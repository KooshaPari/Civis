# Stub fill plan: agent-D

You are **agent-D**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **57** stubs across 2 crates.

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

### FR-CIV-0104-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-001/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_001.rs`

### FR-CIV-0104-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-002/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_002.rs`

### FR-CIV-0104-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-003/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_003.rs`

### FR-CIV-0104-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-004/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_004.rs`

### FR-CIV-0104-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-005/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_005.rs`

### FR-CIV-0104-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-006/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_006.rs`

### FR-CIV-0104-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-007/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_007.rs`

### FR-CIV-0104-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-008/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_008.rs`

### FR-CIV-0104-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-009/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_009.rs`

### FR-CIV-0104-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-0104-010/`
- stub file: `crates/engine/tests/fr_fr_civ_0104_010.rs`

### FR-CIV-3D-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-001/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_001.rs`

### FR-CIV-3D-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-002/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_002.rs`

### FR-CIV-3D-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-003/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_003.rs`

### FR-CIV-3D-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-004/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_004.rs`

### FR-CIV-3D-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-005/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_005.rs`

### FR-CIV-3D-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-006/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_006.rs`

### FR-CIV-3D-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-007/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_007.rs`

### FR-CIV-3D-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-008/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_008.rs`

### FR-CIV-3D-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-009/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_009.rs`

### FR-CIV-3D-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-010/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_010.rs`

### FR-CIV-3D-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-011/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_011.rs`

### FR-CIV-3D-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-012/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_012.rs`

### FR-CIV-3D-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-013/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_013.rs`

### FR-CIV-3D-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-014/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_014.rs`

### FR-CIV-3D-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-3d-015/`
- stub file: `crates/engine/tests/fr_fr_civ_3d_015.rs`

### FR-CIV-MARKET-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-001/`
- stub file: `crates/economy/tests/fr_fr_civ_market_001.rs`

### FR-CIV-MARKET-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-002/`
- stub file: `crates/economy/tests/fr_fr_civ_market_002.rs`

### FR-CIV-MARKET-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-003/`
- stub file: `crates/economy/tests/fr_fr_civ_market_003.rs`

### FR-CIV-MARKET-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-004/`
- stub file: `crates/economy/tests/fr_fr_civ_market_004.rs`

### FR-CIV-MARKET-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-005/`
- stub file: `crates/economy/tests/fr_fr_civ_market_005.rs`

### FR-CIV-MARKET-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-006/`
- stub file: `crates/economy/tests/fr_fr_civ_market_006.rs`

### FR-CIV-MARKET-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-007/`
- stub file: `crates/economy/tests/fr_fr_civ_market_007.rs`

### FR-CIV-MARKET-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-market-008/`
- stub file: `crates/economy/tests/fr_fr_civ_market_008.rs`

### FR-CIV-QOL-100
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-100/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_100.rs`

### FR-CIV-QOL-110
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-110/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_110.rs`

### FR-CIV-QOL-120
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-120/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_120.rs`

### FR-CIV-QOL-130
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-130/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_130.rs`

### FR-CIV-QOL-140
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-140/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_140.rs`

### FR-CIV-QOL-150
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-150/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_150.rs`

### FR-CIV-QOL-160
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-160/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_160.rs`

### FR-CIV-QOL-170
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-170/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_170.rs`

### FR-CIV-QOL-180
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-180/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_180.rs`

### FR-CIV-QOL-190
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-190/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_190.rs`

### FR-CIV-QOL-200
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-200/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_200.rs`

### FR-CIV-QOL-210
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-210/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_210.rs`

### FR-CIV-QOL-220
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-220/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_220.rs`

### FR-CIV-QOL-230
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-qol-230/`
- stub file: `crates/engine/tests/fr_fr_civ_qol_230.rs`

### FR-CIV-VERIFY-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-001/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_001.rs`

### FR-CIV-VERIFY-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-002/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_002.rs`

### FR-CIV-VERIFY-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-003/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_003.rs`

### FR-CIV-VERIFY-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-004/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_004.rs`

### FR-CIV-VERIFY-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-005/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_005.rs`

### FR-CIV-VERIFY-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-006/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_006.rs`

### FR-CIV-VERIFY-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-007/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_007.rs`

### FR-CIV-VERIFY-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-008/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_008.rs`

### FR-CIV-VERIFY-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-009/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_009.rs`

### FR-CIV-VERIFY-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-verify-010/`
- stub file: `crates/engine/tests/fr_fr_civ_verify_010.rs`

