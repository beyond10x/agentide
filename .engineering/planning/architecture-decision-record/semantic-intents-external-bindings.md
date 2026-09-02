---
format: aep.planning-md/1
id: architecture-decision-record:semantic-intents-external-bindings
kind: architecture-decision-record
status: accepted
title: Semantic intents and external bindings
summary: Keep model-visible meaning separate from deployment-selected implementations and options.
relations:
- decides: product-requirements:agentide-v1
revision: 2
---
## Decision

ESS commands are the semantic authority. A closed AgentIDE intent profile projects them to model-facing names and safety declarations. Rust embedding applications register typed implementation ports; deployment configuration selects one registered port and supplies operator-only options.

There is no arbitrary executable plugin bag, model-selected binding, or automatic fallback after refusal.

## Consequences

The CLI and Harness expose identical intent names and schemas. Binding versions and non-secret option digests participate in preview identity. Zero or multiple applicable bindings fail closed.
