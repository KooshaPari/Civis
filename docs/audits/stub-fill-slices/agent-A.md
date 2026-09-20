# Stub fill plan: agent-A

You are **agent-A**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **47** stubs across 1 crates.

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

### FR-CIV-BRUSH-01
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-01/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_01.rs`

### FR-CIV-BRUSH-02
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-02/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_02.rs`

### FR-CIV-BRUSH-03
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-03/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_03.rs`

### FR-CIV-BRUSH-04
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-04/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_04.rs`

### FR-CIV-BRUSH-05
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-05/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_05.rs`

### FR-CIV-BRUSH-06
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-06/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_06.rs`

### FR-CIV-BRUSH-07
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-07/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_07.rs`

### FR-CIV-BRUSH-08
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-08/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_08.rs`

### FR-CIV-BRUSH-09
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-09/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_09.rs`

### FR-CIV-BRUSH-10
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-10/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_10.rs`

### FR-CIV-BRUSH-11
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-11/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_11.rs`

### FR-CIV-BRUSH-12
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-12/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_12.rs`

### FR-CIV-BRUSH-13
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-brush-13/`
- stub file: `crates/engine/tests/fr_fr_civ_brush_13.rs`

### FR-CIV-RTS-NATION-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-nation-001/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_nation_001.rs`

### FR-CIV-RTS-NATION-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-nation-002/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_nation_002.rs`

### FR-CIV-RTS-RENDER-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-render-001/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_render_001.rs`

### FR-CIV-RTS-RENDER-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-render-002/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_render_002.rs`

### FR-CIV-RTS-RENDER-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-render-003/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_render_003.rs`

### FR-CIV-RTS-RENDER-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-render-004/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_render_004.rs`

### FR-CIV-RTS-RENDER-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-render-005/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_render_005.rs`

### FR-CIV-RTS-ZOOM-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-rts-zoom-001/`
- stub file: `crates/engine/tests/fr_fr_civ_rts_zoom_001.rs`

### FR-CIV-VEHICLE-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-001/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_001.rs`

### FR-CIV-VEHICLE-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-002/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_002.rs`

### FR-CIV-VEHICLE-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-003/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_003.rs`

### FR-CIV-VEHICLE-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-004/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_004.rs`

### FR-CIV-VEHICLE-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-005/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_005.rs`

### FR-CIV-VEHICLE-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-010/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_010.rs`

### FR-CIV-VEHICLE-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-011/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_011.rs`

### FR-CIV-VEHICLE-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-012/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_012.rs`

### FR-CIV-VEHICLE-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-013/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_013.rs`

### FR-CIV-VEHICLE-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-014/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_014.rs`

### FR-CIV-VEHICLE-020
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-020/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_020.rs`

### FR-CIV-VEHICLE-021
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-021/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_021.rs`

### FR-CIV-VEHICLE-022
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-022/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_022.rs`

### FR-CIV-VEHICLE-023
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-023/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_023.rs`

### FR-CIV-VEHICLE-024
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-024/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_024.rs`

### FR-CIV-VEHICLE-030
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-030/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_030.rs`

### FR-CIV-VEHICLE-040
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-040/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_040.rs`

### FR-CIV-VEHICLE-041
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-041/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_041.rs`

### FR-CIV-VEHICLE-042
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-042/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_042.rs`

### FR-CIV-VEHICLE-043
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-043/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_043.rs`

### FR-CIV-VEHICLE-044
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-044/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_044.rs`

### FR-CIV-VEHICLE-045
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-045/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_045.rs`

### FR-CIV-VEHICLE-046
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-046/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_046.rs`

### FR-CIV-VEHICLE-047
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-047/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_047.rs`

### FR-CIV-VEHICLE-050
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-050/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_050.rs`

### FR-CIV-VEHICLE-060
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-vehicle-060/`
- stub file: `crates/engine/tests/fr_fr_civ_vehicle_060.rs`

