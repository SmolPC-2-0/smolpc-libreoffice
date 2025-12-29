<script lang="ts">
  interface Props {
    onSend: (message: string) => void;
    disabled?: boolean;
  }

  let { onSend, disabled = false }: Props = $props();

  let inputValue = $state('');

  function handleSubmit() {
    const trimmed = inputValue.trim();
    if (trimmed && !disabled) {
      onSend(trimmed);
      inputValue = '';
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      handleSubmit();
    }
  }
</script>

<div class="chat-input-container">
  <textarea
    bind:value={inputValue}
    onkeydown={handleKeydown}
    placeholder={disabled ? 'Generating response...' : 'Type your message... (Enter to send, Shift+Enter for new line)'}
    {disabled}
    rows="3"
  ></textarea>
  <button onclick={handleSubmit} disabled={disabled || !inputValue.trim()}>
    {disabled ? 'Sending...' : 'Send'}
  </button>
</div>

<style>
  .chat-input-container {
    display: flex;
    gap: 0.75rem;
    padding: 1rem;
    background-color: #242424;
    border-top: 1px solid #3a3a3a;
  }

  textarea {
    flex: 1;
    padding: 0.75rem;
    background-color: #1a1a1a;
    border: 1px solid #3a3a3a;
    border-radius: 6px;
    color: #e0e0e0;
    font-family: inherit;
    font-size: 0.95rem;
    resize: vertical;
    min-height: 60px;
  }

  textarea:focus {
    outline: none;
    border-color: #4a7a9a;
  }

  textarea:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  textarea::placeholder {
    color: #707070;
  }

  button {
    padding: 0.75rem 1.5rem;
    background-color: #4a7a9a;
    color: #ffffff;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    align-self: flex-end;
  }

  button:hover:not(:disabled) {
    background-color: #5a8aaa;
  }

  button:disabled {
    background-color: #3a3a3a;
    color: #707070;
    cursor: not-allowed;
  }
</style>
