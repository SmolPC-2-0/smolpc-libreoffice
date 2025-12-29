<script lang="ts">
  import type { ChatMessage } from '$lib/types/chat';

  interface Props {
    message: ChatMessage;
  }

  let { message }: Props = $props();
</script>

<div class="message {message.role}">
  <div class="message-header">
    <span class="role">{message.role === 'user' ? 'You' : 'Assistant'}</span>
    <span class="timestamp">{message.timestamp.toLocaleTimeString()}</span>
  </div>
  <div class="message-content">
    {message.content}
  </div>
</div>

<style>
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

  .message.user {
    background-color: #2a4a6a;
    margin-left: 2rem;
  }

  .message.assistant {
    background-color: #2a2a2a;
    margin-right: 2rem;
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
</style>
