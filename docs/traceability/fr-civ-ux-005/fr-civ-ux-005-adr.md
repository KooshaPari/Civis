# ADR: FR-CIV-UX-005 -- User experience

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-UX-005
> Epic: FR-CIV-UX

## Context

FR-CIV-UX-005 is part of the FR-CIV-UX epic. This functional requirement captures: User experience.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/godot-ref/scripts/camera.gd:43`
- `clients/godot-ref/scripts/ui.tscn:107`
- `clients/godot-ref/scripts/ui.tscn:111`
- `clients/godot-ref/scripts/ui.tscn:115`
- `docs/development-guide/fr-unreal-agent-playbook.md:116`
- `web/dashboard/src/bottom_bar.tsx:434`
- `web/dashboard/src/store.tsx:295`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-UX-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the User experience requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
