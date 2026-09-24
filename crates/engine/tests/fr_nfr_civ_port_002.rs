//! Real coverage for FR-NFR-CIV-PORT-002 — backend selection tradeoff ADR.
//!
//! This file replaces the previous placeholder body for
//! FR-NFR-CIV-PORT-002 with assertions against the acceptance signal in
//! `docs/traceability/fr-nfr-civ-port-002/fr-nfr-civ-port-002-intent.md`:
//!
//! * `docs/adr/backend-selection-dlss-vs-solari.md` exists and is ≥ 200
//!   words (i.e. not a stub), documenting the DLSS-requires-Vulkan vs
//!   Solari-requires-DX12 tradeoff.
//! * The ADR is referenced from the `clients/` Cargo feature-flag
//!   documentation for the `dlss` / `solari` features.
//!
//! The behavioral counterpart — `civ_server::portability::BACKEND_SELECTION_ADR`
//! resolving to the same live document — lives in
//! `crates/server/tests/fr_nfr_civ_port_002.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read_repo(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

const ADR: &str = "docs/adr/backend-selection-dlss-vs-solari.md";
const CLIENT_MANIFEST: &str = "clients/bevy-ref/Cargo.toml";
const INTENT: &str = "docs/traceability/fr-nfr-civ-port-002/fr-nfr-civ-port-002-intent.md";

/// Happy path: the ADR exists, is bound to FR-NFR-CIV-PORT-002, and is a
/// substantive document (≥ 200 words, per the NFR measurable target) —
/// not a one-line placeholder.
#[test]
fn backend_selection_adr_exists_and_is_substantive() {
    let adr = read_repo(ADR);
    let words = adr.split_whitespace().count();
    assert!(
        words >= 200,
        "{ADR} must be ≥ 200 words per NFR-CIV-PORT-002, found {words}"
    );
    assert!(
        adr.contains("FR-NFR-CIV-PORT-002"),
        "{ADR} must carry the FR trace line"
    );
    assert!(
        adr.contains("Status: Accepted"),
        "{ADR} must record an acceptance status"
    );
}

/// Happy path: the ADR documents the actual tradeoff — both feature names,
/// both backends, the mutual exclusivity framing, the selection policy, and
/// what becomes unavailable on the non-selected backend.
#[test]
fn adr_documents_tradeoff_selection_policy_and_losses() {
    let adr = read_repo(ADR);
    let lower = adr.to_ascii_lowercase();
    for token in ["dlss", "solari", "vulkan", "dx12"] {
        assert!(
            lower.contains(token),
            "{ADR} must document the {token:?} side of the tradeoff"
        );
    }
    assert!(
        lower.contains("mutually exclusive"),
        "{ADR} must record the mutual-exclusivity framing from the NFR"
    );
    assert!(
        lower.contains("selection policy"),
        "{ADR} must state when each backend is selected"
    );
    assert!(
        lower.contains("unavailable on the non-selected"),
        "{ADR} must state which features are unavailable on the non-selected backend"
    );
    // Both cargo features must be named so feature-flag readers find them.
    assert!(
        lower.contains("`solari` cargo feature") || lower.contains("solari cargo feature"),
        "{ADR} must reference the `solari` cargo feature"
    );
    assert!(
        lower.contains("dlss") && lower.contains("cargo feature"),
        "{ADR} must explain how the `dlss` feature surface is exposed"
    );
    // Selection policy must point at the implementing code, not prose only.
    for code_ref in ["native_backend.rs", "gpu_features.rs"] {
        assert!(
            adr.contains(code_ref),
            "{ADR} must reference the implementing module {code_ref}"
        );
    }
}

/// Happy path: the Bevy client manifest references the ADR from inside its
/// `[features]` documentation, next to the `solari` feature definition, and
/// the comment covers the `dlss` side of the tradeoff too.
#[test]
fn client_manifest_references_the_adr_from_feature_docs() {
    let manifest = read_repo(CLIENT_MANIFEST);
    let adr_pos = manifest
        .find(ADR)
        .unwrap_or_else(|| panic!("{CLIENT_MANIFEST} must reference {ADR}"));
    let features_pos = manifest
        .find("[features]")
        .expect("client manifest must have a [features] section");
    assert!(
        adr_pos > features_pos,
        "the ADR reference must live inside the [features] documentation"
    );
    assert!(
        manifest.contains("solari = ["),
        "{CLIENT_MANIFEST} must define the `solari` feature"
    );
    // The comment block that carries the ADR reference must also name dlss.
    let comment_start = manifest[..adr_pos]
        .rfind('\n')
        .and_then(|i| manifest[..i].rfind('\n'))
        .unwrap_or(0);
    let comment_end = manifest[adr_pos..]
        .find("solari = [")
        .map(|i| adr_pos + i)
        .unwrap_or(manifest.len());
    let comment_zone = manifest[comment_start..comment_end].to_ascii_lowercase();
    assert!(
        comment_zone.contains("dlss"),
        "the feature-doc comment referencing the ADR must cover the `dlss` side"
    );
}

/// Edge case + traceability: the intent doc's named statement of record is
/// the NFR file, and that statement itself names this exact ADR path — so
/// spec, ADR, and manifest all agree on one path.
#[test]
fn intent_and_nfr_statement_agree_on_the_adr_path() {
    let intent = read_repo(INTENT);
    assert!(intent.contains("FR-NFR-CIV-PORT-002"));
    assert!(
        intent.contains("docs/reference/non-functional-requirements.md"),
        "the intent doc must reference the NFR statement of record"
    );

    let nfr = read_repo("docs/reference/non-functional-requirements.md");
    let pos = nfr
        .find("### NFR-CIV-PORT-002")
        .expect("NFR statement must contain §PORT-002");
    let section = &nfr[pos..];
    assert!(
        section.contains(ADR),
        "the NFR-CIV-PORT-002 statement must name the same ADR path {ADR}"
    );
}
