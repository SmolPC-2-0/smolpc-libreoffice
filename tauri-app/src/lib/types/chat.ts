export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  toolCalls?: string[];
  isThinking?: boolean;
  timestamp: Date;
}
