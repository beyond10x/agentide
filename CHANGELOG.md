# Changelog

## Unreleased

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
