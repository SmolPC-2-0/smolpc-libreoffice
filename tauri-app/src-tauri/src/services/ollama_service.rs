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
            client: Client::new(),
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
            anyhow::bail!("Chat request failed: {}", response.status());
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

        let response = self.client.post(&url).json(&req).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Chat stream request failed: {}", response.status());
        }

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();

            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Convert bytes to string and add to buffer
                        if let Ok(text) = std::str::from_utf8(&chunk) {
                            buffer.push_str(text);

                            // Process complete JSON objects (separated by newlines)
                            while let Some(newline_pos) = buffer.find('\n') {
                                let line = buffer[..newline_pos].trim().to_string();
                                buffer = buffer[newline_pos + 1..].to_string();

                                if line.is_empty() {
                                    continue;
                                }

                                // Try to parse as StreamChunk
                                match serde_json::from_str::<StreamChunk>(&line) {
                                    Ok(stream_chunk) => {
                                        if tx.send(Ok(stream_chunk.clone())).is_err() {
                                            // Receiver dropped, stop streaming
                                            return;
                                        }

                                        // If done, stop streaming
                                        if stream_chunk.done {
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(anyhow::anyhow!(
                                            "Failed to parse stream chunk: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", e)));
                        return;
                    }
                }
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
