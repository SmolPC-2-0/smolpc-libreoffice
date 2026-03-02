# Migration Plan: LibreOffice AI to Tauri 2.0 + Svelte 5 + Rust

## Executive Summary

Migrate the Windows-only LibreOfficeAI desktop application (C#/WinUI) to a cross-platform Tauri 2.0 application with Svelte 5 frontend and Rust backend, while preserving the existing Python MCP server for LibreOffice integration.

**Key Architecture Decision:** Hybrid approach - Rewrite the desktop app layer in Tauri/Rust/Svelte while keeping the battle-tested Python helper.py for UNO bridge operations.

## Implementation Update (March 2, 2026)

Initial engine migration work has started in `tauri-app`:

- Added provider-aware runtime config (`ai_provider`, `smolpc_engine_url`) with backward-compatible defaults
- Added unified AI stream command (`chat_stream_ai`) that routes to:
  - `OllamaService` (existing full tool-calling path)
  - `SmolpcEngineService` (preview path using `/health` + `/generate`)
- Updated frontend stores/components to use provider-agnostic stream events (`ai-stream-chunk`, `ai-stream-error`)
- Updated dependency initialization to validate selected provider (Ollama or smolpc-engine)
- Updated settings UI for provider selection and engine URL configuration
- Added initial smolpc-engine tool interoperability:
  - stream payload tool call extraction (`tool_calls` where available)
  - fallback JSON tool call extraction from assistant output for MCP execution
- Added explicit smolpc-engine tool request contract in generation payload:
  - includes `tools` and `tool_choice: auto` fields for compatible runtimes
- Added native streamed tool-call delta handling in `SmolpcEngineService`:
  - accumulates split `tool_calls[].function.arguments` fragments across stream chunks
  - finalizes merged tool calls on stream completion before MCP execution
  - covered with parser/accumulator unit tests for partial and invalid payloads
- Added provider-agnostic model discovery command (`list_ai_models`) with flexible parsing for smolpc-engine `/models` responses
- Saving settings now re-runs dependency and MCP initialization with updated provider settings (no app restart required)
- Added tool-call safety guards (per-response cap + recursion-depth cap + unknown tool filtering)

Current limitation in this phase:

- End-to-end document workflows on `smolpc_engine` still need full Windows validation against real daemon/runtime variants.

---

## Current Architecture Analysis

### Application Stack
- **Frontend:** C# WinUI 3 (XAML-based, Windows-only)
- **Backend Services:** C# (.NET 8.0) with MVVM pattern
- **AI Engine:** Embedded Ollama (local model runtime)
- **Voice Input:** Whisper.net (local speech-to-text)
- **MCP Server:** Python executables (main.exe, libre.exe)
- **LibreOffice Integration:** Python UNO bridge (helper.py)

### Critical Services to Migrate

1. **OllamaService** - Process lifecycle, HTTP API communication
2. **ChatService** - Message orchestration, tool calling, streaming
3. **ToolService** - MCP server connection, tool discovery
4. **DocumentService** - File scanning, document tracking
5. **AudioService** - Microphone recording (Windows MediaCapture)
6. **WhisperService** - Local audio transcription
7. **ConfigurationService** - Settings management

### Key Integration Points

- **Ollama:** HTTP API on `localhost:11434` (5-min timeout)
- **MCP Server:** Spawned process communicating via stdio
- **Helper.py:** Socket server on port 8765 (JSON protocol)
- **LibreOffice:** Headless UNO socket on port 2002
- **File System:** Document scanning, path management

---

## Migration Architecture

### Technology Stack

```
┌─────────────────────────────────────────┐
│     Frontend: Svelte 5 (TypeScript)     │
│  - Runes for reactive state             │
│  - Chat UI components                   │
│  - Voice recording controls             │
│  - Settings & help pages                │
└────────────────┬────────────────────────┘
                 │ @tauri-apps/api
                 │ invoke(), listen()
┌────────────────▼────────────────────────┐
│      Backend: Tauri 2.0 (Rust)          │
│  - Process management                   │
│  - HTTP client (Ollama)                 │
│  - Audio capture (cpal)                 │
│  - MCP protocol handler                 │
│  - File system operations               │
│  - State management                     │
└────────────────┬────────────────────────┘
                 │ spawns & manages
        ┌────────┴────────┬──────────────┐
        │                 │              │
┌───────▼────────┐ ┌──────▼──────┐ ┌────▼─────────┐
│ Ollama Process │ │ Python MCP  │ │  LibreOffice │
│                │ │   Server    │ │  (headless)  │
│ HTTP: 11434    │ │ stdio/8765  │ │  UNO: 2002   │
└────────────────┘ └──────┬──────┘ └──────────────┘
                          │
                   ┌──────▼──────┐
                   │  helper.py  │
                   │ UNO Bridge  │
                   └─────────────┘
```

### Core Dependencies (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2.0", features = ["process", "fs-write-all", "fs-read-dir", "path-all"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"

# Audio support
cpal = "0.15"
hound = "3.5"

# MCP protocol
serde_jsonrpc = "0.1"  # or implement custom
async-trait = "0.1"

# State management
parking_lot = "0.12"
dashmap = "6"
```

---

## Implementation Plan

### Phase 1: Project Setup & Infrastructure

#### 1.1 Initialize Tauri Project
```bash
npm create tauri-app@latest
# Select:
# - Template: Svelte + TypeScript
# - Package manager: npm/pnpm
# - Tauri version: 2.0
```

#### 1.2 Configure Rust Backend Structure
```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
└── src/
    ├── main.rs
    ├── lib.rs
    ├── commands/          # Tauri commands
    │   ├── mod.rs
    │   ├── ollama.rs      # Ollama integration
    │   ├── chat.rs        # Chat orchestration
    │   ├── audio.rs       # Audio recording
    │   ├── mcp.rs         # MCP server management
    │   ├── documents.rs   # File operations
    │   └── config.rs      # Settings management
    ├── services/          # Core services
    │   ├── mod.rs
    │   ├── ollama_service.rs
    │   ├── process_manager.rs
    │   ├── mcp_client.rs
    │   └── audio_service.rs
    ├── models/            # Data structures
    │   ├── mod.rs
    │   ├── chat.rs        # ChatMessage, etc.
    │   ├── config.rs      # Settings struct
    │   └── mcp.rs         # MCP types
    └── utils/
        ├── mod.rs
        └── paths.rs       # Path resolution
```

#### 1.3 Configure Frontend Structure
```
src/
├── lib/
│   ├── components/
│   │   ├── Chat/
│   │   │   ├── ChatMessage.svelte
│   │   │   ├── ChatInput.svelte
│   │   │   ├── ToolCallDisplay.svelte
│   │   │   └── ThinkingIndicator.svelte
│   │   ├── VoiceRecorder.svelte
│   │   ├── DocumentList.svelte
│   │   └── LoadingScreen.svelte
│   ├── stores/
│   │   ├── chat.svelte.ts       # Runes-based state
│   │   ├── config.svelte.ts
│   │   ├── audio.svelte.ts
│   │   └── app.svelte.ts
│   ├── types/
│   │   ├── chat.ts
│   │   ├── config.ts
│   │   └── mcp.ts
│   └── utils/
│       ├── tauri.ts             # Tauri API wrappers
│       └── formatting.ts
├── routes/
│   ├── +page.svelte             # Main chat page
│   ├── settings/+page.svelte
│   └── help/+page.svelte
└── app.css
```

### Phase 2: Core Service Migration

#### 2.1 Process Management Service

**File:** `src-tauri/src/services/process_manager.rs`

```rust
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

pub struct ProcessManager {
    ollama_process: Arc<RwLock<Option<Child>>>,
    mcp_process: Arc<RwLock<Option<Child>>>,
    libreoffice_process: Arc<RwLock<Option<Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            ollama_process: Arc::new(RwLock::new(None)),
            mcp_process: Arc::new(RwLock::new(None)),
            libreoffice_process: Arc::new(RwLock::new(None)),
        }
    }

    pub fn start_ollama(&self, ollama_path: &str, models_dir: &str) -> Result<()> {
        let child = Command::new(ollama_path)
            .arg("serve")
            .env("OLLAMA_MODELS", models_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        *self.ollama_process.write() = Some(child);
        Ok(())
    }

    pub fn start_mcp_server(&self, python_path: &str, script_path: &str) -> Result<()> {
        let child = Command::new(python_path)
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        *self.mcp_process.write() = Some(child);
        Ok(())
    }

    pub fn cleanup(&self) {
        // Kill all processes on shutdown
        if let Some(mut process) = self.ollama_process.write().take() {
            let _ = process.kill();
        }
        if let Some(mut process) = self.mcp_process.write().take() {
            let _ = process.kill();
        }
        if let Some(mut process) = self.libreoffice_process.write().take() {
            let _ = process.kill();
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
```

**Tauri Command:** `src-tauri/src/commands/mcp.rs`

```rust
#[tauri::command]
pub async fn start_mcp_server(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let config = state.config.read();
    let python_path = config.python_path.clone();
    let script_path = app_handle.path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("mcp_server/main.py");

    state.process_manager
        .start_mcp_server(&python_path, &script_path.to_string_lossy())
        .map_err(|e| e.to_string())?;

    Ok("MCP server started".to_string())
}
```

#### 2.2 Ollama HTTP Client Service

**File:** `src-tauri/src/services/ollama_service.rs`

```rust
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

pub struct OllamaService {
    client: Client,
    base_url: String,
}

impl OllamaService {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes
            .build()
            .unwrap();

        Self { client, base_url }
    }

    pub async fn generate(&self, request: OllamaRequest) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let result: OllamaResponse = response.json().await?;
        Ok(result.response)
    }

    pub async fn stream_generate(
        &self,
        request: OllamaRequest,
        callback: impl Fn(String) -> (),
    ) -> Result<()> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let mut stream = response.bytes_stream();
        use futures::stream::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            callback(text.to_string());
        }

        Ok(())
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        // Parse and return model list
        Ok(vec![])
    }

    pub async fn pull_model(&self, model_name: String) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);
        self.client
            .post(&url)
            .json(&json!({ "name": model_name }))
            .send()
            .await?;
        Ok(())
    }
}
```

**Tauri Commands:** `src-tauri/src/commands/ollama.rs`

```rust
#[tauri::command]
pub async fn query_ollama(
    state: tauri::State<'_, AppState>,
    model: String,
    prompt: String,
    tools: Option<Vec<serde_json::Value>>,
) -> Result<String, String> {
    let ollama = state.ollama_service.lock();

    let request = OllamaRequest {
        model,
        prompt,
        stream: false,
        tools,
    };

    ollama.generate(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stream_ollama(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    model: String,
    prompt: String,
) -> Result<(), String> {
    let ollama = state.ollama_service.lock().clone();

    let request = OllamaRequest {
        model,
        prompt,
        stream: true,
        tools: None,
    };

    ollama.stream_generate(request, |chunk| {
        app_handle.emit("ollama-token", chunk).ok();
    })
    .await
    .map_err(|e| e.to_string())
}
```

#### 2.3 Audio Recording Service

**File:** `src-tauri/src/services/audio_service.rs`

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use hound::{WavWriter, WavSpec};
use anyhow::Result;

pub struct AudioService {
    device: Device,
    config: StreamConfig,
    recording_data: Arc<Mutex<Vec<f32>>>,
    stream: Arc<Mutex<Option<Stream>>>,
}

impl AudioService {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device available"))?;

        let config = device.default_input_config()?.into();

        Ok(Self {
            device,
            config,
            recording_data: Arc::new(Mutex::new(Vec::new())),
            stream: Arc::new(Mutex::new(None)),
        })
    }

    pub fn start_recording(&self) -> Result<()> {
        let data_clone = Arc::clone(&self.recording_data);

        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                data_clone.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )?;

        stream.play()?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<Vec<f32>> {
        *self.stream.lock().unwrap() = None;
        let data = self.recording_data.lock().unwrap().clone();
        self.recording_data.lock().unwrap().clear();
        Ok(data)
    }

    pub fn save_wav(&self, data: &[f32], path: &str) -> Result<()> {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec)?;

        for sample in data {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }

        writer.finalize()?;
        Ok(())
    }
}
```

**Tauri Commands:** `src-tauri/src/commands/audio.rs`

```rust
#[tauri::command]
pub async fn start_recording(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.audio_service
        .start_recording()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let data = state.audio_service
        .stop_recording()
        .map_err(|e| e.to_string())?;

    let temp_dir = app_handle.path().temp_dir()
        .map_err(|e| e.to_string())?;
    let audio_path = temp_dir.join("recording.wav");

    state.audio_service
        .save_wav(&data, &audio_path.to_string_lossy())
        .map_err(|e| e.to_string())?;

    Ok(audio_path.to_string_lossy().to_string())
}
```

#### 2.4 MCP Client Implementation

**File:** `src-tauri/src/services/mcp_client.rs`

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Child, ChildStdin, ChildStdout};
use std::io::{BufRead, BufReader, Write};
use anyhow::Result;

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: u64,
}

