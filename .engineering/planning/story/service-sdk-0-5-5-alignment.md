---
format: aep.planning-md/1
id: story:service-sdk-0-5-5-alignment
kind: story
status: implemented
title: Align AgentIDE with Service SDK 0.5.5
summary: Regenerate the hosted AgentIDE service against the released Connectors 0.5.11 factory graph.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
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
revision: 6
---
# Story: Align AgentIDE with Service SDK 0.5.6

## Outcome

Devcenter can embed AgentIDE's generated Connector service through the same Connectors 0.5.11 factory contract used by the hosted runtime.

## Context

Connectors 0.5.11 fixes valid GitLab OAuth refresh responses that omit an unchanged scope. Service SDK 0.5.6 carries that contract and keeps Connector-only services library-only.

## Acceptance

- AgentIDE's package and generated service pin the merged Service SDK 0.5.6 commit.
- The workspace resolves one Connectors 0.5.11 generated-service factory graph.
- Connector-only generation emits no standalone HTTP host.
- AgentIDE reports version 0.3.2 and `cargo xtask gate` passes.

## Out of Scope

Changing AgentIDE intents, its browser renderer protocol, or its declared Connector-only delivery.

## Open Questions

None.
