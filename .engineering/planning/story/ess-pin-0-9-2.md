---
format: aep.planning-md/1
id: story:ess-pin-0-9-2
kind: story
status: draft
title: Advance the ESS pin from 0.9.2 to a current tag
summary: agentide-xtask takes ess-compiler, ess-domain and ess-realization at 0.9.2 while ESS's latest tag is 0.24.0; moving them is a release-shaped change, not a manifest edit.
tags:
- dependencies
- pins
relations:
- serves: vision:agent-first-coding-surface
revision: 1
---
## Acceptance

- `crates/agentide-xtask/Cargo.toml:21-23` takes `ess-compiler`, `ess-domain` and `ess-realization`
  at one current ESS tag (`0.24.0` or later), with `tag` and `version = "="` agreeing on that
  version.
- `cargo xtask gate` passes against the new pin, including its ESS validation and generated-byte
  drift steps.
- Every generated tree whose bytes the new ESS changes is regenerated through its own verb
  (`cargo xtask generate-realizations`, `cargo xtask generate-surface-profile`), not hand-edited.
- `AGENTS.md` § Pins no longer records a held `0.9.2`.

## Measured gap, 2026-09-15

`crates/agentide-xtask/Cargo.toml:21-23` pins `tag = "0.9.2", version = "=0.9.2"` three times.
ESS's latest tag is `0.24.0` (`git ls-remote --tags https://github.com/beyond10x/ess`), fifteen
minor versions later. Recorded by the beyond10x org-state review 2026-09-15-run2, lane
`10-long-tail` F7, ledger `ORG-0106`.

## Why this is release-shaped and was not done with the note

The bump rebuilds the gate against a different compiler and a different realization format, so it
carries generated-byte changes and an API delta across fifteen minor versions, and it ends in this
repository's own release. The fix pass that recorded the gap was documentation-only and explicitly
barred from pin bumps, so the gap is written down here rather than closed there.

## Scope

- `crates/agentide-xtask/Cargo.toml` (3 lines)
- whatever `cargo xtask generate-*` rewrites under the generated trees
- `AGENTS.md` § Pins
