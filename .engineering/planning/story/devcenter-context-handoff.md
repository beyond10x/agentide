---
format: aep.planning-md/1
id: story:devcenter-context-handoff
kind: story
status: implemented
title: Seal DevCenter context handoff
summary: Give untrusted prompt attachments one AgentIDE-owned draft contract and sealing path.
relations:
- derived_from: story:hosted-protocol-hardening
- serves: vision:agent-first-coding-surface
scope:
- confidence: inferred
  path: .engineering/planning/journal.jsonl
- confidence: inferred
  path: .engineering/planning/story/devcenter-context-handoff.md
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: contracts/fixtures/hosted
- confidence: inferred
  path: contracts/schemas/hosted
- confidence: inferred
  path: crates/agentide-contracts/src/hosted.rs
- confidence: inferred
  path: crates/agentide-xtask/src/main.rs
- confidence: inferred
  path: docs/running-modes.md
- confidence: inferred
  path: generated/service/runtime/realization-plan.json
- confidence: inferred
  path: realizations/10-agentide-standalone-linux.yaml
revision: 7
---
# Seal the DevCenter context handoff

## Context

DevCenter receives deliberately attached browser bytes before any authenticated actor provenance exists. AgentIDE owns the model-facing context contract, so it must also own the untrusted draft shape and the one transition that seals it for an authenticated human. This keeps browser input distinct from server-derived authority and makes drift a compile-time and fixture-gate failure.

## Acceptance

AgentIDE publishes and validates an untrusted context-selection draft schema and golden vector, seals it into a digest-identical authenticated selection, and DevCenter consumes that exact contract through its complete browser and Rust gates.

## Scope

The renderer-neutral contract, generated schema and vector, contract tests, release metadata, realization metadata, and public documentation allowlist.
