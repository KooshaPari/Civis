# Research: FR-CIV-LEGENDS-006 -- Legend and narrative system

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-LEGENDS-006
> Epic: FR-CIV-LEGENDS

## Research Question

What is the best approach to implement Legend and narrative system within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-LEGENDS epic and is expected to be implemented in `crates/legends/src/`.

### Existing Code References
- `crates/legends/src/graph.rs:77`
- `crates/legends/src/graph.rs:89`
- `crates/legends/src/graph.rs:589`
- `crates/legends/src/graph.rs:607`

### Test References
- `crates/legends/tests/fr_legends_completion.rs:8`
- `crates/legends/tests/fr_legends_completion.rs:376`
- `crates/legends/tests/fr_legends_completion.rs:379`
- `crates/legends/tests/fr_legends_completion.rs:417`
- `crates/legends/tests/fr_legends_completion.rs:431`
- `crates/legends/tests/fr_legends_completion.rs:468`
- `crates/legends/tests/fr_legends_completion.rs:476`

## Findings

### Codebase Analysis
- The `crates/legends/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/legends/src/`
2. Add integration tests in `crates/legends/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/legends/` crate documentation
