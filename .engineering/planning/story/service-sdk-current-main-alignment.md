---
format: aep.planning-md/1
id: story:service-sdk-current-main-alignment
kind: story
status: implemented
title: Align AgentIDE generated service with Service SDK current main
relations:
- derived_from: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
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
## Outcome

AgentIDE's generated service factory composes with the independently promoted current Connectors runtime through the exact current Service SDK runtime identity.

## Acceptance

- The service package pins the exact current Service SDK default-branch commit.
- AgentIDE retains its proven 0.5.6 authoring generator, so this change does not silently perform the separate realization-plan migration exposed by the 0.5.8 generator.
- `cargo xtask generate-service` owns every generated change; the generated diff is limited to dependency coordinates and its manifest digests.
- `cargo xtask gate` passes with the current generated runtime dependency.
