use crate::models::config::AiProvider;
use crate::models::ollama::{ChatMessage, ChatRequest, Model, StreamChunk, Tool};
use crate::services::ollama_service::OllamaService;
use crate::services::smolpc_engine_service::{EngineGenerateRequest, SmolpcEngineService};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatStreamRequest {
    pub provider: AiProvider,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smolpc_engine_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAiModelsResponse {
    pub models: Vec<Model>,
}

#[tauri::command]
pub async fn list_ai_models(
    provider: AiProvider,
    ollama_url: Option<String>,
    smolpc_engine_url: Option<String>,
) -> Result<ListAiModelsResponse, String> {
    match provider {
        AiProvider::Ollama => {
            let service = OllamaService::new(
                ollama_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            );
            match service.list_models().await {
                Ok(response) => Ok(ListAiModelsResponse {
                    models: response.models,
                }),
                Err(e) => Err(format!("Failed to list Ollama models: {}", e)),
            }
        }
        AiProvider::SmolpcEngine => {
            let service = SmolpcEngineService::new(
                smolpc_engine_url.unwrap_or_else(|| "http://127.0.0.1:19432".to_string()),
            );
            match service.list_models().await {
                Ok(models) => Ok(ListAiModelsResponse { models }),
                Err(e) => Err(format!("Failed to list smolpc-engine models: {}", e)),
            }
        }
    }
}

#[tauri::command]
pub async fn chat_stream_ai(app: AppHandle, request: AiChatStreamRequest) -> Result<(), String> {
    match request.provider {
        AiProvider::Ollama => start_ollama_stream(app, request).await,
        AiProvider::SmolpcEngine => start_smolpc_engine_stream(app, request).await,
    }
}

async fn start_ollama_stream(app: AppHandle, request: AiChatStreamRequest) -> Result<(), String> {
    let ollama_url = request
        .ollama_url
        .unwrap_or_else(|| "http://localhost:11434".to_string());

    let chat_request = ChatRequest {
        model: request.model,
        messages: request.messages,
        stream: true,
        tools: request.tools,
    };

    tokio::spawn(async move {
        let service = OllamaService::new(ollama_url);

        let mut rx = match service.chat_stream(chat_request).await {
            Ok(rx) => rx,
            Err(e) => {
                log::error!("Failed to start Ollama stream: {}", e);
                let _ = app.emit("ai-stream-error", format!("{}", e));
                return;
            }
        };

        while let Some(chunk_result) = rx.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Err(e) = app.emit("ai-stream-chunk", &chunk) {
                        log::error!("Failed to emit AI stream chunk: {}", e);
                        break;
                    }
                    if chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Ollama stream error: {}", e);
                    let _ = app.emit("ai-stream-error", format!("{}", e));
                    break;
                }
            }
        }
    });

    Ok(())
}

async fn start_smolpc_engine_stream(
    app: AppHandle,
    request: AiChatStreamRequest,
) -> Result<(), String> {
    let engine_url = request
        .smolpc_engine_url
        .unwrap_or_else(|| "http://127.0.0.1:19432".to_string());

    tokio::spawn(async move {
        let service = SmolpcEngineService::new(engine_url);

        let mut rx = match service
            .generate_stream(EngineGenerateRequest {
                model: request.model.clone(),
                messages: request.messages,
                tools: request.tools,
                temperature: request.temperature,
                max_tokens: request.max_tokens,
            })
            .await
        {
            Ok(rx) => rx,
            Err(e) => {
                log::error!("Failed to start smolpc-engine stream: {}", e);
                let _ = app.emit("ai-stream-error", format!("{}", e));
                return;
            }
        };

        while let Some(chunk_result) = rx.recv().await {
            match chunk_result {
                Ok(chunk) => {
                    let stream_chunk = StreamChunk {
                        model: request.model.clone(),
                        message: ChatMessage {
                            role: "assistant".to_string(),
                            content: chunk.content,
                            tool_calls: chunk.tool_calls.clone(),
                        },
                        done: chunk.done,
                        tool_calls: chunk.tool_calls,
                    };

                    if let Err(e) = app.emit("ai-stream-chunk", &stream_chunk) {
                        log::error!("Failed to emit smolpc-engine chunk: {}", e);
                        break;
                    }

                    if stream_chunk.done {
                        break;
                    }
                }
                Err(e) => {
                    log::error!("smolpc-engine stream error: {}", e);
                    let _ = app.emit("ai-stream-error", format!("{}", e));
                    break;
                }
            }
        }
    });

    Ok(())
}
