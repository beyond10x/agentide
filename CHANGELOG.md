# Changelog

## Unreleased

## 0.3.6 - 2026-09-05

- Regenerate the hosted service with Service SDK 0.5.10 so its Connector factory composes with
  the Connectors 0.6.4 Smart Git runtime and shares the exact service dependency identity.

## 0.3.5 - 2026-09-05

- Regenerate the hosted service with Service SDK 0.5.9 so its Connector factory composes through
  the exact Connectors 0.6.2 Git broker and current authority contract.

## 0.3.4 - 2026-09-05

- Add a transport-neutral workbench controller and equivalent Vue and Vanilla renderer targets
  with shared Monaco editor and Ghostty terminal leaf adapters.
- Stream safely rendered Markdown in agent chat and replace untyped observations with versioned,
  typed workbench frames, events, and semantic actions.
- Generate actor-private, event-sourced workbench operations from ESS so a hosted product can
  restore panes, open files, cursor state, and diffs without browser-owned persistence.

## 0.3.3 - 2026-09-04

- Isolate pinned AEP and ESS command installation from restored Cargo binaries so the release gate
  deterministically publishes the existing AgentIDE renderer and Service SDK 0.5.6 / Connectors
  0.5.11 generated-service update.

## 0.3.2 - 2026-09-04

- Regenerate the hosted Connector-only service with Service SDK 0.5.6 so AgentIDE composes through
  the same Connectors 0.5.11 authority-refresh and factory contract as Devcenter without acquiring
  an unrelated standalone HTTP host.

## 0.3.1 - 2026-09-03

- Migrate the generated Connector-only hosted service to Service SDK 0.4.2 with explicit operation
  scopes and the breaking `service-definition/3` contract.

## 0.3.0 - 2026-09-03

- Add a versioned, transport-neutral browser renderer protocol with immutable frames, transient
  host events, typed semantic actions, and discoverable target manifests.
- Split the embedded browser host from presentation, ship feature-equivalent Vanilla DOM and Vue
  targets at explicit comparison routes, and refuse transport or persistence APIs inside targets.
- Add shared renderer conformance tests and reproducible Chromium/build benchmarks so framework
  choices can be based on measured bundle, startup, update, memory, install, and build costs.
## 0.2.1 - 2026-09-03

- Add an explicit provenance-free `ContextSelectionDraft` contract for interactive renderers and
  one AgentIDE-owned transition that seals complete bytes with a server-derived actor, canonical
  authority source, immutable source revision, and observation time.
- Publish and gate a standalone JSON Schema and golden vector for the renderer-to-hosted-runtime
  attachment handoff so DevCenter and future clients cannot silently drift from model context.

## 0.2.0 - 2026-09-03

- Promote the published `0.1.4` Linux archive and its exact SHA-256 digest into the standalone
  realization declaration.
- Publish JSON Schemas and golden vectors for the hosted actor, context, inventory, grant, diff,
  file, tree, and terminal contracts, and verify their JSON Schema and semantic conformance in the
  repository gate.
- Canonically seal context packs, actor-specific intent inventories, and server-resolved diff
  projections with reproducible SHA-256 digests; reject changed attachment bytes, duplicate tools,
  inconsistent diff summaries, invalid provenance, and contradictory pagination or lifecycle state
  with stable refusal codes.
- Separate coordination, context, and inventory revisions in `ActorView`, bind authority grants to
  their session at both inventory resolution and dispatch, and validate delegated grant
  intersections without widening intent, path, risk, or expiry.
- Define typed terminal control, lifecycle, replay, and server-frame contracts plus the exact
  eight-byte network-order sequence prefix and 4 MiB replay bound used by hosted PTY transport.
- Replace renderer-oriented raw JSON records with typed workbench panes, durable context records,
  file modification state, diff hunks/lines/status, terminal workspace access, and tree entry kinds.
- Pin Harness 0.11.1 and Service SDK 0.3.4 so hosted consumers use released per-turn environment
  refresh, restart-safe approval checkpoints, and the current generated service layer.

