# Target Architecture (LibreOffice + Shared Engine)

Last updated: March 2, 2026
Source of truth: this design plus migration implementation changes in this repo.

## Goals

1. Replace Ollama transport with shared engine transport.
2. Keep current MCP tool execution behavior from user perspective.
3. Preserve token streaming UX and cancellation.
4. Add stronger runtime diagnostics for supportability.

## Component Responsibilities

## Frontend (`Svelte`)

1. Build user conversation and tool context.
2. Start engine stream via Tauri command.
3. Parse planner JSON output from model text.
4. Execute MCP tools via existing MCP commands.
5. Loop until model emits `final_answer`.
6. Render stream, metrics, errors, and final messages.

## Backend (`Rust/Tauri`)

1. Resolve/cached shared engine client via `smolpc-engine-client`.
2. Expose engine commands to frontend:
   - list/load/unload/status/check-model/generate-stream/cancel
3. Normalize stream events and errors for frontend.
4. Expose readiness checks and backend status for loading and diagnostics UI.

## Shared Engine Host (External sidecar)

1. Serve inference endpoints and model lifecycle.
2. Handle queueing/single-flight/cancel.
3. Provide backend selection and fallback behavior (CPU/DirectML).

## MCP Subsystem (Unchanged)

1. Tool list/discovery remains from existing MCP client.
2. Tool execution remains local JSON-RPC over stdio to MCP server.
3. Tool result messages feed model in next loop step.

## End-to-End Data Flow (Single User Turn)

1. User sends message.
2. Frontend assembles conversation + planner instructions + tool manifest.
3. Frontend calls `engine_chat_stream`.
4. Rust backend streams tokens from shared engine and emits `engine-stream-*` events.
5. Frontend collects streamed assistant text.
6. Frontend parses final text as planner JSON.
7. If planner says `tool_call`:
   - validate tool and args
   - call MCP tool
   - append `tool` message with result
   - repeat from step 2
8. If planner says `final_answer`:
   - append assistant final response
   - complete turn.

## State Ownership

1. Chat state and planner loop state: frontend store.
2. Engine client lifecycle and host spawn policy: Rust backend.
3. MCP process and tool invocation: existing Rust MCP modules.
4. Persistent preferences: existing config service with migrated fields.

## Key Contract Changes in App Layer

1. Event names:
   - old: `ollama-stream-chunk`, `ollama-stream-error`
   - new: `engine-stream-chunk`, `engine-stream-metrics`, `engine-stream-error`, `engine-stream-done`
2. Command names:
   - old: `chat_stream`, `list_ollama_models`
   - new: `engine_chat_stream`, `engine_list_models` (+ status/cancel commands)
3. Settings:
   - old primary endpoint field: `ollama_url`
   - new primary endpoint field: `engine_port` (default 19432) and optional backend debug overrides.

