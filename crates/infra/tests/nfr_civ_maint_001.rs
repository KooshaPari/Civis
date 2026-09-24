//! NFR-CIV-MAINT-001 — quality-gate.sh coverage-threshold contract.

use std::path::PathBuf;
use std::process::Command;

/// Find a bash capable of executing the repo's quality-gate script.
fn find_bash() -> Option<String> {
    // Prefer Git's MSYS bash explicitly: a PATH `bash` may resolve to WSL's
    // System32 shim, which cannot open C:/... script paths (and boots slowly).
    for candidate in [
        r"C:\Program Files\Git\usr\bin\bash.exe",
        r"C:\Program Files\Git\bin\bash.exe",
        "bash",
    ] {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Run `quality-gate.sh` against `project_dir` and return (stdout, stderr).
fn run_quality_gate(bash: &str, script: &std::path::Path, project_dir: &std::path::Path) -> (String, String) {
    // Git Bash mangles backslash paths passed as argv; forward slashes are
    // accepted by bash and by the Win32 API it invokes underneath.
    let script_arg = script.to_string_lossy().replace('\\', "/");
    let out = Command::new(bash)
        .arg(&script_arg)
        .current_dir(project_dir)
        .env("PROJECT_DIR", project_dir)
        // Verbose must stay on: in non-verbose mode log_gate's PASS/SKIP echo
        // guard evaluates false as the last command, so the function returns
        // nonzero and quality-gate.sh's `set -euo pipefail` aborts before the
        // summary that reports the coverage threshold. Verbose prints the
        // per-gate lines and completes with the summary (verified on this host).
        .env("QUALITY_GATE_VERBOSE", "true")
        .output()
        .expect("quality-gate.sh must be executable");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// quality-gate.sh gate 5 reports its coverage threshold (default 80%) in the
// 9-gate summary for a stack-less project, and `quality-gate.yml`
// `thresholds.coverage` overrides it.
// NFR-CIV-MAINT-001 — coverage gate threshold is observable and configurable.
#[test]
fn nfr_civ_maint_001_quality_gate_reports_configurable_coverage_threshold() {
    let Some(bash) = find_bash() else {
        eprintln!("bash not available on this host; cannot execute quality-gate.sh");
        return;
    };
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let script = repo
        .join("scripts")
        .join("quality")
        .join("quality-gate.sh");
    assert!(
        script.is_file(),
        "quality-gate.sh must live at {}",
        script.display()
    );

    let tmp = tempfile::tempdir().expect("temp project dir");
    let project = tmp.path();

    // Run 1 — no config: the coverage gate must surface its default threshold.
    let (stdout, stderr) = run_quality_gate(&bash, &script, project);
    assert!(
        stdout.contains("Gate 5: Coverage (>=80%)"),
        "default coverage threshold must be 80%\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    // Run 2 — quality-gate.yml thresholds.coverage raises it to 90.
    // (Requires yq, which load_config needs to read the YAML; without yq the
    // override path cannot execute on this host, so only the default is checked.)
    let has_yq = Command::new("yq")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if has_yq {
        std::fs::write(project.join("quality-gate.yml"), "thresholds:\n  coverage: 90\n")
            .expect("write config");
        let (stdout, stderr) = run_quality_gate(&bash, &script, project);
        assert!(
            stdout.contains("Gate 5: Coverage (>=90%)"),
            "quality-gate.yml thresholds.coverage must override to 90%\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    } else {
        eprintln!("yq unavailable; skipping the thresholds.coverage override assertion");
    }
}
