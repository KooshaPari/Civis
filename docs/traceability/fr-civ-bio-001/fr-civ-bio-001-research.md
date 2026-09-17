# Research: FR-CIV-BIO-001 -- Biological simulation

> Status: RESEARCH-TEMPLATE (auto-generated 2026-09-17)
> FR: FR-CIV-BIO-001
> Epic: FR-CIV-BIO

## Research Question

What is the best approach to implement Biological simulation within the Civis simulation engine?

## Background

This FR belongs to the FR-CIV-BIO epic and is expected to be implemented in `crates/species/src/`.

### Existing Code References
- `docs/guides/voxel-emergent-vision-and-migration.md:33`
- `docs/guides/voxel-emergent-vision-and-migration.md:45`
- `docs/guides/voxel-emergent-vision-and-migration.md:62`
- `docs/guides/voxel-emergent-vision-and-migration.md:79`
- `docs/guides/voxel-emergent-vision-and-migration.md:97`
- `docs/guides/voxel-emergent-vision-and-migration.md:205`
- `docs/guides/voxel-emergent-vision-and-migration.md:207`
- `docs/reference/agileplus-artifacts-index.md:153`

### Test References
- `crates/build/tests/fr_matrix_batch12.rs:381`
- `crates/build/tests/fr_matrix_batch12.rs:384`

## Findings

### Codebase Analysis
- The `crates/species/` crate is the primary implementation target
- Existing patterns in this crate should be followed for consistency

### Feasibility
- Implementation feasibility: high (patterns exist in the codebase)
- Estimated complexity: medium

## Recommendations

1. Follow existing patterns in `crates/species/src/`
2. Add integration tests in `crates/species/tests/`
3. Update this research doc once implementation begins

## References

- `docs/AGILE_WORKSTREAM.md`
- `crates/species/` crate documentation
