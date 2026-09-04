---
title: AgentIDE boundaries
description: Semantic, hosted, runtime, Harness, surface, and durable-state ownership boundaries.
b10x:
  schema: b10x-doc-page/v1
  audiences: [developer, operator]
  experienceIds: [evaluate-beyond10x-products, build-agent-systems]
  support: preview
  access: public
---

# AgentIDE boundaries

AgentIDE specifies what an agent needs during a coding session without embedding how a particular
machine, organization, or harness performs it.

## Semantic boundary

ESS commands are the semantic authority. An intent describes a desired result—read this file,
verify this change at the focused level, publish this reviewed revision—not an executable path or
provider call. The strict intent profile adds consequence metadata that ESS intentionally does not:
model visibility, implementation port, effect class, risk, and approval requirement.

The request body may contain semantic subjects such as a workspace-relative path, search pattern,
verification level, or process profile. The v2 profile declares human, agent, and automation
audiences; its compatibility loader normalizes the v1 `model` and `operator` exposure vocabulary
before policy evaluation. Request data may not contain:

- a driver or implementation name;
- executable paths or arbitrary arguments for a configured operation;
- credentials or environment variables;
- publication destinations;
- approval policy.

Those values enter through `agentide.bindings/1`, supplied by the operator or embedding host.

## Hosted composition and source authority

Hosted AgentIDE is a generated Service SDK package, not a handwritten `agentide-service` and not a
new file service. [`../service.yaml`](../service.yaml) compiles the ESS session semantics, runtime
obligations, authenticated projections, Connector contribution, Rust package, transport contracts,
and conformance scenarios into `generated/service/`. Deployments bind that generated factory to
Eventlog/PostgreSQL; a local composition may bind the same factory to Eventlog/SQLite.

Service SDK owns coordination persistence and delivery mechanics: session aggregates, optimistic
versions, idempotency, event streams, authenticated projections, and effect-journal recovery.
AgentIDE records opaque project, workspace-session, materialization, context-pin, grant, approval,
process, and evidence references. Context pins carry a Workspace selector and complete-content
digest, not a copied source buffer. It never stores project file bytes.

Workspace remains the single project/source authority already used by DevCenter's `/projects`
feature. It resolves Connector authority, pins an exact GitLab revision, and owns bounded tree/file
operations, immutable-base versus working-materialization diffs, and publication preparation.
Substrate owns the materialized bytes, guarded writes, processes, PTYs, confinement, and replay.
Connectors owns provider credentials, current GitLab access, grants, and live publication effects.
DevCenter and AgentIDE therefore use the same Workspace APIs and opaque Substrate references; there
is no browser-computed authoritative diff and no parallel AgentIDE file repository.

```text
DevCenter / AgentIDE clients
          |  session context, events, grants
          v
generated AgentIDE Service SDK factory ---- Eventlog (PostgreSQL or SQLite)
          |
          | opaque project/session selectors and governed operations
          v
Workspace ---- Connectors (GitLab authority/publication)
    |
    +---------- Substrate (base + working bytes, process, PTY)
```

## Runtime boundary

`agentide-core` owns planning, exact-plan approval, the event journal, and deterministic projection.
It calls the small `IntentPort` trait only after an intent has a binding and sufficient authority.
The port returns either an observation or a named refusal; it cannot silently select a weaker
implementation.

The standalone/local `agentide-substrate` port is implemented with `b10x-harness-substrate`, pinned to an
exact Harness revision. It adopts the existing checkout as a guarded Substrate workspace, reads and
writes only through guarded file operations, and executes argv-only operations only when Substrate
reports the required confinement facts. No shell command line is assembled.

## Harness embedding boundary

A Harness-native integration publishes the released model-visible intent schemas through a
`ToolPort` and binds their implementations at composition time. It can replace the standalone port
with workflow, collaboration, delivery, or deployment implementations while keeping these stable:

1. intent names and input schemas;
2. exact plan digest and approval semantics;
3. event envelopes and cursor ordering;
4. named refusal behavior;
5. snapshot and virtual-workbench projection.

Harness owns credentials, subjects, policies, tool publication, and lifecycle. AgentIDE owns no
Harness global state, and Harness does not need to depend on AgentIDE: generated contract adapters
can implement the same port from either side.

The first concrete composition is `agentide-harness`. It turns the bound model-visible subset of
the intent profile into a flat Harness `ToolPort`; each input schema comes from its generated ESS
command schema. The embedding host removes `session_id` and `request_id` from what the model may
supply and injects those values from the active session and Harness call instead. Consequence and
risk metadata map into Harness envelopes, while the qualified ESS command is recorded as the
operation reached by each call.

