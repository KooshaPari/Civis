# Plan: FR-CIV-POLITY-001 -- Polity system

> Date: 2026-09-17
> FR: FR-CIV-POLITY-001
> Epic: FR-CIV-POLITY
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/diplomacy/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Polity system logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/design/master-roadmap.md:25`
- `docs/design/polities-markets.md:37`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-CIV-POLITY
- Implementing crate: `crates/diplomacy/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p diplomacy`
2. `cargo test -p diplomacy`
3. `cargo clippy -p diplomacy`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
