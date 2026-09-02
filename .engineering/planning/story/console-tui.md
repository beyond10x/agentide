---
format: aep.planning-md/1
id: story:console-tui
kind: story
status: implemented
title: Build the Harness-hosted console TUI
summary: Render and control the virtual coding workbench in a terminal on the shared Harness-facing contracts.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 4
---
## Acceptance

- `agentide tui` renders the same event-derived workbench projection as the browser and snapshots.
- The TUI exposes virtual panes, open files, focus, cursor location, diffs, processes, approvals, agents, and evidence without reading hidden model state.
- The standalone target runs its effects through the Substrate-backed binding; the Harness target can host the same surface over its ToolPort and lifecycle.
- Keyboard actions invoke semantic intents and remain replayable from the durable event log.
