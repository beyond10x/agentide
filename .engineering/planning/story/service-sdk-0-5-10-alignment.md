---
format: aep.planning-md/1
id: story:service-sdk-0-5-10-alignment
kind: story
status: implemented
title: Align AgentIDE with Service SDK 0.5.10
summary: Regenerate the hosted service for the Connectors 0.6.4 factory identity.
relations:
- derived_from: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: cited
  path: .engineering/planning
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: generated/service
- confidence: cited
  path: service.yaml
revision: 7
---
# AgentIDE Service SDK 0.5.10 alignment

## Outcome

Regenerate the hosted service package against the exact published Service SDK 0.5.10 commit so its Connector factory composes with the Connectors 0.6.4 Smart Git runtime. Prepare the Rust patch release 0.3.6 after its complete repository gate.

## Scope

Change only the SDK runtime coordinate in service.yaml, generator-owned service output, the Rust release version and Cargo lockfile, changelog, and governed planning evidence. Preserve semantic commands, browser package coordinates, and existing immutable realization artifacts.

## Acceptance

- The owned service output is generated from the declared package and byte-current.
- Its runtime SDK dependencies use the same exact published commit.
- The complete cargo xtask gate passes.
- The candidate is handed to the coordinating agent for bot publication before the composed host consumes it.

## Evidence

The declared package now uses the published Service SDK 0.5.10 commit 14bb3ba65b3f0f2b8a577bd9f30961bfc0a92ad9. cargo xtask generate-service rewrote only the generated Rust and console dependency manifests and their ownership digests. The exact generated runtime, catalogue, Connectors, HTTP, and conformance dependencies all use that commit. Cargo lock regeneration changes only the AgentIDE patch versions and the SDK/Connectors dependency source chain.

The existing xtask-only service-builder 0.5.6 remains the declared generator. Its build dependencies do not cross the generated runtime service boundary. Full gate evidence follows validation with the CI-pinned specification tools.

On 2026-09-05, the complete cargo xtask gate exited 0 using CI-pinned AEP 0.52.0 and ESS 0.9.2 with Node 22.23.1. Validation covered contract/profile and realization agreement, generated ESS and service bytes, the generated service scenario, workspace Rust check/tests/doc tests and all-target Clippy with warnings denied, redacted fixtures, the browser typecheck and 21 browser tests, and the production build with no tracked browser or ESS output drift. git diff --check passed.

A normal-edge cargo tree for agentide-generated-service confirms the runtime, engine, Eventlog, Connectors, catalogue, and HTTP SDK crates use only 0.5.10 at 14bb3ba65b3f0f2b8a577bd9f30961bfc0a92ad9, with Connectors service/protocol dependencies at 0.6.4 commit dbdd285c629d8b93bb685cc5a89a270316978ce5. The gate log is retained at target/agentide-sdk-alignment-gate.log. The candidate is ready for coordinating-agent bot publication.
