# Research: FR-CIV-ECON-015 -- Economy and joule allocation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-ECON-015
> Epic: FR-CIV-ECON

## Research Question

What is the best approach to implement Economy and joule allocation within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-ECON epic and is expected to be implemented in `crates/economy/src/`.

### Existing Code References
- `crates/economy/src/chains.rs:2`

### Test References
- `crates/economy/src/chains.rs:391`
- `crates/economy/src/chains.rs:414`
- `crates/economy/src/chains.rs:444`
- `crates/economy/src/chains.rs:466`
- `crates/economy/src/chains.rs:494`
- `crates/economy/src/chains.rs:537`
- `crates/economy/src/chains.rs:577`
- `crates/economy/src/chains.rs:598`

## Findings

### Codebase Analysis
- The `crates/economy/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/economy/src/`
2. Add integration tests in `crates/economy/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/economy/` crate documentation
