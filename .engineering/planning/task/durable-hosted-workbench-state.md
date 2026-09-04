---
format: aep.planning-md/1
id: task:durable-hosted-workbench-state
kind: task
status: active
title: Persist the hosted workbench surface
summary: Realize actor-private AgentIDE workbench state and consume it through Devcenter's authenticated host port.
relations:
- decomposes: story:hosted-browser-workbench-controller
- serves: vision:agent-first-coding-surface
revision: 3
---
# Task: Persist the hosted workbench surface

## Outcome

The ESS-declared AgentIDE Workbench becomes an owner-private event-sourced aggregate. Devcenter's authenticated host port snapshots and mutates it while Workspace remains the sole file-content authority.

## Acceptance

- Generated AgentIDE operations initialize and snapshot the Workbench and durably apply open/close file, open/close/focus pane, cursor, and diff transitions.
- Surface projections derive owner and scope from the current coding session and remain invisible to another authenticated actor.
- Devcenter uses only authenticated BFF routes for durable workbench state; browser storage is not used.
- Approval projection includes only pending approvals belonging to tasks in the current coding session.
- ESS scenarios and focused runtime/BFF/host tests demonstrate reload restoration and cross-actor refusal.

## Scope

- `service/runtime.yaml` — cited
- `service/scenarios/session.yaml` — cited
- `spec/agentide/domains/surface.yaml` — cited
- `generated/service` — inferred
- `crates/devcenter-http/src/lib.rs` — inferred
- `frontend/src/api/client.ts` — inferred
- `frontend/src/features/workbench/devcenterWorkbenchHost.ts` — inferred
