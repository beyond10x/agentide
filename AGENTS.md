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

## Repository operations

Use managed worktrees. Commit and push through the private Atlas bot wrapper. Never add credentials
or bot-authenticated wrappers to this public repository.

## Gate

```console
cargo xtask gate
```

The gate validates AEP, ESS, contract/profile agreement, Rust, the browser build, fixture redaction,
and generated-byte drift. Read the command's own exit status.

## Releases

Tags are bare SemVer. Cut an annotated tag only from fully gated `main`; publish the checksummed
single binary and verify the GitHub Release is authored by `b10x-bot[bot]`.
