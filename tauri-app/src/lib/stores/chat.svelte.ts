import type { ChatMessage } from '$lib/types/chat';
import type { ChatMessage as OllamaChatMessage, ToolCall } from '$lib/types/ollama';
import type { AppSettings } from '$lib/types/settings';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { mcpStore } from './mcp.svelte';

const MAX_TOOL_CHAIN_DEPTH = 4;
const MAX_TOOL_CALLS_PER_RESPONSE = 8;

class ChatStore {
  messages = $state<ChatMessage[]>([]);
  isGenerating = $state(false);
  currentStreamingMessage = $state('');

  messageCount = $derived(this.messages.length);

  addMessage(message: Omit<ChatMessage, 'timestamp'>) {
    const msg: ChatMessage = {
      role: message.role,
      content: message.content,
      timestamp: new Date()
    };
    if (message.toolCalls) {
      msg.toolCalls = message.toolCalls;
    }
    this.messages.push(msg);
  }

  clearMessages() {
    this.messages = [];
  }

  async sendMessage(content: string, settings: AppSettings) {
    this.addMessage({
      role: 'user',
      content
    });

    await this.generateResponse(settings, 0);
  }

  private async generateResponse(settings: AppSettings, toolRound: number) {
    this.isGenerating = true;
    this.currentStreamingMessage = '';

    const supportsToolCalling = mcpStore.status.running;
    const tools = supportsToolCalling
      ? mcpStore.getOllamaTools()
      : undefined;

    const aiMessages: OllamaChatMessage[] = [];
    const toolNames = tools?.map((tool) => tool.function.name).join(', ') || '';
    const providerHint = settings.ai_provider === 'smolpc_engine'
      ? [
          'You are currently using smolpc-engine integration.',
          'If native tool-calling is available, return structured tool calls.',
          'If not, output a JSON object with this exact shape and no extra text:',
          '{"tool_calls":[{"function":{"name":"<tool_name>","arguments":{...}}}]}',
          'After tool results are provided in the conversation, continue with a normal assistant response.'
        ].join('\n')
      : '';

    const customPrompt = settings.system_prompt || '';
    const systemPrompt = [
      'You are a helpful assistant with access to tools for managing documents via LibreOffice.',
      toolNames
        ? `You have the following tools available: ${toolNames}. Use them when the user asks you to create, edit, or manage documents. Always prefer using your tools over giving manual instructions.`
        : '',
      providerHint,
      customPrompt
    ]
      .filter(Boolean)
      .join('\n\n');

    aiMessages.push({ role: 'system', content: systemPrompt });

    for (const msg of this.messages) {
      const aiMessage: OllamaChatMessage = { role: msg.role, content: msg.content };
      if (msg.toolCalls && msg.toolCalls.length > 0) {
        aiMessage.tool_calls = msg.toolCalls;
      }
      aiMessages.push(aiMessage);
    }

    let toolCalls: ToolCall[] | undefined;
    let unlistenChunk: (() => void) | undefined;
    let unlistenError: (() => void) | undefined;
    let streamCompleted = false;

    const cleanupListeners = () => {
      if (unlistenChunk) {
        unlistenChunk();
        unlistenChunk = undefined;
      }
      if (unlistenError) {
        unlistenError();
        unlistenError = undefined;
      }
    };

    try {
      unlistenChunk = await listen<any>('ai-stream-chunk', async (event) => {
        const chunk = event.payload;

        if (chunk?.message?.content) {
          this.currentStreamingMessage += chunk.message.content;
        }

        if (supportsToolCalling) {
          const chunkToolCalls = chunk?.message?.tool_calls ?? chunk?.tool_calls;
          if (chunkToolCalls) {
            toolCalls = chunkToolCalls;
          }
        }

        if (chunk?.done && !streamCompleted) {
          streamCompleted = true;
          cleanupListeners();
          await this.handleStreamComplete(toolCalls, settings, toolRound);
        }
      });

      unlistenError = await listen<string>('ai-stream-error', (event) => {
        if (streamCompleted) return;
        streamCompleted = true;
        cleanupListeners();
        console.error('AI stream error:', event.payload);
        this.isGenerating = false;
        this.currentStreamingMessage = '';
      });

      await invoke('chat_stream_ai', {
        request: {
          provider: settings.ai_provider,
          model: settings.selected_model,
          messages: aiMessages,
          tools,
          ollamaUrl: settings.ollama_url,
          smolpcEngineUrl: settings.smolpc_engine_url,
          temperature: settings.temperature,
          maxTokens: settings.max_tokens
        }
      });
    } catch (error) {
      cleanupListeners();
      console.error('Failed to start AI chat:', error);
      this.isGenerating = false;
      this.currentStreamingMessage = '';
    }
  }

