//! Real coverage for FR-NFR-CIV-PORT-001 — target platform matrix.
//!
//! This file replaces the previous placeholder body for
//! FR-NFR-CIV-PORT-001 with assertions against the acceptance signal in
//! `docs/traceability/fr-nfr-civ-port-001/fr-nfr-civ-port-001-intent.md`:
//!
//! * The NFR statement (`docs/reference/non-functional-requirements.md`
//!   § NFR-CIV-PORT-001) pins all four platform/backend/target rows.
//! * The measurable target and verification method (CI matrix job
//!   `build/platform-matrix`) are recorded in the statement.
//! * CI actually builds on all three host operating systems.
//!
//! The behavioral counterpart — row-for-row parity between the spec
//! table and `civ_server::portability::PLATFORM_MATRIX` — lives in
//! `crates/server/tests/fr_nfr_civ_port_001.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read_repo(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

const NFR_DOC: &str = "docs/reference/non-functional-requirements.md";
const INTENT: &str = "docs/traceability/fr-nfr-civ-port-001/fr-nfr-civ-port-001-intent.md";

/// Extract the `### NFR-CIV-PORT-001 …` section body (up to the next heading).
fn port_001_section() -> String {
    let doc = read_repo(NFR_DOC);
    let heading = "### NFR-CIV-PORT-001";
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("{NFR_DOC} must contain a `{heading}` section"));
    let rest = &doc[start..];
    let after_first_line = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
    let tail = &rest[after_first_line..];
    let end = tail
        .find("\n### ")
        .or_else(|| tail.find("\n## "))
        .map(|i| i + after_first_line)
        .unwrap_or(rest.len());
    rest[..end].to_owned()
}

/// Happy path: the statement pins the exact four-row platform/backend
/// matrix (Windows Vulkan primary, Windows DX12 DLSS/Solari, macOS Metal,
/// Linux Vulkan) with their canonical build targets.
#[test]
fn nfr_statement_pins_all_four_platform_rows() {
    let section = port_001_section();
    for row in [
        "| Windows 10/11 | Vulkan (primary) | `x86_64-pc-windows-msvc` |",
        "| Windows 10/11 | DX12 (DLSS/Solari path) | `x86_64-pc-windows-msvc` |",
        "| macOS 12+ | Metal | `aarch64-apple-darwin` |",
        "| Linux (Ubuntu 22.04+) | Vulkan | `x86_64-unknown-linux-gnu` |",
    ] {
        assert!(
            section.contains(row),
            "NFR-CIV-PORT-001 statement must pin the row {row:?}"
        );
    }
}

/// Happy path: the measurable target (`cargo build --release` on all four
/// configurations) and the CI verification method (`build/platform-matrix`)
/// are recorded in the statement.
#[test]
fn measurable_target_and_verification_method_are_recorded() {
    let section = port_001_section();
    assert!(
        section.contains("cargo build --release"),
        "NFR-CIV-PORT-001 must state the `cargo build --release` measurable target"
    );
    assert!(
        section.contains("build/platform-matrix"),
        "NFR-CIV-PORT-001 must name the `build/platform-matrix` CI verification method"
    );
}

/// Edge case: exactly four matrix rows in the table, three distinct target
/// triples (Windows appears twice: Vulkan primary + DX12 path), so a fifth
/// row or a collapsed Windows row would break traceability.
#[test]
fn matrix_table_shape_has_four_rows_and_three_distinct_triples() {
    let section = port_001_section();
    let triple_rows: Vec<&str> = section
        .lines()
        .filter(|l| {
            l.starts_with("| ") && (l.contains("`x86_64") || l.contains("`aarch64"))
        })
        .collect();
    assert_eq!(
        triple_rows.len(),
        4,
        "expected exactly 4 platform rows, got {}: {triple_rows:?}",
        triple_rows.len()
    );

    let mut triples: Vec<&str> = triple_rows
        .iter()
        .filter_map(|row| {
            row.split('`').nth(1)
        })
        .collect();
    triples.sort_unstable();
    triples.dedup();
    assert_eq!(
        triples,
        [
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-gnu",
        ],
        "the matrix must cover exactly three distinct target triples"
    );
}

/// Enforcement: CI compiles the codebase on all three host OSes, which is
/// what catches a regression of any matrix row before it lands on main.
#[test]
fn ci_workflow_builds_on_all_three_host_operating_systems() {
    let ci = read_repo(".github/workflows/ci.yml");
    for token in ["ubuntu", "windows", "macos"] {
        assert!(
            ci.contains(token),
            "ci.yml must include a {token} runner so the platform matrix is enforced"
        );
    }
}

/// Traceability: the intent doc points at the NFR statement that carries
/// the full matrix.
#[test]
fn intent_doc_links_the_nfr_statement() {
    let intent = read_repo(INTENT);
    assert!(intent.contains("FR-NFR-CIV-PORT-001"));
    assert!(
        intent.contains(NFR_DOC),
        "the intent doc must reference {NFR_DOC} as the statement of record"
    );
}
