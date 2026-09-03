---
format: aep.planning-md/1
id: story:service-sdk-0-4-alignment
kind: story
status: implemented
title: Align AgentIDE with Service SDK 0.4
summary: Regenerate AgentIDE on the breaking Service SDK 0.4 contract formats.
relations:
- derived_from: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: generated
- confidence: cited
  path: service.yaml
- confidence: cited
  path: service/runtime.yaml
revision: 7
---
# Align AgentIDE with Service SDK 0.4

## Outcome

AgentIDE continues to generate and run its service surface from the current Service SDK after the intentional contract-format cut.

## Context

Service SDK 0.4 removes the prior definition, runtime-IR, and client-plan formats. AgentIDE must update its declarative package and regenerate owned outputs in the same release chain.

## Acceptance

- AgentIDE pins Service SDK 0.4.2 and the exact Connectors 0.5.6 revision.
- Its service definition uses only the new supported format, declares Connector-only delivery, and assigns an exact scope to every operation.
- `cargo xtask generate-service`, `cargo xtask generate-realizations`, and the full `cargo xtask gate` complete without generated drift.
- The release version is 0.3.1 and no compatibility artifacts for the old formats remain.

## Out of Scope

New AgentIDE product behavior, Atlas or Website updates, and deployment changes.

## Open Questions

None.

## Scope

- cited: `service.yaml`, `service/runtime.yaml`, `generated/`, `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and generator tasks.
