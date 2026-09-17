# Research: FR-CIV-WEB-007 -- Web client

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-WEB-007
> Epic: FR-CIV-WEB

## Research Question

What is the best approach to implement Web client within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-WEB epic and is expected to be implemented in `crates/server/src/`.

### Existing Code References
- `docs/development-guide/fr-web-spectator.md:36`
- `docs/development-guide/pr-296-merge-readiness.md:34`
- `docs/IMPLEMENTATION_STATUS.md:57`
- `docs/roadmap/product-quality-ladder.md:30`
- `web/dashboard/src/babylon_scene.tsx:15`
- `web/dashboard/src/lib/rendererMode.ts:1`
- `web/dashboard/src/scene_view.tsx:5`
- `web/src/rendererMode.mjs:1`

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
