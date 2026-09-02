---
format: aep.planning-md/1
id: architecture-decision-record:substrate-default-driver
kind: architecture-decision-record
status: accepted
title: Substrate is the standalone execution boundary
summary: Use Substrate for all standalone workspace and process effects.
relations:
- decides: product-requirements:agentide-v1
revision: 2
---
## Decision

The standalone binary embeds or connects to Substrate for guarded filesystem access, argv-only process execution, limits, secret slots, and egress apertures. It records an intent before dispatch and records the resulting Substrate observation.

## Refusal rule

If the required Substrate capability is absent, the intent is withheld or refused by name. AgentIDE never substitutes direct host filesystem or process access.
