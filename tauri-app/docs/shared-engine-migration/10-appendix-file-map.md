# Appendix: File Map and Interface Mapping

Last updated: March 2, 2026
Source of truth: repository files listed below.

## Key LibreOffice Files (Current)

1. Inference commands:
   - `tauri-app/src-tauri/src/commands/ollama.rs`
2. Ollama transport:
   - `tauri-app/src-tauri/src/services/ollama_service.rs`
3. Startup dependency checks:
   - `tauri-app/src-tauri/src/commands/system.rs`
4. Chat loop/store:
   - `tauri-app/src/lib/stores/chat.svelte.ts`
5. MCP tool conversion/execution:
   - `tauri-app/src/lib/stores/mcp.svelte.ts`
6. App init gating:
   - `tauri-app/src/lib/stores/app.svelte.ts`
7. Settings model and persistence:
   - `tauri-app/src/lib/types/settings.ts`
   - `tauri-app/src-tauri/src/models/config.rs`
   - `tauri-app/src-tauri/src/services/config_service.rs`

## Key Shared Engine Files (Reference: `smolpc-codehelper` `origin/codex/shared-engine-v1`)

1. API contract docs:
   - `docs/ENGINE_API.md`
   - `docs/APP_ONBOARDING_PLAYBOOK.md`
   - `docs/SMOLPC_SUITE_INTEGRATION.md`
2. Engine host:
   - `crates/smolpc-engine-host/src/main.rs`
3. Engine client:
   - `crates/smolpc-engine-client/src/lib.rs`
4. Backend/model/runtime internals:
   - `crates/smolpc-engine-core/src/inference/backend.rs`
   - `crates/smolpc-engine-core/src/models/loader.rs`
   - `crates/smolpc-engine-core/src/models/registry.rs`

## Old-to-New Interface Mapping

## Tauri Commands

1. `list_ollama_models` -> `engine_list_models`
2. `chat_stream` -> `engine_chat_stream`
3. `check_ollama_running` / `check_ollama` -> `check_engine_ready`
4. new additions:
   - `engine_status`
   - `engine_cancel_generation`
   - `engine_load_model`
   - `engine_check_model_exists`

## Frontend Events

1. `ollama-stream-chunk` -> `engine-stream-chunk`
2. `ollama-stream-error` -> `engine-stream-error`
3. add:
   - `engine-stream-metrics`
   - `engine-stream-done`

## Settings

1. legacy:
   - `ollama_url` (compat read only)
2. target:
   - `selected_model`
   - `engine_port`
   - `force_backend` (optional debug)
   - existing user preferences (`temperature`, `max_tokens`, paths)

## Glossary

1. Shared engine: local daemon + client/core crates used across SmolPC apps.
2. Planner loop: app-level model output parsing that decides tool call vs final answer.
3. MCP: Model Context Protocol subsystem used for LibreOffice tools.
4. DML: DirectML backend option in shared engine.

