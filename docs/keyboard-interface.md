# Keyboard interface

The console and browser surfaces share the same conceptual commands. A shortcut changes durable
state only by invoking its semantic intent.

| Key | Intent | Result |
|---|---|---|
| `I` | none | Edit and submit the next prompt to the native Harness loop. |
| `1` | none | Show the streamed agent transcript. |
| `2` | none | Show the focused virtual workbench pane. |
| `O` | `file_open` | Prompt for a workspace-relative path and open an editor pane. |
| `D` | `diff_show` | Open or focus a changes pane. |
| `Tab` | `pane_focus` | Focus the next virtual pane. |
| `X` | `pane_close` | Close the focused pane without changing its underlying file. |
| `R` | none | Refresh the current projection. |
| `Y` | approval | Grant the exact plan currently displayed by the Harness approval gate. |
| `N` / `Esc` | approval | Deny the exact plan; the effect does not happen. |
| `Up` / `Down` | none | Scroll the active agent or workbench view. |
| `Ctrl/⌘ K` | none | Open the browser command palette. |
| `Q` | none | Leave the TUI; the session remains active. |

Prompt and approval keys are present only in Harness mode, selected by passing both `--base-url`
and `--model`. The projection-only TUI keeps the original file, diff, focus, close, and quit keys.

Planned follow-on bindings can add process input, agent lanes, evidence jumps, and publication
without adding renderer-specific state. The intent profile already reserves those operations, and
unavailable ones are explicitly withheld by the default binding document.
