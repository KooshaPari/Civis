# Research: FR-CIV-WEB-008 -- Web client

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-WEB-008
> Epic: FR-CIV-WEB

## Research Question

What is the best approach to implement Web client within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-WEB epic and is expected to be implemented in `crates/server/src/`.

### Existing Code References
- `docs/development-guide/fr-web-spectator.md:37`
- `docs/IMPLEMENTATION_STATUS.md:58`
- `web/dashboard/src/lib/authoring.ts:95`

### Test References
> _No test coverage yet._

## Findings

### Codebase Analysis
- The `crates/server/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/server/src/`
2. Add integration tests in `crates/server/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/server/` crate documentation
