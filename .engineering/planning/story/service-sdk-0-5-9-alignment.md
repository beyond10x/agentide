---
format: aep.planning-md/1
id: story:service-sdk-0-5-9-alignment
kind: story
status: implemented
title: Align AgentIDE with Service SDK 0.5.9
summary: Regenerate the service package for the Connectors 0.6.2 factory identity.
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
  path: package.json
- confidence: cited
  path: service.yaml
revision: 5
---
# AgentIDE Service SDK 0.5.9 alignment\n\n## Outcome\n\nRegenerate the owned service package against exact Service SDK commit 93cd33452f929f675daaf3af307a70e642a8b53d so its generated Connector factory composes with Connectors 0.6.2.\n\n## Acceptance\n\n- Generated Rust, catalogue, documentation, deployment, and conformance output is byte-current.\n- The repository's complete gate passes.\n- AgentIDE 0.3.5 is bot-authored on the repository default branch and released from an annotated tag.\n\n## Scope\n\nOnly the SDK coordinate, generator-owned output, release version, changelog, lockfiles, and governed planning evidence may change.\n