pub struct McpClient {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpClient {
    pub fn new(process: &mut Child) -> Result<Self> {
        let stdin = process.stdin.take()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin"))?;
        let stdout = BufReader::new(
            process.stdout.take()
                .ok_or_else(|| anyhow::anyhow!("Failed to get stdout"))?
        );

        Ok(Self {
            stdin,
            stdout,
            next_id: 1,
        })
    }

    pub fn call_tool(&mut self, tool_name: String, args: Value) -> Result<Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": tool_name,
                "arguments": args
            }),
            id: self.next_id,
        };

        self.next_id += 1;

        // Send request
        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;

        // Read response
        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;

        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

        if let Some(error) = response.error {
            anyhow::bail!("MCP error: {:?}", error);
        }

        Ok(response.result.unwrap_or(Value::Null))
    }

    pub fn list_tools(&mut self) -> Result<Vec<String>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: Value::Null,
            id: self.next_id,
        };

        self.next_id += 1;

        let request_str = serde_json::to_string(&request)?;
        writeln!(self.stdin, "{}", request_str)?;
        self.stdin.flush()?;

        let mut response_line = String::new();
        self.stdout.read_line(&mut response_line)?;

        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;
        // Parse and return tool names
        Ok(vec![])
    }
}
```

### Phase 3: Frontend Migration (Svelte 5)

#### 3.1 State Management with Runes

**File:** `src/lib/stores/chat.svelte.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  toolCalls?: string[];
  isThinking?: boolean;
}

