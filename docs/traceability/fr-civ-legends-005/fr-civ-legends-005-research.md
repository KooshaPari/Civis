# Research: FR-CIV-LEGENDS-005 -- Legend and narrative system

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-LEGENDS-005
> Epic: FR-CIV-LEGENDS

## Research Question

What is the best approach to implement Legend and narrative system within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-LEGENDS epic and is expected to be implemented in `crates/legends/src/`.

### Existing Code References
- `crates/legends/src/graph.rs:163`
- `crates/legends/src/query.rs:85`
- `crates/legends/src/query.rs:92`
- `crates/legends/src/query.rs:119`
- `crates/legends/src/query.rs:137`
- `crates/legends/src/query.rs:145`
- `crates/legends/src/query.rs:217`
- `crates/legends/src/query.rs:237`

### Test References
- `crates/legends/tests/fr_legends_completion.rs:7`
- `crates/legends/tests/fr_legends_completion.rs:287`
- `crates/legends/tests/fr_legends_completion.rs:290`
- `crates/legends/tests/fr_legends_completion.rs:367`

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
