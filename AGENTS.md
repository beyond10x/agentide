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

This repository owns the public source and presentation allowlist in `b10x.docs.yaml`; the unified [beyond10x Website](https://beyond10x.github.io/docs/agentide/) passively collects those declared files from the exact commit in `website/sources.lock.json`. Atlas owns discovery grouping/order; Website and Docs System own rendering, shared components, search, and feeds. Do not add a standalone docs deployer or put App credentials in this public repository. If Atlas catalogs a former Pages workflow, that file remains repository-owned validation: preserve its bespoke checks while keeping exact read-only permissions, an unconditional pull-request trigger, and no deployment primitives. Project Pages at `/agentide/` is only the generated redirect façade in `.github/workflows/b10x-docs-pages.yml`.

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
