# Changelog

## Unreleased

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
