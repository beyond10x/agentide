# AgentIDE

AgentIDE is an agent-first coding-session interface. It gives a model stable semantic intents such
as `code_read`, `code_verify`, `file_open`, and `code_publish`; an embedding application supplies
the concrete implementation, arguments, credentials, destinations, and authority policy.

```text
agent / browser / TUI
          │
          ▼
 typed semantic intent ──► exact plan ──► authority ──► implementation port
          │                                      │                │
          └──────────────── durable event journal ◄────────────────┘
                                  │
                     shared session projection
```

The released binary includes three surfaces over that projection:

- a JSON CLI optimized for agent tool use;
- an embedded local browser workbench;
- a keyboard-driven console TUI built on the same Harness/Substrate execution adapter.

Virtual panes, open files, focus, cursor positions, diffs, approvals, processes, agent lanes, and
evidence are session state rather than UI-private state. They can be replayed and rendered by a
future Harness-native surface without changing the intent vocabulary.

## Build and run

AgentIDE currently targets Linux because its standalone effects use Substrate confinement.

```shell-session
cargo build --locked --release -p agentide-cli
target/release/agentide session start --workspace . --objective "Implement the change"
target/release/agentide snapshot --session <session-id>
target/release/agentide tui --session <session-id>
target/release/agentide serve --session <session-id>
```

Without model options, `tui` is a projection-only workbench. Add a model connection to run the
native Harness loop inside the same terminal surface:

```shell-session
target/release/agentide tui --session <session-id> \
  --base-url https://api.example/v1 \
  --model model-id \
  --api-key-env MODEL_API_KEY
```

The credential value is read from the named source for each request and is never stored in the
connection configuration or AgentIDE journal. `--wire anthropic-messages` selects Harness's second
provider projection; `--oauth-token-env` and `--oauth-token-file` select user-bound token sources.

In Harness mode, `Ctrl+K` opens the command palette, `Ctrl+P` quick-opens a file, `i` opens the
prompt line, `Tab` moves among visible regions, `[`/`]` changes the durable pane, and `y`/`n`
resolves an exact-plan approval. The adaptive terminal surface shares its theme and interaction
profile with the browser workbench. See the [Harness TUI guide](docs/harness-tui.md) for the full
execution, navigation, and authority flow.

An agent invokes authority-free observations directly:

```shell-session
agentide intent call --session "$SESSION" code_read --input '{"path":"src/lib.rs"}'
agentide intent call --session "$SESSION" file_open --input '{"path":"src/lib.rs","line":42}'
```

A mutating intent is two-phase and bound to its exact SHA-256 plan:

```shell-session
agentide intent preview --session "$SESSION" code_edit \
  --input '{"path":"src/lib.rs","content":"...","expected_sha256":"..."}'
agentide approval grant --session "$SESSION" --plan "$PLAN_DIGEST"
agentide intent resume --session "$SESSION" --plan "$PLAN_DIGEST" \
  --input '{"path":"src/lib.rs","content":"...","expected_sha256":"..."}'
```

The original input is not persisted with a pending plan; resume must supply matching bytes. Session
state lives under `${XDG_STATE_HOME:-$HOME/.local/state}/agentide`, never in the target workspace.
A model request cannot select a driver or executable. Missing Substrate facts, bindings,
profiles, approvals, or recovery facts produce named refusals; AgentIDE never falls back to direct
host effects.

## Contracts and embedding

- `spec/agentide/` is the ESS semantic authority.
- `contracts/intent-profile.yaml` adds model-facing consequence and binding declarations.
- `contracts/surface-profile.yaml` defines strict, renderer-neutral presentation and interaction
  rules shared by the browser and console.
- `contracts/default-bindings.yaml` is the standalone Substrate binding supplied from outside the
  semantic request.
- `contracts/schemas/` contains immutable v1 transport and configuration schemas.
- `agentide_core::IntentPort` is the implementation seam a standalone or Harness host binds.
- `agentide_harness::ports` publishes the bound ESS schemas through Harness and pairs the tool port
  with the exact-plan approval port used by the TUI.
- `.engineering/planning/` is the AEP-governed plan and evidence graph.

See [architecture](docs/architecture.md) for the exact boundaries and [keyboard interface](docs/keyboard-interface.md)
for the shared surface operations.

Run the complete gate with:

```shell-session
cargo xtask gate
```

The gate validates AEP, compiles ESS through the pinned compiler, checks generated IR drift,
validates profile/binding coverage, runs Rust tests and Clippy, type-checks the browser, checks built
asset drift, and scans replay fixtures for sensitive data.

<!-- b10x-docs:start -->
## Documentation

[AgentIDE documentation](https://beyond10x.github.io/docs/agentide/) · [Start](https://beyond10x.github.io/) · [Ecosystem](https://beyond10x.github.io/ecosystem/) · [Impact](https://beyond10x.github.io/changes/) · [Releases](https://beyond10x.github.io/releases/)
<!-- b10x-docs:end -->
