use crate::models::ollama::{ChatMessage, ChatRequest, Model};
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
#[tauri::command]
pub async fn chat_stream(
    app: AppHandle,
    ollama_url: String,
    request: ChatStreamRequest,
) -> Result<(), String> {
    let service = OllamaService::new(ollama_url);

    let chat_request = ChatRequest {
        model: request.model,
        messages: request.messages,
        stream: true,
        tools: None,
    };

    match service.chat_stream(chat_request).await {
        Ok(mut rx) => {
            // Spawn a task to handle the stream
            tokio::spawn(async move {
                while let Some(chunk_result) = rx.recv().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // Emit chunk to frontend
                            if let Err(e) = app.emit("ollama-stream-chunk", &chunk) {
                                log::error!("Failed to emit stream chunk: {}", e);
                                break;
                            }

                            // If done, break
                            if chunk.done {
                                break;
                            }
                        }
                        Err(e) => {
                            log::error!("Stream error: {}", e);
                            // Emit error to frontend
                            if let Err(emit_err) = app.emit("ollama-stream-error", format!("{}", e)) {
                                log::error!("Failed to emit error: {}", emit_err);
                            }
                            break;
                        }
                    }
                }
            });

            Ok(())
        }
        Err(e) => Err(format!("Failed to start chat stream: {}", e)),
    }
}

/// Check if Ollama is running
#[tauri::command]
pub async fn check_ollama_running(ollama_url: String) -> Result<bool, String> {
    let service = OllamaService::new(ollama_url);
    Ok(service.is_running().await)
}
