---
format: aep.planning-md/1
id: story:harness-integration
kind: story
status: draft
title: Bind AgentIDE intents into Harness
summary: Implement native ToolPort projection and Harness-specific operation bindings after v1.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 1
---
## Acceptance

- Harness publishes the same names, schemas, envelopes, and subjects.
- Existing native approvals, delegation, and workflow events are reused rather than duplicated.
- AgentIDE depends on Harness; Harness has no AgentIDE dependency.
- The standalone and Harness binding sets pass the same semantic conformance suite.
