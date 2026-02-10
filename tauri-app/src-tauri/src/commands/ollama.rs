use crate::models::ollama::{ChatMessage, ChatRequest, Model, Tool};
use crate::services::ollama_service::OllamaService;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListModelsResponse {
    pub models: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatStreamRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

/// List available Ollama models
#[tauri::command]
pub async fn list_ollama_models(
    ollama_url: String,
) -> Result<ListModelsResponse, String> {
    let service = OllamaService::new(ollama_url);

    match service.list_models().await {
        Ok(response) => Ok(ListModelsResponse {
            models: response.models,
        }),
        Err(e) => Err(format!("Failed to list models: {}", e)),
    }
}

/// Start streaming chat with Ollama
/// Emits 'ollama-stream-chunk' events to the frontend
/// Returns immediately — all work happens in a background task
#[tauri::command]
pub async fn chat_stream(
    app: AppHandle,
    ollama_url: String,
    request: ChatStreamRequest,
) -> Result<(), String> {
    let chat_request = ChatRequest {
        model: request.model,
        messages: request.messages,
        stream: true,
        tools: request.tools,
    };

    // Spawn the entire HTTP request + streaming in a background task
    // so the frontend invoke returns immediately
    tokio::spawn(async move {
        let service = OllamaService::new(ollama_url);

        let mut rx = match service.chat_stream(chat_request).await {
            Ok(rx) => rx,
            Err(e) => {
                log::error!("Failed to start chat stream: {}", e);
                let _ = app.emit("ollama-stream-error", format!("{}", e));
                return;
            }
        };

        while let Some(chunk_result) = rx.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    let done = chunk.done;
                    if let Err(e) = app.emit("ollama-stream-chunk", &chunk) {
                        log::error!("Failed to emit stream chunk: {}", e);
                        break;
                    }
                    if done {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Stream error: {}", e);
                    let _ = app.emit("ollama-stream-error", format!("{}", e));
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Check if Ollama is running
#[tauri::command]
pub async fn check_ollama_running(ollama_url: String) -> Result<bool, String> {
    let service = OllamaService::new(ollama_url);
    Ok(service.is_running().await)
}
