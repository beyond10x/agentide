---
format: aep.planning-md/1
id: story:legacy-gitlab-startup-alignment
kind: story
status: implemented
title: Align composition with safe legacy GitLab startup
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
  path: generated/service
- confidence: cited
  path: service.yaml
revision: 6
---
## Outcome

Consume the exact published compatible provider revision that leaves legacy GitLab connections unusable while keeping the host available for verified reconnection. Align the nominal Connector service factory types across Service SDK, generated AgentIDE and Todo, and the composed Devcenter service. This is a source-pin and release-identity update; the semantic service contracts remain unchanged.

## Acceptance

Regenerate owned outputs through their declared generator, refresh locks, and run the complete repository gate. Publish exact source coordinates for the provider-first deployment. Legacy credentials must not acquire a current grant from configuration; recovery requires the existing verified connect flow.

## Evidence

Aligned the generated runtime service with Service SDK 0.5.11 at 0118bd3f9d63ead5d525fb39324b1e5e13c4ab1a, which consumes the tested Connectors legacy-startup correction at 235558c11f5fc2e4b8f8440474fb975df49d5329. Refreshed generated output through cargo xtask generate-service and refreshed the Cargo lock. The complete cargo xtask gate passed, including AEP, ESS, Rust, 21 browser tests, browser build, and generated-byte drift. The source release identity is 0.3.7; this change is consumed by the composed host and does not claim a new standalone binary release.
