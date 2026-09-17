# Plan: FR-CIV-BIO-001 -- Biological simulation

> Date: 2026-09-17
> FR: FR-CIV-BIO-001
> Epic: FR-CIV-BIO
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/species/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Biological simulation logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/guides/voxel-emergent-vision-and-migration.md:33`
- `docs/guides/voxel-emergent-vision-and-migration.md:45`
- `docs/guides/voxel-emergent-vision-and-migration.md:62`
- `docs/guides/voxel-emergent-vision-and-migration.md:79`
- `docs/guides/voxel-emergent-vision-and-migration.md:97`
- `docs/guides/voxel-emergent-vision-and-migration.md:205`
- `docs/guides/voxel-emergent-vision-and-migration.md:207`
- `docs/reference/agileplus-artifacts-index.md:153`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:381`
- `crates/build/tests/fr_matrix_batch12.rs:384`

## Dependencies

- Epic: FR-CIV-BIO
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
