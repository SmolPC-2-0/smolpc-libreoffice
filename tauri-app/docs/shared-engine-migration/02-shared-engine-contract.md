# Shared Engine Contract (Integration Reference)

Last updated: March 2, 2026
Source of truth: `smolpc-codehelper` branch `origin/codex/shared-engine-v1`, especially `docs/ENGINE_API.md`, `crates/smolpc-engine-host/src/main.rs`, and `crates/smolpc-engine-client/src/lib.rs`.

## Transport and Auth

1. Base URL default: `http://127.0.0.1:19432`.
2. All endpoints require bearer token:
   - `Authorization: Bearer <token>`.
3. Token runtime file: `%LOCALAPPDATA%/SmolPC/engine-runtime/engine-token.txt`.
4. Client library can create/read token and spawn host.

## Engine Control Endpoints

1. `GET /engine/health`
2. `GET /engine/meta`
3. `GET /engine/status`
4. `POST /engine/load` body: `{ "model_id": "<id>" }`
5. `POST /engine/unload` body: `{ "force": false }`
6. `POST /engine/cancel`
7. `POST /engine/check-model` body: `{ "model_id": "<id>" }`
8. `POST /engine/shutdown`

## OpenAI-Compatible Endpoints

1. `GET /v1/models`
2. `POST /v1/chat/completions`

Supported request fields:
1. `model`
2. `messages`
3. `stream`
4. `max_tokens`
5. `temperature`
6. `top_k`
7. `top_p`
8. `repetition_penalty`
9. `repetition_penalty_last_n`

## Stream Semantics

1. Stream payload sends chat token chunks (`chat.completion.chunk`).
2. Exactly one metrics event (`chat.completion.metrics`) per completion.
3. Terminal event includes `[DONE]`.
4. Stream errors are structured JSON with codes:
   - `INFERENCE_GENERATION_CANCELLED`
   - `ENGINE_STREAM_ERROR`

## Scheduling and Limits

1. Single active generation globally.
2. Queue capacity default: 3.
3. Queue timeout default: 60 seconds.
4. Queue full: HTTP 429.
5. Queue timeout: HTTP 504.

## Backend Status Contract Fields

Key fields consumed by app diagnostics:
1. `active_backend` (`cpu` or `directml`)
2. `active_artifact_backend`
3. `runtime_engine`
4. `available_backends`
5. `selection_state`
6. `selection_reason`
7. `selected_device_id`
8. `selected_device_name`
9. `dml_gate_state`
10. `dml_gate_reason`
11. `failure_counters`
12. `force_override`

## Environment Variables Relevant to Integration

1. `SMOLPC_MODELS_DIR`
2. `SMOLPC_ENGINE_PORT`
3. `SMOLPC_FORCE_EP=cpu|dml`
4. `SMOLPC_DML_DEVICE_ID`
5. `SMOLPC_ENGINE_HOST_BIN` (for explicit host path override)

## Contract Boundaries (Important)

1. Integrate against documented API and client crate behavior only.
2. Do not parse internal logs as API.
3. Do not depend on internal host module paths.
4. Protocol major version must match (`1.x` expected in current branch).

## Current Limitation Affecting LibreOffice Migration

No contract-level tool-calling schema (`tools`/`tool_calls`) exists in current shared engine API surface. LibreOffice must implement tool planning and tool dispatch at app layer.

