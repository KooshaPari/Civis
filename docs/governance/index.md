# Governance Hub

v0.26 governance exists to keep release, schema, research, and QA decisions aligned with the current platform wave.

## Authority Map

| Area | Source of truth | Notes |
|---|---|---|
| Release scope | `docs/release/v0.26.0-PLAN.md` | Current wave plan and release priorities. |
| Release procedure | `docs/release/process.md` | How a release is prepared, checked, and published. |
| Schema rules | `docs/reference/schema-governance.md` | How schemas are added, changed, and validated. |
| QA policy | `docs/qa/index.md` | Test gates, isolation rules, and proof expectations. |
| Desktop companion stack | `docs/ADR-companion-tfm-downgrade.md` and `docs/adr/ADR-011-desktop-companion.md` | Keep the stable TFM decision and the historical ADR in sync. |
| Research intake | `docs/research/index.md` | Temporary research notes and what they should resolve into. |

## v0.26 Drift Controls

| Drift type | Required control | Exit condition |
|---|---|---|
| Research drift | Each active research note must end with a decision, a next action, or a pointer to the owning ADR/spec. | No open-ended "exploration only" notes in the active set. |
| Release drift | Release steps must reference a concrete artifact, gate, or owner. | Every step is reproducible from docs alone. |
| Schema drift | Schema changes must declare validation impact and fixture impact. | Schema docs, fixtures, and validators agree. |
| QA drift | QA gates must name the test class, script, or command that enforces them. | Each rule has one documented enforcement path. |

## Working Rule

Prefer small, dated documents over long running prose. When a note stops being exploratory, move it into the governing doc set and leave the draft behind only if it still has active value.

