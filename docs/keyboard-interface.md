# Keyboard interface

The console and browser surfaces share the same conceptual commands. A shortcut changes durable
state only by invoking its semantic intent.

| Key | Intent | Result |
|---|---|---|
| `O` | `file_open` | Prompt for a workspace-relative path and open an editor pane. |
| `D` | `diff_show` | Open or focus a changes pane. |
| `Tab` | `pane_focus` | Focus the next virtual pane. |
| `X` | `pane_close` | Close the focused pane without changing its underlying file. |
| `R` | none | Refresh the current projection. |
| `Ctrl/⌘ K` | none | Open the browser command palette. |
| `Q` | none | Leave the TUI; the session remains active. |

Planned follow-on bindings can add terminal input, agent lanes, process cancellation, evidence
jumps, and publication without adding renderer-specific state. The intent profile already reserves
those operations, and unavailable ones are explicitly withheld by the default binding document.
