# ADR: FR-CIV-RESEARCH-030 -- Technology research

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RESEARCH-030
> Epic: FR-CIV-RESEARCH

## Context

FR-CIV-RESEARCH-030 is part of the FR-CIV-RESEARCH epic. This functional requirement captures: Technology research.

Implementing crate: `crates/research/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/research/src/lib.rs:512`

## Decision

TBD -- The architectural decision for FR-CIV-RESEARCH-030 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/research/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Technology research requirement in the simulation

### Negative
- Adds complexity to the research crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/research/`
2. **Option B**: Extract into a dedicated sub-crate
