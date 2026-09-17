# ADR: FR-CIV-MOD-001 -- Mod system (WASM)

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-MOD-001
> Epic: FR-CIV-MOD

## Context

FR-CIV-MOD-001 is part of the FR-CIV-MOD epic. This functional requirement captures: Mod system (WASM).

Implementing crate: `crates/mod-host/src/`

### Referenced Source
- `docs/design/modding-platform.md:27`
- `docs/design/modding-platform.md:53`
- `docs/specs/CIV-0700-modding-api-spec.md:2356`

### Test Coverage
- `crates/mod-host/src/lib.rs:1404`
- `crates/mod-host/tests/fr_matrix_batch10.rs:275`

## Decision

TBD -- The architectural decision for FR-CIV-MOD-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/mod-host/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Mod system (WASM) requirement in the simulation

### Negative
- Adds complexity to the mod-host crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/mod-host/`
2. **Option B**: Extract into a dedicated sub-crate
