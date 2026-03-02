# Windows Shared-Engine Validation Handoff (March 2, 2026)

## Scope
Validate LibreOffice app branch `codex/libreoffice-engine-migration-start` against CodeHelper engine branch `codex/shared-engine-v1` on Windows.

Target checks:
- Shared engine auth and endpoints (`/engine/health`, `/v1/models`, `/v1/chat/completions` stream)
- LibreOffice provider configuration (`SmolPC Engine (Shared v1)`, `http://127.0.0.1:19432`)
- Dependency checks + MCP startup
- End-to-end tool-call smoke tests
- Ollama regression check

---

## Branch + Commit Snapshot

### LibreOffice repo
- Branch: `codex/libreoffice-engine-migration-start`
- Commit: `bed55cde78274ac18a038e00f8196a00b8fe0189`

### CodeHelper repo
- Branch: `codex/shared-engine-v1`
- Commit validated during run: `d118723a9f8b1ef1a706ab5e3c39edc875817994`

---

## Outcome Summary

### Passed
- LibreOffice branch checkout + `npm install`
- CodeHelper branch checkout + `npm install` + `cargo build -p smolpc-engine-host`
- Engine token creation + auth path
- `GET /engine/health`
- `GET /v1/models`
- `POST /v1/chat/completions` stream (API-level)
- LibreOffice provider settings corrected to `http://127.0.0.1:19432`
- Dependency checks and MCP startup (`27` tools discovered)
- Smoke test 1/4 (`list tools`) eventually passed after local parser/prompt fixes

### Not yet passed
- Smoke test 2/4 (`create document`)
- Smoke test 3/4 (`add heading`)
- Smoke test 4/4 (`add paragraph`)
- Ollama regression check

---

## Critical Failures Encountered and Fixes

## 1) Engine host failed to start (ONNX runtime mismatch)
Error:
- `ort 2.0.0-rc.11 ... expected >= 1.23.x, got 1.17.1`

Root cause:
- `onnxruntime.dll` resolved from system PATH instead of project runtime libs.

Fixes used:
1. Install correct runtime libs:
   - `bash ./scripts/setup-libs.sh --platform windows-x64 --force`
2. Ensure repo-root `libs` exists and contains:
   - `onnxruntime.dll`
   - `onnxruntime_providers_shared.dll`
   - `DirectML.dll`
   - `onnxruntime-genai.dll`
3. Launch host with:
   - `$env:PATH = "$PWD\\libs;$env:PATH"`
   - `$env:SMOLPC_MODELS_DIR = "$env:LOCALAPPDATA\\SmolPC\\models"`
   - `$env:SMOLPC_ENGINE_TOKEN = ...`

---

## 2) Engine token missing in terminal session
Error:
- `SMOLPC_ENGINE_TOKEN is required`

Fix:
- Set token in the same terminal before `cargo run`:
  - `$env:SMOLPC_ENGINE_TOKEN = (Get-Content "$env:LOCALAPPDATA\SmolPC\engine-runtime\engine-token.txt" -Raw).Trim()`

---

## 3) Model not loaded
Error:
- `No model loaded. Call /engine/load first.`

Fix:
- Manual load call:
  - `POST /engine/load` with `{"model_id":"qwen3-4b-instruct-2507"}`

Notes:
- `qwen3-4b-instruct-2507` is DirectML-required in this host implementation.
- This machine did pass DirectML probe:
  - `selected_device_name: Intel(R) Iris(R) Xe Graphics`
  - `directml_probe_passed: true`

---

## 4) Shared model artifacts missing
Error examples:
- CPU model file missing
- setup script dependency failures (`onnx_ir`, `torch`, `transformers`, etc.)

Fix path used:
1. Install Python deps (incrementally as prompted by script errors).
2. Run:
   - `npm run model:setup:qwen3`
3. Success criteria observed:
   - `Qwen3 shared model setup complete.`
   - `Validation result: VALID`

---

## 5) LibreOffice app blocked on loading screen (engine "Not Found")
Root cause:
- Settings contained stale engine URL (`http://localhost:11435`).

Fix:
- Update `%APPDATA%\\libreoffice-ai\\settings.json`:
  - `"ai_provider": "smolpc_engine"`
  - `"smolpc_engine_url": "http://127.0.0.1:19432"`
  - `"selected_model": "qwen3-4b-instruct-2507"`

---

## 6) Tool-call chain failures in app

### 6a) Fallback JSON often malformed by model
Examples observed:
- Leading `:`
- `tool Calls` instead of `tool_calls`
- extra braces

Local fix applied in LibreOffice frontend:
- File: `tauri-app/src/lib/stores/chat.svelte.ts`
- Added tolerant JSON candidate parsing/repair path for fallback tool-call extraction.

