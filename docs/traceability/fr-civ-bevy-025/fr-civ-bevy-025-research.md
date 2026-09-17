# Research: FR-CIV-BEVY-025 -- Bevy rendering client

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-BEVY-025
> Epic: FR-CIV-BEVY

## Research Question

What is the best approach to implement Bevy rendering client within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-BEVY epic and is expected to be implemented in `crates/engine/src/`.

### Existing Code References
- `clients/bevy-ref/README.md:26`
- `docs/development-guide/p-w1-kickoff.md:136`
- `justfile:149`

### Test References
- `clients/bevy-ref/src/live_pick.rs:286`
- `clients/bevy-ref/src/live_pick.rs:306`
- `clients/bevy-ref/src/live_stream.rs:712`
- `crates/build/tests/fr_matrix_batch12.rs:338`
- `crates/build/tests/fr_matrix_batch12.rs:341`

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
