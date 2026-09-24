//! Real coverage for FR-NFR-CIV-DEV-HYGIENE-001 — history-purge plan
//! (build-artifact trees).
//!
//! This file replaces the previous placeholder body for
//! FR-NFR-CIV-DEV-HYGIENE-001 with assertions against the acceptance
//! signal defined in the intent doc:
//! `docs/traceability/fr-nfr-civ-dev-hygiene-001/fr-nfr-civ-dev-hygiene-001-intent.md`.
//!
//! The FR is procedural (a documented, gated procedure rather than
//! executable code), so the behavioral surface is the plan document
//! itself:
//!
//! * `docs/ops/history-purge-plan.md` exists and carries the FR trace
//!   line that ties it to NFR-CIV-DEV-HYGIENE-001.
//! * The plan documents the full `git-filter-repo` procedure: mirror
//!   backup first, freeze merges, the exact purge path set, and the
//!   post-purge verification command.
//! * The plan is a substantive document (not a one-line placeholder)
//!   and the intent doc links back to it.

use std::fs;
use std::path::{Path, PathBuf};

/// Resolve a repository-root-relative path from the engine crate manifest.
fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

/// Read a repository-root-relative file as UTF-8, panicking with the path.
fn read_repo(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

const PLAN: &str = "docs/ops/history-purge-plan.md";
const INTENT: &str =
    "docs/traceability/fr-nfr-civ-dev-hygiene-001/fr-nfr-civ-dev-hygiene-001-intent.md";

/// Happy path: the plan exists, is tracked at the exact path the FR names,
/// and carries the `Trace:` line that binds it to FR-NFR-CIV-DEV-HYGIENE-001.
#[test]
fn history_purge_plan_exists_with_fr_trace_line() {
    let plan = read_repo(PLAN);
    assert!(
        plan.contains("Trace: NFR-CIV-DEV-HYGIENE-001"),
        "{PLAN} must carry the `Trace: NFR-CIV-DEV-HYGIENE-001` line"
    );
    assert!(
        plan.contains("Status:"),
        "{PLAN} must declare an execution status"
    );
    assert!(
        plan.contains("git-filter-repo"),
        "{PLAN} must name the chosen purge tool"
    );
}

/// The plan must document the complete procedure: the safety gates come
/// *before* the destructive rewrite, the purge path set is exact, and the
/// post-purge acceptance command is spelled out.
#[test]
fn history_purge_plan_documents_gated_procedure_and_verification() {
    let plan = read_repo(PLAN);

    // The five path patterns PR #364 untracked (all must be purged).
    for pattern in [
        "target-check-build",
        "target-check-test2",
        "target-check-clippy3",
        "target-ci",
        ".target-*",
    ] {
        assert!(
            plan.contains(pattern),
            "{PLAN} must list the purge path pattern {pattern:?}"
        );
    }

    // Safety gates precede the destructive `git filter-repo` invocation.
    let backup_pos = plan
        .find("Mirror backup")
        .expect("plan must gate the purge behind a mirror backup");
    let freeze_pos = plan
        .find("Freeze merges")
        .expect("plan must gate the purge behind a merge freeze");
    let rewrite_pos = plan
        .find("git filter-repo --invert-paths")
        .expect("plan must contain the actual filter-repo command");
    assert!(
        backup_pos < rewrite_pos,
        "mirror backup must be scheduled before the history rewrite"
    );
    assert!(
        freeze_pos < rewrite_pos,
        "merge freeze must be scheduled before the history rewrite"
    );

    // Post-purge acceptance signal from the intent doc: the grep that must
    // return 0 after the purge executes.
    assert!(
        plan.contains("git rev-list --objects --all"),
        "{PLAN} must include the post-purge object-enumeration verification command"
    );
    assert!(
        plan.contains("target-check|target-ci"),
        "{PLAN} must include the grep pattern used to verify the purge"
    );
}

/// Edge cases: the plan is substantive (not a one-line placeholder), it is
/// explicitly a plan rather than a claim of completion, and the intent doc
/// points back at the same plan path so the traceability loop closes.
#[test]
fn history_purge_plan_is_substantive_and_intent_links_back() {
    let plan = read_repo(PLAN);
    let words = plan.split_whitespace().count();
    assert!(
        words >= 150,
        "{PLAN} looks like a placeholder: only {words} words"
    );
    assert!(
        !plan.contains("Status: DONE"),
        "{PLAN} claims completion, but the FR records the purge as gated/planned"
    );

    // Alternatives section proves the tool choice was deliberate.
    assert!(
        plan.contains("Alternatives considered"),
        "{PLAN} must record rejected alternatives (BFG, do-nothing)"
    );

    let intent = read_repo(INTENT);
    assert!(
        intent.contains(PLAN),
        "the intent doc must reference {PLAN} as its plan artifact"
    );
    assert!(
        intent.contains("FR-NFR-CIV-DEV-HYGIENE-001"),
        "the intent doc must name FR-NFR-CIV-DEV-HYGIENE-001"
    );
}
