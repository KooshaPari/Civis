# NFR Matrix — Civis Non-Functional Requirements

**Purpose**: trace the 34 NFRs (non-functional requirements) enumerated
in `docs/traceability/COVERAGE_AUDIT.md` §256–271.  This is the
canonical NFR table; `civis-tracelinks.md` references it.

**Schema** (mirrors `fr-3d-matrix.md`):

| Column | Meaning |
|---|---|
| NFR ID | the canonical NFR ID |
| Category | one of `performance`, `reliability`, `security`, `maintainability`, `portability`, `scalability`, `observability`, `safety`, `interoperability` |
| Family | which subsystem/cargo workspace this NFR applies to |
| Status | one of `planned`, `dormant`, `recovered`, `implemented`, `in-progress` |
| Owner | the responsible team/subsystem |
| Trace link | pointer to spec/code that satisfies the NFR |
| Notes | free-form |

**Status legend** (same as `fr-emergence-matrix.md`):
- `planned` — design sketched but no validation.
- `dormant` — phase currently a no-op; the NFR is intentionally
  de-scoped.
- `recovered` — re-discovered and given a concrete traceability row.
- `implemented` — code + tests + measurement present.
- `in-progress` — partial implementation; row tracks remaining work.

---

## Categories

The 34 NFRs span 9 categories.  Source: `COVERAGE_AUDIT.md` §256–271
(34 NFRs proposed scope, 2026-06-26 audit).

| Category | NFRs in this matrix | Typical owner |
|---|---|---|
| performance | TBD | engine / runtime |
| reliability | TBD | engine / supervision |
| security | TBD | auth / audit |
| maintainability | TBD | substrate / tooling |
| portability | TBD | substrate / driver |
| scalability | TBD | substrate / wave |
| observability | TBD | phenoobs / trace |
| safety | TBD | engine / faction |
| interoperability | TBD | driver / bridge |

---

## NFR rows

| NFR ID | Category | Family | Status | Owner | Trace link | Notes |
|---|---|---|---|---|---|---|
| `NFR-CIV-001` | performance | engine | planned | engine | tbd | Reserved — populated as each NFR is given a row |
| `NFR-CIV-002` | performance | engine | planned | engine | tbd | Reserved |
| `NFR-CIV-003` | reliability | engine | planned | engine | tbd | Reserved |
| `NFR-CIV-004` | reliability | supervision | planned | substrate | tbd | Reserved |
| `NFR-CIV-005` | security | auth | planned | substrate | tbd | Reserved |
| `NFR-CIV-006` | security | audit | planned | substrate | tbd | Reserved |
| `NFR-CIV-007` | maintainability | substrate | planned | substrate | tbd | Reserved |
| `NFR-CIV-008` | portability | driver | planned | substrate | tbd | Reserved |
| `NFR-CIV-009` | scalability | wave | planned | substrate | tbd | Reserved |
| `NFR-CIV-010` | observability | trace | planned | phenoobs | tbd | Reserved |
| `NFR-CIV-011` | safety | faction | planned | engine | tbd | Reserved |
| `NFR-CIV-012` | interoperability | bridge | planned | substrate | tbd | Reserved |
| `NFR-CIV-013..034` | various | various | planned | various | tbd | Reserved (22 rows remaining; populated as each NFR is given a row) |

---

## Aggregate counts (this matrix)

- Total NFRs: 34 (per `COVERAGE_AUDIT.md` §256–271)
- Rows populated: 0 (placeholder rows only)
- Rows planned: 34

**Action item**: replace each placeholder row with concrete
spec/code/measurement pointers.  See `COVERAGE_AUDIT.md` §4 follow-up #6.

---

## See also

- `docs/traceability/COVERAGE_AUDIT.md` §256–271 (34-NFR source list)
- `docs/traceability/fr-3d-matrix.md` (functional-requirement analogue)
- `docs/traceability/fr-emergence-matrix.md` (171-ID emergence matrix)
- `docs/traceability/index.md` (hub — links here)
- `civis-tracelinks.md` (civic-domain tracelinks, references NFRs)
- `tools/audit-fr-coverage/audit.sh` (regen script — see follow-up #5)
