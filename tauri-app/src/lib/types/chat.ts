export interface ChatMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  toolCalls?: string[];
  isThinking?: boolean;
  timestamp: Date;
}
