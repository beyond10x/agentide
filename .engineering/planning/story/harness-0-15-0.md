---
format: aep.planning-md/1
id: story:harness-0-15-0
kind: story
status: implemented
title: Move to harness 0.15.0 (Substrate 0.7.17)
summary: Pin harness 0.15.0, build the model port through b10x-harness-llm over llm 0.8.0, take Registry from tools; drops the four deleted harness wire and credential crates and Substrate 0.5.0.
tags:
- dependencies
- pins
relations:
- serves: vision:agent-first-coding-surface
revision: 5
---
## Acceptance

AgentIDE builds against harness `0.15.0` (Substrate `0.7.17`) and reaches its model only through
harness's public llm adapter, with the behaviour of `agentide run` / `agentide tui` unchanged:

- `Cargo.toml` pins every `b10x-harness-*` crate at the `0.15.0` tag's commit with
  `version = "=0.15.0"`, and pins no `b10x-harness-credential`, `-messages`, `-responses`, `-http`
  or `-toolchain`.
- `crates/agentide-substrate` takes `Registry` from the `tools` repository's toolchain crate through
  `b10x-harness-substrate`'s re-export, and pins no `tools` coordinate of its own, so `Cargo.lock`
  holds one copy of that crate and `Toolchain::from_providers` receives the type it was compiled
  against.
- `Cargo.lock` resolves `b10x-substrate-*` at `0.7.17` and `b10x-llm-*` at `0.8.0`, and no Substrate
  below `0.7.13`.
- `crates/agentide-harness` builds a `ModelPort` through `b10x-harness-llm` for both wires, with the
  four credential sources `docs/harness-tui.md:52-56` documents: none, an API key read from a named
  variable for each request, an OAuth token read from a named variable for each request, and an OAuth
  token document re-read for each request at an optional JSON pointer. The API key source is
  `Source::resolver` over `llm_credentials::environment::EnvironmentResolver` with
  `Presentation::ApiKey`, so the variable is still read on every request.
- `cargo xtask gate` exits 0 on the branch.

## Measured, 2026-10-20

| what | where |
|---|---|
| 7 harness pins at rev `0f2edfef` (`=0.11.1`) | `Cargo.toml:29-35` |
| imports of the deleted crates | `crates/agentide-harness/src/lib.rs:17-20` |
| Responses and Messages clients built from the deleted crates | `crates/agentide-harness/src/lib.rs:104-128` |
| OAuth file source from `b10x-harness-credential` | `crates/agentide-harness/src/lib.rs:174` |
| deleted crates as dependencies | `crates/agentide-harness/Cargo.toml:17,19,20` |
| Substrate in the lock | `b10x-substrate-host`, `b10x-substrate-wire` `0.5.0` |
| every other harness name AgentIDE imports (`harness-loop`, `-wire`, `-substrate`, `-toolchain`) | still exported on harness `origin/main` `0.14.1` |
| `b10x_harness_toolchain::Registry` | `crates/agentide-substrate/src/lib.rs:12,37`; harness's tools-1 wave deletes the crate, `Registry` moves to `tools` `crates/tool-toolchain/src/lib.rs:353` with the same `builtins()` and `resolve(&Path, Option<&[String]>)` |

harness `0.14.0` deleted the four wire and credential crates; model calls go through llm `0.8.0`.
harness `0.15.0` publishes its llm adapter as the library crate `b10x-harness-llm`, so this move swaps
imports and writes no adapter (Atlas ADR 0097, Consequences: each consumer moves in its own wave).

## Scope

- `Cargo.toml`, `Cargo.lock`
- `crates/agentide-harness/Cargo.toml`, `crates/agentide-harness/src/lib.rs`
- `crates/agentide-substrate/Cargo.toml` (drops `b10x-harness-toolchain`),
  `crates/agentide-substrate/src/lib.rs` (one import)
- `crates/agentide-cli/src/harness_tui.rs` only if the connect error type changes
- `CHANGELOG.md`

## Release

agent-platform pins agentide `0.3.5`, so the merge ends in an agentide release through § Releases of
`AGENTS.md`. The release is `0.3.8`: `CHANGELOG.md` and `Cargo.toml` already name `0.3.6` and `0.3.7`,
and neither was tagged or released (`gh release list`: newest `0.3.5`).
