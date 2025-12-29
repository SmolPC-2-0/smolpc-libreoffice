use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,
    pub selected_model: String,
    pub python_path: String,
    pub documents_path: String,
    pub libreoffice_path: Option<String>,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: "http://localhost:11434".to_string(),
            selected_model: "qwen2.5:7b".to_string(),
            python_path: "python3".to_string(),
            documents_path: dirs::document_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            libreoffice_path: None,
            theme: "dark".to_string(),
        }
    }
}
