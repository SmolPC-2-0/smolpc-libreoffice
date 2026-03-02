pub mod commands;
pub mod models;
pub mod services;
pub mod utils;

use parking_lot::RwLock;
use std::sync::Arc;

use crate::models::config::AppConfig;
use crate::services::{
    config_service::ConfigService, mcp_client::McpClient, process_manager::ProcessManager,
};

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub process_manager: Arc<ProcessManager>,
    pub mcp_client: Arc<McpClient>,
}

impl AppState {
    pub fn new() -> Self {
        let config = ConfigService::load().unwrap_or_default();

        Self {
            config: Arc::new(RwLock::new(config)),
            process_manager: Arc::new(ProcessManager::new()),
            mcp_client: Arc::new(McpClient::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::system::check_python,
            commands::system::check_ollama,
            commands::system::check_smolpc_engine,
            commands::system::check_libreoffice,
            commands::mcp::start_mcp_server,
            commands::mcp::check_mcp_status,
            commands::mcp::stop_mcp_server,
            commands::mcp::list_mcp_tools,
            commands::mcp::call_mcp_tool,
            commands::ai::list_ai_models,
            commands::ai::chat_stream_ai,
            commands::ollama::list_ollama_models,
            commands::ollama::chat_stream,
            commands::ollama::check_ollama_running,
            commands::config::load_settings,
            commands::config::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