### 6b) Engine rejected follow-up `tool` role
Error:
- `400 Bad Request - {"error":"unsupported message role: tool"}`

Local fix applied in LibreOffice Rust adapter:
- File: `tauri-app/src-tauri/src/services/smolpc_engine_service.rs`
- Map outgoing `tool` messages to supported role (`user`) with prefixed content `Tool result:\n...`.

### 6c) "I don't have tool access" response for tool list prompt
Local prompt adjustment:
- File: `tauri-app/src/lib/stores/chat.svelte.ts`
- Added explicit system instruction to list available tools when asked.

Result:
- Smoke 1/4 prompt eventually returned actual tool list.

---

## Local Code Changes Made During This Session

## Modified files
- `tauri-app/src/lib/stores/chat.svelte.ts`
  - fallback parser hardening for malformed tool-call JSON
  - additional system-prompt instruction for tool-list requests
- `tauri-app/src-tauri/src/services/smolpc_engine_service.rs`
  - role normalization for `tool` -> `user` for shared-engine compatibility

## Also modified in working tree (not edited in this session)
- `tauri-app/src-tauri/Cargo.toml` (shows as modified in git status; verify before commit)

---

## Current Working Status (Where to Resume)

Smoke test state:
- 1/4 list tools: PASS
- 2/4 create document: FAIL
  - Model called `create_blank_document` with wrong args:
    - sent: `document_name`, `folder_path`
    - required by MCP schema: `filename` (and possibly additional required args depending on tool schema)

Latest failure snippet:
- `Field required ... filename`

This means tool invocation is now happening, but argument selection is still unreliable.

---

## Resume Checklist (Next Person)

1. Start engine host in one terminal:
```powershell
cd C:\Users\mathi\smolpc\smolpc-codehelper
$env:PATH = "$PWD\libs;$env:PATH"
$env:SMOLPC_MODELS_DIR = "$env:LOCALAPPDATA\SmolPC\models"
$env:SMOLPC_ENGINE_TOKEN = (Get-Content "$env:LOCALAPPDATA\SmolPC\engine-runtime\engine-token.txt" -Raw).Trim()
cargo run -p smolpc-engine-host -- --port 19432 --data-dir "$env:LOCALAPPDATA\SmolPC\engine" --app-version dev
```

2. In second terminal, verify and load model:
```powershell
$token = (Get-Content "$env:LOCALAPPDATA\SmolPC\engine-runtime\engine-token.txt" -Raw).Trim()
$headers = @{ Authorization = "Bearer $token" }
Invoke-RestMethod -Headers $headers http://127.0.0.1:19432/engine/health
Invoke-RestMethod -Headers $headers http://127.0.0.1:19432/v1/models
$payload = @{ model_id = "qwen3-4b-instruct-2507" } | ConvertTo-Json -Compress
Invoke-RestMethod -Method Post -Headers $headers -ContentType "application/json" -Uri "http://127.0.0.1:19432/engine/load" -Body $payload
```

3. Ensure LibreOffice app settings file:
`%APPDATA%\libreoffice-ai\settings.json`
```json
{
  "ai_provider": "smolpc_engine",
  "smolpc_engine_url": "http://127.0.0.1:19432",
  "selected_model": "qwen3-4b-instruct-2507"
}
```

4. Run app:
```powershell
cd C:\Users\mathi\smolpc\smolpc-libreoffice\tauri-app
npm run tauri dev
```

5. Continue smoke tests:
- `Create a document called "engine-smoke-test" in my Documents folder.`
- `Add a heading "Engine Integration Test" to engine-smoke-test.`
- `Add a paragraph saying "This document was created through shared-engine-v1 tool calling."`

6. If argument mismatches continue:
- Ask model for strict JSON with exact MCP parameter names OR
- Improve tool-selection prompt with tool schemas OR
- Add backend-side argument alias normalization for known tool names.

---

## Recommended Follow-up Engineering Tasks

1. Add auto-load behavior before first shared-engine generation:
- If `No model loaded`, call `/engine/load` automatically using selected model.

2. Include `tools` in shared `/v1/chat/completions` payload if supported by host contract (or confirm unsupported).

3. Add robust JSON repair in Rust path as well (currently frontend fallback is compensating for malformed output).

4. Add integration test for tool-call roundtrip with malformed-but-repairable JSON.

5. Investigate occasional host crash:
- `STATUS_STACK_BUFFER_OVERRUN (0xc0000409)` seen once.

6. Run required Ollama regression after shared-engine flow is stable.

---

## Notes on Performance

- Streaming on this hardware is noticeably slow (expected on constrained/IGPU systems with local inference).
- Functional correctness should be validated separately from throughput.

