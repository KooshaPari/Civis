# ADR: FR-CIV-ASSET-MANI-001 -- Civ Asset Mani

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ASSET-MANI-001
> Epic: FR-CIV-ASSET-MANI

## Context

FR-CIV-ASSET-MANI-001 is part of the FR-CIV-ASSET-MANI epic. This functional requirement captures: Civ Asset Mani.

Implementing crate: `crates/asset-pipeline/src/`

### Referenced Source
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3212`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ASSET-MANI-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/asset-pipeline/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Asset Mani requirement in the simulation

### Negative
- Adds complexity to the asset-pipeline crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/asset-pipeline/`
2. **Option B**: Extract into a dedicated sub-crate
