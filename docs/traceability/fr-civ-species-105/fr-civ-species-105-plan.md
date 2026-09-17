# Plan: FR-CIV-SPECIES-105 -- Species definitions

> Date: 2026-09-17
> FR: FR-CIV-SPECIES-105
> Epic: FR-CIV-SPECIES
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/species/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Species definitions logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/design/species-sentience.md:78`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-CIV-SPECIES
- Implementing crate: `crates/species/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p species`
2. `cargo test -p species`
3. `cargo clippy -p species`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
