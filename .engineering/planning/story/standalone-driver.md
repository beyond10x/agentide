---
format: aep.planning-md/1
id: story:standalone-driver
kind: story
status: implemented
title: Build the standalone driver
summary: Implement the journaled Rust engine, Substrate binding set, and single-binary CLI.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 4
---
## Acceptance

- The CLI starts and reopens sessions outside the workspace.
- Read operations execute immediately; mutations preview, authorize, journal, dispatch, and recover.
- Substrate capability gaps produce named refusals with no host fallback.
- JSON and JSONL output keep stdout machine-readable and diagnostics on stderr.
