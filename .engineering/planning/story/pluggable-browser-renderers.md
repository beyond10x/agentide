---
format: aep.planning-md/1
id: story:pluggable-browser-renderers
kind: story
status: active
title: Pluggable browser renderer targets
summary: Render the same AgentIDE state through transport-neutral Vanilla and Vue targets and compare them.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: inferred
  path: contracts
- confidence: inferred
  path: crates/agentide-cli
- confidence: inferred
  path: crates/agentide-contracts
- confidence: inferred
  path: crates/agentide-xtask
- confidence: inferred
  path: web
revision: 4
---
## Outcome

AgentIDE exposes one versioned, transport-neutral browser renderer protocol and interchangeable Vanilla DOM and Vue targets. Every target consumes the same state and transient events, emits the same typed semantic actions, and remains unaware of HTTP, authentication, routing, storage, polling, and sockets.

## Acceptance criteria

- Renderer frame, event, action, target manifest, and lifecycle interfaces are public, versioned, strict, and covered by golden fixtures.
- Vanilla and Vue targets pass one semantic, accessibility, responsive-layout, and action-trace conformance suite.
- The local host adapter owns all API calls and serves the same live session at explicit Vanilla and Vue comparison routes.
- Static checks refuse transport and persistence APIs inside renderer targets.
- Repeatable runtime and build benchmarks publish bundle, startup, update, streaming, memory, install, cold-build, and warm-build results for both targets.
- Devcenter consumes the exact Vue renderer release through a hosted adapter without moving Identity, Workspace, Agent Platform, terminal transport, or authority into the renderer.

## Evidence required

- AgentIDE full gate and renderer conformance report.
- Pinned Chromium runtime and pinned Node build benchmark report.
- Devcenter full gate, authenticated hosted-workbench smoke test, and successful agent turn.
