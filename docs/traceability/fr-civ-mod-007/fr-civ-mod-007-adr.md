# ADR: FR-CIV-MOD-007 -- Mod system (WASM)

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-MOD-007
> Epic: FR-CIV-MOD

## Context

FR-CIV-MOD-007 is part of the FR-CIV-MOD epic. This functional requirement captures: Mod system (WASM).

Implementing crate: `crates/mod-host/src/`

### Referenced Source
- `docs/design/modding-platform.md:33`
- `docs/design/modding-platform.md:234`
- `docs/specs/CIV-0700-modding-api-spec.md:2404`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-MOD-007 needs to be finalized based on implementation exploration.

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
