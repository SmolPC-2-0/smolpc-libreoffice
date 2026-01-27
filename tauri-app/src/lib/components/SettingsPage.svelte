<script lang="ts">
  import { onMount } from 'svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  onMount(async () => {
    await settingsStore.loadSettings();
    await settingsStore.loadAvailableModels();
  });

  async function handleSave() {
    try {
      await settingsStore.saveSettings();
      onClose();
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert('Failed to save settings. Please try again.');
    }
  }

  function handleReset() {
    if (confirm('Reset all settings to defaults?')) {
      settingsStore.resetToDefaults();
    }
  }
</script>

<div class="settings-page">
  <div class="settings-header">
    <h1>Settings</h1>
    <button class="close-button" onclick={onClose} aria-label="Close settings">
      ×
    </button>
  </div>

  <div class="settings-content">
    <!-- Ollama Configuration -->
    <section class="settings-section">
      <h2>Ollama Configuration</h2>

      <div class="setting-item">
        <label for="ollama-url">Ollama URL</label>
        <input
          id="ollama-url"
          type="text"
          value={settingsStore.settings.ollama_url}
          oninput={(e) => settingsStore.updateSetting('ollama_url', e.currentTarget.value)}
          placeholder="http://localhost:11434"
        />
        <span class="setting-description">URL where Ollama is running</span>
      </div>

      <div class="setting-item">
        <label for="model-select">Model</label>
        <select
          id="model-select"
          value={settingsStore.settings.selected_model}
          onchange={(e) => settingsStore.updateSetting('selected_model', e.currentTarget.value)}
        >
          {#if settingsStore.availableModels.length === 0}
            <option value={settingsStore.settings.selected_model}>
              {settingsStore.settings.selected_model}
            </option>
          {:else}
            {#each settingsStore.availableModels as model}
              <option value={model.name}>{model.name}</option>
            {/each}
          {/if}
        </select>
        <span class="setting-description">
          {settingsStore.availableModels.length} models available
        </span>
      </div>

      <div class="setting-item">
        <label for="temperature">Temperature</label>
        <div class="slider-container">
          <input
            id="temperature"
            type="range"
            min="0"
            max="2"
            step="0.1"
            value={settingsStore.settings.temperature ?? 0.7}
            oninput={(e) => settingsStore.updateSetting('temperature', parseFloat(e.currentTarget.value))}
          />
          <span class="slider-value">{(settingsStore.settings.temperature ?? 0.7).toFixed(1)}</span>
        </div>
        <span class="setting-description">Controls randomness (0 = focused, 2 = creative)</span>
      </div>
    </section>

    <!-- Paths -->
    <section class="settings-section">
      <h2>Paths</h2>

      <div class="setting-item">
        <label for="python-path">Python Path</label>
        <input
          id="python-path"
          type="text"
          value={settingsStore.settings.python_path}
          oninput={(e) => settingsStore.updateSetting('python_path', e.currentTarget.value)}
          placeholder="python3"
        />
        <span class="setting-description">Path to Python executable</span>
      </div>

      <div class="setting-item">
        <label for="documents-path">Documents Folder</label>
        <input
          id="documents-path"
          type="text"
          value={settingsStore.settings.documents_path}
          oninput={(e) => settingsStore.updateSetting('documents_path', e.currentTarget.value)}
          placeholder="~/Documents"
        />
        <span class="setting-description">Default folder for created documents</span>
      </div>

      <div class="setting-item">
        <label for="libreoffice-path">LibreOffice Path (Optional)</label>
        <input
          id="libreoffice-path"
          type="text"
          value={settingsStore.settings.libreoffice_path ?? ''}
          oninput={(e) => settingsStore.updateSetting('libreoffice_path', e.currentTarget.value || null)}
          placeholder="Auto-detect"
        />
        <span class="setting-description">Leave empty for auto-detection</span>
      </div>
    </section>

    <!-- UI Preferences -->
    <section class="settings-section">
      <h2>Appearance</h2>

      <div class="setting-item">
        <label for="theme">Theme</label>
        <select
          id="theme"
          value={settingsStore.settings.theme}
          onchange={(e) => settingsStore.updateSetting('theme', e.currentTarget.value as 'dark' | 'light')}
        >
          <option value="dark">Dark</option>
          <option value="light">Light</option>
        </select>
        <span class="setting-description">Color scheme for the application</span>
      </div>
    </section>

    <!-- Advanced -->
    <section class="settings-section">
      <h2>Advanced</h2>

      <div class="setting-item">
        <label for="system-prompt">System Prompt (Optional)</label>
        <textarea
          id="system-prompt"
          value={settingsStore.settings.system_prompt ?? ''}
          oninput={(e) => settingsStore.updateSetting('system_prompt', e.currentTarget.value || undefined)}
          placeholder="You are a helpful AI assistant..."
          rows="4"
        ></textarea>
        <span class="setting-description">Custom instructions for the AI</span>
      </div>

      <div class="setting-item">
        <label for="max-tokens">Max Tokens</label>
        <input
          id="max-tokens"
          type="number"
          min="256"
          max="8192"
          value={settingsStore.settings.max_tokens ?? 2048}
          oninput={(e) => settingsStore.updateSetting('max_tokens', parseInt(e.currentTarget.value))}
        />
        <span class="setting-description">Maximum length of generated responses</span>
      </div>
    </section>
  </div>

  <div class="settings-footer">
    <button class="button-secondary" onclick={handleReset}>
      Reset to Defaults
    </button>
    <div class="footer-actions">
      <button class="button-secondary" onclick={onClose}>
        Cancel
      </button>
      <button class="button-primary" onclick={handleSave} disabled={settingsStore.isSaving}>
        {settingsStore.isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  </div>
</div>

<style>
  .settings-page {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #1a1a1a;
    color: #e0e0e0;
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 1px solid #2a2a2a;
  }

  .settings-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .close-button {
    background: none;
    border: none;
    color: #888;
    font-size: 2rem;
    cursor: pointer;
    line-height: 1;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-button:hover {
    color: #e0e0e0;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  .settings-section {
    margin-bottom: 2.5rem;
  }

  .settings-section h2 {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0 0 1.5rem 0;
    color: #4a9eff;
  }

  .setting-item {
    margin-bottom: 1.5rem;
  }

  .setting-item label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #b0b0b0;
  }

  .setting-item input[type="text"],
  .setting-item input[type="number"],
  .setting-item select,
  .setting-item textarea {
    width: 100%;
    padding: 0.75rem;
    background-color: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 0.95rem;
    font-family: inherit;
  }

  .setting-item input:focus,
  .setting-item select:focus,
  .setting-item textarea:focus {
    outline: none;
    border-color: #4a9eff;
  }

  .slider-container {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .slider-container input[type="range"] {
    flex: 1;
  }

  .slider-value {
    min-width: 3rem;
    text-align: right;
    font-weight: 600;
    color: #4a9eff;
  }

  .setting-description {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.85rem;
    color: #666;
  }

  .settings-footer {
    display: flex;
    justify-content: space-between;
    padding: 1.5rem 2rem;
    border-top: 1px solid #2a2a2a;
  }

  .footer-actions {
    display: flex;
    gap: 1rem;
  }

  .button-primary,
  .button-secondary {
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    font-size: 0.95rem;
  }

  .button-primary {
    background-color: #4a9eff;
    color: white;
  }

  .button-primary:hover:not(:disabled) {
    background-color: #3a8eef;
  }

  .button-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button-secondary {
    background-color: #2a2a2a;
    color: #e0e0e0;
  }

  .button-secondary:hover {
    background-color: #3a3a3a;
  }
</style>
