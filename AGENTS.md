# AGENTS.md — agentide

## Serves

- **O1 — governed reach.** AgentIDE exposes semantic coding intents and delegates every effect to a
  capability-bearing implementation port.
- **O2 — decisions as data, with evidence.** Plans, approvals, refusals, outcomes, and evidence are
  durable typed events.
- **O3 — any harness, observed and compared.** The standalone and Harness surfaces share one intent
  catalogue and conformance suite.

## Boundaries

- ESS commands are semantic authority. Generated contracts and UI projections are not authority.
- A model request never chooses an implementation, executable, credential, destination, or policy.
- Mutations are previewed and durably recorded before dispatch. Approval names the exact plan digest.
- Missing capability is a named refusal. Never fall back from Substrate to direct host effects.
- Session state lives outside the target workspace and contains no secret values.
- AgentIDE composes Harness; Harness never depends on AgentIDE.
- Released contract bundle directories are immutable. Breaking changes add a successor.
- Command-line, server, storage, authority, and execution code is Rust. Browser UI code may be
  TypeScript and is embedded as built assets in the Rust binary.

## Public audience

- Write `README.md` for evaluators and adopters. Lead with installation and the first successful
  run, explain ESS in user-benefit terms, and link to technical internals instead of putting them in
  the onboarding path.
- `agentide run` is the primary local entrypoint. It creates a recoverable session before opening
  the workbench, keeps model endpoint and model selection explicit, and never discovers or persists
  credentials.
- The release installer defaults to the latest stable GitHub Release, verifies the matching archive
  checksum, supports only declared platforms, installs without sudo, and never embeds a version in
  the public one-line command. The Cargo alternative intentionally follows current `main` source.

## Repository operations

Use managed worktrees. Commit and push through the private Atlas bot wrapper. Never add credentials
or bot-authenticated wrappers to this public repository.

This repository is a satellite leaf. It owns its source, gate, tag, GitHub release, and post-release
realization commit, then hands exact published coordinates to an Atlas-based coordinator. Do not
mutate Atlas, Website source locks, documentation snapshots, or façade delivery from this leaf.

## Pins

There are **two** ESS pins in this repository and they are deliberately at different versions.

