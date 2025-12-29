import { invoke } from '@tauri-apps/api/core';
import type { McpTool, McpStatus, ToolResult } from '$lib/types/mcp';
import type { OllamaTool } from '$lib/types/ollama';

class McpStore {
  status = $state<McpStatus>({ running: false });
  tools = $state<McpTool[]>([]);
  isInitialized = $state(false);

  async checkStatus(): Promise<void> {
    try {
      this.status = await invoke<McpStatus>('check_mcp_status');
    } catch (error) {
      console.error('Failed to check MCP status:', error);
      this.status = { running: false, error_message: String(error) };
    }
  }

  async loadTools(): Promise<void> {
    try {
      this.tools = await invoke<McpTool[]>('list_mcp_tools');
      this.isInitialized = true;
      console.log(`Loaded ${this.tools.length} MCP tools`);
    } catch (error) {
      console.error('Failed to load MCP tools:', error);
      this.tools = [];
    }
  }

  async callTool(name: string, arguments_: any): Promise<ToolResult> {
    try {
      const result = await invoke<ToolResult>('call_mcp_tool', {
        name,
        arguments: arguments_
      });
      return result;
    } catch (error) {
      console.error(`Failed to call tool ${name}:`, error);
      throw error;
    }
  }

  /**
   * Convert MCP tools to Ollama tool format
   */
  getOllamaTools(): OllamaTool[] {
    return this.tools.map(tool => ({
      type: 'function' as const,
      function: {
        name: tool.name,
        description: tool.description,
        parameters: tool.input_schema || {}
      }
    }));
  }

  /**
   * Get a tool by name
   */
  getTool(name: string): McpTool | undefined {
    return this.tools.find(t => t.name === name);
  }
}

export const mcpStore = new McpStore();
