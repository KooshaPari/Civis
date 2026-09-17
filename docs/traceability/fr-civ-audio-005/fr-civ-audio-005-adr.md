# ADR: FR-CIV-AUDIO-005 -- Audio system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-AUDIO-005
> Epic: FR-CIV-AUDIO

## Context

FR-CIV-AUDIO-005 is part of the FR-CIV-AUDIO epic. This functional requirement captures: Audio system.

Implementing crate: `crates/audio/src/`

### Referenced Source
- `crates/audio/src/lib.rs:24`
- `crates/audio/src/sfx.rs:30`
- `crates/audio/src/sfx.rs:58`
- `crates/audio/src/sfx.rs:62`
- `crates/audio/src/sfx.rs:83`
- `crates/audio/src/triggers.rs:1`
- `docs/design/audio-direction.md:298`

### Test Coverage
- `crates/audio/src/sfx.rs:409`
- `crates/audio/src/triggers.rs:266`
- `crates/build/tests/fr_matrix_batch12.rs:191`
- `crates/build/tests/fr_matrix_batch12.rs:194`

## Decision

TBD -- The architectural decision for FR-CIV-AUDIO-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/audio/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Audio system requirement in the simulation

### Negative
- Adds complexity to the audio crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/audio/`
2. **Option B**: Extract into a dedicated sub-crate
