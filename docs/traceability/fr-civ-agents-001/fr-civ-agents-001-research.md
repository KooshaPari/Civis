# Research: FR-CIV-AGENTS-001 -- Agent systems

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-AGENTS-001
> Epic: FR-CIV-AGENTS

## Research Question

What is the best approach to implement Agent systems within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-AGENTS epic and is expected to be implemented in `crates/ai/src/`.

### Existing Code References
- `docs/design/emergence-dashboard.md:7`
- `docs/development-guide/fr-3d-additions.md:57`

### Test References
- `crates/agents/src/lib.rs:710`
- `crates/agents/src/lib.rs:737`

## Findings

### Codebase Analysis
- The `crates/ai/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/ai/src/`
2. Add integration tests in `crates/ai/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/ai/` crate documentation
