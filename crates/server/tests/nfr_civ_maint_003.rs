//! NFR-CIV-MAINT-003 — the 9-gate quality script enforces the code
//! duplication ceiling: `jscpd --threshold 5` runs as gate 8, detected
//! clusters fail the gate, and a failing gate exits non-zero to block merge.

use std::path::PathBuf;

fn quality_gate_script() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/quality/quality-gate.sh");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

// NFR-CIV-MAINT-003 — quality-gate.sh wires a jscpd duplication gate with a
// 5% ceiling: NFR traceability tag, threshold default, gate definition,
// main() invocation, clone-output classification, FAIL branch, non-zero exit.
#[test]
fn nfr_civ_maint_003_quality_gate_enforces_duplication_ceiling() {
    let script = quality_gate_script();

    // The gate carries this NFR's traceability tag.
    assert!(
        script.contains("# NFR-CIV-MAINT-003 — code duplication ceiling"),
        "gate 8 must carry the NFR-CIV-MAINT-003 traceability tag"
    );
    // Default ceiling: 5%.
    assert!(
        script.contains("DUPLICATION_THRESHOLD=5"),
        "duplication threshold must default to 5 percent"
    );
    // gate 8 is defined and invoked from main().
    assert!(
        script.contains("gate_8_duplication()"),
        "gate 8 must be defined"
    );
    assert!(
        script.contains("    gate_8_duplication\n"),
        "main() must invoke gate_8_duplication"
    );
    // jscpd runs against the whole project with the configured threshold.
    assert!(
        script.contains("--threshold \"${DUPLICATION_THRESHOLD}\""),
        "jscpd must run with --threshold ${DUPLICATION_THRESHOLD}"
    );
    // Detected clone output is classified as a gate failure.
    assert!(
        script.contains("\"duplicat.*found|clones found\""),
        "jscpd clone output must be classified as FAIL"
    );
    assert!(
        script.contains("log_gate 8 \"Duplication (<${DUPLICATION_THRESHOLD}%)\" FAIL"),
        "a detected duplication cluster must log gate 8 FAIL"
    );
    // Failed gates block the merge by exiting non-zero.
    assert!(
        script.contains("[[ ${FAILED_GATES} -gt 0 ]] && exit 1"),
        "failed gates must exit non-zero"
    );
    // The gate script is fail-fast shell.
    assert!(script.contains("set -euo pipefail"));
}
