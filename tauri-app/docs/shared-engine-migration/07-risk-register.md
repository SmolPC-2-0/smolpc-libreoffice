# Risk Register

Last updated: March 2, 2026
Source of truth: this register plus `tauri-app/ENGINEERING_ISSUES.md`.

## Legend

1. Severity: P1 high, P2 medium, P3 low.
2. Likelihood: High, Medium, Low.

## Risks

## R-01 Planner output invalid JSON

1. Severity: P1
2. Likelihood: Medium
3. Impact: tool loop stalls or returns user-visible failure.
4. Mitigation:
   - strict parser + one repair retry
   - clear fallback message
   - iteration guardrails

## R-02 Tool schema mismatch

1. Severity: P1
2. Likelihood: Medium
3. Impact: model calls tools with wrong arguments; execution failures.
4. Mitigation:
   - validate against MCP tool schema before execution
   - include schema summaries in planner prompt
   - feed tool error back to model for correction attempt

## R-03 Stream listener leaks/regression

1. Severity: P2
2. Likelihood: Medium
3. Impact: duplicate chunks/events, unstable UI behavior.
4. Mitigation:
   - single lifecycle scope per request
   - guaranteed cleanup in `finally`
   - regression tests for repeated send/cancel/error paths

## R-04 Engine host discovery failure

1. Severity: P1
2. Likelihood: Medium
3. Impact: app cannot start inference.
4. Mitigation:
   - use `smolpc-engine-client` discovery behavior
   - validate packaged sidecar presence
   - improve startup diagnostics and operator runbook

## R-05 Missing runtime libraries

1. Severity: P1
2. Likelihood: Medium
3. Impact: ONNX runtime init failure at host startup.
4. Mitigation:
   - bundle required `libs/*`
   - preflight runtime checks
   - actionable setup/troubleshooting docs

## R-06 Backend fallback confusion

1. Severity: P2
2. Likelihood: Medium
3. Impact: users expect GPU but see CPU behavior.
4. Mitigation:
   - surface `engine/status` backend fields in diagnostics
   - document debug overrides (`SMOLPC_FORCE_EP`, `SMOLPC_DML_DEVICE_ID`)

## R-07 Queue pressure under parallel requests

1. Severity: P2
2. Likelihood: Low
3. Impact: 429/504 failures during bursts.
4. Mitigation:
   - app-level retry policy with backoff for retryable states
   - clear user feedback on queue timeout/full conditions

## R-08 Existing MCP technical debt

1. Severity: P1
2. Likelihood: Medium
3. Impact: deadlocks/process leaks unrelated to engine but affects perceived migration quality.
4. Mitigation:
   - prioritize ENG-001 and ENG-002 from `ENGINEERING_ISSUES.md` in same migration window
   - add regression tests for timeout/restart paths

## Watch Items (Not blockers for initial migration)

1. Shared engine currently lacks native tool-calling protocol.
2. Startup probe can sometimes prefer CPU on DML-capable systems (known deferred issue in shared engine docs).

