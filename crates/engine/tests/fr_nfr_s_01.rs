//! Real coverage for FR-NFR-S-01 — max simultaneous WebSocket clients
//! (> 100 concurrent clients at 10 ticks/sec).
//!
//! This file replaces the previous placeholder body for FR-NFR-S-01 with
//! assertions against the acceptance signal in
//! `docs/traceability/fr-nfr-s-01/fr-nfr-s-01-intent.md`:
//!
//! * The §10.3 Scalability NFR row in
//!   `docs/models/civ-sim/TECHNICAL_SPEC.md` pins the metric, the strict
//!   `> 100` threshold, the 10 ticks/sec cadence, and the
//!   `tokio-tungstenite` load-test method.
//! * The behavioral 101-client load test exists, is bound to this FR ID,
//!   and carries real assertions (no placeholder markers).
//!
//! The behavioral load test itself lives in
//! `crates/server/tests/fr_nfr_s_01.rs`.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read_repo(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

const TECH_SPEC: &str = "docs/models/civ-sim/TECHNICAL_SPEC.md";
const INTENT: &str = "docs/traceability/fr-nfr-s-01/fr-nfr-s-01-intent.md";
const LOAD_TEST: &str = "crates/server/tests/fr_nfr_s_01.rs";

/// Extract the `### 10.3 Scalability` section body of the technical spec.
fn scalability_section() -> String {
    let doc = read_repo(TECH_SPEC);
    let heading = "### 10.3 Scalability";
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("{TECH_SPEC} must contain a `{heading}` section"));
    let rest = &doc[start..];
    let after_first_line = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
    let tail = &rest[after_first_line..];
    let end = tail
        .find("\n### ")
        .map(|i| i + after_first_line)
        .unwrap_or(rest.len());
    rest[..end].to_owned()
}

/// Happy path: the NFR row exists and pins all four properties of the
/// scalability target — metric name, strict `> 100` threshold, 10
/// ticks/sec cadence, and the tokio-tungstenite load-test method.
#[test]
fn nfr_row_pins_the_scalability_target() {
    let section = scalability_section();
    let row = section
        .lines()
        .find(|l| l.contains("NFR-S-01"))
        .expect("§10.3 must contain the NFR-S-01 row")
        .to_owned();

    assert!(
        row.contains("Max simultaneous WebSocket clients"),
        "NFR-S-01 row must name the metric: {row}"
    );
    assert!(
        row.contains("&gt; 100") || row.contains("> 100"),
        "NFR-S-01 row must pin the strict > 100 threshold: {row}"
    );
    assert!(
        row.contains("100 clients at 10 ticks/sec"),
        "NFR-S-01 row must pin the 10 ticks/sec cadence: {row}"
    );
    assert!(
        row.contains("tokio-tungstenite"),
        "NFR-S-01 row must name the tokio-tungstenite load-test method: {row}"
    );
}

/// Happy path: the behavioral load test exists, is bound to FR-NFR-S-01,
/// uses the strict > 100 client count (101), and contains real async
/// assertions rather than placeholder markers.
#[test]
fn behavioral_load_test_exists_with_real_assertions() {
    let test = read_repo(LOAD_TEST);
    assert!(
        test.contains("FR-NFR-S-01"),
        "{LOAD_TEST} must reference FR-NFR-S-01 for traceability"
    );
    assert!(
        test.contains("const CLIENTS: usize = 101"),
        "{LOAD_TEST} must spin up 101 clients (smallest integer > 100)"
    );
    assert!(
        test.contains("#[tokio::test"),
        "{LOAD_TEST} must be a real async test"
    );
    assert!(
        test.contains("assert!"),
        "{LOAD_TEST} must contain real assertions"
    );
    for marker in ["Stub: TDD-red", "Epic: auto-generated"] {
        assert!(
            !test.contains(marker),
            "{LOAD_TEST} must not carry the placeholder marker {marker:?}"
        );
    }
}

/// Edge case: the threshold is deliberately strict (`> 100`, not
/// `>= 100`) so there is slack above the number — the intent doc records
/// that reasoning, and the behavioral test honors it with 101 clients.
#[test]
fn threshold_is_strictly_greater_than_100_per_intent() {
    let intent = read_repo(INTENT);
    assert!(intent.contains("FR-NFR-S-01"));
    assert!(
        intent.contains("greater than 100"),
        "the intent doc must record the strict-greater-than threshold reasoning"
    );
    assert!(
        intent.contains("docs/models/civ-sim/TECHNICAL_SPEC.md"),
        "the intent doc must reference the technical spec statement of record"
    );
}
