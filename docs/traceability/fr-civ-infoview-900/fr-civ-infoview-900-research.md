# Research: FR-CIV-INFOVIEW-900 -- Info views and inspectors

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-INFOVIEW-900
> Epic: FR-CIV-INFOVIEW

## Research Question

What is the best approach to implement Info views and inspectors within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-INFOVIEW epic and is expected to be implemented in `crates/hud/src/`.

### Existing Code References
- `clients/bevy-ref/src/info_views.rs:17`
- `clients/bevy-ref/src/info_views.rs:380`
- `docs/agileplus/epics/civ-w4-perception.md:9`
- `docs/agileplus/epics/civ-w4-perception.md:28`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:214`
- `docs/design/master-roadmap.md:22`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:12`

### Test References
- `clients/bevy-ref/src/info_views.rs:738`
- `clients/bevy-ref/src/info_views.rs:759`
- `clients/bevy-ref/src/info_views.rs:773`

## Findings

### Codebase Analysis
- The `crates/hud/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/hud/src/`
2. Add integration tests in `crates/hud/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/hud/` crate documentation
