# ADR: FR-CIV-AI-001 -- Artificial intelligence

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-AI-001
> Epic: FR-CIV-AI

## Context

FR-CIV-AI-001 is part of the FR-CIV-AI epic. This functional requirement captures: Artificial intelligence.

Implementing crate: `crates/ai/src/`

### Referenced Source
- `crates/ai/src/lib.rs:7`
- `crates/ai/src/lib.rs:55`
- `docs/design/civ-ai-crate.md:33`
- `docs/design/civ-ai-crate.md:81`
- `docs/design/master-roadmap.md:27`
- `docs/design/master-roadmap.md:199`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-AI-001 needs to be finalized based on implementation exploration.

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