  private async handleStreamComplete(
    toolCalls: ToolCall[] | undefined,
    settings: AppSettings,
    toolRound: number
  ) {
    const assistantMessage = this.currentStreamingMessage;
    this.currentStreamingMessage = '';
    const parsedToolCalls = toolCalls ?? this.extractToolCallsFromText(assistantMessage);

    this.addMessage({
      role: 'assistant',
      content: assistantMessage,
      toolCalls: parsedToolCalls
    });

    if (parsedToolCalls && parsedToolCalls.length > 0) {
      if (toolRound >= MAX_TOOL_CHAIN_DEPTH) {
        this.addMessage({
          role: 'tool',
          content: `Tool chain stopped after ${MAX_TOOL_CHAIN_DEPTH} rounds to avoid infinite loops.`
        });
        this.isGenerating = false;
        return;
      }

      let boundedToolCalls = parsedToolCalls;
      if (parsedToolCalls.length > MAX_TOOL_CALLS_PER_RESPONSE) {
        boundedToolCalls = parsedToolCalls.slice(0, MAX_TOOL_CALLS_PER_RESPONSE);
        this.addMessage({
          role: 'tool',
          content: `Tool call list truncated to ${MAX_TOOL_CALLS_PER_RESPONSE} entries for safety.`
        });
      }

      const executableCalls: Array<{ name: string; args: any }> = [];
      for (const toolCall of boundedToolCalls) {
        const toolName = toolCall.function?.name?.trim();
        if (!toolName) {
          this.addMessage({
            role: 'tool',
            content: 'Skipped invalid tool call: missing tool name.'
          });
          continue;
        }

        if (!mcpStore.getTool(toolName)) {
          this.addMessage({
            role: 'tool',
            content: `Skipped unknown tool call: ${toolName}`
          });
          continue;
        }

        executableCalls.push({
          name: toolName,
          args: this.normalizeToolArguments(toolCall.function.arguments)
        });
      }

      for (const executableCall of executableCalls) {
        try {
          const result = await mcpStore.callTool(executableCall.name, executableCall.args);
          const toolResultText = result.content.map((content) => content.text).join('\n');

          this.addMessage({
            role: 'tool',
            content: `Tool ${executableCall.name} result:\n${toolResultText}`
          });
        } catch (error) {
          console.error('Tool execution error:', error);
          this.addMessage({
            role: 'tool',
            content: `Tool execution failed (${executableCall.name}): ${error}`
          });
        }
      }

      if (executableCalls.length > 0) {
        await this.generateResponse(settings, toolRound + 1);
      } else {
        this.isGenerating = false;
      }
      return;
    }

    this.isGenerating = false;
  }

  private normalizeToolArguments(rawArgs: unknown): any {
    if (typeof rawArgs === 'string') {
      try {
        return rawArgs.trim().length > 0 ? JSON.parse(rawArgs) : {};
      } catch {
        return {};
      }
    }

    if (rawArgs && typeof rawArgs === 'object') {
      return rawArgs;
    }

    return {};
  }

  private extractToolCallsFromText(content: string): ToolCall[] | undefined {
    const trimmed = content.trim();
    if (!trimmed) return undefined;

    // Direct JSON payload
    const direct = this.tryParseToolCallJson(trimmed);
    if (direct && direct.length > 0) {
      return direct;
    }

    // JSON fenced block payload
    const fenceMatch = trimmed.match(/```json\\s*([\\s\\S]*?)\\s*```/i);
    if (fenceMatch?.[1]) {
      const parsed = this.tryParseToolCallJson(fenceMatch[1].trim());
      if (parsed && parsed.length > 0) {
        return parsed;
      }
    }

    return undefined;
  }

  private tryParseToolCallJson(raw: string): ToolCall[] | undefined {
    try {
      const parsed = JSON.parse(raw);

      if (Array.isArray(parsed)) {
        const validated = parsed.filter(this.isToolCallShape);
        return validated.length > 0 ? validated : undefined;
      }

      if (parsed && typeof parsed === 'object') {
        if (Array.isArray(parsed.tool_calls)) {
          const validated = parsed.tool_calls.filter(this.isToolCallShape);
          return validated.length > 0 ? validated : undefined;
        }

        if (this.isToolCallShape(parsed)) {
          return [parsed];
        }
      }
    } catch {
      return undefined;
    }

    return undefined;
  }

  private isToolCallShape(value: any): value is ToolCall {
    return Boolean(
      value &&
      typeof value === 'object' &&
      value.function &&
      typeof value.function.name === 'string' &&
      'arguments' in value.function
    );
  }
}

export const chatStore = new ChatStore();
