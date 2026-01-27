use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Ollama Configuration
    pub ollama_url: String,
    pub selected_model: String,

    // Paths
    pub python_path: String,
    pub documents_path: String,
    pub libreoffice_path: Option<String>,

    // UI Preferences
    pub theme: String,

    // Advanced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            selected_model: "phi3:latest".to_string(),
            python_path: "python3".to_string(),
            documents_path: dirs::document_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            libreoffice_path: None,
            theme: "dark".to_string(),
            system_prompt: None,
            temperature: Some(0.7),
            max_tokens: Some(2048),
        }
    }
}
