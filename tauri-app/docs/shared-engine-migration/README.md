# Shared Engine Migration Handoff

Last updated: March 2, 2026
Source of truth: this folder (`tauri-app/docs/shared-engine-migration`) plus referenced code paths in `smolpc-libreoffice` and `smolpc-codehelper` (`origin/codex/shared-engine-v1`).

## Purpose

This documentation package is the handoff source for migrating `smolpc-libreoffice` from Ollama to the shared SmolPC local engine, while preserving current working document workflows.

The goal is decision-complete implementation guidance that another engineer or AI can execute without design ambiguity.

## Locked Decisions

1. Tool execution remains app-owned (planner JSON protocol), not engine-native tool calling.
2. LibreOffice backend uses `smolpc-engine-client` for connect/spawn/auth/protocol lifecycle.
3. Default model policy uses `qwen3-4b-instruct-2507` with `qwen2.5-coder-1.5b` fallback.
4. Migration remains Windows-first.

## Reading Order

1. [01-current-state.md](./01-current-state.md)
2. [02-shared-engine-contract.md](./02-shared-engine-contract.md)
3. [03-target-architecture.md](./03-target-architecture.md)
4. [04-migration-plan.md](./04-migration-plan.md)
5. [05-tool-planner-protocol.md](./05-tool-planner-protocol.md)
6. [06-validation-test-matrix.md](./06-validation-test-matrix.md)
7. [07-risk-register.md](./07-risk-register.md)
8. [08-rollout-and-cutover.md](./08-rollout-and-cutover.md)
9. [09-operator-runbook.md](./09-operator-runbook.md)
10. [10-appendix-file-map.md](./10-appendix-file-map.md)

## Scope and Non-Goals

In scope:
1. Replace Ollama inference path with shared engine path.
2. Preserve current tool-based LibreOffice behavior (document creation and heading insertion are known working baselines).
3. Keep MCP integration model unchanged.

Out of scope:
1. Rewriting MCP server protocol or helper internals.
2. Adding engine-side native tool-calling protocol in this phase.
3. Broader product redesign unrelated to inference migration.

