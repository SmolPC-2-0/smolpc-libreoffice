# Engineering Issues Backlog

Last updated: March 2, 2026

This file tracks non-blocking and latent engineering issues found during code review.
Document creation and heading insertion currently work on Windows, so these are primarily
stability and maintainability risks to address in follow-up work.

## Related migration handoff docs

For the complete Ollama-to-shared-engine migration package, see:

- [`docs/shared-engine-migration/README.md`](docs/shared-engine-migration/README.md)

## Priority legend

- P0: must fix immediately
- P1: high risk, schedule soon
- P2: medium risk, schedule in next maintenance pass
- P3: low risk, fix opportunistically

## Open issues

### ENG-001 - MCP client can deadlock after a timeout

- Priority: P1
- Status: Open
- Area: Rust MCP transport
- Files:
  - `src-tauri/src/services/mcp_client.rs`
- Summary:
  - `send_request` spawns a blocking reader thread per request. If request timeout triggers,
    that thread can continue holding the stdout mutex, preventing later requests from reading.
- User impact:
  - Tool calling can become permanently stuck until app restart after one bad timeout.
- Suggested fix:
  - Replace per-request thread spawn + mutex lock with a single dedicated reader loop and
    request/response correlation by JSON-RPC id.
  - Ensure timeout path does not leave any lock held by background work.
- Verification:
  - Add an integration test that forces one response timeout, then confirms next request succeeds.

### ENG-002 - MCP process lifecycle leaks on failed init/restart

- Priority: P1
- Status: Open
- Area: Rust MCP process management
- Files:
  - `src-tauri/src/commands/mcp.rs`
  - `src-tauri/src/services/mcp_client.rs`
- Summary:
  - Failed `initialize()` path returns an error status without stopping the started child process.
  - `start()` does not stop an existing child before replacing stored handles.
- User impact:
  - Can leave orphan helper/server processes and create inconsistent state after retries.
- Suggested fix:
  - On init failure, call `stop()` before returning failure.
  - In `start()`, short-circuit if already running or stop and replace cleanly.
  - Add an explicit restart command path and tests.
- Verification:
  - Start, fail init intentionally, retry start, then confirm only one child process is alive.

### ENG-003 - Chat stream event listeners are not fully cleaned up

- Priority: P2
- Status: Open
- Area: Frontend chat store
- Files:
  - `src/lib/stores/chat.svelte.ts`
- Summary:
  - Success path clears the chunk listener but not always the error listener.
  - Invoke failure path can leave both listeners registered.
- User impact:
  - Duplicate events over time, repeated callback execution, and unstable chat state.
- Suggested fix:
  - Manage both unlisten callbacks together and always clean up in `finally`.
  - Use per-request scope object/abort token to prevent stale listeners from mutating state.
- Verification:
  - Send multiple chats with forced errors and confirm listener count remains constant.

### ENG-004 - Settings are partially wired and can be misleading

- Priority: P2
- Status: Open
- Area: Config and settings integration
- Files:
  - `src-tauri/src/commands/config.rs`
  - `src-tauri/src/commands/mcp.rs`
  - `src-tauri/src/services/mcp_client.rs`
  - `src/lib/components/SettingsPage.svelte`
- Summary:
  - UI exposes `python_path`, `documents_path`, `libreoffice_path`, but runtime paths are not fully honored.
  - `python_path` is read from config but currently ignored by `McpClient::start`.
- User impact:
  - Users may change settings and expect behavior changes that do not happen.
- Suggested fix:
  - Wire all path settings into runtime command paths.
  - Update in-memory `AppState` config after save, not only disk config.
  - Hide or mark settings as experimental until fully implemented.
- Verification:
  - Change each setting, restart/reinitialize services, and confirm effective runtime path changes.

### ENG-005 - Windows Python version check may false-negative

- Priority: P3
- Status: Open
- Area: Dependency detection
- Files:
  - `src-tauri/src/commands/system.rs`
- Summary:
  - `check_python` only reads stdout, but some Python installations report version via stderr.
- User impact:
  - "Python not found" false negatives on some machines.
- Suggested fix:
  - Parse both stdout and stderr for version output.
  - Optionally run `-c "import sys; print(sys.version)"` as fallback.
- Verification:
  - Test detection against `python`, `python3`, and `py` variants on Windows.

### ENG-006 - MCP launcher scripts drift between dev and bundled copies

- Priority: P2
- Status: Open
- Area: Python resource packaging
- Files:
  - `resources/mcp_server/main.py`
  - `src-tauri/resources/mcp_server/main.py`
- Summary:
  - Two copies of `main.py` have diverged, which can cause behavior differences in dev vs packaged app.
- User impact:
  - Works in dev but fails in packaged app (or vice versa), harder debugging.
- Suggested fix:
  - Keep one source-of-truth MCP resource directory and copy/sync at build time.
  - Add CI check to fail if duplicate resource files diverge.
- Verification:
  - Build app and confirm same MCP startup behavior in dev and packaged modes.

### ENG-007 - Svelte warning about non-reactive `messagesContainer`

- Priority: P3
- Status: Open
- Area: Frontend app shell
- Files:
  - `src/App.svelte`
- Summary:
  - Vite/Svelte warns that `messagesContainer` is updated but not declared with `$state`.
- User impact:
  - Usually low risk today, but can lead to subtle update issues as component evolves.
- Suggested fix:
  - Follow current Svelte guidance for bind targets (or make intent explicit with local state pattern).
- Verification:
  - `npm run build` completes without Svelte reactivity warnings.

## Suggested order for next maintenance pass

1. ENG-001
2. ENG-002
3. ENG-003
4. ENG-004
5. ENG-006
6. ENG-005
7. ENG-007
