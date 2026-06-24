# Plan: Recovered Requirements — Phantom-ID Triage Batch 1 (civ-021)

Batch-1 audit of the 786 `CODE-ONLY-no-spec` rows in
`docs/audits/fr-matrix.json` (top 100 by ref count, classified).

## Phased WBS

### Phase 1: Audit (E2.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| A1.1 | Load `fr-matrix.json`; filter `status == CODE-ONLY-no-spec` (786 rows) | — | Done |
| A1.2 | Group by `id` prefix; count `code_refs` + `test_refs` per ID | A1.1 | Done |
| A1.3 | Take top 100 by ref count; build evidence map (file:line snippets) | A1.2 | Done |
| A1.4 | Classify each: (a) REAL, (b) STALE, (c) RENAME | A1.3 | Done |
| A1.5 | Cross-check class-(a) against existing spec files (`docs/specs/requirements/*`, `docs/specs/CIV-*`, `docs/design/*`, `docs/agileplus/epics/civ-w*`, `docs/development-guide/*`) | A1.4 | Done |

### Phase 2: Doc (E2.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| D2.1 | `docs/audits/phantom-triage-batch1.md` (verdicts + evidence) | A1.* | Done |
| D2.2 | `agileplus-specs/civ-021-recovered-requirements/meta.json` (FR list) | A1.* | Done |
| D2.3 | `agileplus-specs/civ-021-recovered-requirements/spec.md` (12 stubs) | A1.* | Done |
| D2.4 | `agileplus-specs/civ-021-recovered-requirements/plan.md` (this file) | D2.2 | Done |

### Phase 3: PR (E2.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| P3.1 | Commit on `docs/phantom-id-triage` (worktree `D:/civis-build/triage`) | D2.* | Done |
| P3.2 | `git push` to `origin/docs/phantom-id-triage` | P3.1 | Pending |
| P3.3 | `gh pr create --draft` with `Trace:` block | P3.2 | Pending |
| P3.4 | `gh pr ready` to leave draft state once review is acknowledged | P3.3 | Pending |

### Phase 4: Follow-up (out of scope here, filed as separate batches)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| F4.1 | Triage batches 2..N (686 - 100 = 586 remaining rows) | P3.* | Future |
| F4.2 | Backfill `docs/reference/FR_TRACKER.md` + `CODE_ENTITY_MAP.md` with the 12 new stubs | P3.* | Future |
| F4.3 | Re-run `_id_inventory_v3.py` and confirm the 12 stubs flip from CODE-ONLY to COVERED | F4.2 | Future |
| F4.4 | Decide on the FR-CIV-ECON-001-MARKET / FR-CIV-ECON-002 / -002-JOULE hyphen-collapse (RENAME candidates) | P3.* | Future |

## DAG Dependencies

```
A1.1 -> A1.2 -> A1.3 -> A1.4 -> A1.5
                          |       |
                          v       v
                         D2.1    D2.2 -> D2.3 -> D2.4
                                              |
                                              v
                                             P3.1 -> P3.2 -> P3.3 -> P3.4
```

## Risk / Notes

- The matrix undercount method recognises `docs/specs/requirements/*` and
  `docs/traceability/*` as spec homes. We extended that to
  `docs/specs/CIV-*`, `docs/agileplus/epics/civ-w*`, `docs/design/*`, and
  `docs/development-guide/*`. If a future audit script narrows this back,
  re-classify accordingly.
- The user-facing naming for this spec was `civ-019-recovered-requirements`,
  but `civ-019` is taken. We use `civ-021` (next free) and explain the
  deviation in the PR `Trace:` block.
- Class-(a) `cov: no` rows are stubs only; they do not block the verify
  gate. Class-(a) `cov: yes` rows are re-confirmations of existing
  coverage and need no new doc.