**The ESS libraries are pinned at `0.24.0`, the current tag.** `crates/agentide-xtask/Cargo.toml:21-23`
take `ess-compiler`, `ess-domain` and `ess-realization` at `tag = "0.24.0", version = "=0.24.0"`
(ESS's latest tag on 2026-09-15). The surface this crate uses — `ess_compiler::source::SourceMap`,
`ess_domain::spec::{RawSpecFile, Specification}`, `ess_domain::system::Source`,
`ess_compiler::resolve::compile_locating`, `EssIr`, `ess_realization::{RealizationSpec, compile}` —
is unchanged from `0.9.2`, so the bump needed no source edit and `Cargo.lock` moved only those three
packages' version and source lines. `cargo check -p agentide-xtask --locked` is green against it, and
so are the gate's `validate_ess` and `validate_realizations`: `EssIr::to_canonical_json()` from
`0.24.0` still equals `generated/ess/ir.json` byte for byte, and `docs/running-modes.md` still
regenerates byte-identically (`ess-realization/1` is still accepted beside the newer
`ess-realization/2`).

**The ESS *CLI* is deliberately held at `0.9.2`** — `.github/workflows/ci.yml:49` installs it, and
`validate_generated_ess` shells out to it (`AGENTIDE_ESS_BIN`, else `ess` on `PATH`) rather than to
the pinned library. What moving that pin costs was measured on 2026-09-15 by regenerating
`spec/agentide` with ESS CLI `0.23.0` into a scratch tree and comparing it to `generated/ess`:

| what | result |
|---|---|
| `generated/ess/ir.json` and 4 other files | byte-identical |
| the other **125** files | differ **only** in the contract digest, `<hex>` → `slice-sha256/2:<hex>` |
| `.ess-output/state.json` | **new** — a publication ledger the newer CLI writes beside its output, with no flag to suppress it. **Handled:** `read_tree` (`crates/agentide-xtask/src/main.rs:1287`) now skips dot-prefixed entries, under two unit tests. None of the four trees it reads holds a dot-prefixed artifact, so no committed byte moved. |

Nothing structural is left. Moving the CLI tag needs `generated/ess` regenerated with the **same** tag
the workflow installs — 125 files, the digest field only, through the CLI and never by hand — and a
complete `cargo xtask gate` green with both pins at once. Tracked by `story:ess-pin-0-9-2` in
`.engineering/planning/`. Do not move the CLI tag outside that story, and keep the two pins'
relationship written down here when either one moves.

## Gate

```console
cargo xtask gate
```

The gate validates AEP, ESS, contract/profile agreement, Rust, the browser build, fixture redaction,
and generated-byte drift. Read the command's own exit status. Before declaring a public release
complete, also verify the latest-release installer and the unpinned anonymous Cargo installation.

Regenerate exclusively owned output through `cargo xtask generate-service`, `cargo xtask
generate-realizations`, or `cargo xtask generate-surface-profile` after changing its corresponding
source declaration. Never hand-edit those generated trees.

## Releases

Tags are bare SemVer. Cut an annotated tag only from fully gated `main`; publish the checksummed
single binary and verify the GitHub Release is authored by `b10x-bot[bot]`.

Keep realization declarations authoritative for running-mode documentation. Release preparation may
retain the previous immutable binary artifact. After the new GitHub Release exists, promote its
archive URL and exact digest in a separate commit and regenerate the realization reference.

<!-- b10x-docs-operations:start -->
## Public documentation operations

This repository owns the public source and presentation allowlist in `b10x.docs.yaml`. The generated credential-free `.github/workflows/b10x-docs-bundle.yml` passively packages only those declared files for the exact successful `main` commit; it must never run repository code. Atlas selects the latest successful bundle with every other catalog source, and Website plus Docs System own rendering, shared components, search, and feeds. Do not add a standalone docs deployer or put App credentials in this public repository. If Atlas catalogs a former Pages workflow, that file remains repository-owned validation: preserve its bespoke checks while keeping exact read-only permissions, an unconditional pull-request trigger, and no deployment primitives. Project Pages at `/agentide/` is only the generated stable redirect façade in `.github/workflows/b10x-docs-pages.yml`; content-only publication never rebuilds it.

From the complete organization workspace, verify the contract with a clean Atlas checkout at the current remote `main`. Set `B10X_ATLAS_CHECKOUT` to a managed Atlas worktree when the primary checkout is dirty or stale; never infer command availability from the primary alone.

```bash
atlas_checkout="${B10X_ATLAS_CHECKOUT:-atlas}"
atlas_head="$(git -C "$atlas_checkout" rev-parse HEAD)"
atlas_main="$(git -C "$atlas_checkout" ls-remote origin refs/heads/main | awk '{print $1}')"
test -z "$(git -C "$atlas_checkout" status --porcelain)"
test "$atlas_head" = "$atlas_main"
cargo run --manifest-path "$atlas_checkout/Cargo.toml" --locked -q -- \
  --store "$atlas_checkout/catalog/store" docs reconcile --workspace . --check
```

Keep internal plans, stories, ADRs, decisions, worklogs, security material, and research out of the public allowlist unless a repository authority explicitly declares them public.
<!-- b10x-docs-operations:end -->

<!-- b10x-release-operations:start -->
## Release completion

An ordinary release completes after this repository's exact tag, required source checks,
published release and required artifacts are verified. A pushed tag with unfinished checks or
uploads is queued; report it as released only after those requirements succeed.

Atlas reconciliation and public documentation publication run asynchronously. Do not wait for
Atlas or Website, update Website source locks or bootstrap snapshots, promote consumer pins,
release shared docs tooling, or redeploy documentation façades as part of an ordinary source
release. Report documentation as pending unless its publication was actually verified. A background
documentation failure does not invalidate a successful source release.

Keep this repository's provenance, correctness, security, compatibility and artifact verification
requirements. Shared rendering, routing or delivery-control changes still require their relevant
integration gates. A release request does not authorize deployment or downstream releases.
Repositories without a release unit retain their existing publication policy. This completion
boundary supersedes older instructions that attach synchronous documentation ceremony to each
source release.
<!-- b10x-release-operations:end -->
