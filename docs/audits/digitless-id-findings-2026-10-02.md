# Digitless requirement IDs — findings (2026-10-02)

`docs/audits/_gather_ids.py` `ID_RE` requires a numeric final segment, so 254
FR/NFR-shaped tokens with no digits are invisible to the audit. Broadening the
regex was rejected: 156 of them are namespace heads such as `FR-CIV-SPECIES`
that already cover 10,126 occurrences, and admitting them would mint ~156
phantom namespace rows.

## Enumeration

98 of the 254 have no numbered sibling anywhere in the tree, so they are
candidates for a literal allowlist.

| Bucket | Count | Disposition |
|---|---|---|
| Backed by an authoritative spec/design/index document | 22 | several are placeholders (`FR-SAVE-NNN`, `FR-to-test`) and still must not be admitted blind |
| No spec backing, digitless appears only in its own crate | 76 | see below |
| **Admitted** | **0** | no token survived source validation |

## The one genuine finding: a self-minted alias

`crates/species/src/speciation.rs` implemented the entire Hamming-distance
speciation behaviour that `docs/design/species-sentience.md:120-124` defines as
`FR-CIV-SPECIES-300..304`, but tagged itself with an invented digitless ID.

Consequences, both real and both now fixed:

1. `FR-CIV-SPECIES-300..304` were reported `SPEC-ONLY` while code and tests
   existed. The matrix claimed a spec gap that does not exist.
2. Admitting the alias would have added a phantom row for behaviour that is
   already covered five times over.

Fixed by re-tagging the source with the authoritative IDs, not by admitting the
alias. `FR-CIV-GENETICS-010/011` are *distinct* requirements ("emits new species
record", defined at `docs/development-guide/fr-3d-additions.md:45`), so they were
kept and cross-referenced rather than merged.

## Remaining 76 are spec gaps, not matrix rows

Nine of the unbacked tokens are code with no spec anywhere — the digitless ID is
the symptom of behaviour nobody wrote down:

`FR-CIV-LEGAL-PRECEDENT` (`crates/laws/src/precedent.rs`),
`FR-CIV-NICHE-ADAPT` (`crates/species/src/niche.rs`),
`FR-CIV-TECH-OBSOLETE` (`crates/research/src/tech_obsolete.rs`),
`FR-CIV-phasewire`, `FR-CLIENT-godbuttons`, `FR-CONTENT-SEEDMIX`,
`FR-CONTENT-STARTCOND`, `FR-ENGINE-phaseorder`, `FR-RELIG-readapi`.

These need spec authoring, not an ID. Adding them to the matrix would assert
that a requirement exists when none does.

## Rejected heuristics

- **Word overlap.** Matching a token's words against design docs "confirmed" 52
  aliases, but `FR-CIV-TAX-POLICY` matched `docs/design/audio-direction.md` on
  the word "policy". Rejected as a false-positive generator.
- **Broadening `ID_RE`.** Would admit ~156 namespace heads as rows.
- **`FR-CIV-MORALE`** is explicitly labelled a phantom in its own source, so it
  is not a requirement despite the ID-shaped name.

## Related finding: `docs/traceability/index.md` is a stale snapshot

Regenerated status comparison, all 1231 rows parsed:

| Measure | Count |
|---|---:|
| Rows in `index.md` | 1231 |
| IDs also present in the matrix | 1229 |
| Status agrees with the matrix | 163 |
| Status disagrees with the matrix | 435 |
| Not comparable (absent from the matrix) | 631 |
| Only in `index.md` | 2 (`FR-CIV-TACTICS-001-`, `FR-CIV-TACTICS-025-`, both known phantoms) |
| Only in the matrix | 201 |

The file is dated 2026-09-16 by its own header and its generator no longer
exists in the repo. It uses a status vocabulary the matrix does not emit at all:
631 rows claim `CODE-ONLY-no-spec`, which the matrix has never produced.

The matrix is authoritative where they differ. Spot-checked three of the 435
disagreements against source:

- `FR-AI-001` — `index.md` says `SPEC-ONLY`, matrix says `COVERED`. The matrix
  is right: `docs/FR.md:43` defines it, `crates/ai/src/decision.rs:3` cites it,
  and `crates/ai/tests/fr_fr_ai_001.rs:1` tests it.
- `FR-CIV-CULT-001` — `index.md` says `IMPL-NO-TEST`, matrix says `COVERED`.
  Specified in `agileplus-specs/civ-009-culture-diffusion/spec.md:24`.
- `FR-CIV-AGENTS-000` — `index.md` says `COVERED`, matrix says `SELF-TEST-ONLY`.
  The matrix is stricter, which is the correct direction for a needs-test row.

Patching the five `SPECIES-300..304` rows by hand would have been cosmetic, so it
was not done. The file needs regeneration from the matrix, or retirement as a
duplicate source of truth. Tracked separately.

The `Completeness` column is not a coverage signal. It ranges 3/7 to 7/7 and is
computed from seven documentation artefacts existing per ID, not from whether
behaviour is implemented. An ID can score 7/7 and still be `SPEC-ONLY`, so the
column must not be read as implementation status.
