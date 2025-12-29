use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC request to MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl McpRequest {
    pub fn new(id: u64, method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
        }
    }
}

/// JSON-RPC response from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
}

/// MCP tool invocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: Value,
}

/// MCP tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP initialization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitParams {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

impl Default for McpInitParams {
    fn default() -> Self {
        Self {
            protocol_version: "2024-11-05".to_string(),
            capabilities: McpCapabilities {
                tools: Some(serde_json::json!({})),
            },
            client_info: ClientInfo {
                name: "tauri-app".to_string(),
                version: "0.1.0".to_string(),
            },
        }
    }
}
