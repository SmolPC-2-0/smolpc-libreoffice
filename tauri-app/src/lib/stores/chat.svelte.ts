import type { ChatMessage } from '$lib/types/chat';
import type { ChatMessage as OllamaChatMessage, ToolCall } from '$lib/types/ollama';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { mcpStore } from './mcp.svelte';

class ChatStore {
  messages = $state<ChatMessage[]>([]);
  isGenerating = $state(false);
  currentStreamingMessage = $state('');

  messageCount = $derived(this.messages.length);

  addMessage(message: Omit<ChatMessage, 'timestamp'>) {
    this.messages.push({
      ...message,
      timestamp: new Date()
    });
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

    // Build message history
    const ollamaMessages: OllamaChatMessage[] = this.messages.map(msg => ({
      role: msg.role,
      content: msg.content
    }));

    try {
      let toolCalls: ToolCall[] | undefined;

      // Set up listener for stream chunks
      const unlisten = await listen<any>('ollama-stream-chunk', (event) => {
        const chunk = event.payload;

        if (chunk.message && chunk.message.content) {
          this.currentStreamingMessage += chunk.message.content;
        }

        // Capture tool calls if present
        if (chunk.message && chunk.message.tool_calls) {
          toolCalls = chunk.message.tool_calls;
        }

        if (chunk.done) {
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

      // Get MCP tools if available
      const tools = mcpStore.status.running ? mcpStore.getOllamaTools() : undefined;

      // Start streaming
      await invoke('chat_stream', {
        ollamaUrl,
        request: {
          model,
          messages: ollamaMessages,
          tools
        }
      });

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

    // Add assistant message
    this.addMessage({
      role: 'assistant',
      content: assistantMessage
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
