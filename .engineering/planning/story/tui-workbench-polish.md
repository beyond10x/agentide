---
format: aep.planning-md/1
id: story:tui-workbench-polish
kind: story
status: implemented
title: Adaptive Harness workbench TUI
summary: Deliver reducer-driven navigation, adaptive layout, branded rendering, structured panes, and consequence-complete approvals.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 4
---
## Outcome

The Harness-native terminal becomes an adaptive, keyboard-discoverable AgentIDE workbench driven by a deterministic reducer and a pure renderer over durable session state.

## Acceptance criteria

- `Ctrl+K`, `Ctrl+P`, region focus, pane cycling, prompt, help, refresh, close, and approval keys reduce to local state or released semantic intentions with no alternate execution path.
- Compact, standard, wide, and too-small terminal layouts remain usable and resize-safe.
- Transcript, editor, diff, activity, context, and approval widgets retain textual meaning without color or Unicode.
- Region scroll positions are independent; hidden regions have explicit overlay access.
- Approval shows the exact arguments and plan digests plus Harness effects, risk, access, idempotency, and subjects; denial is always reachable.

## Evidence required

- Reducer and approval safety tests.
- Ratatui test-backend matrices at 80x24, 120x32, and 180x50.
- Truecolor, 256-color, 16-color, ASCII, and `NO_COLOR` fallback checks.
- Browser type check/build and full repository gate.
