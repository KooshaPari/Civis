# Research: FR-SOC-CIV-002 -- Social civilisation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-SOC-CIV-002
> Epic: FR-SOC-CIV

## Research Question

What is the best approach to implement Social civilisation within the Civis simulation engine?

## Background

This FR belongs to the FR-SOC-CIV epic and is expected to be implemented in `crates/civ-institutions/src/`.

### Existing Code References
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4355`
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4571`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/civ-institutions/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/civ-institutions/src/`
2. Add integration tests in `crates/civ-institutions/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/civ-institutions/` crate documentation
