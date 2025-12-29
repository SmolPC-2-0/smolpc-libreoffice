import { invoke } from '@tauri-apps/api/core';
import type { DependencyStatus, McpStatus } from '$lib/types/system';

class AppStore {
  pythonStatus = $state<DependencyStatus | null>(null);
  ollamaStatus = $state<DependencyStatus | null>(null);
  libreofficeStatus = $state<DependencyStatus | null>(null);
  mcpStatus = $state<McpStatus | null>(null);

  isInitializing = $state(true);

  allDependenciesReady = $derived(
    this.pythonStatus?.installed &&
    this.ollamaStatus?.installed
    // LibreOffice and MCP are optional (MCP needed for document features)
  );

  async initialize() {
    this.isInitializing = true;

    try {
      // Check all dependencies in parallel
      const [python, ollama, libreoffice] = await Promise.all([
        invoke<DependencyStatus>('check_python'),
        invoke<DependencyStatus>('check_ollama'),
        invoke<DependencyStatus>('check_libreoffice')
      ]);

      this.pythonStatus = python;
      this.ollamaStatus = ollama;
      this.libreofficeStatus = libreoffice;

      // If Python and Ollama are ready, start MCP server
      if (python.installed && ollama.installed) {
        const mcpStatus = await invoke<McpStatus>('start_mcp_server');
        this.mcpStatus = mcpStatus;
      } else {
        this.mcpStatus = {
          running: false,
          error_message: 'Dependencies not met. Python and Ollama are required.'
        };
      }
    } catch (error) {
      console.error('Failed to check dependencies:', error);
    } finally {
      this.isInitializing = false;
    }
  }
}

export const appStore = new AppStore();
