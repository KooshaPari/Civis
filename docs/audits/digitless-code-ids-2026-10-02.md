# Digitless code-side IDs — 9 tokens classified (2026-10-02)

Nine ID tokens that tag real code have no numeric segment, so `scripts/traceability/`
`_gather_ids.py` never sees them and they never reach `docs/audits/fr-matrix.json`.
This is why they survived: **the audit could not detect them by construction.**

## Method

1. Enumerate every repo occurrence with `git grep`.
2. For each, read the module's own doc comment to learn the behaviour it claims.
3. Search the spec corpus for that behaviour's distinctive vocabulary.
4. Decide the verdict by reading the hits, not by counting them.

A count of 0 is evidence of no backing. A count of 60 is not evidence of backing:
for `FR-CIV-TECH-OBSOLETE` the 60 hits were 17 `supersed` matches about deprecated
*clients* and 43 `deprecat` matches about the same. Every non-zero result in this
investigation was a false positive. Every zero result was real.

## Verdicts

| ID | Verdict | Owner | Implementation |
|---|---|---|---|
| `FR-CIV-LEGAL-PRECEDENT` | UNDOCUMENTED | — | `crates/laws/src/precedent.rs:1` |
| `FR-CIV-NICHE-ADAPT` | UNDOCUMENTED | — | `crates/species/src/niche.rs:1` |
| `FR-CIV-TECH-OBSOLETE` | UNDOCUMENTED | — | `crates/research/src/tech_obsolete.rs:2` |
| `FR-CIV-phasewire` | UNDOCUMENTED | — | `crates/engine/src/engine.rs:2397` |
| `FR-CLIENT-godbuttons` | UNDOCUMENTED | — | `crates/server/src/ws_bridge.rs:2206,` |
| `FR-CONTENT-SEEDMIX` | MISFILED_DUPLICATE | FR-API-001 | `crates/engine/src/scenario.rs:1` |
| `FR-CONTENT-STARTCOND` | MISFILED_DUPLICATE | FR-API-001 | `crates/engine/src/scenario.rs:43` |
| `FR-ENGINE-phaseorder` | UNDOCUMENTED | — | `crates/engine/src/engine.rs:2484` |
| `FR-RELIG-readapi` | UNDOCUMENTED | — | `crates/civis-mcp/src/server.rs:1764` |

## What each verdict means

**MISFILED_DUPLICATE (2).** `FR-CONTENT-SEEDMIX` and `FR-CONTENT-STARTCOND` are the
same requirement as `FR-API-001`. `crates/engine/src/scenario.rs:1` already declares
`FR-API-001` at the top of the file, and `agileplus-specs/civ-013-research-api/spec.md:25`
defines `FR-API-001` as the scenario YAML schema covering *"map dimensions, entity
placement, starting conditions, policy parameters"*. `SeedWeight` and
`ScenarioStartingConditions` are fields of that same schema struct.

`FR-CONTENT` has **zero** rows in the matrix. The namespace is entirely self-minted.

**UNDOCUMENTED (7).** Real, tested, `#[forbid(unsafe_code)]`, ADR-referenced
implementation with no requirement anywhere. These are code-first features that were
never specified. They are not gaps in the matrix; they are gaps in the *specs*.

## The template-discovery problem (affects all 1430 rows)

While checking `FR-CIV-TECH-003` I read `docs/traceability/fr-civ-tech-003/` and found
`intent.md` and `adr.md` are **template-filled, not authored**. `fr-civ-tech-003-adr.md`
reads, in full:

```
## Decision

TBD -- The architectural decision for FR-CIV-TECH-003 needs to be finalized based on
implementation exploration.
```

and its *Test Coverage* section says `_No test coverage yet._` while the matrix calls the
same FR `COVERED`. The `intent.md` acceptance criteria are equally generic:
`cargo test -p research` passes; the simulation runs without errors.

These template files are counted as `spec_refs` on every row they touch. They are counted
as **documentation artefacts**, and the regenerated `docs/traceability/index.md` now says
so explicitly. They are not treated as requirements text, and the `Artifacts` column was
relabelled in 95b506cf for exactly this reason.

**Open question this raises, deliberately not resolved here.** 1430 rows × ~6 template
refs each means several thousand `spec_refs` entries that restate the ID rather than
specify behaviour. If the spec_refs budget for a row is downgraded from 9 to its authored
sources, how many rows change status? That is a measurable question and it is not
measured yet. Recording it rather than silently keeping the credit.

## Blast radius of the recommended fix

For the 2 misfiled duplicates, re-tagging `scenario.rs` to `FR-API-001` at the two sites
and deleting the two self-minted labels removes 2 phantom requirements without adding
rows. `FR-API-001` is already `COVERED`, so status totals do not move. These 9 tokens
remain outside the matrix either way, because the gatherer requires a numeric segment.

Per standing policy, none of the 9 was admitted to the matrix. Admitting them would mint
9 requirements from code comments, which is the exact defect this audit exists to find.

## Verification

```
matrix rows                                   1430  (unchanged)
digitless tokens admitted                     0
verdicts with a missing owner                 0
FR-CONTENT / FR-ENGINE / FR-RELIG rows        0  (namespaces absent from matrix)
```
