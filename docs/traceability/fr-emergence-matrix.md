# FR Emergence Matrix — Civis Phase 3

**Purpose**: trace the 171 emergence + dormant-phase + charter-umbrella
`FR-` IDs identified in `docs/traceability/COVERAGE_AUDIT.md` §3.1, §3.2,
§3.3, and the `FR-CIV-0100` family.  This is the canonical row-level
source for the §3 IDs; `civis-tracelinks.md` and
`emergent-systems-tracelinks.md` link here.

**Schema** (mirrors `fr-3d-matrix.md`):

| Column | Meaning |
|---|---|
| FR ID | the canonical functional-requirement ID |
| Family | one of `emergence`, `dormant`, `charter` |
| Sub-cluster | the audit's sub-grouping (e.g. `civ-019`, `civ-021-batchA`) |
| Phase | `emergence` for §3.1, `dormant` for §3.2, `recovered` for §3.3, `charter` for `FR-CIV-0100` |
| Status | one of `planned`, `dormant`, `recovered`, `implemented`, `in-progress` |
| Owner | the FR's responsible subsystem or family |
| Trace link | pointer to spec/code that satisfies the FR (or `tbd` if not yet found) |
| Notes | free-form |

**Status legend** (from `COVERAGE_AUDIT.md` §3 follow-up #4):
- `planned` — code is sketched but no tests/coverage.
- `dormant` — phase is currently a no-op stub (see
  `crates/engine/src/emergence.rs:1-2`); the FRs are intentionally
  de-scoped until a re-emergence signal arrives.
- `recovered` — re-discovered in `civ-021` (and similar) and given a
  concrete traceability row here.
- `implemented` — code + tests + coverage present.
- `in-progress` — partial implementation; row tracks remaining work.

---

## Charter umbrella (1 row)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-0100` | charter | charter umbrella | charter | planned | engine | tbd | Top-level umbrella for all `civ-0XX` IDs; covers §3 emergence, §3 dormant, §3 recovered.  See `civis-tracelinks.md` and `emergent-systems-tracelinks.md`. |

---

## FR-CIV-EMERG-001..005 (5 rows — civ-019 emergence-metrics dashboard)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERG-001` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 1 |
| `FR-CIV-EMERG-002` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 2 |
| `FR-CIV-EMERG-003` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 3 |
| `FR-CIV-EMERG-004` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 4 |
| `FR-CIV-EMERG-005` | emergence | civ-019 metrics dashboard | emergence | planned | engine | tbd | Top-level dashboard metric 5 |

---

## FR-CIV-EMERGENCE-001..006 (6 rows — civ-021 recovered-requirements batch A)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-001` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 1 (batch A) |
| `FR-CIV-EMERGENCE-002` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 2 (batch A) |
| `FR-CIV-EMERGENCE-003` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 3 (batch A) |
| `FR-CIV-EMERGENCE-004` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 4 (batch A) |
| `FR-CIV-EMERGENCE-005` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 5 (batch A) |
| `FR-CIV-EMERGENCE-006` | emergence | civ-021 batch A | recovered | recovered | engine | tbd | Recovered requirement 6 (batch A) |

---

## FR-CIV-EMERGENCE-010..013 (4 rows — civ-021 recovered-requirements batch B)

| FR ID | Family | Sub-cluster | Phase | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|---|
| `FR-CIV-EMERGENCE-010` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 10 (batch B) |
| `FR-CIV-EMERGENCE-011` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 11 (batch B) |
| `FR-CIV-EMERGENCE-012` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 12 (batch B) |
| `FR-CIV-EMERGENCE-013` | emergence | civ-021 batch B | recovered | recovered | engine | tbd | Recovered requirement 13 (batch B) |

---

## Aggregate counts (this matrix)

- Charter umbrella: 1 row (`FR-CIV-0100`)
- Emergence (`civ-019`): 5 rows
- Recovered (`civ-021` batch A): 6 rows
- Recovered (`civ-021` batch B): 4 rows
- **Total: 16 rows**

The remaining 155 IDs (171 − 16) are listed in
`docs/traceability/COVERAGE_AUDIT.md` §3.1, §3.2, §3.3 but are not yet
given individual rows here.  They are tracked in bulk via the
audit summary and will be added in subsequent PRs.

---

## See also

- `docs/traceability/COVERAGE_AUDIT.md` §3.1–§3.3 (171-ID source list)
- `docs/traceability/fr-3d-matrix.md` (existing 3D rendering matrix)
- `docs/traceability/index.md` (hub — links here)
- `civis-tracelinks.md` (civic-domain tracelinks, references §3)
- `emergent-systems-tracelinks.md` (emergence-family tracelinks, references §3)
- `tools/audit-fr-coverage/audit.sh` (regen script — see follow-up #5)
