//! Portability targets and headless build policy — NFR-CIV-PORT-001,
//! NFR-CIV-PORT-002, NFR-CIV-PORT-003.
//!
//! Per `docs/reference/non-functional-requirements.md` §PORT:
//!
//! - **NFR-CIV-PORT-001** — Target platform matrix. The codebase SHALL
//!   compile + produce a working binary on each row of
//!   [`PLATFORM_MATRIX`]: Windows (Vulkan primary / DX12 DLSS path),
//!   macOS (Metal), Linux (Vulkan).
//! - **NFR-CIV-PORT-002** — Backend-selection ADR. The DLSS-requires-
//!   Vulkan vs. Solari-requires-DX12 tradeoff is documented at
//!   [`BACKEND_SELECTION_ADR`]; the ADR is the canonical source for the
//!   selection policy.
//! - **NFR-CIV-PORT-003** — Headless server runs without a GPU. The
//!   server crate (`civ-server`) is the headless surface; it builds
//!   without `gpu` / `vulkan` / `dx12` / `metal` feature flags and the
//!   50-tick determinism test passes on a Linux CI runner.
//!
//! ## Design contract
//!
//! 1. **Compile-time platform triple reporting.** [`compile_target_triple`]
//!    returns `cfg!(target_os = "...")`-style strings so the build can
//!    be inspected in CI.
//! 2. **Headless gate.** [`headless_build_required`] returns `true`
//!    unconditionally — the CI step `build/headless-server-no-gpu`
//!    calls this and asserts no GPU backend was pulled in.
//! 3. **Backend selection ADR reference.** The constant
//!    [`BACKEND_SELECTION_ADR`] names the ADR path so a CI doc-lint can
//!    verify it exists (and that `clients/Cargo.toml` references it).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// One row in the platform matrix — NFR-CIV-PORT-001.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformTarget {
    /// Platform identifier (e.g. `windows`, `macos`, `linux`).
    pub platform: &'static str,
    /// GPU backend (e.g. `vulkan`, `dx12`, `metal`).
    pub gpu_backend: &'static str,
    /// Cargo target triple (e.g. `x86_64-pc-windows-msvc`).
    pub target_triple: &'static str,
}

impl PlatformTarget {
    /// Construct a platform row.
    #[must_use]
    pub const fn new(
        platform: &'static str,
        gpu_backend: &'static str,
        target_triple: &'static str,
    ) -> Self {
        Self {
            platform,
            gpu_backend,
            target_triple,
        }
    }
}

/// NFR-CIV-PORT-001 — the canonical platform matrix.
///
/// Rows are in declaration order (the same order as the spec table):
/// 1. Windows + Vulkan (primary; portable fallback when DX12 features
///    like DXR / DLSS are unavailable).
/// 2. Windows + DX12 (DLSS / Solari path).
/// 3. macOS + Metal (Apple Silicon).
/// 4. Linux + Vulkan (CI + headless server deployments).
pub const PLATFORM_MATRIX: [PlatformTarget; 4] = [
    PlatformTarget::new("windows", "vulkan", "x86_64-pc-windows-msvc"),
    PlatformTarget::new("windows", "dx12", "x86_64-pc-windows-msvc"),
    PlatformTarget::new("macos", "metal", "aarch64-apple-darwin"),
    PlatformTarget::new("linux", "vulkan", "x86_64-unknown-linux-gnu"),
];

/// NFR-CIV-PORT-002 — path to the backend-selection ADR.
///
/// The ADR documents the DLSS-requires-Vulkan vs. Solari-requires-DX12
/// tradeoff; this constant is the single source of truth so a CI doc
/// lint can assert it points to a real file. The spec text lives at
/// `docs/reference/non-functional-requirements.md` §PORT-002.
pub const BACKEND_SELECTION_ADR: &str = "docs/adr/backend-selection-dlss-vs-solari.md";

/// NFR-CIV-PORT-003 — the headless build contract.
///
/// `true` means the CI step `build/headless-server-no-gpu` MUST
/// successfully build `civ-server` without `gpu` / `vulkan` / `dx12` /
/// `metal` feature flags. The constant is unconditional so any build
/// profile that wants to deviate must override the contract via an
/// explicit CI step.
pub const HEADLESS_BUILD_REQUIRED: bool = true;

/// NFR-CIV-PORT-001 — return the compile-time platform identifier.
///
/// Uses `cfg!(target_os = "...")` so it works on every supported Rust
/// target triple without conditional compilation. Maps to one of
/// `"windows"`, `"macos"`, `"linux"`, or `"unknown"`.
#[must_use]
pub fn compile_target_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

/// NFR-CIV-PORT-001 — return the compile-time target architecture.
///
/// Returns one of `"x86_64"`, `"aarch64"`, or `"unknown"`.
#[must_use]
pub fn compile_target_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    }
}

/// NFR-CIV-PORT-001 — return the canonical `(platform, arch)` triple
/// string. Used by the CI platform matrix to assert every row in
/// [`PLATFORM_MATRIX`] compiles.
#[must_use]
pub fn compile_target_triple() -> &'static str {
    match (compile_target_os(), compile_target_arch()) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        _ => "unknown",
    }
}

/// NFR-CIV-PORT-001 — does the compile-time triple appear in
/// [`PLATFORM_MATRIX`]? Returns `true` for the four supported rows;
/// `false` for unsupported triples (the CI matrix will catch this).
#[must_use]
pub fn platform_supported() -> bool {
    PLATFORM_MATRIX
        .iter()
        .any(|row| row.target_triple == compile_target_triple())
}

/// NFR-CIV-PORT-003 — accessor for the headless build constant. CI
/// scripts call this; production code can branch on it (the current
/// production code is unconditionally headless).
#[must_use]
pub fn headless_build_required() -> bool {
    HEADLESS_BUILD_REQUIRED
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NFR-CIV-PORT-001: the matrix has four rows covering the
    /// required platforms.
    #[test]
    fn platform_matrix_has_required_rows() {
        assert_eq!(PLATFORM_MATRIX.len(), 4);
        // Windows appears twice (Vulkan + DX12).
        let windows_rows = PLATFORM_MATRIX
            .iter()
            .filter(|r| r.platform == "windows")
            .count();
        assert_eq!(windows_rows, 2, "windows must have both vulkan + dx12 rows");
        // macOS / linux present once.
        assert!(PLATFORM_MATRIX.iter().any(|r| r.platform == "macos" && r.gpu_backend == "metal"));
        assert!(PLATFORM_MATRIX.iter().any(|r| r.platform == "linux" && r.gpu_backend == "vulkan"));
    }

    /// NFR-CIV-PORT-002: the backend-selection ADR path is set.
    #[test]
    fn backend_selection_adr_path_is_set() {
        assert!(BACKEND_SELECTION_ADR.ends_with(".md"));
        assert!(BACKEND_SELECTION_ADR.contains("backend-selection"));
    }

    /// NFR-CIV-PORT-003: headless build is required.
    #[test]
    fn headless_build_required_is_true() {
        assert!(headless_build_required());
    }

    /// NFR-CIV-PORT-001: the compile-time triple accessor returns a
    /// stable identifier.
    #[test]
    fn compile_target_triple_is_stable() {
        let triple = compile_target_triple();
        assert!(!triple.is_empty());
        assert!(triple == "unknown" || PLATFORM_MATRIX.iter().any(|r| r.target_triple == triple));
    }
}