use crate::models::ollama::{ChatRequest, ChatResponse, ModelsResponse, StreamChunk};
use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Client;
use tokio::sync::mpsc;

pub struct OllamaService {
    client: Client,
    base_url: String,
}

impl OllamaService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<ModelsResponse> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list models: {}", response.status());
        }

        let models = response.json::<ModelsResponse>().await?;
        Ok(models)
    }

    /// Send a chat request without streaming (returns complete response)
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);
        let mut req = request.clone();
        req.stream = false;

        let response = self.client.post(&url).json(&req).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Ollama chat failed ({}): {}", status, body);
            anyhow::bail!("Chat request failed: {} - {}", status, body);
        }

        let chat_response = response.json::<ChatResponse>().await?;
        Ok(chat_response)
    }

    /// Stream chat responses token by token
    /// Returns a channel that yields StreamChunk items
    pub async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<mpsc::UnboundedReceiver<Result<StreamChunk>>> {
        let url = format!("{}/api/chat", self.base_url);
        let mut req = request.clone();
        req.stream = true;

        // Log the request for debugging
        if let Ok(json_str) = serde_json::to_string(&req) {
            log::info!("Ollama chat request (len={}): {}", json_str.len(), &json_str[..json_str.len().min(500)]);
        }
        log::info!("Sending request to {}", url);

        let response = self.client.post(&url).json(&req).send().await?;
        log::info!("Ollama responded with status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Ollama chat stream failed ({}): {}", status, body);
            anyhow::bail!("Chat stream request failed: {} - {}", status, body);
        }

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            let process_line = |line: &str, tx: &mpsc::UnboundedSender<Result<StreamChunk>>| -> bool {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return false; // continue processing
                }
                log::info!("Ollama chunk: {}", &trimmed[..trimmed.len().min(200)]);
                match serde_json::from_str::<StreamChunk>(trimmed) {
                    Ok(stream_chunk) => {
                        let done = stream_chunk.done;
                        if tx.send(Ok(stream_chunk)).is_err() {
                            return true; // receiver dropped, stop
                        }
                        done
                    }
                    Err(e) => {
                        log::error!("Failed to parse Ollama chunk: {} — raw: {}", e, &trimmed[..trimmed.len().min(500)]);
                        let _ = tx.send(Err(anyhow::anyhow!("Failed to parse stream chunk: {}", e)));
                        false
                    }
                }
            };

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(text) = std::str::from_utf8(&chunk) {
                            buffer.push_str(text);

                            while let Some(newline_pos) = buffer.find('\n') {
                                let line = buffer[..newline_pos].to_string();
                                buffer = buffer[newline_pos + 1..].to_string();
                                if process_line(&line, &tx) {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Ollama stream error: {}", e);
                        let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", e)));
                        return;
                    }
                }
            }

            // Process any remaining data in the buffer after stream ends
            if !buffer.trim().is_empty() {
                log::info!("Processing remaining buffer: {}", &buffer[..buffer.len().min(200)]);
                process_line(&buffer, &tx);
            }
        });

        Ok(rx)
    }

    /// Check if Ollama is running
    pub async fn is_running(&self) -> bool {
        let url = format!("{}/api/version", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }
}
