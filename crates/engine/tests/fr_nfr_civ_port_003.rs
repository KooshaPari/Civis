//! Real coverage for FR-NFR-CIV-PORT-003 — headless server on all platforms.
//!
//! This file replaces the previous placeholder body for
//! FR-NFR-CIV-PORT-003 with assertions against the acceptance signal in
//! `docs/traceability/fr-nfr-civ-port-003/fr-nfr-civ-port-003-intent.md`:
//!
//! * The NFR statement requires `cargo build -p civlab-server
//!   --no-default-features` on all three OS targets and the 50-tick
//!   determinism test to pass on a GPU-less runner.
//! * Behavioral: two same-seed simulations advanced 50 ticks produce
//!   byte-identical state — the headless determinism contract the NFR
//!   depends on, exercised directly on engine code with no GPU involved.
//! * The headless server manifest declares no GPU backend dependency.
//!
//! The behavioral counterpart on `civ_server::portability`
//! (`HEADLESS_BUILD_REQUIRED`) lives in
//! `crates/server/tests/fr_nfr_civ_port_003.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use civ_engine::Simulation;

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read_repo(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

const NFR_DOC: &str = "docs/reference/non-functional-requirements.md";
const INTENT: &str = "docs/traceability/fr-nfr-civ-port-003/fr-nfr-civ-port-003-intent.md";

/// Extract the `### NFR-CIV-PORT-003 …` section body.
fn port_003_section() -> String {
    let doc = read_repo(NFR_DOC);
    let heading = "### NFR-CIV-PORT-003";
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

/// Happy path: the statement pins the no-GPU build command, the three OS
/// targets, and the 50-tick determinism acceptance signal.
#[test]
fn nfr_statement_requires_no_gpu_build_and_determinism() {
    let section = port_003_section();
    assert!(
        section.contains("cargo build -p civlab-server --no-default-features"),
        "NFR-CIV-PORT-003 must state the `--no-default-features` headless build command"
    );
    assert!(
        section.contains("Windows, macOS, Linux"),
        "NFR-CIV-PORT-003 must cover all three OS targets"
    );
    assert!(
        section.contains("50-tick determinism"),
        "NFR-CIV-PORT-003 must name the 50-tick determinism acceptance signal"
    );
    assert!(
        section.contains("no GPU"),
        "NFR-CIV-PORT-003 must state the GPU-less runner requirement"
    );
}

/// Behavioral (the acceptance signal itself): the headless determinism
/// contract — same seed ⇒ identical state after exactly 50 ticks. Runs on
/// plain engine code, no GPU backend or feature flags involved.
#[test]
fn fifty_tick_determinism_same_seed_identical_state() {
    const SEED: u64 = 0x50_77_30_03; // FR-NFR-CIV-PORT-003 marker seed
    let mut a = Simulation::with_seed(SEED);
    let mut b = Simulation::with_seed(SEED);
    a.advance_ticks(50);
    b.advance_ticks(50);

    assert_eq!(a.state.tick, 50, "sim A must reach tick 50");
    assert_eq!(b.state.tick, 50, "sim B must reach tick 50");
    let state_a = format!("{:?}", a.state);
    let state_b = format!("{:?}", b.state);
    assert_eq!(
        state_a, state_b,
        "same-seed simulations must be state-identical after 50 headless ticks"
    );
}

/// Edge case: determinism must be independent of tick segmentation —
/// advancing 25 + 25 ticks lands on the same state as one 50-tick run, so
/// no phase boundary leaks into the world state.
#[test]
fn fifty_tick_determinism_is_segmentation_invariant() {
    const SEED: u64 = 0x50_77_30_03;
    let mut one_shot = Simulation::with_seed(SEED);
    one_shot.advance_ticks(50);

    let mut segmented = Simulation::with_seed(SEED);
    segmented.advance_ticks(25);
    segmented.advance_ticks(25);

    assert_eq!(segmented.state.tick, one_shot.state.tick);
    assert_eq!(
        format!("{:?}", segmented.state),
        format!("{:?}", one_shot.state),
        "25 + 25 segmented ticks must equal a single 50-tick advance"
    );
}

/// Happy path: the headless server manifest declares no GPU backend
/// (wgpu / bevy / vendor APIs), so `--no-default-features` builds cannot
/// pull a GPU dependency in on a CI runner without a discrete GPU.
#[test]
fn headless_server_manifest_declares_no_gpu_backend() {
    let manifest = read_repo("crates/server/Cargo.toml").to_ascii_lowercase();
    for token in ["wgpu", "bevy", "vulkan", "dx12", "metal", "nvidia", "dlss"] {
        assert!(
            !manifest.contains(token),
            "crates/server/Cargo.toml must not depend on the GPU-related token {token:?}"
        );
    }
}

/// Traceability: the intent doc points at the NFR statement of record.
#[test]
fn intent_doc_links_the_nfr_statement() {
    let intent = read_repo(INTENT);
    assert!(intent.contains("FR-NFR-CIV-PORT-003"));
    assert!(
        intent.contains(NFR_DOC),
        "the intent doc must reference {NFR_DOC} as the statement of record"
    );
}
