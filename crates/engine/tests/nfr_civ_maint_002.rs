//! NFR-CIV-MAINT-002 — the quality-gate complexity caps must stay in lockstep
//! with the NFR specification they enforce.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

// NFR-CIV-MAINT-002 — scripts/quality/quality-gate.sh Gate 7 enforces the
// cyclomatic (≤ 10) and cognitive (≤ 15) caps that
// docs/reference/non-functional-requirements.md §NFR-CIV-MAINT-002 declares,
// and the gate is actually wired to radon cc, ruff C901, and gocyclo.
#[test]
fn nfr_civ_maint_002_quality_gate_complexity_caps_match_nfr_spec() {
    let script = read("scripts/quality/quality-gate.sh");
    let spec = read("docs/reference/non-functional-requirements.md");

    // Gate defaults must equal the NFR caps.
    assert!(
        script.contains("CYCLOMATIC_MAX=10"),
        "quality-gate.sh must default CYCLOMATIC_MAX to the NFR cap of 10"
    );
    assert!(
        script.contains("COGNITIVE_MAX=15"),
        "quality-gate.sh must default COGNITIVE_MAX to the NFR cap of 15"
    );

    // The NFR statement must pin the same numbers the gate enforces.
    assert!(
        spec.contains("cyclomatic complexity of 10"),
        "NFR-CIV-MAINT-002 spec must declare cyclomatic cap 10"
    );
    assert!(
        spec.contains("cognitive complexity of 15"),
        "NFR-CIV-MAINT-002 spec must declare cognitive cap 15"
    );

    // Gate 7 is the enforcing code path: it must consult both caps and wire
    // the three language complexity checkers named by the gate.
    let start = script
        .find("gate_7_complexity()")
        .expect("quality-gate.sh must define gate_7_complexity");
    let body = &script[start..];
    assert!(
        body.contains("CYCLOMATIC_MAX"),
        "gate 7 must consult CYCLOMATIC_MAX"
    );
    assert!(
        body.contains("COGNITIVE_MAX"),
        "gate 7 must report the COGNITIVE_MAX cap"
    );
    assert!(body.contains("radon cc"), "gate 7 must run radon cyclomatic check");
    assert!(
        body.contains("C901"),
        "gate 7 must run ruff's cognitive-complexity rule (C901)"
    );
    assert!(body.contains("gocyclo"), "gate 7 must run gocyclo for Go sources");
}
