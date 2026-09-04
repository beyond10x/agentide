---
format: aep.planning-md/1
id: story:release-agentide-0-3-3
kind: story
status: implemented
title: Release AgentIDE 0.3.3
summary: Publish the already-gated AgentIDE dependency update through the repaired deterministic release path.
relations:
- implements: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
revision: 5
---
## Outcome

AgentIDE 0.3.3 is published from current main with the Service SDK 0.5.6 and Connectors 0.5.11 generated service plus the repaired CI tool installation.

## Acceptance

- Workspace and lock versions agree on 0.3.3.
- Changelog names the release-gate repair and dependency payload.
- The full repository gate passes on the release commit.
- Tag 0.3.3 is cut from merged main and the bot-authored GitHub release succeeds.

## Scope

- `Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
