use crate::models::config::AppConfig;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ConfigService;

impl ConfigService {
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        let app_dir = config_dir.join("libreoffice-ai");
        fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("settings.json"))
    }

    pub fn load() -> Result<AppConfig> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(AppConfig::default());
        }

        let contents = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(config: &AppConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let contents = serde_json::to_string_pretty(config)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
