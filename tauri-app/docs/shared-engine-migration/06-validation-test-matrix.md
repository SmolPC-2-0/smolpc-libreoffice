# Validation and Test Matrix

Last updated: March 2, 2026
Source of truth: this matrix plus CI/build test outputs when implementation begins.

## Test Strategy

1. Unit tests for parser/config/error mapping.
2. Integration tests for engine commands and streaming.
3. End-to-end functional tests for LibreOffice tool workflows.
4. Hardware matrix tests for backend selection behavior.

## Unit Tests

## Planner parser

1. Valid `tool_call` JSON parses and validates.
2. Valid `final_answer` JSON parses and validates.
3. Fenced JSON can be recovered.
4. Invalid JSON fails with deterministic error.
5. Unknown `type` fails.
6. Unknown `tool_name` fails.
7. Missing required fields fail.

## Settings migration

1. Legacy config with `ollama_url` loads without crash.
2. Missing model defaults to `qwen3-4b-instruct-2507`.
3. Optional `engine_port` default is `19432`.
4. Optional `force_backend` accepts only `cpu|dml`.

## Error normalization

1. Stream cancellation maps to `INFERENCE_GENERATION_CANCELLED`.
2. Engine runtime stream failures map to `ENGINE_STREAM_ERROR`.
3. 429 queue full is surfaced with retry guidance.
4. 504 queue timeout is surfaced with retry guidance.

## Integration Tests (Rust Command Layer)

1. `check_engine_ready` returns healthy when host reachable.
2. `engine_list_models` returns at least one model.
3. `engine_load_model` succeeds for configured model.
4. `engine_chat_stream` emits chunk -> metrics -> done sequence.
5. `engine_cancel_generation` interrupts stream and emits cancellation.
6. `engine_status` returns backend diagnostics fields.

## End-to-End Functional Scenarios

1. User asks: "Create a document about climate change."
   - expected: planner `tool_call`, MCP execution succeeds, final answer returned.
2. User asks: "Add a heading called Project Goals."
   - expected: heading insertion flow succeeds.
3. Multi-step request:
   - create document, add heading, add paragraph.
   - expected: loop completes within limits and final answer summarizes.
4. Tool failure scenario:
   - MCP tool fails once.
   - expected: graceful failure message and no deadlock.

## Backend/Hardware Matrix

1. CPU-only machine:
   - app still completes chats and tool loop.
2. DirectML-capable machine (auto):
   - backend status shows directml selected or explicit fallback reason.
3. Forced DML:
   - `SMOLPC_FORCE_EP=dml` behavior verified.
4. Forced invalid device ID:
   - explicit error surfaced and captured in diagnostics.

## Acceptance Criteria

Migration is accepted when all are true:

1. Baseline workflows still work (document creation + heading insertion).
2. No active dependence on Ollama HTTP endpoints remains.
3. Stream UX remains token-by-token and cancelable.
4. Tool loop remains automatic (planner-driven).
5. Diagnostic status fields are available in UI or support logs.

