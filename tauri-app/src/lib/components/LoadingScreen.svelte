<script lang="ts">
  import type { DependencyStatus, McpStatus } from '$lib/types/system';

  interface Props {
    pythonStatus: DependencyStatus | null;
    ollamaStatus: DependencyStatus | null;
    libreofficeStatus: DependencyStatus | null;
    mcpStatus: McpStatus | null;
  }

  let { pythonStatus, ollamaStatus, libreofficeStatus, mcpStatus }: Props = $props();

  function getStatusBadge(dep: DependencyStatus | null) {
    if (!dep) return { class: 'checking', text: 'Checking...' };
    if (dep.installed) return { class: 'ready', text: '✓ Ready' };
    return { class: 'not-found', text: '✗ Not Found' };
  }

  function getMcpStatusBadge(status: McpStatus | null) {
    if (!status) return { class: 'checking', text: 'Checking...' };
    if (status.running) return { class: 'ready', text: '✓ Running' };
    return { class: 'not-found', text: '✗ Not Running' };
  }
</script>

<div class="loading-screen">
  <div class="card">
    <h2>LibreOffice AI</h2>
    <p class="subtitle">Checking system dependencies...</p>

    <div class="dependencies">
      <!-- Python Status -->
      <div class="dependency-item">
        <span class="dep-name">Python 3.12+</span>
        <span class="badge {getStatusBadge(pythonStatus).class}">
          {getStatusBadge(pythonStatus).text}
        </span>
      </div>
      {#if pythonStatus?.error_message}
        <div class="alert error">
          {pythonStatus.error_message}
        </div>
      {/if}

      <!-- Ollama Status -->
      <div class="dependency-item">
        <span class="dep-name">Ollama</span>
        <span class="badge {getStatusBadge(ollamaStatus).class}">
          {getStatusBadge(ollamaStatus).text}
        </span>
      </div>
      {#if ollamaStatus?.error_message}
        <div class="alert error">
          {ollamaStatus.error_message}
        </div>
      {/if}

      <!-- LibreOffice Status -->
      <div class="dependency-item">
        <span class="dep-name">LibreOffice (optional)</span>
        <span class="badge {getStatusBadge(libreofficeStatus).class}">
          {getStatusBadge(libreofficeStatus).text}
        </span>
      </div>
      {#if libreofficeStatus?.error_message}
        <div class="alert warning">
          {libreofficeStatus.error_message}
        </div>
      {/if}

      <!-- MCP Server Status -->
      <div class="dependency-item">
        <span class="dep-name">MCP Server</span>
        <span class="badge {getMcpStatusBadge(mcpStatus).class}">
          {getMcpStatusBadge(mcpStatus).text}
        </span>
      </div>
      {#if mcpStatus?.error_message}
        <div class="alert error">
          {mcpStatus.error_message}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    background-color: #1a1a1a;
    color: #e0e0e0;
  }

  .card {
    background-color: #2a2a2a;
    border: 1px solid #3a3a3a;
    border-radius: 8px;
    padding: 2rem;
    width: 100%;
    max-width: 500px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  }

  h2 {
    font-size: 1.75rem;
    font-weight: bold;
    margin-bottom: 0.5rem;
    color: #ffffff;
  }

  .subtitle {
    color: #a0a0a0;
    margin-bottom: 1.5rem;
    font-size: 0.95rem;
  }

  .dependencies {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .dependency-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background-color: #242424;
    border-radius: 6px;
  }

  .dep-name {
    font-weight: 500;
    color: #e0e0e0;
  }

  .badge {
    padding: 0.35rem 0.75rem;
    border-radius: 4px;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .badge.checking {
    background-color: #4a5568;
    color: #cbd5e0;
  }

  .badge.ready {
    background-color: #2d5a2d;
    color: #9ae69a;
  }

  .badge.not-found {
    background-color: #5a2d2d;
    color: #e69a9a;
  }

  .alert {
    padding: 0.75rem;
    border-radius: 6px;
    font-size: 0.9rem;
    margin-top: 0.5rem;
  }

  .alert.error {
    background-color: #4a2020;
    border: 1px solid #6a3030;
    color: #e69a9a;
  }

  .alert.warning {
    background-color: #4a4020;
    border: 1px solid #6a6030;
    color: #e6d89a;
  }
</style>
