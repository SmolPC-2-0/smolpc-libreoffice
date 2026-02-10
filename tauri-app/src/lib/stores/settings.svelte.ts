import type { AppSettings } from '$lib/types/settings';
import type { Model } from '$lib/types/ollama';
import { DEFAULT_SETTINGS } from '$lib/types/settings';
import { invoke } from '@tauri-apps/api/core';

class SettingsStore {
  settings = $state<AppSettings>(DEFAULT_SETTINGS);
  availableModels = $state<Model[]>([]);
  isLoading = $state(false);
  isSaving = $state(false);

  async loadSettings(): Promise<void> {
    this.isLoading = true;
    try {
      const loadedSettings = await invoke<AppSettings>('load_settings');
      this.settings = loadedSettings;
      console.log('Settings loaded successfully');
    } catch (error) {
      console.error('Failed to load settings:', error);
      // Use defaults if loading fails
      this.settings = { ...DEFAULT_SETTINGS };
    } finally {
      this.isLoading = false;
    }
  }

  async saveSettings(): Promise<void> {
    this.isSaving = true;
    try {
      await invoke('save_settings', { config: this.settings });
      console.log('Settings saved successfully');
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    } finally {
      this.isSaving = false;
    }
  }

  async loadAvailableModels(): Promise<void> {
    try {
      const response = await invoke<{ models: Model[] }>('list_ollama_models', {
        ollamaUrl: this.settings.ollama_url
      });
      this.availableModels = response.models;
      console.log(`Loaded ${response.models.length} Ollama models`);
    } catch (error) {
      console.error('Failed to load Ollama models:', error);
      this.availableModels = [];
    }
  }

  updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]): void {
    this.settings[key] = value;
  }

  resetToDefaults(): void {
    this.settings = { ...DEFAULT_SETTINGS };
  }
}

export const settingsStore = new SettingsStore();
