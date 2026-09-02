---
format: aep.planning-md/1
id: story:tui-surface-profile
kind: story
status: implemented
title: Versioned AgentIDE surface profile
summary: Define and validate the shared adaptive layout, interaction, keymap, and terminal fallback contract.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 4
---
## Outcome

A strict public `agentide.surface-profile/1` contract makes renderer regions, viewport classes, interaction modes, semantic action references, key bindings, theme roles, and terminal fallbacks explicit without moving execution semantics out of ESS.

## Acceptance criteria

- Unknown fields, duplicate identifiers or key chords, unknown intents, unsafe approval actions, unreachable hidden regions, invalid focus defaults, malformed colors, and missing ASCII/reduced-color fallbacks are refused.
- The console and browser consume deterministic projections of the same checked-in profile.
- Compact, standard, and wide viewports retain canvas, composer, status, and exact-plan approval access.
- Presentation fields remain outside ESS; AEP contains evidence rather than a duplicate widget schema.

## Evidence required

- Contract positive and negative tests.
- TUI/browser generated-profile drift check.
- Repository gate result.