Required intents use paired ports. Harness asks its `ApprovalPort`; that port previews the call in
AgentIDE and shows the resulting plan digest. Approval durably grants that digest and leaves the
exact input waiting for the paired `ToolPort`; denial durably removes it. A required intent that
reaches the tool port without that handshake is refused as `harness.approval_missing`. Thus the TUI
does not introduce a second, weaker mutation path and Harness cannot execute a plan different from
the one the operator saw.

## Surface boundary

The browser, JSON CLI, and console TUI are renderers over `agentide.snapshot/1` and
`agentide.event/1`. File and pane operations are semantic intents, so a renderer does not keep the
authoritative list of open files or focused pane privately. Source contents are observations, not
session state, and are not written to the journal.

The browser has an additional explicit target boundary. A framework-neutral AgentIDE controller
converts host observations into an immutable `agentide.renderer-frame/2`, delivers ordered
transient `agentide.renderer-event/2` values, and handles `agentide.renderer-action/2` values
emitted by the target. Frame v2 has closed, typed tree, editor, diff, chat, terminal, preparation,
coordination, and refusal projections; the v1 arbitrary observation bag is not part of the current
contract. A target implements only `mount`, `update`, `deliver`, and `destroy`; it cannot know an
HTTP route, bearer, polling interval, WebSocket, storage key, or deployment.

Vanilla DOM and Vue are independent realizations of that same controller contract. Both accept the
same optional Monaco editor and Ghostty terminal leaf adapters. The adapters only translate render,
input, resize, and teardown calls; they never acquire transport, persistence, authorization, or
workspace ownership. Unsaved editor bytes remain in the browser controller until an explicit save,
including across snapshot refreshes and optimistic-concurrency refusals. Assistant Markdown is
escaped and rendered continuously as ordered deltas arrive; raw HTML and non-HTTP link schemes are
never activated.

The local host owns the loopback API binding. A hosted product such as DevCenter owns its Identity,
Workspace, Agent Platform, streaming, and terminal adapters. Both may select a renderer target, but
neither grants that target authority or exposes transport details through the renderer contract.

ESS describes renderer-neutral panes and workbench intentions. The separate, versioned
`agentide.surface-profile/1` contract describes how a surface makes those semantics reachable:
regions, adaptive viewport classes, interaction modes, keymaps, local actions, intent references,
semantic theme roles, and fallbacks. It is strict rather than an arbitrary widget-property bag.
AEP records the feature story and its evidence; it does not duplicate either semantic or visual
contracts.

The DevCenter workbench renders the hosted actor view natively. Actor-private pane, tab, cursor,
editor-buffer, and terminal-selection state does not enter another actor's view. Context pins and
explicit prompt attachments are shared. Unsaved editor bytes remain client-local. A model turn is
built from a freshly resolved `ActorView`; at most eight complete selections, 32 KiB each and 64 KiB
combined, may be injected, further capped at ten percent of the model context window.

Interactive terminals use Substrate's existing JSON WebSocket PTY protocol and deployment-declared
profiles. AgentIDE does not introduce a host shell or a second binary terminal protocol. Human open
requires an `interactive_terminal` grant; agents receive structured process intents rather than raw
keystrokes by default. Terminal metadata is durable, while raw scrollback reaches model context only
through an explicit bounded selection.

This separation allows a Harness app-server, terminal host, or model-native tool surface to inject
the projection directly without inheriting the standalone HTTP server or command-line parser.

The native TUI runs the synchronous Harness loop on a worker thread. A channel carries neutral
`LoopEvent` values to the terminal renderer and carries one approval decision back; the model loop
remains blocked until that decision arrives. The terminal owns no alternate tool semantics: file,
diff, focus, and close shortcuts invoke the same AgentIDE intents as the model-facing surface.
The TUI reducer is deterministic over key and resize events, and the renderer is a pure projection
of reducer state, the durable snapshot, and the validated surface profile.

## Durable state and sensitive data

Standalone session records are atomically stored outside the target workspace. Hosted records use
the generated Service SDK/Eventlog aggregate and authenticated projection. The journal carries plan
digests, outcomes, refusals, pane metadata, and evidence references. It must not contain secrets,
hidden model reasoning, raw model conversations, or copied source contents. Sanitized fixtures are
checked by the repository gate.
