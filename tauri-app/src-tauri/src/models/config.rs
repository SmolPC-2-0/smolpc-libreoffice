use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiProvider {
    Ollama,
    SmolpcEngine,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self::Ollama
    }
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_selected_model() -> String {
    "phi3:latest".to_string()
}

fn default_smolpc_engine_url() -> String {
    "http://127.0.0.1:19432".to_string()
}

fn default_python_path() -> String {
    "python3".to_string()
}

fn default_documents_path() -> String {
    dirs::document_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_temperature() -> Option<f32> {
    Some(0.7)
}

fn default_max_tokens() -> Option<u32> {
    Some(2048)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    // AI Provider Configuration
    pub ai_provider: AiProvider,

    // Ollama Configuration
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    #[serde(default = "default_selected_model")]
    pub selected_model: String,
    #[serde(default = "default_smolpc_engine_url")]
    pub smolpc_engine_url: String,

    // Paths
    #[serde(default = "default_python_path")]
    pub python_path: String,
    #[serde(default = "default_documents_path")]
    pub documents_path: String,
    pub libreoffice_path: Option<String>,

    // UI Preferences
    #[serde(default = "default_theme")]
    pub theme: String,

    // Advanced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "default_temperature"
    )]
    pub temperature: Option<f32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default = "default_max_tokens"
    )]
    pub max_tokens: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_provider: AiProvider::default(),
            ollama_url: default_ollama_url(),
            selected_model: default_selected_model(),
            smolpc_engine_url: default_smolpc_engine_url(),
            python_path: default_python_path(),
            documents_path: default_documents_path(),
            libreoffice_path: None,
            theme: default_theme(),
            system_prompt: None,
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}
