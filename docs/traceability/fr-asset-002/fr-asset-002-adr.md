# ADR: FR-ASSET-002 -- Asset

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-ASSET-002
> Epic: FR-ASSET

## Context

FR-ASSET-002 is part of the FR-ASSET epic. This functional requirement captures: Asset.

Implementing crate: `crates/asset-pipeline/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-ASSET-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/asset-pipeline/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Asset requirement in the simulation

### Negative
- Adds complexity to the asset-pipeline crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/asset-pipeline/`
2. **Option B**: Extract into a dedicated sub-crate
