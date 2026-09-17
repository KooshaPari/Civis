# ADR: FR-CIV-AI-013 -- Artificial intelligence

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-AI-013
> Epic: FR-CIV-AI

## Context

FR-CIV-AI-013 is part of the FR-CIV-AI epic. This functional requirement captures: Artificial intelligence.

Implementing crate: `crates/ai/src/`

### Referenced Source
- `docs/design/civ-ai-crate.md:45`
- `docs/design/civ-ai-crate.md:257`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-AI-013 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/ai/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Artificial intelligence requirement in the simulation

### Negative
- Adds complexity to the ai crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/ai/`
2. **Option B**: Extract into a dedicated sub-crate
