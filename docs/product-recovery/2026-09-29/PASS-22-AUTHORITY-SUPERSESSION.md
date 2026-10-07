# Civis authority and supersession ledger — pass 22

Date 2026-09-30.

This ledger determines which authored sources may generate **candidate** obligations and which can currently bind the mature product. It does not delete historical requirements.

## Observed source chronology

| Source | Date / status | Observed role |
|---|---|---|
| PRD.md v1.0 | 2026-02-21 / APPROVED | Early CivLab product baseline: deterministic headless simulation, multi-client platform/research framing |
| FUNCTIONAL_REQUIREMENTS.md v1.0 | 2026-03-25 / DRAFT | Detailed FR expansion tracing to PRD epics; still requires bit-identical determinism, logged RNG draws and multiple clients |
| emergence-charter.md correction | 2026-05-29 | Explicit later correction: global determinism is NOT a requirement; snapshots preserve actual state; do not enforce deterministic replay |
| emergence-charter.md scope boundary | 2026-05-31 | Explicit later correction: multiplayer/co-op/spectator OUT of v1; WebSocket is single-player client/server transport |
| ADR-determinism-dropped.md | 2026-05-30 / Accepted as recorded | Explicitly records determinism supersession |
| current user recovery assignment | 2026-09-29 onward | Normative recovery policy: distinguish intent/design/implementation/proposal; do not promote old assistant/design material accidentally; recover mature current intent |

## Provisional authority order

For a direct contradiction, use the most specific later accepted/current authority rather than status labels alone:

1. current explicit user instruction / recovered explicit current user intent;
2. later explicit accepted scope/design corrections;
3. earlier approved product documents for subjects not superseded;
4. draft requirement catalogues as **candidate-obligation sources**, not automatic accepted obligations;
5. design documents, reports and generated trace catalogues as candidate/supporting sources according to recovered acceptance provenance;
6. implementation as evidence of what exists, never self-authorizing product intent.

Confidence/volume does not change this order.

## Consequences

### Determinism

PRD/FUNCTIONAL_REQUIREMENTS deterministic-replay clauses are **historical/superseded for the main product** where they conflict with the May 29–30 correction.

Examples include:
- FR-CORE-003 bit-identical transition/replay requirements;
- FR-CORE-004 all RNG draws logged to guarantee reproducibility;
- any NFR requiring global bit-identical outcomes merely because the earlier CivLab product did.

An explicitly scoped subsystem may still opt into deterministic semantics, but that requires its own accepted contract.

### Multiplayer / many clients

PRD and draft requirements describing simultaneous multiplayer/co-op/spectator behavior do not bind v1 against the May 31 single-player boundary.

Transport/client separation and multiple renderer code can remain as implementation/history, but are not v1 product obligations by inheritance.

### Unsuperseded subjects

The draft FR catalogue can still contain useful candidate obligations for subjects not contradicted by later intent, e.g. policy evaluation, persistence, economy behavior, interfaces. Each must still pass:
- provenance/acceptance review;
- current mature ontology fit;
- alternatives/SOTA review where architectural;
- semantic decomposition and contradiction attack.

A row is not accepted merely because it says SHALL.

## New main audit: docs/audits/spec-only-triage-2026-09-29.md

Current main `590fad0643eb85cae89edd9e64ed6b991461de6e` adds only this audit over code revision `54d5758970249c8d1f24688ea45920b530e77299`.

The audit is valuable supporting evidence. It establishes several important catalogue-quality problems:
- scanner invisibility for repo-root FUNCTIONAL_REQUIREMENTS.md;
- dead/unmounted symbols incorrectly counted as implementation;
- non-existent named test suites;
- 155 range-manufactured emergence IDs;
- non-Rust implementation surfaces omitted by Rust-only scanning;
- namespace collisions and inventory-invisible IDs.

However, its chosen rule — authored per-ID acceptance criteria imply a REAL-GAP even from design documents — is **not adopted as product authority** by this recovery program. It answers whether an authored criterion lacks matching code, not whether that criterion remains accepted current product intent.

Therefore its counts (121 REAL-GAP, etc.) are an audit denominator for that catalogue, not a mature-product requirement denominator or completion score.

## Blocking authority questions still open

- Recover original approval/supersession provenance for unsuperseded Draft FR families.
- Determine which design-doc catalogues were accepted vs remained proposals.
- Resolve the seven namespace collisions without selecting by whichever implementation currently exists.
- Reconcile root/inventory scanning so invalid/generated/missing IDs cannot participate in grading.
- Retire or quarantine the 155 range-manufactured emergence IDs from any completion calculation unless authentic defining obligations are recovered.

No requirement count is inferred from this ledger.
