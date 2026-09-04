---
format: aep.planning-md/1
id: story:repair-release-tool-install
kind: story
status: implemented
title: Isolate pinned release tools from CI caches
summary: Install pinned AEP and ESS binaries into a run-local tool root so restored Cargo binaries cannot break the release gate.
relations:
- implements: epic:agentide-v1
- serves: vision:agent-first-coding-surface
scope:
- confidence: cited
  path: .github/workflows/ci.yml
revision: 5
---
## Outcome

AgentIDE CI installs the exact pinned AEP and ESS revisions without colliding with stale binaries restored by the Rust cache.

## Acceptance

- The specification tools are installed under a run-local root rather than the shared Cargo bin directory.
- The repository gate resolves those run-local binaries on PATH.
- The workflow remains deterministic and retains dependency caches.
- The full repository gate succeeds.

## Scope

- `.github/workflows/ci.yml`
