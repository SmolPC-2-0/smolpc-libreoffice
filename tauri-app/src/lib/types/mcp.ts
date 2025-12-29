export interface McpStatus {
  running: boolean;
  error_message?: string;
}

export interface McpTool {
  name: string;
  description: string;
  input_schema?: any;
}

export interface ToolInvocation {
  name: string;
  arguments: any;
}

export interface ToolContent {
  type: string;
  text: string;
}

export interface ToolResult {
  content: ToolContent[];
  is_error?: boolean;
}
