use crate::models::ollama::{ChatMessage, Model, Tool, ToolCall};
use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct EngineGenerateRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<Tool>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct EngineStreamChunk {
    pub content: String,
    pub done: bool,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize)]
struct EngineMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct EngineGenerationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct EngineGeneratePayload {
    model: String,
    prompt: String,
    messages: Vec<EngineMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<EngineTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
    params: EngineGenerationParams,
}

#[derive(Debug, Serialize)]
struct EngineTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: EngineToolFunction,
}

#[derive(Debug, Serialize)]
struct EngineToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Default, Clone)]
struct ParsedStreamLine {
    content: String,
    done: bool,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_deltas: Vec<ToolCallDelta>,
}

#[derive(Debug, Default, Clone)]
struct ToolCallDelta {
    index: usize,
    name: Option<String>,
    arguments_chunk: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct PendingToolCall {
    name: Option<String>,
    arguments_buffer: String,
}

#[derive(Debug, Default, Clone)]
struct ToolCallAccumulator {
    pending: BTreeMap<usize, PendingToolCall>,
}

impl ToolCallAccumulator {
    fn absorb_complete_calls(&mut self, calls: &[ToolCall]) {
        for (index, call) in calls.iter().enumerate() {
            let entry = self.pending.entry(index).or_default();
            entry.name = Some(call.function.name.clone());
            entry.arguments_buffer = match &call.function.arguments {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            };
        }
    }

    fn absorb_deltas(&mut self, deltas: &[ToolCallDelta]) {
        for delta in deltas {
            let entry = self.pending.entry(delta.index).or_default();

            if let Some(name) = delta
                .name
                .as_ref()
                .map(|name| name.trim())
                .filter(|name| !name.is_empty())
            {
                entry.name = Some(name.to_string());
            }

            if let Some(arguments_chunk) = &delta.arguments_chunk {
                entry.arguments_buffer.push_str(arguments_chunk);
            }
        }
    }

    fn finalize(&self) -> Option<Vec<ToolCall>> {
        let mut calls = Vec::new();

        for pending in self.pending.values() {
            let Some(name) = pending
                .name
                .as_ref()
                .map(|name| name.trim())
                .filter(|name| !name.is_empty())
            else {
                continue;
            };

            let arguments = match parse_arguments_buffer(&pending.arguments_buffer) {
                Some(parsed) => parsed,
                None => continue,
            };

            calls.push(ToolCall {
                function: crate::models::ollama::ToolCallFunction {
                    name: name.to_string(),
                    arguments,
                },
            });
        }

        if calls.is_empty() {
            None
        } else {
            Some(calls)
        }
    }
}

pub struct SmolpcEngineService {
    client: Client,
    base_url: String,
}

impl SmolpcEngineService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(5))
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
        }
    }

    pub async fn is_running(&self) -> bool {
        let url = format!("{}/health", self.base_url.trim_end_matches('/'));
        self.client
            .get(url)
            .send()
            .await
            .map(|resp| resp.status().is_success())
            .unwrap_or(false)
    }

    pub async fn generate_stream(
        &self,
        request: EngineGenerateRequest,
    ) -> Result<mpsc::UnboundedReceiver<Result<EngineStreamChunk>>> {
        let messages = request
            .messages
            .iter()
            .map(|msg| EngineMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            })
            .collect::<Vec<_>>();

        let payload = EngineGeneratePayload {
            model: request.model,
            prompt: flatten_prompt(&request.messages),
            messages,
            tools: request.tools.as_ref().map(|tools| to_engine_tools(tools)),
            tool_choice: request.tools.as_ref().map(|_| "auto".to_string()),
            params: EngineGenerationParams {
                temperature: request.temperature,
                max_tokens: request.max_tokens,
            },
        };

        let url = format!("{}/generate", self.base_url.trim_end_matches('/'));
        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "smolpc-engine generate request failed: {} - {}",
                status,
                body
            );
        }

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut tool_accumulator = ToolCallAccumulator::default();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if let Ok(text) = std::str::from_utf8(&chunk) {
                            buffer.push_str(text);

                            while let Some(newline_pos) = buffer.find('\n') {
                                let line = buffer[..newline_pos].to_string();
                                buffer = buffer[newline_pos + 1..].to_string();

                                if let Some(parsed_line) = parse_stream_line(&line) {
                                    if let Some(calls) = parsed_line.tool_calls.as_ref() {
                                        tool_accumulator.absorb_complete_calls(calls);
                                    }
                                    tool_accumulator.absorb_deltas(&parsed_line.tool_call_deltas);

                                    let tool_calls = parsed_line.tool_calls.clone().or_else(|| {
                                        if parsed_line.done {
                                            tool_accumulator.finalize()
                                        } else {
                                            None
                                        }
                                    });

                                    let chunk = EngineStreamChunk {
                                        content: parsed_line.content,
                                        done: parsed_line.done,
                                        tool_calls,
                                    };

                                    let done = chunk.done;
                                    if tx.send(Ok(chunk)).is_err() {
                                        return;
                                    }
                                    if done {
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("smolpc-engine stream error: {}", e)));
                        return;
                    }
                }
            }

            if !buffer.trim().is_empty() {
                if let Some(parsed_line) = parse_stream_line(&buffer) {
                    if let Some(calls) = parsed_line.tool_calls.as_ref() {
                        tool_accumulator.absorb_complete_calls(calls);
                    }
                    tool_accumulator.absorb_deltas(&parsed_line.tool_call_deltas);

                    let tool_calls = parsed_line.tool_calls.or_else(|| {
                        if parsed_line.done {
                            tool_accumulator.finalize()
                        } else {
                            None
                        }
                    });

                    let chunk = EngineStreamChunk {
                        content: parsed_line.content,
                        done: parsed_line.done,
                        tool_calls,
                    };

                    let done = chunk.done;
                    if tx.send(Ok(chunk)).is_err() {
                        return;
                    }
                    if done {
                        return;
                    }
                }
            }

            let _ = tx.send(Ok(EngineStreamChunk {
                content: String::new(),
                done: true,
                tool_calls: tool_accumulator.finalize(),
            }));
        });

        Ok(rx)
    }

    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let url = format!("{}/models", self.base_url.trim_end_matches('/'));
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("smolpc-engine models request failed: {}", response.status());
        }

        let value = response.json::<Value>().await?;
        let models = parse_models_from_value(&value);

        if models.is_empty() {
            anyhow::bail!("smolpc-engine /models returned no usable model entries");
        }

        Ok(models)
    }
}

