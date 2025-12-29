export interface DependencyStatus {
  name: string;
  installed: boolean;
  version?: string;
  error_message?: string;
}

export interface McpStatus {
  running: boolean;
  error_message?: string;
}
