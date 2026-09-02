---
format: aep.planning-md/1
id: product-requirements:agentide-v1
kind: product-requirements
status: draft
title: AgentIDE v1
summary: A single-binary coding-session driver with abstract intents and externally supplied implementations.
relations:
- serves: vision:agent-first-coding-surface
revision: 2
---
- AgentIDE publishes typed semantic intents rather than executable paths or provider-specific commands.
- A model-visible request cannot supply binding options, credentials, destinations, or authority policy.
- Every mutating intent is previewed, authorized against the exact plan digest, durably recorded, and then dispatched.
- The standalone application uses Substrate for guarded workspace and process effects and never falls back to unconfined execution.
- Virtual panes, open files, cursor state, diffs, terminals, approvals, agents, and evidence are durable renderer-neutral session projections.
- A compact session snapshot replaces repeated repository, process, agent, workflow, and evidence inspection.
- A local web surface, console TUI, and the CLI render the same event-derived state.
- Released intent, event, snapshot, and binding formats are immutable; breaking changes add successors.
- Harness can publish the same intent schemas, host the TUI, and rebind their implementations without importing AgentIDE into Harness.

## Acceptance

- The released binary can start a session, inspect a workspace, manage virtual panes and open files, edit code, run a configured verification, control a process, and publish through a configured implementation.
- Sanitized real-session fixtures replay to deterministic projections across JSON, browser, and TUI renderers.
- Missing bindings, capabilities, approvals, and recovery facts produce named refusals rather than optimistic success.
- The complete repository gate and generated-contract drift checks pass from a clean checkout.
