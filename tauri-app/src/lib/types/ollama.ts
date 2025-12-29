export interface ChatMessage {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  tool_calls?: ToolCall[];
}

export interface ToolCall {
  function: {
    name: string;
    arguments: any;
  };
}

export interface OllamaTool {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: any;
  };
}

export interface Model {
  name: string;
  modified_at: string;
  size: number;
}

export interface StreamChunk {
  model: string;
  message: ChatMessage;
  done: boolean;
}

export interface ChatStreamRequest {
  model: string;
  messages: ChatMessage[];
  tools?: OllamaTool[];
}
