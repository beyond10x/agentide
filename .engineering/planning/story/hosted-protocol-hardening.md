---
format: aep.planning-md/1
id: story:hosted-protocol-hardening
kind: story
status: implemented
title: Harden hosted AgentIDE protocol
summary: Publish self-validating wire contracts and conformance for hosted sessions without duplicating Workspace or Service SDK.
relations:
- derived_from: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: Cargo.toml
- confidence: inferred
  path: README.md
- confidence: inferred
  path: contracts/fixtures
- confidence: inferred
  path: contracts/schemas
- confidence: inferred
  path: crates/agentide-contracts/src/hosted.rs
- confidence: inferred
  path: crates/agentide-harness/src/lib.rs
- confidence: inferred
  path: crates/agentide-xtask/src/main.rs
revision: 7
---
# Harden hosted session protocol

## Outcome

Hosted AgentIDE clients exchange versioned, self-validating actor, context, inventory, diff, file, grant, and terminal records that every renderer and service can verify independently.

## Context

The hosted workbench is now exercised through DevCenter, Workspace, Substrate, Service SDK, and Eventlog. The pilot proved the component boundaries and exposed the need for stable wire schemas, canonical digests, explicit attachment provenance, and terminal transport conformance before additional clients depend on the protocol.

Service SDK remains the session coordination, persistence, event, OpenAPI, and hosted-service layer. Workspace remains the file, diff, process, and PTY authority. This story must not introduce another file store, event store, service framework, or browser-computed authority.

## Acceptance

- Renderer-neutral hosted records publish immutable JSON Schemas and representative golden vectors.
- Format discriminators, SHA-256 values, workspace paths, ranges, revisions, and cross-record digests are validated with stable named refusals.
- Context selections carry actor and source provenance and prove the digest of the exact attached bytes; truncated content is never model-injected.
- Actor view exposes separate coordination, context, and inventory revisions/digests so clients can distinguish which authority changed.
- Terminal JSON controls and lifecycle frames are typed, sequence-aware, bounded, and round-trip through conformance tests; raw PTY bytes remain outside model context.
- Inventory sealing is deterministic and dispatch reauthorizes against current actor, bindings, session state, and grants.
- The v1 intent-profile compatibility loader and actor-specific human/agent inventories remain covered.
- AgentIDE pins exact current released dependencies, passes `cargo xtask gate`, merges through `main`, and cuts an immutable release.

## Out of Scope

- A new persistence backend or filesystem implementation.
- Browser-side authoritative diffing.
- Raw shell authority for agents.
- Remote LSP integration.

## Open Questions

None. The pilot established the ownership boundaries above.
