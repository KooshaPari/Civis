# Traceability-doc evidence: what is real, and what is restated (2026-10-02)

Follow-up to `digitless-code-ids-2026-10-02.md`, which found that
`docs/traceability/fr-civ-tech-003/fr-civ-tech-003-adr.md:33` says, verbatim:

    TBD -- The architectural decision for FR-CIV-TECH-003 needs to be finalized
    based on implementation exploration.

and that the file is credited as spec evidence. That raises a fair question:
does boilerplate inflate the matrix's coverage verdicts?

## Answer: no row is covered by boilerplate, and traceability docs are load-bearing

Four measurements, each with a method that can be checked.

### 1. Byte-exact: strip the channel, re-run the committed generator

`scripts/traceability/gen-fr-audit.py:95` classifies with

    has_spec = bool(row.get("in_specs") or row.get("in_traceability")
                    or row.get("in_func_req"))

To measure the blast radius without reimplementing anything, the committed
inventory was copied into a temp tree, `in_traceability` emptied for all 1408
entries (10179 references removed), and the **real** `gen-fr-audit.py` was run as a
subprocess. Diffing its output against the committed matrix:

| status | committed | traceability stripped |
|---|---|---|
| `COVERED` | 840 | 728 |
| `SELF-TEST-ONLY` | 228 | 228 |
| `SPEC-ONLY` | 197 | 169 |
| `TEST-NO-CODE-REF` | 156 | 116 |
| `IMPL-NO-TEST` | 9 | 9 |
| `CODE-ONLY-no-spec` | 0 | 180 |
| rows | 1430 | 1430 |
| **rows whose status changes** | | **180** |

So traceability docs are where per-ID requirements actually live for 313 rows.
Deleting the channel would **manufacture** 180 unsupported `CODE-ONLY-no-spec`
verdicts rather than fix anything. `SELF-TEST-ONLY` and `IMPL-NO-TEST` are
untouched because those branches never consult `has_spec`.

### 2. Literal: no row rests on boilerplate alone

For the 313 rows whose only spec evidence is `docs/traceability`, each credited
file was tested for the generator's own placeholder strings, quoted verbatim from
the committed tree:

| | rows |
|---|---|
| every credited file is boilerplate | **0** |
| some files are boilerplate, some authored | 135 |
| no credited file is boilerplate | 178 |

**Zero COVERED rows are covered by placeholder text alone.** An intermediate
estimate of 112 was wrong.

### 3. Exact: cited lines carry real content

The strongest test needs no threshold. Delete the ID and every other `FR-`/`NFR-`
token from the cited line, strip markdown decoration and the generator's fixed
labels, then ask whether any characters remain. If nothing remains, the line was
pure ID restatement and carried no evidence of its own.

| | refs |
|---|---|
| spec-side refs examined | 12777 |
| of those, line-level (`file:line`) | 12648 |
| of those, file-level (no line) | 129, all `FUNCTIONAL_REQUIREMENTS.md` |
| **empty: ID restatement only** | 1312 (10.4% of line-level) |
| **content: says something** | 11336 (89.6% of line-level) |
| **line-level citation that fails to resolve** | **0** |
| rows where *every* spec ref is an empty restatement | **0** |

Two things worth stating plainly. First, **no citation in the inventory is
broken**: all 12648 line-level references point at a real file and a real
in-range line, and all 129 file-level references name a file that exists, so there
are no phantom credits of either shape. Second, the 1312 empty restatements are
almost entirely frontmatter: 1225 sit on lines 1-10 (titles, `> Epic:`,
`> Relates to:`, `> Date:`) and only 87 are in the body.

### 4. The six rows where the spec is generator output

`FR-CIV-ENGINE-INT-001/005/011/014`, `FR-CIV-ENGINE-REPLAY-003` and
`FR-CIV-LIFE-035` each credit 3 unique files: `intent`, `adr` and `plan`. Two are
boilerplate; the third, the intent doc, says only

    The product owner requires Civ Engine Int as part of the FR-CIV-ENGINE-INT epic.

where "Civ Engine Int" is the auto-titlecased epic slug. Its Definition of Done is
unchecked (`- [ ] Implementation ... compiles`).

These six are **documentation gaps, not implementation gaps**. The code and tests
are real and specific:

| ID | code | real test evidence |
|---|---|---|
| `FR-CIV-ENGINE-INT-001` | `crates/planet/src/lib.rs:64` | `crates/engine/src/engine/engine_tests.rs:1067` `is_daytime returns sensible day/night` |
| `FR-CIV-ENGINE-INT-005` | `crates/planet/src/lib.rs:81` | `crates/engine/src/engine/engine_tests.rs:1486` `is_daytime returns sensible day/night` |
| `FR-CIV-ENGINE-INT-011` | `crates/engine/src/engine.rs:2791` | `crates/engine/src/engine/engine_tests.rs:1311` `phase_buildings allocates over time` |
| `FR-CIV-ENGINE-INT-014` | `crates/engine/src/engine.rs:1955` | `crates/engine/src/engine/engine_tests.rs:1474` `last_cohort_stats reflects the population` |
| `FR-CIV-ENGINE-REPLAY-003` | `crates/engine/src/engine.rs:1768` | `crates/engine/src/engine/engine_tests.rs:1826` `push_damage records a Damage event` |
| `FR-CIV-LIFE-035` | `crates/agents/src/cluster.rs:76` | `crates/agents/src/cluster.rs:284` `Covers: FR-CIV-LIFE-035` |

