# ADR: FR-CIV-ASSET-002 -- Asset pipeline

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ASSET-002
> Epic: FR-CIV-ASSET

## Context

FR-CIV-ASSET-002 is part of the FR-CIV-ASSET epic. This functional requirement captures: Asset pipeline.

Implementing crate: `crates/asset-pipeline/src/`

### Referenced Source
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437`
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3206`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ASSET-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/asset-pipeline/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Asset pipeline requirement in the simulation

### Negative
- Adds complexity to the asset-pipeline crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/asset-pipeline/`
2. **Option B**: Extract into a dedicated sub-crate
