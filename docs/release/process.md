# Release Process

This process is intentionally narrow. It is the minimum release workflow for the v0.26 platform wave and should stay machine-scannable.

## Release Authority

| Decision | Owner | Canonical input |
|---|---|---|
| Release scope | Manager / release lead | `docs/release/v0.26.0-PLAN.md` |
| Architecture exceptions | ADR owners | `docs/adr/*.md` |
| Schema compatibility | SDK / schema owner | `docs/reference/schema-governance.md` |
| QA gates | QA owner | `docs/qa/index.md` |

## Required Order

| Step | Required evidence |
|---|---|
| 1. Scope confirm | Release plan references the exact wave and open debt items. |
| 2. Doc drift check | No release note contradicts ADR, schema, or QA policy. |
| 3. Validation check | Required gates are named and available before tagging. |
| 4. Freeze check | Any late change has an explicit owner and a rollback path. |
| 5. Publish | Release notes, changelog, and linked references are updated together. |

## Status Table

| Status | Meaning | Action |
|---|---|---|
| Draft | Release is being assembled. | Keep changes scoped and traceable. |
| Ready | Required docs and gates are aligned. | Proceed to final validation. |
| Blocked | A required gate or decision is missing. | Resolve the blocker before tagging. |
| Released | The wave is published. | Archive follow-up work into the next cycle. |

## Evidence Rules

Every release note should point back to one of these artifacts:

- a plan item,
- a schema change record,
- a QA gate,
- or an ADR.

If a statement cannot point to one of those, it is commentary and should not be promoted into the release record.

