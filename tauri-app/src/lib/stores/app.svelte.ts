import { invoke } from '@tauri-apps/api/core';
import type { DependencyStatus, McpStatus } from '$lib/types/system';
import type { AiProvider, AppSettings } from '$lib/types/settings';

class AppStore {
  pythonStatus = $state<DependencyStatus | null>(null);
  aiStatus = $state<DependencyStatus | null>(null);
  libreofficeStatus = $state<DependencyStatus | null>(null);
  mcpStatus = $state<McpStatus | null>(null);

  aiProvider = $state<AiProvider>('ollama');
  aiProviderLabel = $derived(this.aiProvider === 'smolpc_engine' ? 'SmolPC Engine' : 'Ollama');

  isInitializing = $state(true);

  allDependenciesReady = $derived(
    this.pythonStatus?.installed &&
    this.aiStatus?.installed
    // LibreOffice and MCP are optional (MCP needed for document features)
  );

  async initialize(settings: AppSettings) {
    this.isInitializing = true;
    this.aiProvider = settings.ai_provider;

    try {
      const aiCheckPromise = settings.ai_provider === 'smolpc_engine'
        ? invoke<DependencyStatus>('check_smolpc_engine', { engineUrl: settings.smolpc_engine_url })
        : invoke<DependencyStatus>('check_ollama', { ollamaUrl: settings.ollama_url });

      // Check dependencies in parallel
      const [python, ai, libreoffice] = await Promise.all([
        invoke<DependencyStatus>('check_python'),
        aiCheckPromise,
        invoke<DependencyStatus>('check_libreoffice')
      ]);

      this.pythonStatus = python;
      this.aiStatus = ai;
      this.libreofficeStatus = libreoffice;

      // If Python and selected AI provider are ready, start MCP server
      if (python.installed && ai.installed) {
        const mcpStatus = await invoke<McpStatus>('start_mcp_server');
        this.mcpStatus = mcpStatus;
      } else {
        this.mcpStatus = {
          running: false,
          error_message: `Dependencies not met. Python and ${this.aiProviderLabel} are required.`
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