## 0.1.4 - 2026-09-03

- Promote the published `0.1.3` Linux archive and its exact SHA-256 digest into the standalone
  realization declaration.
- Add `agentide run` to create a durable session and open either the model-backed or projection-only
  TUI in one command while preserving the session and an explicit resume command on exit.
- Ship a checksum-verifying latest-release installer for Linux x86_64 and document an unpinned
  Cargo alternative for adopters who intentionally want the latest source from `main`.
- Consume public Service SDK 0.3.4, remove private dependency credentials from CI and releases, and
  make anonymous source installation a supported path.
- Rewrite the README for evaluators and adopters around installation, first run, running modes,
  ESS-derived safety, and links to deeper architecture and contract references.

## 0.1.3 - 2026-09-03

- Add deterministic, retry-safe hosted-session creation keyed by the Workspace session identity.

## 0.1.2 - 2026-09-03

- Consume the ESS realization compiler and CI tool from the published `0.9.2` tag.
- Declare the standalone Linux and hosted DevCenter realizations as typed ESS Realization IR,
  validate them against the exact compiled AgentIDE specification, and generate the running-mode
  reference from those declarations.
- Rework the public quickstart around the released, checksum-verified Linux binary; lead with the
  model-backed local TUI, distinguish the projection-only CLI, TUI, and browser surfaces, and state
  the approval-required hosted boundary and operational prerequisites explicitly.
- Regenerate the hosted coordination service with Service SDK 0.3.2 and its exact Connectors 0.5.3
  factory contract so downstream composed runtimes use one Connector service trait.
- Include lifecycle state in the generated session snapshot so Workspace can distinguish active
  coding sessions from closed sessions when deriving terminal and agent authority.
- Let hosted runtimes convert a digest-sealed, actor-specific `IntentInventory` through the
  AgentIDE-owned Harness adapter, preserving generated command schemas, consequence envelopes, and
  bounded-grant versus exact-plan approval posture without reconstructing the catalogue downstream.
- Add `agentide.intent-profile/2`, renderer-neutral actor, context, inventory, grant, file, canonical
  diff, and terminal contracts, plus compatibility normalization for the v1 exposure model.
- Generate the hosted coordination service from Service SDK instead of adding an AgentIDE-specific
  database or service repository. The package includes authenticated session projections, Eventlog
  persistence, a Connector factory, public contracts, and executable conformance scenarios.
- Extend that generated session aggregate with nested authority grants, Workspace-backed context
  references, and exact approval-checkpoint records. Their authenticated projections and lifecycle
  events use the same Eventlog stream; no source buffer or alternate file store is introduced.
- Add exact create/delete/rename and interactive-terminal semantics, bounded context injection,
  actor-specific tool inventory resolution, confined path grants, expiry/revocation checks, and
  delegated-grant intersection.

## 0.1.1 - 2026-09-02

- Fix the embedded browser workbench asset routing so its stylesheet, JavaScript, and generated
  surface profile load instead of returning 404 responses.
- Add the first native Harness-driven TUI session: ESS-derived AgentIDE intents are published as
  Harness tools, model output and tool activity stream into the workbench, and a `y`/`n` decision
  grants or denies the exact durable AgentIDE plan before a required intent can execute.
- Add a reusable `agentide-harness` composition crate with named credential sources, both Harness
  provider wires, host-bound session/request fields, consequence envelopes, and a paired tool and
  approval port that refuses required intents if the approval half is bypassed.

## 0.1.0 - 2026-09-02

- Define the ESS-owned AgentIDE intent catalogue and strict safety profile.
- Add a journaled Rust engine, managed session state, preview-bound approvals, and deterministic replay.
- Add the Harness/Substrate-backed standalone binding and single-binary CLI, browser, and console
  TUI workbench.
- Make virtual panes, open files, cursor state, diffs, approvals, and timelines replayable session
  projections shared by every renderer.
