import type { ToolCall } from './ollama';

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  toolCalls?: ToolCall[];
  isThinking?: boolean;
  timestamp: Date;
}
