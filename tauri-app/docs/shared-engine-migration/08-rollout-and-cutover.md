# Rollout and Cutover Plan

Last updated: March 2, 2026
Source of truth: this doc and migration execution PR history.

## Rollout Stages

## Stage A: Dual Path (Internal)

1. Keep engine and Ollama code paths selectable via build/runtime flag.
2. Default internal testing to engine path.
3. Use Ollama path only as fallback while parity is validated.

Exit criteria:
1. Validation matrix pass for core scenarios.
2. No P1 blockers from risk register.

## Stage B: Engine Default (Internal QA)

1. Engine path enabled by default.
2. Ollama path disabled by default but still available for emergency regression checks.
3. Collect diagnostics from real test hardware.

Exit criteria:
1. Document creation and heading insertion stable on target Windows hardware.
2. Stream, cancel, and tool loop behavior stable.

## Stage C: Hard Cutover

1. Remove active Ollama inference path from product.
2. Update setup docs to engine-only flow.
3. Keep legacy migration notes for users with older settings files.

Exit criteria:
1. No runtime calls to Ollama endpoints remain.
2. Support runbook covers all expected failures.

## Rollback Strategy

1. Maintain tagged commit for last known Ollama-capable build during transition.
2. If critical regression appears after Stage B:
   - toggle back to Ollama path in internal builds
   - keep migration branch for fixes
3. Do not discard config migration compatibility logic after cutover.

## Monitoring and Evidence to Capture

For each test session:
1. app version
2. OS and hardware
3. selected model
4. backend status snapshot (`active_backend`, `selection_reason`, `dml_gate_state`)
5. failure code and message when errors occur

## Operational Policies

1. `429` queue full:
   - allow retry with jitter.
2. `504` queue timeout:
   - present timeout with retry action.
3. cancellation:
   - treat as expected state, not error-level incident.
4. protocol mismatch:
   - block chat and report incompatible host/app version.

