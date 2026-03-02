# Operator Runbook

Last updated: March 2, 2026
Source of truth: shared engine docs in `smolpc-codehelper` branch `origin/codex/shared-engine-v1` and this migration package.

## Purpose

Provide practical setup and troubleshooting steps for developers/QA during and after migration.

## Preconditions

1. Python installed for MCP server.
2. LibreOffice installed for document tool operations.
3. Shared model artifacts present under `%LOCALAPPDATA%/SmolPC/models` (or `SMOLPC_MODELS_DIR` override).
4. Engine host binary and runtime libraries packaged/discoverable by app.

## Quick Health Check

1. Verify host reachable:
   - `GET /engine/health`
2. Verify protocol:
   - `GET /engine/meta` should report protocol major `1`.
3. Verify backend/model status:
   - `GET /engine/status`

Example PowerShell snippet:

```powershell
$tokenPath = Join-Path $env:LOCALAPPDATA "SmolPC\engine-runtime\engine-token.txt"
$token = (Get-Content $tokenPath -Raw).Trim()
$headers = @{ Authorization = "Bearer $token" }
Invoke-RestMethod -Uri "http://127.0.0.1:19432/engine/health" -Headers $headers
Invoke-RestMethod -Uri "http://127.0.0.1:19432/engine/meta" -Headers $headers
Invoke-RestMethod -Uri "http://127.0.0.1:19432/engine/status" -Headers $headers
```

## Model Lifecycle Check

```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:19432/engine/load" -Method Post -Headers $headers -ContentType "application/json" -Body '{"model_id":"qwen3-4b-instruct-2507"}'
Invoke-RestMethod -Uri "http://127.0.0.1:19432/engine/status" -Headers $headers
```

## Common Failure States and Actions

## Engine not healthy

1. Check host binary discovery path.
2. Check required runtime libraries are present.
3. Check app logs for ONNX runtime initialization errors.

## Protocol mismatch

1. Confirm app uses engine client compatible with host major version.
2. Stop stale host process and relaunch app.

## Queue full (429)

1. Reduce concurrent requests.
2. Retry with short backoff.

## Queue timeout (504)

1. Retry request.
2. Confirm current generation is not stuck.
3. Use cancel endpoint if needed.

## Cancel behavior confusion

1. Cancellation should return `INFERENCE_GENERATION_CANCELLED`.
2. Treat as expected control flow.

## CPU selected when GPU expected

1. Inspect `/engine/status`:
   - `active_backend`
   - `selection_reason`
   - `dml_gate_state`
2. Verify DML artifact presence for selected model.
3. For diagnostics only, run with `SMOLPC_FORCE_EP=dml`.

## Invalid forced device ID

1. If `SMOLPC_DML_DEVICE_ID` is set and invalid, load can fail.
2. Clear or correct env var and retry.

## MCP tool loop failure

1. Verify planner output parse error details in app logs.
2. Verify MCP server is running and tools are listed.
3. Check argument schema mismatch or unknown tool name.

## Incident Report Template

Capture:
1. app version + git commit
2. OS and hardware details
3. selected model and settings
4. request payload summary
5. HTTP status / event error
6. `/engine/status` snapshot
7. whether backend overrides were set

