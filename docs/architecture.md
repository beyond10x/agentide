# AgentIDE boundaries

AgentIDE specifies what an agent needs during a coding session without embedding how a particular
machine, organization, or harness performs it.

## Semantic boundary

ESS commands are the semantic authority. An intent describes a desired result—read this file,
verify this change at the focused level, publish this reviewed revision—not an executable path or
provider call. The strict intent profile adds consequence metadata that ESS intentionally does not:
model visibility, implementation port, effect class, risk, and approval requirement.

The request body may contain semantic subjects such as a workspace-relative path, search pattern,
verification level, or process profile. It may not contain:

- a driver or implementation name;
- executable paths or arbitrary arguments for a configured operation;
- credentials or environment variables;
- publication destinations;
- approval policy.

Those values enter through `agentide.bindings/1`, supplied by the operator or embedding host.

## Runtime boundary

`agentide-core` owns planning, exact-plan approval, the event journal, and deterministic projection.
It calls the small `IntentPort` trait only after an intent has a binding and sufficient authority.
The port returns either an observation or a named refusal; it cannot silently select a weaker
implementation.

The standalone `agentide-substrate` port is implemented with `b10x-harness-substrate`, pinned to an
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

ESS describes renderer-neutral panes and workbench intentions. The separate, versioned
`agentide.surface-profile/1` contract describes how a surface makes those semantics reachable:
regions, adaptive viewport classes, interaction modes, keymaps, local actions, intent references,
semantic theme roles, and fallbacks. It is strict rather than an arbitrary widget-property bag.
AEP records the feature story and its evidence; it does not duplicate either semantic or visual
contracts.

This separation allows a Harness app-server, terminal host, or model-native tool surface to inject
the projection directly without inheriting the standalone HTTP server or command-line parser.

The native TUI runs the synchronous Harness loop on a worker thread. A channel carries neutral
`LoopEvent` values to the terminal renderer and carries one approval decision back; the model loop
remains blocked until that decision arrives. The terminal owns no alternate tool semantics: file,
diff, focus, and close shortcuts invoke the same AgentIDE intents as the model-facing surface.
The TUI reducer is deterministic over key and resize events, and the renderer is a pure projection
of reducer state, the durable snapshot, and the validated surface profile.

## Durable state and sensitive data

Session records are atomically stored outside the target workspace. The journal carries plan
digests, outcomes, refusals, pane metadata, and evidence references. It must not contain secrets,
hidden model reasoning, raw model conversations, or copied source contents. Sanitized fixtures are
checked by the repository gate.
