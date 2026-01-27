use crate::models::config::AppConfig;
use crate::services::config_service::ConfigService;

#[tauri::command]
pub async fn load_settings() -> Result<AppConfig, String> {
    ConfigService::load().map_err(|e| format!("Failed to load settings: {}", e))
}

#[tauri::command]
pub async fn save_settings(config: AppConfig) -> Result<(), String> {
    ConfigService::save(&config).map_err(|e| format!("Failed to save settings: {}", e))
}