class ChatStore {
  messages = $state<ChatMessage[]>([]);
  isGenerating = $state(false);
  currentInput = $state('');

  get messageCount() {
    return $derived(this.messages.length);
  }

  async sendMessage(prompt: string) {
    this.messages.push({
      role: 'user',
      content: prompt
    });

    this.isGenerating = true;

    try {
      // Start streaming
      await invoke('stream_ollama', {
        model: 'mistral',
        prompt: prompt
      });

      // Listen for tokens
      const aiMessage: ChatMessage = {
        role: 'assistant',
        content: ''
      };
      this.messages.push(aiMessage);

      await listen('ollama-token', (event) => {
        aiMessage.content += event.payload;
      });

    } finally {
      this.isGenerating = false;
    }
  }

  clearMessages() {
    this.messages = [];
  }
}

export const chatStore = new ChatStore();
```

**File:** `src/lib/stores/app.svelte.ts`

```typescript
import { invoke } from '@tauri-apps/api/core';

class AppStore {
  ollamaReady = $state(false);
  toolsLoaded = $state(false);
  ollamaStatus = $state('Initializing...');
  toolsStatus = $state('Loading tools...');

  get appLoaded() {
    return $derived(this.ollamaReady && this.toolsLoaded);
  }

  async initialize() {
    // Start MCP server
    try {
      await invoke('start_mcp_server');
      this.toolsLoaded = true;
      this.toolsStatus = 'Tools loaded';
    } catch (err) {
      this.toolsStatus = `Error: ${err}`;
    }

    // Check Ollama status
    try {
      await invoke('check_ollama_status');
      this.ollamaReady = true;
      this.ollamaStatus = 'Ready';
    } catch (err) {
      this.ollamaStatus = `Error: ${err}`;
    }
  }
}

