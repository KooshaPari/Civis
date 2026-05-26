# Schema Governance

This page defines how schema changes move through the v0.26 platform wave.

## Source of Truth

| Layer | Authority | Notes |
|---|---|---|
| Schema definitions | `schemas/*.json` and `schemas/*.yaml` | Machine validation source. |
| Human policy | this page | Change control and drift rules. |
| Test fixtures | `src/Tests/**` and docs fixtures | Must cover parse, rejection, and round-trip where applicable. |
| Pack guidance | `docs/reference/schemas.md` | User-facing schema summary. |

## Change Classes

| Change type | Required response |
|---|---|
| Add schema | Add schema docs and at least one fixture that proves valid and invalid input. |
| Modify schema | Document compatibility impact and update any dependent fixtures. |
| Deprecate field | Keep the old field documented until the migration path is explicit. |
| Remove field | Only after the deprecation path is documented and covered. |
| Rename field | Treat as breaking unless a compatibility alias is preserved. |

## Validation Contract

| Check | Must be true |
|---|---|
| Parse | Valid sample input parses successfully. |
| Reject | Invalid sample input fails for the intended reason. |
| Round-trip | When the shape is serializable, output matches the schema contract. |
| Drift | Docs and fixtures say the same thing about required fields and defaults. |

## Governance Rules

1. Schemas are authoritative over narrative docs.
2. Narrative docs are authoritative over memory and prior notes.
3. If a schema change affects release behavior, the release process must name it explicitly.
4. If a schema change affects QA behavior, the QA index must name the enforcement path.

