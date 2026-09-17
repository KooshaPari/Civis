# Plan: FR-SOC-INS-002 -- Social institution

> Date: 2026-09-17
> FR: FR-SOC-INS-002
> Epic: FR-SOC-INS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/civ-institutions/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Social institution logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1708`
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1999`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-SOC-INS
- Implementing crate: `crates/civ-institutions/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p civ-institutions`
2. `cargo test -p civ-institutions`
3. `cargo clippy -p civ-institutions`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
