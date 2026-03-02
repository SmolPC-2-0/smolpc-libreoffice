# SmolPC Shared Engine Testing on Windows

Use this guide to validate LibreOffice integration against `smolpc-codehelper` branch `codex/shared-engine-v1`.

## 1. Get the LibreOffice Branch

```cmd
cd C:\path\to\smolpc-libreoffice
git fetch origin
git checkout -B codex/libreoffice-engine-migration-start --track origin/codex/libreoffice-engine-migration-start
cd tauri-app
npm install
```

## 2. Get and Build the Engine Host (CodeHelper)

```cmd
cd C:\path\to\smolpc-codehelper
git fetch origin
git checkout -B codex/shared-engine-v1 --track origin/codex/shared-engine-v1
npm install
cargo build -p smolpc-engine-host
```

## 3. Prepare Engine Token and Start Daemon

PowerShell:

```powershell
$runtimeDir = Join-Path $env:LOCALAPPDATA "SmolPC\engine-runtime"
New-Item -ItemType Directory -Force -Path $runtimeDir | Out-Null

$tokenPath = Join-Path $runtimeDir "engine-token.txt"
if (!(Test-Path $tokenPath)) {
  $token = -join ((48..57 + 65..90 + 97..122) | Get-Random -Count 48 | ForEach-Object {[char]$_})
  Set-Content -Path $tokenPath -Value $token
}

$env:SMOLPC_ENGINE_TOKEN = (Get-Content $tokenPath -Raw).Trim()

cd C:\path\to\smolpc-codehelper
cargo run -p smolpc-engine-host -- --port 19432 --data-dir "$env:LOCALAPPDATA\SmolPC\engine" --app-version dev
```

Keep that terminal open.

## 4. Verify Engine Endpoints

In a second PowerShell window:

```powershell
$token = (Get-Content "$env:LOCALAPPDATA\SmolPC\engine-runtime\engine-token.txt" -Raw).Trim()
$headers = @{ Authorization = "Bearer $token" }

Invoke-RestMethod -Headers $headers http://127.0.0.1:19432/engine/health
Invoke-RestMethod -Headers $headers http://127.0.0.1:19432/v1/models
```

Expected:
- `/engine/health` returns `{ ok: true }`
- `/v1/models` returns model data

## 5. Run LibreOffice App Against Shared Engine

```cmd
cd C:\path\to\smolpc-libreoffice\tauri-app
npm run tauri dev
```

In Settings:
1. Provider: `SmolPC Engine (Shared v1)`
2. Engine URL: `http://127.0.0.1:19432`
3. Model ID: choose from dropdown or set manually (example: `qwen3-4b-instruct-2507`)
4. Save

Expected loading statuses:
- Python: ready
- SmolPC Engine: ready
- MCP Server: running

## 6. Smoke Test Prompts

1. `What tools do you have available for LibreOffice?`
2. `Create a document called "engine-smoke-test" in my Documents folder.`
3. `Add a heading "Engine Integration Test" to engine-smoke-test.`
4. `Add a paragraph saying "This document was created through shared-engine-v1 tool calling."`

Expected:
- Assistant streams content.
- Tool messages appear with execution results.
- Document file exists and is updated.

## 7. API Contract Used by LibreOffice Branch

Primary (shared-engine-v1):
- `GET /engine/health` (Bearer token)
- `GET /v1/models` (Bearer token)
- `POST /v1/chat/completions` with `stream: true` (Bearer token)

Compatibility fallback:
- `/health`, `/models`, `/generate` (legacy preview engines)

Tool-call handling:
- Native `tool_calls` parsing when present
- OpenAI-style delta tool-call argument assembly
- JSON fallback from assistant text:
  `{"tool_calls":[{"function":{"name":"<tool_name>","arguments":{...}}}]}`

## 8. Capture Failures

When a step fails, capture:
1. Prompt and observed chat output
2. `npm run tauri dev` logs
3. Engine host terminal logs
4. Engine URL and model ID
5. Result of `/engine/status` (with bearer token)
