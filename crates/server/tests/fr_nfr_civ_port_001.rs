//! Behavioral coverage for FR-NFR-CIV-PORT-001 — target platform matrix.
//!
//! `civ_server::portability` is the implementing code named by
//! NFR-CIV-PORT-001: [`PLATFORM_MATRIX`] is the canonical, compile-time
//! copy of the four-row matrix, and [`compile_target_triple`] /
//! [`platform_supported`] are what CI calls to assert the current host is
//! one of the supported rows.
//!
//! These tests close the loop between code and spec: the constant must
//! match the spec table row-for-row, and the host accessors must stay
//! consistent with the matrix. The doc-level lint of the NFR statement
//! lives in `crates/engine/tests/fr_nfr_civ_port_001.rs`.

use civ_server::portability::{
    compile_target_arch, compile_target_os, compile_target_triple, platform_supported,
    PlatformTarget, PLATFORM_MATRIX,
};

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(rel)
}

fn read_repo(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

/// Extract the `### NFR-CIV-PORT-001 …` section of the NFR statement.
fn port_001_section() -> String {
    let doc = read_repo("docs/reference/non-functional-requirements.md");
    let heading = "### NFR-CIV-PORT-001";
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("NFR statement must contain `{heading}`"));
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

/// Spec-platform label (`Windows 10/11`) → portability identifier (`windows`).
fn platform_id(spec_platform: &str) -> &'static str {
    if spec_platform.starts_with("Windows") {
        "windows"
    } else if spec_platform.starts_with("macOS") {
        "macos"
    } else if spec_platform.starts_with("Linux") {
        "linux"
    } else {
        panic!("unknown spec platform {spec_platform:?}")
    }
}

/// Spec-backend label (`DX12 (DLSS/Solari path)`) → portability identifier (`dx12`).
fn backend_id(spec_backend: &str) -> &'static str {
    match spec_backend.split_whitespace().next().unwrap_or_default().to_ascii_lowercase().as_str()
    {
        "vulkan" => "vulkan",
        "dx12" => "dx12",
        "metal" => "metal",
        other => panic!("unknown spec GPU backend {other:?}"),
    }
}

/// Parse the four `| Platform | GPU Backend | Build Target |` data rows
/// out of the NFR-CIV-PORT-001 statement table.
fn spec_rows() -> Vec<(String, String, String)> {
    port_001_section()
        .lines()
        .filter(|l| l.starts_with("| "))
        .filter_map(|line| {
            let cells: Vec<&str> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            match cells.as_slice() {
                [platform, backend, triple] if triple.starts_with('`') && triple.ends_with('`') => {
                    Some((
                        (*platform).to_owned(),
                        (*backend).to_owned(),
                        triple.trim_matches('`').to_owned(),
                    ))
                }
                _ => None,
            }
        })
        .collect()
}

/// Happy path: `PLATFORM_MATRIX` mirrors the spec table row-for-row, in
/// declaration order — platform, GPU backend, and target triple.
#[test]
fn platform_matrix_matches_nfr_spec_table_row_for_row() {
    let spec = spec_rows();
    assert_eq!(spec.len(), 4, "spec table must have exactly 4 rows: {spec:?}");
    assert_eq!(
        PLATFORM_MATRIX.len(),
        spec.len(),
        "code matrix and spec table must have the same row count"
    );

    for (code, (spec_platform, spec_backend, spec_triple)) in
        PLATFORM_MATRIX.iter().zip(spec.iter())
    {
        assert_eq!(
            code.platform,
            platform_id(spec_platform),
            "platform mismatch for row {spec_platform:?}"
        );
        assert_eq!(
            code.gpu_backend,
            backend_id(spec_backend),
            "GPU backend mismatch for row {spec_platform:?} / {spec_backend:?}"
        );
        assert_eq!(
            code.target_triple, spec_triple,
            "target triple mismatch for row {spec_platform:?}"
        );
    }
}

/// Happy path: the compile-time host accessors agree with the matrix —
/// the host triple is reported, and `platform_supported` is exactly the
/// statement "the host triple appears in `PLATFORM_MATRIX`".
#[test]
fn host_accessors_agree_with_the_matrix() {
    let os = compile_target_os();
    let arch = compile_target_arch();
    let triple = compile_target_triple();
    assert!(
        matches!(os, "windows" | "macos" | "linux"),
        "host OS must be one of the three supported platforms, got {os:?}"
    );

    let in_matrix = PLATFORM_MATRIX
        .iter()
        .any(|row| row.target_triple == triple);
    assert_eq!(
        platform_supported(),
        in_matrix,
        "platform_supported() must equal matrix membership for {triple}"
    );

    // The three canonical host configurations from the spec must all be
    // recognized as supported rows.
    if let ("windows" | "linux", "x86_64") | ("macos", "aarch64") = (os, arch) {
        assert!(
            platform_supported(),
            "canonical host {arch}-{os} ({triple}) must be a supported matrix row"
        );
    }
}

/// Edge cases: four rows, three distinct triples (Windows is listed twice —
/// Vulkan primary and the DX12 DLSS/Solari path), and only the three
/// sanctioned backends appear.
#[test]
fn matrix_shape_and_backend_edge_cases() {
    assert_eq!(PLATFORM_MATRIX.len(), 4);

    let windows_backends: Vec<&str> = PLATFORM_MATRIX
        .iter()
        .filter(|row| row.platform == "windows")
        .map(|row| row.gpu_backend)
        .collect();
    assert_eq!(
        windows_backends,
        ["vulkan", "dx12"],
        "Windows must keep both the Vulkan-primary and DX12 rows, in spec order"
    );

    let mut triples: Vec<&str> = PLATFORM_MATRIX
        .iter()
        .map(|row| row.target_triple)
        .collect();
    triples.sort_unstable();
    triples.dedup();
    assert_eq!(triples.len(), 3, "three distinct triples across four rows");

    for PlatformTarget {
        platform,
        gpu_backend,
        ..
    } in PLATFORM_MATRIX.iter()
    {
        assert!(matches!(*platform, "windows" | "macos" | "linux"));
        assert!(matches!(*gpu_backend, "vulkan" | "dx12" | "metal"));
    }
}
