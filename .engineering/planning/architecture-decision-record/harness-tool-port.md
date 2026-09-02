---
format: aep.planning-md/1
id: architecture-decision-record:harness-tool-port
kind: architecture-decision-record
status: accepted
title: Harness integration uses ToolPort
summary: Project intents into Harness ToolSpec, Envelope, Subject, and LoopEvent contracts.
relations:
- decides: product-requirements:agentide-v1
revision: 2
---
## Decision

The native integration lives in AgentIDE and depends on Harness public crates. It projects intent contracts into `ToolPort`; Harness remains the owner of its loop, approval port, delegation, workflows, and provider-neutral events.

Declarative Toolchain YAML is not the normative seam because it cannot preserve the full structured schemas, dynamic subjects, and action lifecycle without weakening them.
