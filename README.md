# AgentIDE

AgentIDE is an actor-aware coding-session protocol and client. It gives humans, agents, and
automation stable semantic intents such as `code_read`, `code_verify`, `file_open`, and
`code_publish`; a composing application supplies the concrete implementation, arguments,
credentials, destinations, and authority policy.

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

The hosted form is generated from [`service.yaml`](service.yaml) with Service SDK. Service SDK and
Eventlog provide its service layer, authenticated projections, event persistence, and deployment
store selection; AgentIDE does not introduce another PostgreSQL repository or a source-file store.
DevCenter is the primary hosted human surface, while the CLI, TUI, and local browser remain peer
clients.

## Choose how to run it

Start with the **model-backed local TUI** if you want to evaluate AgentIDE as an interactive coding
environment. The same Linux binary also provides a projection-only TUI, a JSON CLI for automation,
and a loopback browser workbench. A fifth realization composes AgentIDE into DevCenter, but that
hosted surface is approval-required and has no public self-service endpoint.

The exact distinctions—interaction shape, attachment boundary, support, availability, immutable
artifact, and ESS surface—are generated from two checked `ess-realization/1` declarations in
[`docs/running-modes.md`](docs/running-modes.md).

## Install the released Linux binary

The public release is the self-service path. Source builds currently need access to the private
Service SDK dependency used to generate the hosted package, so cloning the public repository alone
is not a complete build path.

```bash
version=0.1.1
target=x86_64-unknown-linux-gnu
archive="agentide-${version}-${target}.tar.gz"
base="https://github.com/beyond10x/agentide/releases/download/${version}"

curl --fail --location --remote-name "${base}/${archive}"
curl --fail --location --remote-name "${base}/SHA256SUMS"
grep -F "  ${archive}" SHA256SUMS | sha256sum --check
tar -xzf "${archive}"
cp "agentide-${version}-${target}/agentide" ./agentide
./agentide --version
```

Expected verification output:

```shell-session
agentide-0.1.1-x86_64-unknown-linux-gnu.tar.gz: OK
agentide 0.1.1
```

The checksum for that archive is also locked into the standalone ESS realization. AgentIDE
currently targets Linux because process execution relies on Linux Substrate confinement.

## Quickstart: model-backed local TUI

Run these commands from an existing workspace. Starting a session adopts that workspace through
Substrate and stores AgentIDE state outside it.

```bash
./agentide session start \
  --workspace . \
  --objective "Implement and verify the change"
```

The command returns JSON. Copy the printed `session_id` into `SESSION_ID`:

```json
{
  "format": "agentide.session-started/1",
  "session_id": "<session-id>",
  "workspace": "<absolute-workspace-path>",
  "next": "agentide snapshot --session <session-id>"
}
```

Configure a model endpoint and name the credential source. The value of `MODEL_API_KEY` is read for
each request; neither the value nor the model conversation is written to the AgentIDE journal.

```bash
export SESSION_ID="<session-id>"
export AGENTIDE_BASE_URL="https://api.example/v1"
export AGENTIDE_MODEL="model-id"
export MODEL_API_KEY="<credential>"

./agentide tui \
  --session "$SESSION_ID" \
  --api-key-env MODEL_API_KEY
```

You should see the session objective, transcript and workbench tabs, open files, activity, and any
plan waiting for approval. Press `i` to prompt the agent. `Ctrl+K` opens the command palette,
`Ctrl+P` quick-opens a file, `Tab` moves among regions, and `y` or `n` resolves the exact plan shown
by the approval gate. See the [Harness TUI guide](docs/harness-tui.md) for model wires, credential
sources, budgets, and the full interaction flow.

## Other local surfaces

Projection-only TUI—same durable workbench, no model connection:

```bash
./agentide tui --session "$SESSION_ID"
```

JSON CLI—suited to scripts and agent tool adapters:

```bash
./agentide snapshot --session "$SESSION_ID"
./agentide intent call --session "$SESSION_ID" code_read \
  --input '{"path":"src/lib.rs"}'
```

Local browser—start the loopback-only server, then open `http://127.0.0.1:7788/`:

```bash
./agentide serve --session "$SESSION_ID"
```

The browser server is a local projection and manual interaction surface. It does not run the
Harness model loop; use the model-backed TUI for that.

