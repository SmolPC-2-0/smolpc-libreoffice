use crate::AppState;
use crate::models::mcp::{McpTool, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpStatus {
    pub running: bool,
    pub error_message: Option<String>,
}

fn get_mcp_resource_path(app: &AppHandle) -> Result<PathBuf, String> {
    // In development, use the local resources directory
    // In production, Tauri will bundle these files

    if cfg!(debug_assertions) {
        // Development mode - cargo runs from src-tauri/, go up to tauri-app/
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let mcp_dir = current_dir.join("resources/mcp_server");
        // If the venv exists here, use this path; otherwise try parent directory
        if mcp_dir.join(".venv").exists() {
            Ok(mcp_dir)
        } else {
            let parent_mcp_dir = current_dir.join("../resources/mcp_server");
            if parent_mcp_dir.exists() {
                log::info!("Using parent directory MCP path: {:?}", parent_mcp_dir);
                Ok(parent_mcp_dir)
            } else {
                Ok(mcp_dir) // Fall back to original path
            }
        }
    } else {
        // Production mode - use bundled resources
        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource directory: {}", e))?;
        Ok(resource_dir.join("mcp_server"))
    }
}

#[tauri::command]
pub async fn start_mcp_server(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<McpStatus, String> {
    let python_path = {
        let config = state.config.read();
        config.python_path.clone()
    };

    let mcp_dir = get_mcp_resource_path(&app)?;

    // Start the MCP server process
    match state.mcp_client.start(&python_path, mcp_dir) {
        Ok(_) => {
            // Initialize the MCP connection
            if let Err(e) = state.mcp_client.initialize() {
                log::error!("Failed to initialize MCP: {}", e);
                return Ok(McpStatus {
                    running: false,
                    error_message: Some(format!("Failed to initialize MCP: {}", e)),
                });
            }

            // Discover tools
            if let Err(e) = state.mcp_client.list_tools() {
                log::warn!("Failed to list tools: {}", e);
            }

            Ok(McpStatus {
                running: true,
                error_message: None,
            })
        }
        Err(e) => Ok(McpStatus {
            running: false,
            error_message: Some(format!("Failed to start MCP server: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn check_mcp_status(
    state: State<'_, AppState>,
) -> Result<McpStatus, String> {
    let running = state.mcp_client.is_running();
    Ok(McpStatus {
        running,
        error_message: if !running {
            Some("MCP server is not running".to_string())
        } else {
            None
        },
    })
}

#[tauri::command]
pub async fn stop_mcp_server(
    state: State<'_, AppState>,
) -> Result<McpStatus, String> {
    match state.mcp_client.stop() {
        Ok(_) => Ok(McpStatus {
            running: false,
            error_message: None,
        }),
        Err(e) => Ok(McpStatus {
            running: true,
            error_message: Some(format!("Failed to stop MCP server: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn list_mcp_tools(
    state: State<'_, AppState>,
) -> Result<Vec<McpTool>, String> {
    state.mcp_client
        .get_tools()
        .into_iter()
        .map(Ok)
        .collect()
}

#[tauri::command]
pub async fn call_mcp_tool(
    state: State<'_, AppState>,
    name: String,
    arguments: Value,
) -> Result<ToolResult, String> {
    state.mcp_client
        .call_tool(name, arguments)
        .map_err(|e| format!("Tool call failed: {}", e))
}
