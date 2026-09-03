---
format: aep.planning-md/1
id: story:pluggable-browser-renderers
kind: story
status: implemented
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
revision: 6
---
## Outcome

AgentIDE exposes one versioned, transport-neutral browser renderer protocol and interchangeable
Vanilla DOM and Vue targets. Every target consumes the same state and transient events, emits the
same typed semantic actions, and remains unaware of HTTP, authentication, routing, storage,
polling, and sockets.

## Acceptance criteria

- Renderer frame, event, action, target manifest, and lifecycle interfaces are public, versioned,
  strict, and covered by golden fixtures.
- Vanilla and Vue targets pass one semantic, accessibility, and action-trace conformance suite.
- The local host adapter owns all API calls and serves the same live session at explicit Vanilla
  and Vue comparison routes.
- Static checks refuse transport and persistence APIs inside renderer targets.
- A repeatable pinned-Chromium benchmark reports bundle bytes, navigation, mount, 50-frame update,
  and available heap observations for both targets.
- Devcenter consumes the exact Vue renderer release without moving Identity, Workspace, Agent
  Platform, terminal transport, or authority into the renderer.

## Evidence required

- AgentIDE full gate and renderer conformance report.
- Pinned Chromium runtime and Node build benchmark report.
- Devcenter full gate and successful deployment.
