# Stub fill plan: agent-B

You are **agent-B**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **55** stubs across 4 crates.

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

### FR-CIV-ARCH-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-arch-006/`
- stub file: `crates/engine/tests/fr_fr_civ_arch_006.rs`

### FR-CIV-DET-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-det-001/`
- stub file: `crates/engine/tests/fr_fr_civ_det_001.rs`

### FR-CIV-LANG-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-lang-004/`
- stub file: `crates/engine/tests/fr_fr_civ_lang_004.rs`

### FR-CIV-LANG-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-lang-006/`
- stub file: `crates/engine/tests/fr_fr_civ_lang_006.rs`

### FR-CIV-LANG-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-lang-007/`
- stub file: `crates/engine/tests/fr_fr_civ_lang_007.rs`

### FR-CIV-LANG-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-lang-008/`
- stub file: `crates/engine/tests/fr_fr_civ_lang_008.rs`

### FR-CIV-LANG-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-lang-010/`
- stub file: `crates/engine/tests/fr_fr_civ_lang_010.rs`

### FR-CIV-LEGENDS-BROWSER-09
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-browser-09/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_browser_09.rs`

### FR-CIV-LEGENDS-CAUSAL-06
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-causal-06/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_causal_06.rs`

### FR-CIV-LEGENDS-GAP-12
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-gap-12/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_gap_12.rs`

### FR-CIV-LEGENDS-INSPECT-08
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-inspect-08/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_inspect_08.rs`

### FR-CIV-LEGENDS-NARRATOR-13
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-narrator-13/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_narrator_13.rs`

### FR-CIV-LEGENDS-PERSIST-11
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-persist-11/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_persist_11.rs`

### FR-CIV-LEGENDS-PRESIM-10
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-presim-10/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_presim_10.rs`

### FR-CIV-LEGENDS-PRODUCER-03
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-producer-03/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_producer_03.rs`

### FR-CIV-LEGENDS-RESOLVE-04
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-resolve-04/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_resolve_04.rs`

### FR-CIV-LEGENDS-SIG-05
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-legends-sig-05/`
- stub file: `crates/legends/tests/fr_fr_civ_legends_sig_05.rs`

### FR-CIV-MCP-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mcp-002/`
- stub file: `crates/civis-mcp/tests/fr_fr_civ_mcp_002.rs`

### FR-CIV-MCP-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mcp-004/`
- stub file: `crates/civis-mcp/tests/fr_fr_civ_mcp_004.rs`

### FR-CIV-MCP-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mcp-005/`
- stub file: `crates/civis-mcp/tests/fr_fr_civ_mcp_005.rs`

### FR-CIV-MCP-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mcp-006/`
- stub file: `crates/civis-mcp/tests/fr_fr_civ_mcp_006.rs`

### FR-CIV-MOD-000
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-000/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_000.rs`

### FR-CIV-MOD-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-002/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_002.rs`

### FR-CIV-MOD-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-003/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_003.rs`

### FR-CIV-MOD-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-004/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_004.rs`

### FR-CIV-MOD-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-005/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_005.rs`

### FR-CIV-MOD-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-006/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_006.rs`

### FR-CIV-MOD-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-007/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_007.rs`

### FR-CIV-MOD-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-008/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_008.rs`

### FR-CIV-MOD-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-009/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_009.rs`

### FR-CIV-MOD-010
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-010/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_010.rs`

### FR-CIV-MOD-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-011/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_011.rs`

### FR-CIV-MOD-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-012/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_012.rs`

### FR-CIV-MOD-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-013/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_013.rs`

### FR-CIV-MOD-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-014/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_014.rs`

### FR-CIV-MOD-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-015/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_015.rs`

### FR-CIV-MOD-016
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-016/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_016.rs`

### FR-CIV-MOD-017
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-017/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_017.rs`

### FR-CIV-MOD-018
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-018/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_018.rs`

### FR-CIV-MOD-019
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-019/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_019.rs`

### FR-CIV-MOD-020
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-mod-020/`
- stub file: `crates/mod-host/tests/fr_fr_civ_mod_020.rs`

### FR-DET-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-001/`
- stub file: `crates/engine/tests/fr_fr_det_001.rs`

### FR-DET-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-002/`
- stub file: `crates/engine/tests/fr_fr_det_002.rs`

### FR-DET-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-003/`
- stub file: `crates/engine/tests/fr_fr_det_003.rs`

### FR-DET-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-004/`
- stub file: `crates/engine/tests/fr_fr_det_004.rs`

### FR-DET-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-005/`
- stub file: `crates/engine/tests/fr_fr_det_005.rs`

### FR-DET-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-006/`
- stub file: `crates/engine/tests/fr_fr_det_006.rs`

### FR-DET-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-det-007/`
- stub file: `crates/engine/tests/fr_fr_det_007.rs`

### FR-DOC-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-doc-001/`
- stub file: `crates/engine/tests/fr_fr_doc_001.rs`

### FR-GUARD-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-guard-001/`
- stub file: `crates/engine/tests/fr_fr_guard_001.rs`

### FR-GUARD-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-guard-002/`
- stub file: `crates/engine/tests/fr_fr_guard_002.rs`

### FR-INT-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-int-001/`
- stub file: `crates/engine/tests/fr_fr_int_001.rs`

### FR-MET-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-met-001/`
- stub file: `crates/engine/tests/fr_fr_met_001.rs`

### FR-METRICS-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-metrics-004/`
- stub file: `crates/engine/tests/fr_fr_metrics_004.rs`

### FR-METRICS-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-metrics-005/`
- stub file: `crates/engine/tests/fr_fr_metrics_005.rs`

