---
title: Keyboard interface
description: Keys shared by the AgentIDE console and browser workbench projections.
b10x:
  schema: b10x-doc-page/v1
  audiences: [adopter, developer]
  experienceIds: [build-agent-systems]
  support: preview
  access: public
---

# Keyboard interface

The console and browser surfaces share the same conceptual commands. A shortcut changes durable
state only by invoking its semantic intent.

| Key | Intent | Result |
|---|---|---|
| `I` | none | Edit and submit the next prompt to the native Harness loop. |
| `1` | none | Show the streamed agent transcript. |
| `2` | none | Show the focused virtual workbench pane. |
| `Ctrl+K` | none | Open the profile-defined command palette. |
| `Ctrl+P` | `file_open` | Quick-open a workspace-relative path or an already open file. |
| `D` | `diff_show` | Open or focus a changes pane. |
| `Tab` / `Shift+Tab` | none | Move focus among regions visible in the current viewport. |
| `[` / `]` | `pane_focus` | Focus the previous or next durable virtual pane. |
| `X` | `pane_close` | Close the focused pane without changing its underlying file. |
| `R` | none | Refresh the current projection. |
| `?` | none | Open the keymap help overlay. |
| `Y` | approval | Grant the exact plan currently displayed by the Harness approval gate. |
| `N` / `Esc` | approval | Deny the exact plan; the effect does not happen. |
| Arrows | none | Navigate a modal or scroll the independently focused region. |
| `Q` | none | Leave the TUI; the session remains active. |

Prompt and approval keys are present only in Harness mode, selected by passing both `--base-url`
and `--model`. The projection-only TUI keeps the original file, diff, focus, close, and quit keys.

The public `agentide.surface-profile/1` contract is the source for TUI modes and key bindings.
Commands which affect the coding session reference released AgentIDE intents; local commands only
change renderer focus, overlays, or input state. Planned follow-on bindings can add process input,
agent lanes, evidence jumps, and publication without adding a new execution path.
