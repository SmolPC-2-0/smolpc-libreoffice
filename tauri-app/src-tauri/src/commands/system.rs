use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let version = if stdout.trim().is_empty() {
                    stderr
                } else {
                    stdout
                };
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
pub async fn check_ollama(ollama_url: Option<String>) -> Result<DependencyStatus, String> {
    let base_url = ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string());
    let version_url = format!("{}/api/version", base_url.trim_end_matches('/'));

    match reqwest::get(version_url).await {
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
pub async fn check_smolpc_engine(engine_url: Option<String>) -> Result<DependencyStatus, String> {
    let base_url = engine_url.unwrap_or_else(|| "http://127.0.0.1:19432".to_string());
    let token = discover_engine_token();
    let client = reqwest::Client::new();

    let shared_health_url = format!("{}/engine/health", base_url.trim_end_matches('/'));
    let mut shared_request = client.get(&shared_health_url);
    if let Some(token) = token.as_ref() {
        shared_request = shared_request.bearer_auth(token);
    }

    match shared_request.send().await {
        Ok(response) if response.status().is_success() => {
            let body = response.text().await.unwrap_or_default();
            return Ok(DependencyStatus {
                name: "SmolPC Engine".to_string(),
                installed: true,
                version: if body.trim().is_empty() {
                    Some("shared-engine-v1".to_string())
                } else {
                    Some(format!("shared-engine-v1 {}", body.trim()))
                },
                error_message: None,
            });
        }
        Ok(response)
            if response.status() == reqwest::StatusCode::UNAUTHORIZED
                || response.status() == reqwest::StatusCode::FORBIDDEN =>
        {
            return Ok(DependencyStatus {
                name: "SmolPC Engine".to_string(),
                installed: false,
                version: None,
                error_message: Some(
                    "smolpc-engine host requires bearer token. Set SMOLPC_ENGINE_TOKEN or ensure %LOCALAPPDATA%\\SmolPC\\engine-runtime\\engine-token.txt exists."
                        .to_string(),
                ),
            });
        }
        Ok(response)
            if response.status() != reqwest::StatusCode::NOT_FOUND
                && response.status() != reqwest::StatusCode::METHOD_NOT_ALLOWED =>
        {
            return Ok(DependencyStatus {
                name: "SmolPC Engine".to_string(),
                installed: false,
                version: None,
                error_message: Some(format!(
                    "smolpc-engine health check failed at /engine/health: {}",
                    response.status()
                )),
            });
        }
        _ => {}
    }

    let legacy_health_url = format!("{}/health", base_url.trim_end_matches('/'));
    let mut legacy_request = client.get(&legacy_health_url);
    if let Some(token) = token.as_ref() {
        legacy_request = legacy_request.bearer_auth(token);
    }

    match legacy_request.send().await {
        Ok(response) if response.status().is_success() => {
            let body = response.text().await.unwrap_or_default();
            Ok(DependencyStatus {
                name: "SmolPC Engine".to_string(),
                installed: true,
                version: if body.trim().is_empty() {
                    Some("legacy-preview".to_string())
                } else {
                    Some(format!("legacy-preview {}", body.trim()))
                },
                error_message: None,
            })
        }
        _ => Ok(DependencyStatus {
            name: "SmolPC Engine".to_string(),
            installed: false,
            version: None,
            error_message: Some(
                "smolpc-engine daemon not running. Expected shared-engine-v1 at /engine/health (default http://127.0.0.1:19432).".to_string(),
            ),
        }),
    }
}

fn discover_engine_token() -> Option<String> {
    if let Some(token) = std::env::var("SMOLPC_ENGINE_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
    {
        return Some(token);
    }

    let token_path = default_engine_token_path()?;
    let token = std::fs::read_to_string(token_path).ok()?;
    let trimmed = token.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn default_engine_token_path() -> Option<PathBuf> {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return Some(
            PathBuf::from(local_app_data)
                .join("SmolPC")
                .join("engine-runtime")
                .join("engine-token.txt"),
        );
    }

    dirs::data_local_dir().map(|path| {
        path.join("SmolPC")
            .join("engine-runtime")
            .join("engine-token.txt")
    })
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
