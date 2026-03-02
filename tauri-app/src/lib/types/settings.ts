export type AiProvider = 'ollama' | 'smolpc_engine';

export interface AppSettings {
  // AI Provider Configuration
  ai_provider: AiProvider;

  // Ollama Configuration
  ollama_url: string;
  selected_model: string;
  smolpc_engine_url: string;

  // Paths
  python_path: string;
  documents_path: string;
  libreoffice_path: string | null;

  // UI Preferences
  theme: 'dark' | 'light';

  // Advanced
  system_prompt?: string;
  temperature?: number;
  max_tokens?: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
  ai_provider: 'ollama',
  ollama_url: 'http://localhost:11434',
  selected_model: 'qwen2.5-coder:7b',
  smolpc_engine_url: 'http://127.0.0.1:19432',
  python_path: 'python',
  documents_path: '~/Documents',
  libreoffice_path: null,
  theme: 'dark',
  temperature: 0.7,
  max_tokens: 2048
};
