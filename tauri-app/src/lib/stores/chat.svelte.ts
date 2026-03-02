import type { ChatMessage } from '$lib/types/chat';
import type { ChatMessage as OllamaChatMessage, ToolCall } from '$lib/types/ollama';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { mcpStore } from './mcp.svelte';
import { settingsStore } from './settings.svelte';

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

  async sendMessage(content: string, model: string, ollamaUrl: string) {
    // Add user message
    this.addMessage({
      role: 'user',
      content
    });

    await this.generateResponse(model, ollamaUrl);
  }

  private async generateResponse(model: string, ollamaUrl: string) {
    // Start generating
    this.isGenerating = true;
    this.currentStreamingMessage = '';

    // Build message history with system prompt
    const ollamaMessages: OllamaChatMessage[] = [];

    // Add system prompt with tool awareness
    const tools = mcpStore.status.running ? mcpStore.getOllamaTools() : undefined;
    const toolNames = tools?.map(t => t.function.name).join(', ') || '';
    const customPrompt = settingsStore.settings.system_prompt || '';
    const systemPrompt = [
      'You are a helpful assistant with access to tools for managing documents via LibreOffice.',
      toolNames ? `You have the following tools available: ${toolNames}. Use them when the user asks you to create, edit, or manage documents. Always prefer using your tools over giving manual instructions.` : '',
      customPrompt
    ].filter(Boolean).join('\n\n');

    ollamaMessages.push({ role: 'system', content: systemPrompt });

    // Add conversation history
    for (const msg of this.messages) {
      const ollamaMsg: OllamaChatMessage = { role: msg.role, content: msg.content };
      if (msg.toolCalls && msg.toolCalls.length > 0) {
        ollamaMsg.tool_calls = msg.toolCalls;
      }
      ollamaMessages.push(ollamaMsg);
    }

    try {
      let toolCalls: ToolCall[] | undefined;

      // Set up listener for stream chunks
      const unlisten = await listen<any>('ollama-stream-chunk', (event) => {
        const chunk = event.payload;
        console.log('Stream chunk received:', JSON.stringify(chunk).slice(0, 300));

        if (chunk.message && chunk.message.content) {
          this.currentStreamingMessage += chunk.message.content;
        }

        // Capture tool calls if present
        if (chunk.message && chunk.message.tool_calls) {
          toolCalls = chunk.message.tool_calls;
          console.log('Tool calls detected:', toolCalls);
        }

        if (chunk.done) {
          console.log('Stream done. Tool calls:', toolCalls);
          // Handle completion
          this.handleStreamComplete(toolCalls, model, ollamaUrl, unlisten);
        }
      });

      // Set up error listener
      const unlistenError = await listen<string>('ollama-stream-error', (event) => {
        console.error('Stream error:', event.payload);
        this.isGenerating = false;
        this.currentStreamingMessage = '';
        unlistenError();
        unlisten();
      });

      // Start streaming
      console.log(`Starting chat_stream: model=${model}, messages=${ollamaMessages.length}, tools=${tools?.length ?? 0}`);
      await invoke('chat_stream', {
        ollamaUrl,
        request: {
          model,
          messages: ollamaMessages,
          tools
        }
      });
      console.log('chat_stream invoke returned successfully');

    } catch (error) {
      console.error('Failed to start chat:', error);
      this.isGenerating = false;
      this.currentStreamingMessage = '';
    }
  }

  private async handleStreamComplete(
    toolCalls: ToolCall[] | undefined,
    model: string,
    ollamaUrl: string,
    unlisten: () => void
  ) {
    const assistantMessage = this.currentStreamingMessage;
    this.currentStreamingMessage = '';

    // Add assistant message (include tool_calls if present so Ollama sees them in history)
    this.addMessage({
      role: 'assistant',
      content: assistantMessage,
      toolCalls: toolCalls
    });

    // If there are tool calls, execute them
    if (toolCalls && toolCalls.length > 0) {
      console.log('Executing tool calls:', toolCalls);

      for (const toolCall of toolCalls) {
        try {
          const toolName = toolCall.function.name;
          const toolArgs = toolCall.function.arguments;

          console.log(`Calling tool: ${toolName} with args:`, toolArgs);

          // Execute the tool
          const result = await mcpStore.callTool(toolName, toolArgs);

          // Add tool result to messages
          const toolResultText = result.content.map(c => c.text).join('\n');
          this.addMessage({
            role: 'tool',
            content: `Tool ${toolName} result:\n${toolResultText}`
          });

        } catch (error) {
          console.error('Tool execution error:', error);
          this.addMessage({
            role: 'tool',
            content: `Tool execution failed: ${error}`
          });
        }
      }

      // Unlisten to the previous stream
      unlisten();

      // Generate final response with tool results
      await this.generateResponse(model, ollamaUrl);
    } else {
      // No tool calls, just finish
      this.isGenerating = false;
      unlisten();
    }
  }
}

export const chatStore = new ChatStore();