export const appStore = new AppStore();
```

#### 3.2 Main Chat Component

**File:** `src/routes/+page.svelte`

```svelte
<script lang="ts">
  import { chatStore } from '$lib/stores/chat.svelte';
  import { appStore } from '$lib/stores/app.svelte';
  import ChatMessage from '$lib/components/Chat/ChatMessage.svelte';
  import ChatInput from '$lib/components/Chat/ChatInput.svelte';
  import LoadingScreen from '$lib/components/LoadingScreen.svelte';
  import { onMount } from 'svelte';

  let chatContainer: HTMLDivElement;

  onMount(async () => {
    await appStore.initialize();
  });

  $effect(() => {
    // Auto-scroll to bottom when new messages arrive
    if (chatContainer) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }
  });

  async function handleSend(message: string) {
    await chatStore.sendMessage(message);
  }
</script>

{#if !appStore.appLoaded}
  <LoadingScreen
    ollamaStatus={appStore.ollamaStatus}
    toolsStatus={appStore.toolsStatus}
  />
{:else}
  <div class="app">
    <header>
      <button onclick={() => chatStore.clearMessages()}>New Chat</button>
      <a href="/settings">Settings</a>
      <a href="/help">Help</a>
    </header>

    <div class="chat-container" bind:this={chatContainer}>
      {#each chatStore.messages as message}
        <ChatMessage {message} />
      {/each}
    </div>

    <ChatInput
      onSend={handleSend}
      disabled={chatStore.isGenerating}
    />
  </div>
{/if}
```

### Phase 4: Configuration & Bundle Management

#### 4.1 Tauri Configuration

**File:** `src-tauri/tauri.conf.json`

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "dmg", "deb"],
    "resources": [
      "mcp_server/**/*",
      "ollama/**/*"
    ],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "permissions": [
    "fs:read-all",
    "fs:write-all",
    "process:allow-spawn",
    "path:allow-all"
  ]
}
```

#### 4.2 Python MCP Server Integration

**Strategy:** Keep existing Python files, bundle them as resources

```
src-tauri/
└── resources/
    └── mcp_server/
        ├── main.py
        ├── libre.py
        ├── helper.py
        ├── helper_utils.py
        └── helper_test_functions.py
```

**Access in Rust:**
```rust
let mcp_path = app_handle.path()
    .resource_dir()?
    .join("mcp_server/main.py");
```

### Phase 5: Whisper Integration

**Option 1:** Use Rust Whisper bindings
- `whisper-rs` crate (bindings to whisper.cpp)
- Bundle ggml models as resources
- Implement in Rust for better performance

**Option 2:** Keep Python subprocess
- Call Python Whisper library
- Simpler migration, proven approach

**Recommended:** Option 1 for better integration

**File:** `src-tauri/src/services/whisper_service.rs`

```rust
// Using whisper-rs crate
use whisper_rs::{WhisperContext, WhisperParams};
use anyhow::Result;

pub struct WhisperService {
    context: WhisperContext,
}

impl WhisperService {
    pub fn new(model_path: &str) -> Result<Self> {
        let context = WhisperContext::new(model_path)?;
        Ok(Self { context })
    }

    pub fn transcribe(&self, audio_path: &str) -> Result<String> {
        let params = WhisperParams::default();
        let result = self.context.full(params, audio_path)?;
        Ok(result.text)
    }
}
```

---

## Key Migration Challenges & Solutions

### Challenge 1: Cross-Platform Audio Recording
**Current:** Windows MediaCapture API
**Solution:** Use `cpal` crate (cross-platform)
**Trade-off:** More low-level, requires manual WAV encoding

### Challenge 2: MCP Protocol in Rust
**Current:** OllamaSharp.ModelContextProtocol library
**Solution:** Implement custom MCP client using stdio/JSON-RPC
**Trade-off:** More code to maintain, but full control

### Challenge 3: Process Lifecycle Management
**Current:** .NET process management
**Solution:** Tokio async process spawning with cleanup handlers
**Trade-off:** Need to handle SIGTERM/SIGINT correctly on all platforms

### Challenge 4: Observable State Pattern
**Current:** INotifyPropertyChanged, ObservableCollection
**Solution:** Svelte 5 runes + Tauri event system
**Trade-off:** Different paradigm, but more modern and reactive

### Challenge 5: Python UNO Bridge
**Current:** Bundled as .exe (Nuitka)
**Solution:** Keep as Python files, ensure Python runtime available
**Trade-off:** Require Python installation or bundle Python runtime

---

## Python Runtime Strategy

### Option A: Bundle Python (Recommended)
- Include Python 3.12 runtime in Tauri resources
- Cross-platform Python installers
- App is fully self-contained

### Option B: Require System Python
- Check for Python 3.12+ at startup
- Simpler bundle, smaller app size
- Risk: User may not have Python

### Option C: Compile to Binary
- Continue using Nuitka/PyInstaller
- Bundle compiled executables
- No Python dependency

**Recommendation:** Option A for best user experience

---

## Migration Phases Timeline

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Initialize Tauri project
- [ ] Set up Rust project structure
- [ ] Implement ProcessManager
- [ ] Bundle Python MCP server as resource
- [ ] Test process spawning on all platforms

### Phase 2: Service Layer (Week 3-4)
- [ ] Implement OllamaService (HTTP client)
- [ ] Implement MCP client (stdio communication)
- [ ] Implement AudioService (cpal)
- [ ] Implement ConfigurationService
- [ ] Implement DocumentService

### Phase 3: Frontend (Week 5-6)
- [ ] Create Svelte 5 stores (runes-based)
- [ ] Build chat interface
- [ ] Build voice recording UI
- [ ] Build settings page
- [ ] Build help page

### Phase 4: Integration (Week 7)
- [ ] Wire up Tauri commands to services
- [ ] Implement event system for streaming
- [ ] Add tool calling integration
- [ ] Test end-to-end workflows

### Phase 5: Whisper Integration (Week 8)
- [ ] Integrate whisper-rs
- [ ] Bundle GGML models
- [ ] Test transcription accuracy
- [ ] Optimize for performance

### Phase 6: Testing & Polish (Week 9-10)
- [ ] Cross-platform testing (Windows, macOS, Linux)
- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] UI/UX polish
- [ ] Documentation

---

## Critical Files Reference

### From Current Codebase (To Reference)
1. `LibreOfficeAI/Services/OllamaService.cs` - Ollama integration patterns
2. `LibreOfficeAI/Services/ChatService.cs` - Chat orchestration logic
3. `LibreOfficeAI/Services/ToolService.cs` - MCP integration
4. `LibreOfficeAI/SystemPrompt.txt` - AI system prompt
5. `LibreOfficeAI/IntentPrompt.txt` - Tool selection prompt
6. `libre-office-mcp/libre-writer/main.py` - Process launcher logic
7. `libre-office-mcp/libre-writer/libre.py` - MCP tool definitions
8. `libre-office-mcp/libre-writer/helper.py` - UNO bridge operations

### New Files to Create
1. `src-tauri/src/services/ollama_service.rs`
2. `src-tauri/src/services/process_manager.rs`
3. `src-tauri/src/services/mcp_client.rs`
4. `src-tauri/src/services/audio_service.rs`
5. `src/lib/stores/chat.svelte.ts`
6. `src/lib/stores/app.svelte.ts`
7. `src/routes/+page.svelte`

---

## Design Decisions (Finalized)

Based on requirements discussion, the following decisions have been made:

### 1. Platform Strategy
- **Primary:** Windows (production target)
- **Secondary:** macOS (development environment)
- **Future:** Linux support (Phase 2)

### 2. Python Runtime - System Python Required
**Decision:** Check for Python 3.12+ at startup, provide helpful error if missing

**Implementation:**
```rust
async fn check_python_version() -> Result<String, String> {
    let output = Command::new("python3")
        .arg("--version")
        .output()
        .map_err(|_| "Python not found. Please install Python 3.12+ from python.org")?;

    let version = String::from_utf8_lossy(&output.stdout);
    // Parse and validate version >= 3.12
    Ok(version.to_string())
}
```

**Rationale:**
- Keeps bundle size small (~50MB vs ~200MB)
- Most dev environments have Python
- Future: Add bundled Python option

### 3. Ollama Management - Pre-installed Assumption
**Decision:** Assume Ollama is installed, show friendly error with download link if not

**Implementation:**
```rust
async fn check_ollama_installed() -> Result<(), String> {
    match reqwest::get("http://localhost:11434/api/version").await {
        Ok(_) => Ok(()),
        Err(_) => Err(
            "Ollama not found. Please install from https://ollama.ai/download"
                .to_string()
        )
    }
}
```

**Error UI:** Modal with "Install Ollama" button linking to download page

**Rationale:**
- Ollama is 500MB-1GB (too large to bundle in MVP)
- One-time user setup acceptable
- Future: Add auto-installer

### 4. LibreOffice Detection - Graceful Degradation
**Decision:** Show friendly warning, app continues without LibreOffice features

**Implementation:**
```rust
async fn check_libreoffice() -> bool {
    // Try to find LibreOffice executable
    let paths = vec![
        "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        "/usr/bin/libreoffice"
    ];

    paths.iter().any(|p| std::path::Path::new(p).exists())
}
```

**UI Behavior:**
- If missing: Yellow banner "LibreOffice not installed. Document features disabled. [Download](link)"
- Chat still works for testing/basic use
- Tool calls fail gracefully with helpful message

**Rationale:**
- LibreOffice is ~300MB (too large for MVP)
- App has value even without document features (chat testing)
- Future: One-click installer

### 5. Whisper Models - Download on First Use
**Decision:** Download model (150MB) when user first uses voice recording

**Implementation:**
```rust
async fn ensure_whisper_model(app_handle: &AppHandle) -> Result<PathBuf> {
    let model_dir = app_handle.path().app_data_dir()?;
    let model_path = model_dir.join("models/ggml-base.bin");

    if !model_path.exists() {
        // Show progress dialog
        app_handle.emit("whisper-download-start", ())?;

        download_file(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
            &model_path,
            |progress| {
                app_handle.emit("whisper-download-progress", progress).ok();
            }
        ).await?;

        app_handle.emit("whisper-download-complete", ())?;
    }

    Ok(model_path)
}
```

**Rationale:**
- Keeps initial download under 100MB
- Users who don't use voice don't download model
- Better UX than large upfront download

### 6. UI Design - Modern Minimal (shadcn-svelte)
**Decision:** Build modern chat interface with shadcn-svelte components

**Layout:**
```
┌─────────────────────────────────────────────────────┐
│  LibreOffice AI                    Settings   Help  │
├──────────┬──────────────────────────────────────────┤
│  Tools   │  Chat Area                               │
│          │  ┌────────────────────────────────────┐  │
│ Document │  │ User: Create a document...         │  │
│ Creation │  │                                    │  │
│          │  │ Assistant: I'll help you create    │  │
│ Document │  │ a document. [Tool: create_blank]   │  │
│ Editing  │  │                                    │  │
│          │  └────────────────────────────────────┘  │
│ Presenta-│                                          │
│ tions    │  ┌────────────────────────────────────┐  │
│          │  │ Type your message...          [🎤] │  │
│ General  │  └────────────────────────────────────┘  │
└──────────┴──────────────────────────────────────────┘
```

**Features:**
- Left sidebar: Collapsible tool categories
- Main: Chat messages with tool call indicators
- Bottom: Input box with voice button
- Dark mode by default
- Minimal, clean aesthetic

**Component Library:** shadcn-svelte
- Modern, customizable components
- Tailwind CSS based
- Accessible by default
- Easy theme customization

**Rationale:**
- Professional, modern look
- Better than recreating C# UI
- Fast to iterate
- Mobile-friendly (future)

### 7. MVP Scope - Phase 1 Features Only

**Phase 1 MVP (Included):**
- ✅ Chat interface (streaming responses)
- ✅ Ollama integration
- ✅ MCP server integration
- ✅ All LibreOffice tool calls (Writer + Impress)
- ✅ Settings page (paths, model selection)
- ✅ Dependency checking (Python, Ollama, LibreOffice)
- ✅ Basic error handling

**Phase 2 (Deferred):**
- ❌ Voice recording + Whisper (code written, feature-flagged)
- ❌ Chat history persistence
- ❌ Tool visualization sidebar
- ❌ Auto-installers for dependencies
- ❌ Linux support
- ❌ Advanced settings

**Rationale:**
- Focus on core functionality first
- Faster time to working prototype
- Voice is complex (cross-platform audio)
- Can enable Phase 2 features incrementally

---

## Revised Implementation Timeline

### Week 1: Project Setup & Infrastructure ✅ COMPLETED
**Deliverable:** Running Tauri app with basic UI

- [x] Initialize Tauri project with Svelte 5 + TypeScript
- [x] Set up Rust project structure (services, commands, models)
- [x] Create basic layout and LoadingScreen component
- [x] Implement dependency checking (Python, Ollama, LibreOffice)
- [x] Bundle Python MCP server files as resources
- [x] Implement MCP process spawning with ProcessManager
- [x] Create MCP commands (start/check/stop)
- [x] Integrate MCP status into app store and UI

**Completed Files:**
- `tauri-app/src-tauri/Cargo.toml` - Dependencies (tauri, tokio, reqwest, parking_lot, anyhow)
- `tauri-app/src-tauri/tauri.conf.json` - Tauri config with resource bundling
- `tauri-app/src-tauri/src/models/config.rs` - AppConfig with defaults
- `tauri-app/src-tauri/src/services/config_service.rs` - Settings persistence
- `tauri-app/src-tauri/src/services/process_manager.rs` - MCP process lifecycle
- `tauri-app/src-tauri/src/commands/system.rs` - Dependency checking commands
- `tauri-app/src-tauri/src/commands/mcp.rs` - MCP server commands
- `tauri-app/src/lib/types/system.ts` - TypeScript types
- `tauri-app/src/lib/stores/app.svelte.ts` - App state with Svelte 5 runes
- `tauri-app/src/lib/stores/chat.svelte.ts` - Chat state with Svelte 5 runes
- `tauri-app/src/lib/components/LoadingScreen.svelte` - Dependency status UI
- `tauri-app/src/App.svelte` - Main app with system status display
- `tauri-app/src-tauri/resources/mcp_server/*` - Bundled Python MCP files

**Status:** ✅ All functionality working. App detects dependencies and shows appropriate UI states.

### Week 2: Backend Services
**Deliverable:** All Rust services functional

- [ ] Implement `ProcessManager` (Ollama, MCP, LibreOffice lifecycle)
- [ ] Implement `OllamaService` (HTTP client with streaming)
- [ ] Implement `McpClient` (stdio/JSON-RPC communication)
- [ ] Implement `ConfigurationService` (settings.json persistence)
- [ ] Implement `DocumentService` (file scanning)
- [ ] Write Tauri commands for all services
- [ ] Add error handling and logging

**Key Files:**
- `src-tauri/src/services/process_manager.rs`
- `src-tauri/src/services/ollama_service.rs`
- `src-tauri/src/services/mcp_client.rs`
- `src-tauri/src/commands/ollama.rs`
- `src-tauri/src/commands/mcp.rs`

### Week 3: Frontend Core
**Deliverable:** Functional chat interface

- [ ] Create Svelte 5 stores with runes (chat, app, config)
- [ ] Build ChatMessage component (user/assistant/tool calls)
- [ ] Build ChatInput component (text input, send button)
- [ ] Build LoadingScreen component (dependency status)
- [ ] Implement token streaming UI
- [ ] Add auto-scroll to bottom
- [ ] Build settings page (model, paths)

**Key Files:**
- `src/lib/stores/chat.svelte.ts`
- `src/lib/stores/app.svelte.ts`
- `src/lib/components/Chat/ChatMessage.svelte`
- `src/lib/components/Chat/ChatInput.svelte`
- `src/routes/settings/+page.svelte`

### Week 4: Integration & Testing
**Deliverable:** End-to-end working application

- [ ] Wire up all Tauri commands to UI
- [ ] Implement MCP tool calling flow
- [ ] Add tool call visualization in chat
- [ ] Test all LibreOffice operations (Writer + Impress)
- [ ] Add error notifications
- [ ] Implement "New Chat" functionality
- [ ] Polish loading states
- [ ] Test on Windows + macOS

**Testing Checklist:**
- [ ] Create document
- [ ] Add text/headings/tables
- [ ] Format text
- [ ] Create presentation
- [ ] Add/edit slides
- [ ] Apply templates
- [ ] Stream responses correctly
- [ ] Handle errors gracefully

### Week 5: Polish & Documentation
**Deliverable:** Production-ready MVP

- [ ] UI polish (animations, transitions)
- [ ] Error message improvements
- [ ] Add keyboard shortcuts (Ctrl+Enter to send)
- [ ] Build help page with examples
- [ ] Write README with setup instructions
- [ ] Create installer (MSI for Windows, DMG for macOS)
- [ ] Performance testing
- [ ] Bug fixes

---

## Technical Architecture Details

### State Management Flow

```
User Input (Svelte)
  ↓
chatStore.sendMessage(prompt)
  ↓
invoke('stream_ollama', { model, prompt, tools })
  ↓
Rust: OllamaService.stream_generate()
  ↓
For each token → emit('ollama-token', token)
  ↓
Svelte: listen('ollama-token') → append to message
  ↓
On tool call → invoke('call_mcp_tool', { name, args })
  ↓
Rust: McpClient.call_tool() → Python MCP → LibreOffice
  ↓
Tool result → Display in chat
```

### Error Handling Strategy

**Rust Side:**
```rust
#[tauri::command]
async fn operation() -> Result<String, String> {
    match perform_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            log::error!("Operation failed: {}", e);
            Err(format!("Error: {}", e))
        }
    }
}
```

**Svelte Side:**
```typescript
try {
    await invoke('operation');
} catch (error) {
    // Show toast notification
    toast.error(error as string);
}
```

**User-Facing Errors:**
- Python not found → "Please install Python 3.12+"
- Ollama not running → "Please start Ollama"
- LibreOffice missing → "Document features disabled. Install LibreOffice?"
- Tool call failed → "Failed to create document: [reason]"

### Configuration Schema

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub ollama_url: String,          // "http://localhost:11434"
    pub selected_model: String,       // "qwen2.5:7b"
    pub python_path: String,          // "python3" or specific path
    pub documents_path: String,       // User's Documents folder
    pub libreoffice_path: Option<String>,
    pub theme: String,                // "dark" | "light"
}
```

**Storage:** `~/.config/libreoffice-ai/settings.json` (Linux/Mac) or `%APPDATA%/LibreOfficeAI/settings.json` (Windows)

### MCP Communication Protocol

**Tool List Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/list",
    "params": null,
    "id": 1
}
```

