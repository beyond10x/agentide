---
format: aep.planning-md/1
id: executable-system-specification:agentide-system
kind: executable-system-specification
status: implemented
title: AgentIDE executable system specification
summary: Typed commands, outcomes, events, views, and intent projection for AgentIDE.
relations:
- specifies: product-requirements:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 2
---
## Authority

The canonical source is the ESS tree under `spec/agentide/`. Generated JSON Schema, OpenAPI, AsyncAPI, intent descriptors, and conformance scenarios are projections and may not be edited as authority.

## Requirements

- ESS commands define semantic intent inputs, outcomes, refusals, and emitted facts.
- `agentide-intent-profile/1` references ESS commands and supplies model-facing consequence envelopes, subject extraction, and port names.
- Every public intent is either bound exactly once or withheld with a named reason.
- The same conformance scenarios run against the standalone and Harness binding sets.
