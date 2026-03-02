# Implementation Plan (Decision Complete)

Last updated: March 2, 2026
Source of truth: this document and linked files in this repo.

## Phase 0: Preparation

## Changes

1. Update Rust version compatibility for shared engine crates.
2. Add dependencies for:
   - `smolpc-engine-client`
   - `smolpc-engine-core`
3. Add docs-only feature flag note for temporary dual-path support.

## Files

1. `tauri-app/src-tauri/Cargo.toml`
2. `tauri-app/src-tauri/Cargo.lock`

## Done Criteria

1. Project compiles with shared engine deps available.
2. No inference-path behavior changes yet.

## Phase 1: Rust Backend Engine Command Layer

## Changes

1. Add engine service/client resolver with cache + reconnect logic.
2. Add Tauri commands:
   - `engine_list_models`
   - `engine_load_model`
   - `engine_unload_model`
   - `engine_chat_stream`
   - `engine_cancel_generation`
   - `engine_status`
   - `engine_check_model_exists`
   - `check_engine_ready`
3. Add event emission:
   - `engine-stream-chunk`
   - `engine-stream-metrics`
   - `engine-stream-error`
   - `engine-stream-done`
4. Keep old Ollama commands available behind temporary compatibility flag only (for staged rollout).

## Files

1. `tauri-app/src-tauri/src/lib.rs`
2. `tauri-app/src-tauri/src/commands/mod.rs`
3. `tauri-app/src-tauri/src/commands/engine.rs` (new)
4. `tauri-app/src-tauri/src/services/engine_service.rs` (new)
5. `tauri-app/src-tauri/src/models/engine.rs` (new)

## Done Criteria

1. Frontend can list models and stream a plain completion from engine.
2. Cancel and status commands function.
3. No MCP tool loop changes yet.

## Phase 2: Frontend Store and Event Migration

## Changes

1. Replace Ollama event listeners with engine event listeners.
2. Replace `list_ollama_models` usage with `engine_list_models`.
3. Introduce engine type definitions and remove direct dependence on Ollama stream types for active path.
4. Ensure listener lifecycle is cleaned up in `finally`.

## Files

1. `tauri-app/src/lib/stores/chat.svelte.ts`
2. `tauri-app/src/lib/stores/settings.svelte.ts`
3. `tauri-app/src/lib/stores/app.svelte.ts`
4. `tauri-app/src/lib/types/engine.ts` (new)
5. `tauri-app/src/lib/components/LoadingScreen.svelte`
6. `tauri-app/src/lib/components/SettingsPage.svelte`
7. `tauri-app/src/App.svelte`

## Done Criteria

1. Chat streams via engine events.
2. Model selector displays engine models.
3. Loading screen shows engine readiness instead of Ollama readiness.

## Phase 3: Planner-Based Tool Loop

## Changes

1. Implement strict planner protocol (see `05-tool-planner-protocol.md`).
2. Parse/validate engine assistant output JSON.
3. If `tool_call`, execute existing MCP call and append `tool` message.
4. Repeat loop until `final_answer`.
5. Add guardrails:
   - max planner iterations per user turn: 8
   - max tool calls per turn: 12
   - malformed response repair retry: 1

## Files

1. `tauri-app/src/lib/stores/chat.svelte.ts`
2. `tauri-app/src/lib/types/chat.ts`
3. Optional parser helper:
   - `tauri-app/src/lib/utils/planner.ts` (new)

## Done Criteria

1. Existing user workflow for document creation works with engine.
2. Existing heading insertion workflow works with engine.
3. Tool loop terminates predictably on malformed output or hard limits.

## Phase 4: Settings and Config Migration

## Changes

1. Keep backward compatibility when loading old config containing `ollama_url`.
2. Migrate defaults:
   - `selected_model`: `qwen3-4b-instruct-2507`
3. Add optional advanced fields:
   - `engine_port` default `19432`
   - `force_backend` (`cpu` or `dml`) for diagnostics mode
4. Remove Ollama-specific labels and references from settings UI.

## Files

1. `tauri-app/src/lib/types/settings.ts`
2. `tauri-app/src-tauri/src/models/config.rs`
3. `tauri-app/src-tauri/src/services/config_service.rs`
4. `tauri-app/src/lib/components/SettingsPage.svelte`

## Done Criteria

1. Existing settings files load without breakage.
2. New installs use engine-first defaults.

## Phase 5: Packaging and Runtime Integration

## Changes

1. Ensure Tauri bundle includes engine host sidecar (`binaries/*`).
2. Ensure runtime libs expected by engine host are available (`libs/*`).
3. Add dev scripts equivalent to shared engine workflow (host prebuild and optional forced backend run).

## Files

1. `tauri-app/src-tauri/tauri.conf.json`
2. `tauri-app/package.json`
3. `tauri-app/scripts/*` (new)
4. `tauri-app/src-tauri/binaries/README.md` (new)
5. `tauri-app/src-tauri/libs/README.md` (new)

## Done Criteria

1. Dev run can connect/spawn host reliably.
2. Packaged app can locate host and required runtime libraries.

## Phase 6: Cleanup and Final Cutover

## Changes

1. Remove Ollama command/service/type code from active build path after parity validation.
2. Update docs and setup guides to engine-first.
3. Keep brief migration notes for legacy users.

## Files

1. `tauri-app/src-tauri/src/commands/ollama.rs` (remove or fully deprecated)
2. `tauri-app/src-tauri/src/services/ollama_service.rs` (remove or deprecated)
3. `tauri-app/src-tauri/src/models/ollama.rs` (remove or compatibility only)
4. `tauri-app/README.md`
5. `tauri-app/GETTING_STARTED_WINDOWS.md`

## Done Criteria

1. No runtime dependency on Ollama endpoints remains.
2. Parity checklist passes (see `06-validation-test-matrix.md`).

