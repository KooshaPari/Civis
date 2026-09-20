# Stub fill plan: agent-F

You are **agent-F**. Your job: convert each STUB-TEST-ONLY ID below into a real, FR-specific assertion.

You have **16** stubs across 2 crates.

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

### FR-CIV-AI-011
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-ai-011/`
- stub file: `crates/ai/tests/fr_fr_civ_ai_011.rs`

### FR-CIV-AI-012
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-ai-012/`
- stub file: `crates/ai/tests/fr_fr_civ_ai_012.rs`

### FR-CIV-AI-013
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-ai-013/`
- stub file: `crates/ai/tests/fr_fr_civ_ai_013.rs`

### FR-CIV-AI-014
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-ai-014/`
- stub file: `crates/ai/tests/fr_fr_civ_ai_014.rs`

### FR-CIV-AI-015
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-ai-015/`
- stub file: `crates/ai/tests/fr_fr_civ_ai_015.rs`

### FR-CIV-INFOVIEW-902
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-902/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_902.rs`

### FR-CIV-INFOVIEW-903
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-903/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_903.rs`

### FR-CIV-INFOVIEW-904
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-904/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_904.rs`

### FR-CIV-INFOVIEW-906
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-906/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_906.rs`

### FR-CIV-INFOVIEW-915
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-915/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_915.rs`

### FR-CIV-INFOVIEW-916
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-916/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_916.rs`

### FR-CIV-INFOVIEW-917
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-917/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_917.rs`

### FR-CIV-INFOVIEW-918
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-918/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_918.rs`

### FR-CIV-INFOVIEW-919
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-919/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_919.rs`

### FR-CIV-INFOVIEW-921
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-921/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_921.rs`

### FR-CIV-INFOVIEW-930
- implementing crate: `UNKNOWN`
- spec dir: `docs/traceability/fr-civ-infoview-930/`
- stub file: `crates/engine/tests/fr_fr_civ_infoview_930.rs`

