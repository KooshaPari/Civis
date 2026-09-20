# Stub fill plan: agent-C

You are **agent-C**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **37** stubs across 1 crates.

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

### FR-CIV-LLM-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-001/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_001.rs`

### FR-CIV-LLM-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-002/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_002.rs`

### FR-CIV-LLM-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-003/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_003.rs`

### FR-CIV-LLM-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-004/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_004.rs`

### FR-CIV-LLM-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-005/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_005.rs`

### FR-CIV-LLM-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-llm-006/`
- stub file: `crates/engine/tests/fr_fr_civ_llm_006.rs`

### FR-CIV-PERF-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-002/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_002.rs`

### FR-CIV-PERF-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-003/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_003.rs`

### FR-CIV-PERF-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-004/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_004.rs`

### FR-CIV-PERF-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-005/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_005.rs`

### FR-CIV-PERF-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-006/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_006.rs`

### FR-CIV-PERF-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-007/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_007.rs`

### FR-CIV-PERF-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-008/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_008.rs`

### FR-CIV-PERF-009
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-009/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_009.rs`

### FR-CIV-PERF-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-011/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_011.rs`

### FR-CIV-PERF-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-012/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_012.rs`

### FR-CIV-PERF-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-013/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_013.rs`

### FR-CIV-PERF-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-014/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_014.rs`

### FR-CIV-PERF-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-015/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_015.rs`

### FR-CIV-PERF-016
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-016/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_016.rs`

### FR-CIV-PERF-017
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-017/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_017.rs`

### FR-CIV-PERF-018
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-018/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_018.rs`

### FR-CIV-PERF-019
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-019/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_019.rs`

### FR-CIV-PERF-020
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-020/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_020.rs`

### FR-CIV-PERF-BUILD-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-build-001/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_build_001.rs`

### FR-CIV-PERF-RT-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-rt-001/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_rt_001.rs`

### FR-CIV-PERF-RT-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-rt-002/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_rt_002.rs`

### FR-CIV-PERF-RT-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-rt-003/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_rt_003.rs`

### FR-CIV-PERF-WEB-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-perf-web-001/`
- stub file: `crates/engine/tests/fr_fr_civ_perf_web_001.rs`

### FR-CIV-POLITY-001
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-001/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_001.rs`

### FR-CIV-POLITY-002
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-002/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_002.rs`

### FR-CIV-POLITY-003
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-003/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_003.rs`

### FR-CIV-POLITY-004
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-004/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_004.rs`

### FR-CIV-POLITY-005
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-005/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_005.rs`

### FR-CIV-POLITY-006
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-006/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_006.rs`

### FR-CIV-POLITY-007
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-007/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_007.rs`

### FR-CIV-POLITY-008
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-polity-008/`
- stub file: `crates/engine/tests/fr_fr_civ_polity_008.rs`