fn flatten_prompt(messages: &[ChatMessage]) -> String {
    messages
        .iter()
        .filter_map(|message| {
            if message.content.trim().is_empty() {
                None
            } else {
                Some(format!("{}: {}", message.role, message.content))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_engine_tools(tools: &[Tool]) -> Vec<EngineTool> {
    tools
        .iter()
        .map(|tool| EngineTool {
            tool_type: tool.tool_type.clone(),
            function: EngineToolFunction {
                name: tool.function.name.clone(),
                description: tool.function.description.clone(),
                parameters: tool.function.parameters.clone(),
            },
        })
        .collect()
}

fn parse_stream_line(line: &str) -> Option<ParsedStreamLine> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let payload = trimmed.strip_prefix("data:").unwrap_or(trimmed).trim();

    if payload.eq_ignore_ascii_case("[DONE]") {
        return Some(ParsedStreamLine {
            content: String::new(),
            done: true,
            tool_calls: None,
            tool_call_deltas: Vec::new(),
        });
    }

    match serde_json::from_str::<Value>(payload) {
        Ok(value) => {
            if let Some(error_message) = extract_error_message(&value) {
                return Some(ParsedStreamLine {
                    content: format!("[smolpc-engine error] {}", error_message),
                    done: true,
                    tool_calls: None,
                    tool_call_deltas: Vec::new(),
                });
            }

            let done = value.get("done").and_then(Value::as_bool).unwrap_or(false);
            let tool_calls = extract_tool_calls(&value);
            let tool_call_deltas = extract_tool_call_deltas(&value);

            if let Some(token) = extract_token_text(&value) {
                return Some(ParsedStreamLine {
                    content: token,
                    done,
                    tool_calls,
                    tool_call_deltas,
                });
            }

            if done || tool_calls.is_some() || !tool_call_deltas.is_empty() {
                return Some(ParsedStreamLine {
                    content: String::new(),
                    done,
                    tool_calls,
                    tool_call_deltas,
                });
            }

            None
        }
        Err(_) => Some(ParsedStreamLine {
            content: payload.to_string(),
            done: false,
            tool_calls: None,
            tool_call_deltas: Vec::new(),
        }),
    }
}

fn extract_error_message(value: &Value) -> Option<String> {
    if let Some(message) = value
        .get("error")
        .and_then(Value::as_str)
        .map(ToString::to_string)
    {
        return Some(message);
    }

    value
        .get("error")
        .and_then(|err| err.get("message"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn extract_token_text(value: &Value) -> Option<String> {
    if let Some(text) = value
        .get("token")
        .and_then(Value::as_str)
        .or_else(|| value.get("text").and_then(Value::as_str))
        .or_else(|| value.get("content").and_then(Value::as_str))
        .map(ToString::to_string)
    {
        return Some(text);
    }

    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| {
            choice
                .get("delta")
                .and_then(|delta| delta.get("content"))
                .and_then(Value::as_str)
                .or_else(|| choice.get("text").and_then(Value::as_str))
        })
        .map(ToString::to_string)
}

fn extract_tool_calls(value: &Value) -> Option<Vec<ToolCall>> {
    if let Some(parsed) = value.get("tool_calls").and_then(parse_tool_calls_value) {
        return Some(parsed);
    }

    if let Some(parsed) = value.get("toolCall").and_then(parse_tool_calls_value) {
        return Some(parsed);
    }

    if let Some(parsed) = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| {
            choice
                .get("delta")
                .and_then(|delta| delta.get("tool_calls"))
                .or_else(|| {
                    choice
                        .get("message")
                        .and_then(|message| message.get("tool_calls"))
                })
        })
        .and_then(parse_tool_calls_value)
    {
        return Some(parsed);
    }

    None
}

fn parse_tool_calls_value(value: &Value) -> Option<Vec<ToolCall>> {
    if let Ok(parsed) = serde_json::from_value::<Vec<ToolCall>>(value.clone()) {
        return Some(parsed);
    }

    if let Ok(single) = serde_json::from_value::<ToolCall>(value.clone()) {
        return Some(vec![single]);
    }

    None
}

fn extract_tool_call_deltas(value: &Value) -> Vec<ToolCallDelta> {
    if let Some(deltas) = value
        .get("tool_calls")
        .and_then(parse_tool_call_deltas_value)
    {
        return deltas;
    }

    if let Some(deltas) = value.get("toolCall").and_then(parse_tool_call_deltas_value) {
        return deltas;
    }

    if let Some(deltas) = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| {
            choice
                .get("delta")
                .and_then(|delta| {
                    delta
                        .get("tool_calls")
                        .or_else(|| delta.get("toolCall"))
                        .or_else(|| delta.get("function_call"))
                })
                .or_else(|| {
                    choice
                        .get("message")
                        .and_then(|message| message.get("tool_calls"))
                })
        })
        .and_then(parse_tool_call_deltas_value)
    {
        return deltas;
    }

    Vec::new()
}

fn parse_tool_call_deltas_value(value: &Value) -> Option<Vec<ToolCallDelta>> {
    if let Some(items) = value.as_array() {
        let deltas = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| parse_tool_call_delta_item(item, index))
            .collect::<Vec<_>>();

        if deltas.is_empty() {
            None
        } else {
            Some(deltas)
        }
    } else {
        parse_tool_call_delta_item(value, 0).map(|delta| vec![delta])
    }
}

fn parse_tool_call_delta_item(value: &Value, fallback_index: usize) -> Option<ToolCallDelta> {
    let index = value
        .get("index")
        .and_then(Value::as_u64)
        .map(|index| index as usize)
        .unwrap_or(fallback_index);

    let function = value.get("function").unwrap_or(value);

    let name = function
        .get("name")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let arguments_chunk = function
        .get("arguments")
        .or_else(|| value.get("arguments"))
        .and_then(|arguments| {
            arguments.as_str().map(ToString::to_string).or_else(|| {
                if arguments.is_object() || arguments.is_array() {
                    Some(arguments.to_string())
                } else {
                    None
                }
            })
        });

    if name.is_none() && arguments_chunk.is_none() {
        return None;
    }

    Some(ToolCallDelta {
        index,
        name,
        arguments_chunk,
    })
}

fn parse_arguments_buffer(buffer: &str) -> Option<Value> {
    let trimmed = buffer.trim();

    if trimmed.is_empty() {
        return Some(Value::Object(serde_json::Map::new()));
    }

    serde_json::from_str::<Value>(trimmed).ok()
}

fn parse_models_from_value(value: &Value) -> Vec<Model> {
    if let Some(array) = value.as_array() {
        return array.iter().filter_map(parse_model_item).collect();
    }

    if let Some(models) = value.get("models").and_then(Value::as_array) {
        return models.iter().filter_map(parse_model_item).collect();
    }

    if let Some(data) = value.get("data").and_then(Value::as_array) {
        return data.iter().filter_map(parse_model_item).collect();
    }

    parse_model_item(value)
        .map(|model| vec![model])
        .unwrap_or_default()
}

fn parse_model_item(value: &Value) -> Option<Model> {
    if let Some(name) = value.as_str() {
        return Some(Model {
            name: name.to_string(),
            modified_at: String::new(),
            size: 0,
        });
    }

    let name = value
        .get("name")
        .or_else(|| value.get("id"))
        .or_else(|| value.get("model"))
        .and_then(Value::as_str)?;

    let modified_at = value
        .get("modified_at")
        .or_else(|| value.get("updated_at"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let size = value.get("size").and_then(Value::as_u64).unwrap_or(0);

    Some(Model {
        name: name.to_string(),
        modified_at,
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_done_marker_line() {
        let chunk = parse_stream_line("data: [DONE]").expect("chunk expected");
        assert!(chunk.done);
        assert!(chunk.content.is_empty());
        assert!(chunk.tool_calls.is_none());
    }

    #[test]
    fn parses_token_payload() {
        let chunk = parse_stream_line(r#"{"token":"Hello","done":false}"#).expect("chunk expected");
        assert!(!chunk.done);
        assert_eq!(chunk.content, "Hello");
        assert!(chunk.tool_calls.is_none());
    }

    #[test]
    fn parses_top_level_tool_calls_array() {
        let chunk = parse_stream_line(
            r#"{"tool_calls":[{"function":{"name":"create_blank_document","arguments":{"filename":"test.odt"}}}]}"#,
        )
        .expect("chunk expected");

        let calls = chunk.tool_calls.expect("tool calls expected");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "create_blank_document");
        assert_eq!(calls[0].function.arguments["filename"], "test.odt");
    }

    #[test]
    fn parses_openai_style_choice_delta_tool_calls() {
        let chunk = parse_stream_line(
            r#"{"choices":[{"delta":{"tool_calls":[{"function":{"name":"add_text","arguments":{"file_path":"a.odt","text":"hi"}}}]}}]}"#,
        )
        .expect("chunk expected");

        let calls = chunk.tool_calls.expect("tool calls expected");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "add_text");
        assert_eq!(calls[0].function.arguments["text"], "hi");
    }

    #[test]
    fn extracts_tool_call_deltas_from_partial_openai_chunk() {
        let chunk = parse_stream_line(
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"add_text","arguments":"{\"file_path\":\"a.odt\",\"text\":\""}}]}}]}"#,
        )
        .expect("chunk expected");

        assert!(chunk.tool_calls.is_some());
        assert_eq!(chunk.tool_call_deltas.len(), 1);
        assert_eq!(chunk.tool_call_deltas[0].index, 0);
        assert_eq!(chunk.tool_call_deltas[0].name.as_deref(), Some("add_text"));
        assert_eq!(
            chunk.tool_call_deltas[0].arguments_chunk.as_deref(),
            Some("{\"file_path\":\"a.odt\",\"text\":\"")
        );
    }

    #[test]
    fn assembles_split_tool_call_arguments() {
        let mut accumulator = ToolCallAccumulator::default();

        let chunk_1 = parse_stream_line(
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"add_text","arguments":"{\"file_path\":\"a.odt\",\"text\":\""}}]}}]}"#,
        )
        .expect("chunk expected");
        accumulator.absorb_deltas(&chunk_1.tool_call_deltas);

        let chunk_2 = parse_stream_line(
            r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"hello\"}"}}]}}]}"#,
        )
        .expect("chunk expected");
        accumulator.absorb_deltas(&chunk_2.tool_call_deltas);

        let calls = accumulator
            .finalize()
            .expect("assembled tool calls expected");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "add_text");
        assert_eq!(calls[0].function.arguments["file_path"], "a.odt");
        assert_eq!(calls[0].function.arguments["text"], "hello");
    }

    #[test]
    fn drops_incomplete_tool_call_arguments() {
        let mut accumulator = ToolCallAccumulator::default();
        accumulator.absorb_deltas(&[ToolCallDelta {
            index: 0,
            name: Some("add_text".to_string()),
            arguments_chunk: Some("{\"file_path\":".to_string()),
        }]);

        assert!(accumulator.finalize().is_none());
    }

    #[test]
    fn parses_models_from_models_array() {
        let value = serde_json::json!({
            "models": [
                {"name": "model-a", "size": 123},
                {"id": "model-b", "updated_at": "2026-03-02T10:00:00Z"}
            ]
        });

        let models = parse_models_from_value(&value);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "model-a");
        assert_eq!(models[0].size, 123);
        assert_eq!(models[1].name, "model-b");
        assert_eq!(models[1].modified_at, "2026-03-02T10:00:00Z");
    }

    #[test]
    fn parses_models_from_string_array() {
        let value = serde_json::json!(["model-x", "model-y"]);
        let models = parse_models_from_value(&value);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "model-x");
        assert_eq!(models[1].name, "model-y");
    }
}
