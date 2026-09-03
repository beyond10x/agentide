---
title: Harness-native TUI
description: Run the recommended local AgentIDE surface with a native Harness model loop and exact-plan approvals.
b10x:
  schema: b10x-doc-page/v1
  audiences: [evaluator, adopter, developer]
  experienceIds: [evaluate-beyond10x-products, build-agent-systems]
  support: preview
  access: public
---

# Harness-native TUI

The Harness mode of `agentide run` is the recommended console surface that composes the AgentIDE
specification with the native Harness agent loop. The lower-level `agentide tui` command attaches
the same surface to an existing session. Both keep four kinds of state separate:

```text
ESS command schema ──► Harness ToolSpec ──► model turn
                              │                 │
                              │          neutral LoopEvent stream
                              ▼                 ▼
                    exact-plan approval ──► terminal projection
                              │
                              ▼
                    AgentIDE IntentPort
```

AgentIDE owns the semantic intent, exact implementation plan, durable approval event, and replayable
workbench. Harness owns model turns, provider-wire projection, tool publication, budgets,
cancellation, and the call-time approval gate. The terminal is a renderer and decision surface; it
does not implement a third execution path.

## Start a session

Create an AgentIDE session for an existing workspace and open the Harness TUI in one command:

```shell-session
agentide run --workspace . --objective "Implement and verify the change" \
  --base-url https://api.example/v1 \
  --model model-id \
  --api-key-env MODEL_API_KEY
```

AgentIDE prints the durable session id and a projection-only resume command before opening the TUI
and again when the TUI exits. To reconnect the model loop later, use `agentide tui --session
<session-id>` with the same explicit connection and credential-source options.

The default wire is `openai-responses`. Use `--wire anthropic-messages` for a Messages-compatible
endpoint. The connection has no ambient credential fallback:

- `--api-key-env NAME` reads an API key from `NAME` for each request;
- `--oauth-token-env NAME` reads a user token from `NAME` for each request;
- `--oauth-token-file FILE --oauth-token-pointer /path` re-reads a token document for each request;
- naming none sends no credential header, which is useful only for an explicitly unauthenticated
  local gateway.

The TUI does not renew credentials in this first slice. Harness cancellation, turn retry,
compaction, tool-result bounds, and the declared `--max-turns` budget still apply.

## Work in the terminal

Press `i` to type a prompt. Harness streams visible text and neutral activity events while the
model works. The surface adapts instead of squeezing fixed columns: compact terminals retain the
canvas, composer, status, and approval capability; standard terminals add the explorer; wide
terminals also show activity and session context. Hidden regions remain available from `Ctrl+K`.

The interaction model is deliberately IDE-like:

- `Ctrl+K` opens the command palette and reports why unavailable actions are disabled;
- `Ctrl+P` accepts a workspace-relative path and filters the current open-file set;
- `Tab`/`Shift+Tab` moves among visible regions, while `[`/`]` changes the durable pane;
- `1` and `2` switch directly between the transcript and workbench;
- `?` opens the keymap generated from the same surface profile used by the renderer.

Editor panes have line numbers, independent horizontal and vertical scrolling, and a cursor-line
treatment. Diff panes preserve textual markers while styling headers, hunks, additions, and
deletions. The dark theme follows the browser workbench and has 256-color, 16-color, ASCII, and
`NO_COLOR` fallbacks.

When a required intent is requested, the loop pauses and the terminal displays:

- the semantic intent and model arguments;
- the Harness effects, risk, access, idempotency, and concrete subjects;
- the externally selected driver and operation;
- the exact plan digest and input digest.

Press `y` to grant that plan or `n`/`Esc` to deny it. The paired tool port refuses a required call
that did not pass through this handshake, even if another embedder accidentally invokes it
directly.

## Surface profile

`contracts/surface-profile.yaml` is the public `agentide.surface-profile/1` contract. It declares
semantic regions, viewport classes, interaction modes, action-to-intent references, key bindings,
theme roles, and terminal fallbacks. It does not define execution semantics: ESS commands remain
the authority, and every effectful UI action reaches the same intent path used by the model.

The contract loader rejects unknown fields, key collisions, unknown intent references, invisible
focus defaults, hidden regions without an overlay route, approval modes that can perform unrelated
actions, missing reduced-color roles, and glyphs without ASCII equivalents. The browser consumes a
deterministically generated TypeScript projection; regenerate it after profile changes with:

```shell-session
cargo xtask generate-surface-profile
```

The Harness conversation is in memory for the lifetime of this first TUI process. AgentIDE's
secret-free intent journal and virtual workbench remain durable. Persisted Harness conversation
resume and direct app-server injection are follow-on host adapters, not hidden behavior in this
surface.
