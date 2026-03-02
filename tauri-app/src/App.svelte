<script lang="ts">
  import { onMount } from 'svelte';
  import { appStore } from '$lib/stores/app.svelte';
  import { chatStore } from '$lib/stores/chat.svelte';
  import { mcpStore } from '$lib/stores/mcp.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import LoadingScreen from '$lib/components/LoadingScreen.svelte';
  import ChatMessage from '$lib/components/ChatMessage.svelte';
  import ChatInput from '$lib/components/ChatInput.svelte';
  import SettingsPage from '$lib/components/SettingsPage.svelte';

  type View = 'chat' | 'settings';

  let messagesContainer = $state<HTMLDivElement | undefined>(undefined);
  let currentView = $state<View>('chat');

  onMount(async () => {
    // Load settings first
    await settingsStore.loadSettings();

    await appStore.initialize(settingsStore.settings);

    // Load MCP tools if server is running
    if (appStore.mcpStatus?.running) {
      await mcpStore.checkStatus();
      await mcpStore.loadTools();
    }
  });

  // Auto-scroll to bottom when new messages arrive
  $effect(() => {
    if (chatStore.messages.length > 0 || chatStore.currentStreamingMessage) {
      setTimeout(() => {
        if (messagesContainer) {
          messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }
      }, 50);
    }
  });

  async function handleSend(message: string) {
    await chatStore.sendMessage(message, settingsStore.settings);
  }

  async function handleSettingsSaved() {
    currentView = 'chat';
    await appStore.initialize(settingsStore.settings);

    if (appStore.mcpStatus?.running) {
      await mcpStore.checkStatus();
      await mcpStore.loadTools();
    }
  }
</script>

{#if appStore.isInitializing || !appStore.allDependenciesReady}
  <LoadingScreen
    pythonStatus={appStore.pythonStatus}
    aiStatus={appStore.aiStatus}
    aiProviderLabel={appStore.aiProviderLabel}
    libreofficeStatus={appStore.libreofficeStatus}
    mcpStatus={appStore.mcpStatus}
  />
{:else if currentView === 'settings'}
  <SettingsPage onClose={() => currentView = 'chat'} onSaved={handleSettingsSaved} />
{:else}
  <div class="app">
    <header>
      <div class="header-content">
        <div>
          <h1>LibreOffice AI</h1>
          <p class="subtitle">Chat with AI to create and edit documents</p>
        </div>
        <button class="settings-button" onclick={() => currentView = 'settings'} aria-label="Settings">
          ⚙️
        </button>
      </div>
    </header>

    <div class="chat-container">
      <div class="messages" bind:this={messagesContainer}>
        {#if chatStore.messages.length === 0}
          <div class="welcome">
            <h2>Welcome to LibreOffice AI</h2>
            <p>Start chatting to create documents, presentations, and more!</p>
            <div class="examples">
              <p class="examples-title">Try asking:</p>
              <ul>
                <li>"Create a document about climate change"</li>
                <li>"Make a presentation about AI"</li>
                <li>"What can you help me with?"</li>
              </ul>
            </div>
          </div>
        {/if}

        {#each chatStore.messages as message}
          <ChatMessage {message} />
        {/each}

        {#if chatStore.currentStreamingMessage}
          <div class="message assistant streaming">
            <div class="message-header">
              <span class="role">Assistant</span>
              <span class="timestamp">streaming...</span>
            </div>
            <div class="message-content">
              {chatStore.currentStreamingMessage}<span class="cursor">▊</span>
            </div>
          </div>
        {/if}
      </div>

      <ChatInput onSend={handleSend} disabled={chatStore.isGenerating} />
    </div>
  </div>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background-color: #1a1a1a;
    color: #e0e0e0;
  }

  header {
    border-bottom: 1px solid #3a3a3a;
    padding: 1.5rem 2rem;
    background-color: #242424;
    flex-shrink: 0;
  }

  .header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: bold;
    color: #ffffff;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: #a0a0a0;
    font-size: 0.9rem;
  }

  .settings-button {
    background: none;
    border: 2px solid #3a3a3a;
    color: #a0a0a0;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .settings-button:hover {
    border-color: #4a9eff;
    color: #4a9eff;
    background-color: rgba(74, 158, 255, 0.1);
  }

  .chat-container {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
    display: flex;
    flex-direction: column;
  }

  .welcome {
    max-width: 600px;
    margin: auto;
    text-align: center;
  }

  .welcome h2 {
    font-size: 1.75rem;
    margin-bottom: 1rem;
    color: #ffffff;
  }

  .welcome p {
    color: #a0a0a0;
    margin-bottom: 2rem;
    font-size: 1.05rem;
  }

  .examples {
    background-color: #242424;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    padding: 1.5rem;
    text-align: left;
  }

  .examples-title {
    font-weight: 600;
    color: #c0c0c0;
    margin-bottom: 0.75rem;
  }

  .examples ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .examples li {
    padding: 0.5rem 0;
    color: #8a8a8a;
    font-style: italic;
  }

  .examples li::before {
    content: "💬 ";
    margin-right: 0.5rem;
  }

  .message {
    margin-bottom: 1rem;
    padding: 1rem;
    border-radius: 8px;
    animation: fadeIn 0.3s ease-in;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .message.assistant {
    background-color: #2a2a2a;
    margin-right: 2rem;
  }

  .message.streaming {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.8;
    }
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
  }

  .role {
    font-weight: 600;
    color: #a0a0a0;
  }

  .timestamp {
    color: #707070;
  }

  .message-content {
    color: #e0e0e0;
    line-height: 1.6;
    white-space: pre-wrap;
    word-wrap: break-word;
  }

  .cursor {
    animation: blink 1s steps(2) infinite;
    color: #4a7a9a;
  }

  @keyframes blink {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0;
    }
  }
</style>
