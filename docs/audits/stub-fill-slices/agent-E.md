# Stub fill plan: agent-E

You are **agent-E**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **34** stubs across 1 crates.

## Workflow per ID

1. Open the spec dir listed for the ID and read the spec, intent, and ADR.
2. Locate the implementing crate (often listed; otherwise infer from the spec or grep the codebase for the ID).
3. Open the stub test file. Replace the placeholder body (currently `let ws = civ_engine::WorldState::default(); assert!(ws.tick == 0);`) with real assertions that exercise the FR's behavior.
4. Remove the `//! Stub: TDD-red ...` line from the file's doc-comment header.
5. Run `cargo test -p <crate> --tests` for the affected crate. Confirm green.
6. Commit with message `test(<crate>): real FR-XYZ-NNN assertions`.

## Constraints

- DO NOT regenerate the audit (`docs/audits/fr-matrix.json` etc.). The maintainer does that once all agents complete.
- DO NOT touch any file outside your assigned stub test files (and the implementing crate's source if you need to add minimal helpers).
- DO NOT modify any other stub test file, even if it looks easy.
- If the FR has no implementation anywhere, write a minimal one in the implementing crate (a struct + method stub is fine — but the test must call into it and assert non-trivial state, not just `Default::default()`).
- When done, post a summary of completed IDs and any blockers back to the manager.

## Assignments

### FR-CIV-CORE-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-006/`
- stub file: `crates/engine/tests/fr_fr_civ_core_006.rs`

### FR-CIV-CORE-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-007/`
- stub file: `crates/engine/tests/fr_fr_civ_core_007.rs`

### FR-CIV-CORE-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-008/`
- stub file: `crates/engine/tests/fr_fr_civ_core_008.rs`

### FR-CIV-CORE-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-009/`
- stub file: `crates/engine/tests/fr_fr_civ_core_009.rs`

### FR-CIV-CORE-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-010/`
- stub file: `crates/engine/tests/fr_fr_civ_core_010.rs`

### FR-CIV-CORE-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-011/`
- stub file: `crates/engine/tests/fr_fr_civ_core_011.rs`

### FR-CIV-CORE-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-012/`
- stub file: `crates/engine/tests/fr_fr_civ_core_012.rs`

### FR-CIV-CORE-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-014/`
- stub file: `crates/engine/tests/fr_fr_civ_core_014.rs`

### FR-CIV-CORE-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-015/`
- stub file: `crates/engine/tests/fr_fr_civ_core_015.rs`

### FR-CIV-CORE-016
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-016/`
- stub file: `crates/engine/tests/fr_fr_civ_core_016.rs`

### FR-CIV-CORE-017
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-017/`
- stub file: `crates/engine/tests/fr_fr_civ_core_017.rs`

### FR-CIV-CORE-018
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-018/`
- stub file: `crates/engine/tests/fr_fr_civ_core_018.rs`

### FR-CIV-CORE-DET-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-det-001/`
- stub file: `crates/engine/tests/fr_fr_civ_core_det_001.rs`

### FR-CIV-CORE-DET-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-det-002/`
- stub file: `crates/engine/tests/fr_fr_civ_core_det_002.rs`

### FR-CIV-CORE-DET-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-core-det-003/`
- stub file: `crates/engine/tests/fr_fr_civ_core_det_003.rs`

### FR-CIV-RTS-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-003/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_003.rs`

### FR-CIV-RTS-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-004/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_004.rs`

### FR-CIV-RTS-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-005/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_005.rs`

### FR-CIV-RTS-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-006/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_006.rs`

### FR-CIV-RTS-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-007/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_007.rs`

### FR-CIV-RTS-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-008/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_008.rs`

### FR-CIV-RTS-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-009/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_009.rs`

### FR-CIV-RTS-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-010/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_010.rs`

### FR-CIV-RTS-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-011/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_011.rs`

### FR-CIV-RTS-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-012/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_012.rs`

### FR-CIV-RTS-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-013/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_013.rs`

### FR-CIV-RTS-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-014/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_014.rs`

### FR-CIV-RTS-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-015/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_015.rs`

### FR-CIV-TERRAIN-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-001/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_001.rs`

### FR-CIV-TERRAIN-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-002/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_002.rs`

### FR-CIV-TERRAIN-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-003/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_003.rs`

### FR-CIV-TERRAIN-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-004/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_004.rs`

### FR-CIV-TERRAIN-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-005/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_005.rs`

### FR-CIV-TERRAIN-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-terrain-006/`
- stub file: `crates/engine/tests/fr_fr_civ_terrain_006.rs`