Implemented, tested, and documented by a generator that did not know what the
requirement was. Fixing them means writing the requirement.

Note that all 1221 `docs/traceability/*/*-spec.md` files self-declare
`> Status: SPEC-TEMPLATE (auto-generated 2026-09-16)`, but only **74 rows credit a
spec.md at all**. The dominant evidence is `intent` (1336 files), `adr` (1144) and
`plan` (1088). So the self-declaration cannot discriminate, which is why it is not
used as a signal.

## The shared-matrix rows are sound

47 rows cite `emergent-systems-tracelinks.md`, `fr-emergence-matrix.md` or
`nfr-matrix.md` instead of a per-ID directory. Checked one by one: **all 47 IDs
appear verbatim in the file that credits them.** Zero phantom credits.
`fr-emergence-matrix.md:339` carries 211 ID mentions in a real requirements table.

`docs/traceability/nfr-matrix.md:52-63` is different. Twelve rows read

    | `NFR-CIV-001` | performance | engine | planned | engine | tbd | Reserved -- populated as each NFR is given a row |

with zero code refs and zero test refs. All twelve are reported `SPEC-ONLY`, which
is the honest verdict for a reserved placeholder, and the file's header says it is
"populated as each NFR is given a row". No change needed.

## What is NOT changing

No status changes. The 180-row strip cost is a measurement, not a proposal. The
findings worth keeping are:

1. **Zero** rows gain coverage from boilerplate alone.
2. **Zero** of 12648 spec citations fail to resolve.
3. **Six** rows are implemented and tested against a spec that only restates the
   epic name. Doc gaps; worth writing the requirements.
4. **178** rows rest on traceability docs containing no boilerplate at all.

Only item 3 is actionable. The rest is measurement.

## Six heuristics rejected along the way

Each was implemented, measured, and discarded because it disagreed with the
source. Recorded so the next person does not re-derive them.

1. **Text-overlap / similarity.** Flagged
   `docs/traceability/fr-asset-pipeline-001/fr-asset-pipeline-001-intent.md` as
   boilerplate. Reading it showed an authored doc naming `export_svg` and
   `ExportError::Encode`. False positive on an authored file.
2. **Reimplemented `has_spec` as a count and `derive_status` as thresholds.**
   Both inventions. `has_spec` is a boolean; `derive_status` has no thresholds.
   I had also been reading `docs/audits/_build_matrix.py`, an older builder that
   emits only 4 statuses and reproduced just 934 of 1430 committed rows. Caught by
   a faithfulness check that refuses to run unless the model reproduces every
   committed status first.
3. **"Bulk commit ⇒ template."** The 2 bulk commits introduced 6888 traceability
   files, but normalizing the ID out of 6025 of them left **4496 distinct
   shapes**, so those commits carried largely bespoke text. The rule is false. A
   second derivation rule, "H1 title equals the recomputed epic slug", fired 44
   times out of 5982 and disagreed with the literal test. Rejected.
4. **Line length as a proxy for content.** A "THIN vs THICK" split that called
   `TBD -- The architectural decision ...` THICK because the line is long. Long
   is not informative; the 49%/51% split that produced is not quoted anywhere
   above. Replaced by measurement 3, which has no threshold.
5. **Importing a script to reuse its function.** `importlib`-ing
   `docs/audits/_build_matrix.py` to call the real `derive_status` **rewrote
   `docs/audits/fr-matrix.json` at import time**, replacing 840 `COVERED` with 815
   and injecting 253 `CODE-ONLY-no-spec` rows. Reverted with `git checkout` and
   re-verified with the suite. Use a subprocess, never an import.
6. **Treating a 3354-ref "unresolved" count as a finding.** It was a bug in my own
   script: the inventory stores some `in_*` values as bare strings, and
   `for r in value` iterated the string character by character, yielding refs
   named `F`, `U`, `N`. The real unresolved count is 0.

## Verification

    matrix rows, committed                          1430
    matrix rows, after stripping in_traceability    1430
    rows added / removed                            0 / 0
    rows whose status changes under the strip       180
    COVERED rows covered by boilerplate alone          0
    spec-side citations examined                   12777
    line-level citations that fail to resolve          0 of 12648
    file-level citations that fail to resolve          0 of 129
    citations that are pure ID restatement          1312 of 12648 (10.4%)
    rows resting only on ID restatement                0
    of the 313 traceability-only rows, fully authored 178
    shared-matrix rows verified to name their IDs   47 / 47
    traceability pytest suite                      109 passed