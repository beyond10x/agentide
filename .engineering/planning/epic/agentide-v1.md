---
format: aep.planning-md/1
id: epic:agentide-v1
kind: epic
status: draft
title: Deliver AgentIDE v1
summary: Specify, implement, validate, publish, and release the first AgentIDE binary.
relations:
- derived_from: product-requirements:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 1
---
## Outcome

Release a public Apache-2.0 Rust application whose ESS contracts generate the standalone coding-session interface and preserve a native Harness integration seam.

## Deliberate boundary

The first release owns the standalone binary and web projection. Native Harness composition is a separately deliverable follow-up over the same contracts.