**Tool Call Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
        "name": "create_blank_document",
        "arguments": {
            "filename": "test.odt",
            "title": "Test Document"
        }
    },
    "id": 2
}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "result": {
        "content": [
            {
                "type": "text",
                "text": "Successfully created document at /path/to/test.odt"
            }
        ]
    },
    "id": 2
}
```

---

## Deployment Strategy

### Development Setup

```bash
# Clone repo
git clone <repo-url>
cd libreoffice-ai-tauri

# Install dependencies
npm install
cd src-tauri
cargo build

# Copy MCP server files
cp -r ../../libre-office-mcp/libre-writer/* resources/mcp_server/

# Run dev mode
cd ..
npm run tauri dev
```

### Production Build

```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS
npm run tauri build -- --target aarch64-apple-darwin

# Outputs:
# Windows: target/release/bundle/msi/LibreOfficeAI_1.0.0_x64.msi
# macOS: target/release/bundle/dmg/LibreOfficeAI_1.0.0_aarch64.dmg
```

### Installation Size Estimates

**MVP Bundle:**
- Tauri runtime: ~5MB
- Rust binary: ~15MB
- Frontend (Svelte): ~2MB
- Python scripts: ~1MB
- Icons/assets: ~1MB
- **Total: ~25MB**

**User Must Install:**
- Python 3.12+: ~50MB
- Ollama: ~500MB + models (varies)
- LibreOffice: ~300MB

---

## Future Enhancements (Phase 2+)

### Voice Input (Week 6-7)
- Integrate `cpal` for audio recording
- Integrate `whisper-rs` for transcription
- Download model on first use
- Add voice button to chat input
- Visual recording indicator

### Chat History (Week 8)
- SQLite database for message persistence
- Chat session management
- Search across history
- Export conversations

### Advanced Features (Week 9-10)
- Tool visualization sidebar (show available tools)
- Custom tool creation (user-defined actions)
- Batch operations (process multiple docs)
- Template library (pre-made documents)

### Cross-Platform Polish (Week 11-12)
- Linux support (test on Ubuntu)
- Mobile-responsive UI
- Accessibility improvements (ARIA labels)
- Internationalization (i18n)

---

## Success Criteria

### MVP Acceptance Criteria

**Functional:**
- [ ] User can send chat messages and receive streaming responses
- [ ] User can create LibreOffice documents via chat
- [ ] User can edit documents (add text, format, tables)
- [ ] User can create presentations
- [ ] User can edit presentations (add slides, format)
- [ ] Settings persist across sessions
- [ ] Errors are shown clearly to user
- [ ] App works offline (after dependencies installed)

**Performance:**
- [ ] First response token within 2 seconds
- [ ] UI remains responsive during generation
- [ ] No memory leaks during extended use
- [ ] App starts in < 3 seconds

**Quality:**
- [ ] No crashes during normal operation
- [ ] All tool calls complete successfully
- [ ] Code is documented (inline comments)
- [ ] README includes setup instructions

---

## Risk Mitigation

### Risk: MCP Protocol Complexity
**Mitigation:** Start with simple JSON-RPC, test early with Python server

### Risk: Cross-Platform Audio Issues
**Mitigation:** Defer to Phase 2, focus on chat-only MVP

### Risk: LibreOffice UNO Errors
**Mitigation:** Reuse existing Python helper.py (battle-tested)

### Risk: Ollama API Changes
**Mitigation:** Pin to specific API version, add version checking

### Risk: Large Bundle Size
**Mitigation:** Use system dependencies (Python, Ollama, LibreOffice)

---

## Appendix: Key Code Snippets

### Main App State (Rust)

```rust
pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub ollama_service: Arc<Mutex<OllamaService>>,
    pub process_manager: Arc<ProcessManager>,
    pub mcp_client: Arc<Mutex<Option<McpClient>>>,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load_or_default();
        let ollama = OllamaService::new(config.ollama_url.clone());

        Self {
            config: Arc::new(RwLock::new(config)),
            ollama_service: Arc::new(Mutex::new(ollama)),
            process_manager: Arc::new(ProcessManager::new()),
            mcp_client: Arc::new(Mutex::new(None)),
        }
    }
}
```

### Chat Store (Svelte 5 Runes)

```typescript
class ChatStore {
    messages = $state<ChatMessage[]>([]);
    isGenerating = $state(false);

    async sendMessage(content: string) {
        this.messages.push({ role: 'user', content });
        this.isGenerating = true;

        const assistantMsg = { role: 'assistant', content: '' };
        this.messages.push(assistantMsg);

        const unlisten = await listen<string>('ollama-token', (event) => {
            assistantMsg.content += event.payload;
        });

        try {
            await invoke('stream_ollama', {
                model: configStore.model,
                prompt: content
            });
        } finally {
            unlisten();
            this.isGenerating = false;
        }
    }
}
```

---

## Conclusion

This plan provides a comprehensive roadmap for migrating the LibreOffice AI application from C#/WinUI to Tauri/Svelte/Rust. The phased approach prioritizes core functionality (chat + LibreOffice integration) while deferring complex features (voice, history) to future iterations.

**Estimated Timeline:** 5 weeks for production-ready MVP

**Next Steps:**
1. Initialize Tauri project
2. Set up project structure
3. Begin Week 1 implementation

Ready to proceed with implementation when approved.
