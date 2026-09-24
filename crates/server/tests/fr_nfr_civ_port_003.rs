//! Behavioral coverage for FR-NFR-CIV-PORT-003 — headless server runs
//! without a GPU backend on all platforms.
//!
//! `civ_server::portability` carries the headless build contract:
//! [`HEADLESS_BUILD_REQUIRED`] / [`headless_build_required`] are what the
//! CI step `build/headless-server-no-gpu` asserts before compiling the
//! server with no GPU feature flags. These tests pin that contract, the
//! GPU-free server manifest, and the agreement between the NFR statement
//! and the portability module about which crate is the headless surface.
//!
//! The 50-tick determinism behavioral test (the NFR's runtime acceptance
//! signal) lives in `crates/engine/tests/fr_nfr_civ_port_003.rs`.

use civ_server::portability::{headless_build_required, HEADLESS_BUILD_REQUIRED};

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(rel)
}

fn read_repo(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

/// Happy path: the headless build contract holds — CI can assert it
/// unconditionally before compiling the server with no GPU features.
#[test]
fn headless_build_contract_is_required() {
    assert!(
        HEADLESS_BUILD_REQUIRED,
        "the no-GPU headless build must be required unconditionally"
    );
    assert!(
        headless_build_required(),
        "the runtime accessor must agree with HEADLESS_BUILD_REQUIRED"
    );
}

/// Happy path + edge cases for the headless manifest: the server package
/// is the `civ-server` headless surface, declares no GPU backend token,
/// and has no `[features] default = [...]` list that could drag a GPU
/// feature into `--no-default-features` builds.
#[test]
fn server_manifest_is_gpu_free_with_no_default_feature_gate() {
    let manifest = read_repo("crates/server/Cargo.toml");
    let lower = manifest.to_ascii_lowercase();

    assert!(
        lower.contains("name = \"civ-server\""),
        "the headless server package must remain named civ-server"
    );
    for token in ["wgpu", "bevy", "vulkan", "dx12", "metal", "nvidia", "dlss"] {
        assert!(
            !lower.contains(token),
            "civ-server must not reference the GPU-related token {token:?}"
        );
    }

    // If a [features] table ever appears, its `default` must stay empty —
    // otherwise `--no-default-features` semantics change for CI.
    if let Some(features_pos) = lower.find("[features]") {
        let after = &lower[features_pos..];
        let section_end = after[1..]
            .find("\n[")
            .map(|i| i + 1)
            .unwrap_or(after.len());
        let features_section = &after[..section_end];
        if let Some(default_pos) = features_section.find("default") {
            let default_line = &features_section[default_pos..];
            let line_end = default_line.find('\n').unwrap_or(default_line.len());
            assert!(
                default_line[..line_end].contains("= []"),
                "civ-server default features must stay empty for headless builds, found: {:?}",
                &default_line[..line_end]
            );
        }
    }
}

/// Traceability: the NFR statement and the portability module agree on the
/// headless surface. The statement uses the historical `civlab-server`
/// name; `portability.rs` must keep documenting the mapping to the real
/// crate (`civ-server`) so CI doc lints and agents don't diverge.
#[test]
fn nfr_statement_and_portability_module_agree_on_headless_surface() {
    let nfr = read_repo("docs/reference/non-functional-requirements.md");
    let start = nfr
        .find("### NFR-CIV-PORT-003")
        .expect("NFR statement must contain §PORT-003");
    let end_rel = nfr[start..]
        .find("\n### ")
        .map(|i| start + i)
        .unwrap_or(nfr.len());
    let section = &nfr[start..end_rel];
    assert!(
        section.contains("cargo build -p civlab-server --no-default-features"),
        "§PORT-003 must pin the `--no-default-features` build command"
    );

    let portability = read_repo("crates/server/src/portability.rs");
    assert!(
        portability.contains("NFR-CIV-PORT-003"),
        "portability.rs must document its NFR-CIV-PORT-003 contract"
    );
    assert!(
        portability.contains("`civ-server`"),
        "portability.rs must document `civ-server` as the headless surface"
    );
    assert!(
        portability.contains("build/headless-server-no-gpu"),
        "portability.rs must name the CI step that enforces the headless build"
    );
}
