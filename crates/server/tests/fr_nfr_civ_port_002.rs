//! Behavioral coverage for FR-NFR-CIV-PORT-002 — backend selection tradeoff ADR.
//!
//! `civ_server::portability::BACKEND_SELECTION_ADR` is the single
//! source-of-truth constant for the ADR path: CI doc lints call it to
//! verify the ADR exists, is substantive, matches the NFR statement, and
//! is referenced from the Bevy client's feature-flag documentation.
//!
//! The doc-level lint of the ADR content itself lives in
//! `crates/engine/tests/fr_nfr_civ_port_002.rs`.

use civ_server::portability::BACKEND_SELECTION_ADR;

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(rel)
}

fn read_repo(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

/// Happy path: the constant names a real, substantive ADR document that
/// covers both sides of the DLSS / Solari tradeoff.
#[test]
fn backend_selection_adr_constant_points_to_a_live_document() {
    assert_eq!(
        BACKEND_SELECTION_ADR, "docs/adr/backend-selection-dlss-vs-solari.md",
        "BACKEND_SELECTION_ADR must remain the canonical ADR path"
    );

    let adr = read_repo(BACKEND_SELECTION_ADR);
    let words = adr.split_whitespace().count();
    assert!(
        words >= 200,
        "the ADR at {BACKEND_SELECTION_ADR} must be ≥ 200 words, found {words}"
    );
    let lower = adr.to_ascii_lowercase();
    for token in ["dlss", "solari", "vulkan", "dx12"] {
        assert!(
            lower.contains(token),
            "the ADR must document the {token:?} side of the tradeoff"
        );
    }
}

/// Happy path: the constant and the NFR statement name the same file — a
/// path drift in either place fails the lint.
#[test]
fn adr_constant_matches_the_nfr_statement_path() {
    let nfr = read_repo("docs/reference/non-functional-requirements.md");
    let start = nfr
        .find("### NFR-CIV-PORT-002")
        .expect("NFR statement must contain §PORT-002");
    let end_rel = nfr[start..]
        .find("\n### ")
        .map(|i| start + i)
        .unwrap_or(nfr.len());
    let section = &nfr[start..end_rel];

    let path_start = section
        .find("docs/adr/")
        .expect("§PORT-002 must name an ADR path under docs/adr/");
    let path_end = path_start
        + section[path_start..]
            .find('`')
            .expect("ADR path must be backtick-quoted");
    let statement_path = &section[path_start..path_end];
    assert_eq!(
        statement_path, BACKEND_SELECTION_ADR,
        "NFR statement and BACKEND_SELECTION_ADR must agree on one path"
    );
}

/// Happy path: the Bevy client manifest references the constant's path, so
/// the feature-flag documentation points readers at the ADR.
#[test]
fn client_manifest_references_the_adr_constant_path() {
    let manifest = read_repo("clients/bevy-ref/Cargo.toml");
    assert!(
        manifest.contains(BACKEND_SELECTION_ADR),
        "clients/bevy-ref/Cargo.toml must reference {BACKEND_SELECTION_ADR}"
    );
    assert!(
        manifest.contains("solari = ["),
        "the `solari` cargo feature must remain defined in the client manifest"
    );
}

/// Edge case: the NFR statement's measurable target requires the ADR to be
/// non-stub (≥ 200 words) — the constant's document satisfies it, and the
/// statement itself promises exactly that bound.
#[test]
fn nfr_statement_requires_a_non_stub_adr() {
    let nfr = read_repo("docs/reference/non-functional-requirements.md");
    let start = nfr
        .find("### NFR-CIV-PORT-002")
        .expect("NFR statement must contain §PORT-002");
    let section = &nfr[start..];
    assert!(
        section.contains("200 words"),
        "§PORT-002 must record the ≥ 200 words measurable target"
    );
}
