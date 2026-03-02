# SmolPC Engine Testing on Windows

Use this guide to validate the `smolpc_engine` integration on a Windows machine.

## 1. Get the Branch

After the feature branch is pushed:

```cmd
cd C:\Users\%USERNAME%\Documents
git clone https://github.com/SmolPC-2-0/smolpc-libreoffice.git
cd smolpc-libreoffice
git fetch origin
git checkout codex/libreoffice-engine-migration-start
```

If you already have the repo:

```cmd
cd C:\path\to\smolpc-libreoffice
git fetch origin
git checkout codex/libreoffice-engine-migration-start
git pull --ff-only origin codex/libreoffice-engine-migration-start
```

## 2. Install/Verify Prerequisites

From `tauri-app`:

```cmd
npm install
cd src-tauri
cargo check
cd ..
```

You still need:
- Python 3.12+
- LibreOffice
- MCP Python deps in `resources\mcp_server\.venv`

## 3. Start the Custom Engine Daemon

Start your engine daemon (the command depends on your engine project) and ensure it listens on `http://localhost:11435` (or your chosen URL).

Quick endpoint checks from PowerShell:

```powershell
Invoke-WebRequest -Uri "http://localhost:11435/health" -UseBasicParsing
Invoke-WebRequest -Uri "http://localhost:11435/models" -UseBasicParsing
```

Minimum expected behavior:
- `GET /health` returns HTTP `200`.
- `GET /models` should return a usable model list (array/object). If not implemented, you can still manually set model ID in app settings.
- `POST /generate` streams newline-delimited JSON or SSE (`data: ...` lines).

## 4. Run the App

```cmd
npm run tauri dev
```

In Settings:
1. Provider: `SmolPC Engine (Preview)`
2. Engine URL: `http://localhost:11435` (or your URL)
3. Model ID: select from dropdown or type manually
4. Save Settings

The loading screen should show:
- Python: ready
- SmolPC Engine: ready
- MCP Server: running

## 5. Smoke Tests

Run these prompts in order:

1. `What tools do you have available for LibreOffice?`
2. `Create a document called "engine-smoke-test" in my Documents folder.`
3. `Add a heading "Engine Integration Test" to engine-smoke-test.`
4. `Add a paragraph saying "This document was created through smolpc-engine tool calling."`

Expected:
- Assistant may stream normal text and/or tool-calling metadata.
- Tool messages should appear in chat with execution results.
- `.odt` file is created and updated correctly.

## 6. Tool-Calling Compatibility Contract

The app sends `/generate` payloads including:
- `model`
- `prompt`
- `messages` (role/content history)
- `tools` and `tool_choice: "auto"` when tools are available
- `params.temperature` and `params.max_tokens`

The app accepts streamed tool calls in these forms:
- Top-level `tool_calls`
- `choices[0].delta.tool_calls`
- `choices[0].message.tool_calls`
- Partial/delta tool arguments split across chunks (now reassembled before execution)

Fallback supported:
- If native tool calls are not emitted, assistant output can return strict JSON:
  `{"tool_calls":[{"function":{"name":"<tool_name>","arguments":{...}}}]}`

## 7. Capture Results

If a test fails, capture:
1. Prompt used
2. Chat output
3. Terminal logs (`npm run tauri dev` window)
4. Whether document/tool side effect happened
5. Engine endpoint URL + model ID

This is enough to isolate whether the issue is parser shape, engine protocol mismatch, or MCP/runtime behavior.
