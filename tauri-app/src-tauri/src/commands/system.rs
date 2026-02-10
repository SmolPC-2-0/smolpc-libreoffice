use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub error_message: Option<String>,
}

#[tauri::command]
pub async fn check_python() -> Result<DependencyStatus, String> {
    // Try python3, python, and py (Windows launcher) in order
    let commands = if cfg!(target_os = "windows") {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python"]
    };

    for cmd in commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).to_string();
                return Ok(DependencyStatus {
                    name: "Python".to_string(),
                    installed: true,
                    version: Some(version.trim().to_string()),
                    error_message: None,
                });
            }
        }
    }

    Ok(DependencyStatus {
        name: "Python".to_string(),
        installed: false,
        version: None,
        error_message: Some("Python 3.12+ not found. Please install from python.org".to_string()),
    })
}

#[tauri::command]
pub async fn check_ollama() -> Result<DependencyStatus, String> {
    match reqwest::get("http://localhost:11434/api/version").await {
        Ok(response) => {
            let version = response.text().await.unwrap_or_default();
            Ok(DependencyStatus {
                name: "Ollama".to_string(),
                installed: true,
                version: Some(version),
                error_message: None,
            })
        }
        Err(_) => Ok(DependencyStatus {
            name: "Ollama".to_string(),
            installed: false,
            version: None,
            error_message: Some("Ollama not running. Please install from ollama.ai".to_string()),
        }),
    }
}

#[tauri::command]
pub async fn check_libreoffice() -> Result<DependencyStatus, String> {
    let paths = vec![
        "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        "/usr/bin/libreoffice",
    ];

    let found = paths.iter().any(|p| std::path::Path::new(p).exists());

    Ok(DependencyStatus {
        name: "LibreOffice".to_string(),
        installed: found,
        version: None,
        error_message: if !found {
            Some("LibreOffice not found. Document features will be disabled.".to_string())
        } else {
            None
        },
    })
}
