# Current State Snapshot

Last updated: March 2, 2026
Source of truth: code paths listed in this document.

## LibreOffice App (Current, Ollama-Based)

## Runtime and startup flow

1. App startup checks Python, Ollama, and LibreOffice in [`src-tauri/src/commands/system.rs`](../../src-tauri/src/commands/system.rs).
2. UI readiness is gated by Python + Ollama in [`src/lib/stores/app.svelte.ts`](../../src/lib/stores/app.svelte.ts).
3. MCP server start occurs when those dependencies are ready in [`src-tauri/src/commands/mcp.rs`](../../src-tauri/src/commands/mcp.rs).

## Inference path

1. Tauri commands are Ollama-specific in [`src-tauri/src/commands/ollama.rs`](../../src-tauri/src/commands/ollama.rs):
   - `list_ollama_models`
   - `chat_stream`
   - `check_ollama_running`
2. Rust service calls Ollama HTTP endpoints (`/api/tags`, `/api/chat`) in [`src-tauri/src/services/ollama_service.rs`](../../src-tauri/src/services/ollama_service.rs).
3. Frontend listens to event stream:
   - `ollama-stream-chunk`
   - `ollama-stream-error`
   in [`src/lib/stores/chat.svelte.ts`](../../src/lib/stores/chat.svelte.ts).

## Tool-calling behavior

1. Current tool orchestration depends on Ollama-native `tools` request field and `tool_calls` response field.
2. MCP tool definitions are converted into Ollama function schema in [`src/lib/stores/mcp.svelte.ts`](../../src/lib/stores/mcp.svelte.ts).
3. Chat loop executes tool calls from model output, appends tool results, then recursively asks the model again in [`src/lib/stores/chat.svelte.ts`](../../src/lib/stores/chat.svelte.ts).

## Settings and config

1. TS settings currently include:
   - `ollama_url`
   - `selected_model`
   - `temperature`
   - `max_tokens`
   in [`src/lib/types/settings.ts`](../../src/lib/types/settings.ts).
2. Rust config uses matching Ollama-first fields in [`src-tauri/src/models/config.rs`](../../src-tauri/src/models/config.rs).
3. Model list UI is populated by `list_ollama_models` in [`src/lib/stores/settings.svelte.ts`](../../src/lib/stores/settings.svelte.ts).

## Shared Engine Branch (CodeHelper `origin/codex/shared-engine-v1`)

## Architecture

1. Engine crates:
   - `crates/smolpc-engine-core`
   - `crates/smolpc-engine-host`
   - `crates/smolpc-engine-client`
2. Host is localhost daemon with bearer token auth and OpenAI-compatible chat endpoint.
3. Client handles connect-or-spawn lifecycle, host discovery, token handling, protocol check, and spawn lock.

## Contract and behavior

1. Engine endpoints are documented in `docs/ENGINE_API.md` (CodeHelper branch).
2. Backend selection and fallback include CPU and DirectML with runtime demotion logic.
3. Queueing behavior is built-in:
   - single active generation
   - queue size default 3
   - 429 queue full
   - 504 queue timeout

## Runtime dependencies and packaging assumptions

1. ONNX Runtime and DirectML DLLs are required in runtime resource paths.
2. Engine host sidecar is expected in packaged resources (`binaries/*`) or discoverable fallback paths.
3. Shared model root defaults to `%LOCALAPPDATA%/SmolPC/models`.

## Main Compatibility Gap

The shared engine contract does not currently expose Ollama-style function-calling fields (`tools` request, `tool_calls` response). It supports text chat completions with stream/non-stream, metrics, and cancel semantics.

This requires app-side planning/execution protocol for tool use.

## Baseline Functional Requirement to Preserve

Known working baseline in current LibreOffice app:
1. document creation
2. heading insertion

Migration is complete only if these remain working under the shared engine path.

