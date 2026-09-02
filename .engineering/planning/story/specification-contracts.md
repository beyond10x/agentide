---
format: aep.planning-md/1
id: story:specification-contracts
kind: story
status: implemented
title: Author executable intent contracts
summary: Create the ESS system, strict intent profile, generated contracts, and conformance fixtures.
relations:
- decomposes: epic:agentide-v1
- serves: vision:agent-first-coding-surface
revision: 4
---
## Acceptance

- ESS validates and compiles deterministically.
- Every exported command has exactly one intent-profile entry with an envelope, subjects, exposure, and implementation port.
- Generated contracts reproduce byte-for-byte and are never hand-edited.
- Adapter options and secret material are absent from model-visible schemas.
