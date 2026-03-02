use crate::models::config::AppConfig;
use crate::services::config_service::ConfigService;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn load_settings() -> Result<AppConfig, String> {
    ConfigService::load().map_err(|e| format!("Failed to load settings: {}", e))
}

#[tauri::command]
pub async fn save_settings(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    ConfigService::save(&config).map_err(|e| format!("Failed to save settings: {}", e))?;
    {
        let mut app_config = state.config.write();
        *app_config = config;
    }
    Ok(())
}
