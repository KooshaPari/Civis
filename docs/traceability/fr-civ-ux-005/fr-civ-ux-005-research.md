# Research: FR-CIV-UX-005 -- User experience

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-UX-005
> Epic: FR-CIV-UX

## Research Question

What is the best approach to implement User experience within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-UX epic and is expected to be implemented in `crates/hud/src/`.

### Existing Code References
- `clients/godot-ref/scripts/camera.gd:43`
- `clients/godot-ref/scripts/ui.tscn:107`
- `clients/godot-ref/scripts/ui.tscn:111`
- `clients/godot-ref/scripts/ui.tscn:115`
- `docs/development-guide/fr-unreal-agent-playbook.md:116`
- `web/dashboard/src/bottom_bar.tsx:434`
- `web/dashboard/src/store.tsx:295`

### Test References
> _No test coverage yet._

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
