---
format: aep.planning-md/1
id: story:session-workbench
kind: story
status: implemented
title: Build the shared session workbench
summary: Render live and replayed sessions through one browser and CLI projection.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 5
---
## Acceptance

- The embedded web application and console TUI show virtual panes, open files, focus, cursor location, diffs, processes, approvals, agent lanes, and evidence.
- Open, close, focus, cursor, and diff actions are semantic surface intents recorded in the same event log.
- CLI snapshots, browser state, and TUI state come from the same projector and cursor semantics.
- Sanitized recent-session fixtures contain no prompts, reasoning, source contents, credentials, or private absolute paths.
