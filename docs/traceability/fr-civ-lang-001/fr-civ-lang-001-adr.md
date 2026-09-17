# ADR: FR-CIV-LANG-001 -- Language and naming

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LANG-001
> Epic: FR-CIV-LANG

## Context

FR-CIV-LANG-001 is part of the FR-CIV-LANG epic. This functional requirement captures: Language and naming.

Implementing crate: `crates/i18n/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-LANG-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/i18n/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Language and naming requirement in the simulation

### Negative
- Adds complexity to the i18n crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/i18n/`
2. **Option B**: Extract into a dedicated sub-crate