![AgentIDE local browser workbench showing a new session, open-file and approval regions, and the durable session timeline](docs/assets/browser-workbench.png)

The browser capture uses a fresh local session with no source files, credentials, or model
conversation loaded.

A mutating JSON intent is two-phase and bound to one SHA-256 plan:

```bash
./agentide intent preview --session "$SESSION_ID" code_edit \
  --input '{"path":"src/lib.rs","content":"...","expected_sha256":"..."}'
./agentide approval grant --session "$SESSION_ID" --plan "$PLAN_DIGEST"
./agentide intent resume --session "$SESSION_ID" --plan "$PLAN_DIGEST" \
  --input '{"path":"src/lib.rs","content":"...","expected_sha256":"..."}'
```

Resume must supply the exact input bytes used for preview; pending plans do not persist the original
input.

## Hosted DevCenter boundary

Hosted AgentIDE is a generated Service SDK package composed into DevCenter with authenticated
session projections and Eventlog persistence. It is not the loopback browser exposed on a public
host, and this repository does not publish a self-hosting chart or public hosted URL. The declared
DevCenter realization is an approval-required preview boundary; its current deployment surface is
paused. Use the released local binary unless you have been given a DevCenter environment and
access.

## Troubleshooting and limits

- `workspace.unreadable` means `--workspace` does not resolve to an existing accessible directory.
- `toolchain.unavailable` means the adopted workspace has no supported toolchain facts. Inspect
  `agentide bindings inspect --session "$SESSION_ID"` after session creation.
- `substrate.exec_refused` usually means a process intent needs confinement facts the host did not
  provide. Process verification requires a delegated cgroup v2 subtree selected by
  `AGENTIDE_CGROUP_ROOT`; AgentIDE does not fall back to an unconfined host process.
- `binding.profile_unavailable` means the semantic verification or process profile is absent from
  the operator-owned binding file. A model cannot add an executable or argv to repair it.
- `harness.approval_missing` means an effectful tool call did not pass through the paired exact-plan
  approval flow. Retry through the displayed `y`/`n` decision rather than bypassing it.
- If `tui` starts without a prompt line, both a model base URL and model id were absent; that is the
  projection-only mode, not a failed agent loop.

Session state lives under `${XDG_STATE_HOME:-$HOME/.local/state}/agentide`, never in the target
workspace. A model request cannot select a driver, executable, credential, destination, or policy.
Missing Substrate facts, bindings, profiles, approvals, or recovery facts produce named refusals;
AgentIDE never falls back to direct host effects.

## Contracts and embedding

- `spec/agentide/` is the ESS semantic authority.
- `contracts/intent-profile-v2.yaml` adds actor audiences, consequence, and authority declarations;
  the v1 profile remains a compatibility input.
- `contracts/surface-profile.yaml` defines strict, renderer-neutral presentation and interaction
  rules shared by the browser and console.
- `contracts/default-bindings.yaml` is the standalone Substrate binding supplied from outside the
  semantic request.
- `contracts/schemas/` contains immutable transport and configuration schemas.
- `service.yaml` and `service/` are the handwritten Service SDK package for hosted coordination;
  `generated/service/` is its exclusive generated output, including the Connector factory,
  transport contracts, Rust service package, and conformance scenarios.
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

After a handwritten ESS or Service SDK definition change, regenerate the exclusively owned hosted
package with `cargo xtask generate-service`. The generated Rust crate is a workspace member, so the
normal locked workspace build compiles the exact Connector factory and Eventlog service used by a
hosted composition; there is no parallel handwritten service implementation.

The gate validates AEP, compiles ESS through the pinned compiler, checks ESS and Service SDK output
drift, compiles and tests the generated service, validates actor/profile/binding coverage, runs Rust
tests and Clippy, type-checks the browser, checks built asset drift, and scans replay fixtures for
sensitive data.

<!-- b10x-docs:start -->
## Documentation

[AgentIDE documentation](https://beyond10x.github.io/docs/agentide/) · [Start](https://beyond10x.github.io/) · [Ecosystem](https://beyond10x.github.io/ecosystem/) · [Impact](https://beyond10x.github.io/changes/) · [Releases](https://beyond10x.github.io/releases/)
<!-- b10x-docs:end -->
