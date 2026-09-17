# Research: FR-CIV-ACT-001 -- Actor lifecycle

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-ACT-001
> Epic: FR-CIV-ACT

## Research Question

What is the best approach to implement Actor lifecycle within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-ACT epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2103`
- `docs/reference/FR_TRACKER.md:28`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:179`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:509`
- `docs/reports/STATUS_REPORT.md:97`

### Test References
- `crates/build/tests/fr_matrix_batch12.rs:115`
- `crates/build/tests/fr_matrix_batch12.rs:118`

## Findings

### Codebase Analysis
- The `crates/engine/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/engine/src/`
2. Add integration tests in `crates/engine/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/engine/` crate documentation
