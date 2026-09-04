---
format: aep.planning-md/1
id: story:hosted-browser-workbench-controller
kind: story
status: implemented
title: Own the hosted browser workbench
summary: Make Vue and Vanilla render one typed AgentIDE controller with shared Monaco and Ghostty adapters.
relations:
- decomposes: epic:agentide-v1
- informed_by: story:pluggable-browser-renderers
- serves: vision:agent-first-coding-surface
scope:
- confidence: cited
  path: spec/agentide/domains/surface.yaml
- confidence: inferred
  path: web/package.json
- confidence: inferred
  path: web/src/adapters
- confidence: inferred
  path: web/src/controller
- confidence: cited
  path: web/src/renderer
revision: 6
---
# Story: Own the hosted browser workbench

## Outcome

AgentIDE owns the complete browser workbench controller and renderer targets; products supply only an authenticated host port and select a target.

## Acceptance

- Renderer frame v2 replaces the unknown observation bag with typed explorer, editor, diff, chat, terminal, coordination, preparation, and refusal projections.
- A framework-neutral controller performs lazy observations and effects through a typed host port with no URL, bearer, storage, or product dependency.
- Vue and Vanilla share Monaco and Ghostty leaf adapters and render the same frames and actions.
- Draft bytes stay client-local; actor-private pane/open-file/cursor state remains durable through AgentIDE.
- Agent chat renders safe Markdown continuously while assistant deltas stream.
- Benchmarks and browser tests compare both targets and prove teardown, replay, themes, conflicts, and accessibility.

## Scope

- `spec/agentide/domains/surface.yaml` — cited
- `web/src/renderer` — cited
- `web/src/controller` — inferred
- `web/src/adapters` — inferred
- `web/package.json` — inferred
