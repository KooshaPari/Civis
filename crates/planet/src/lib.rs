//! civ-planet — Single-planet geology + weather + day/night + moon tides; deterministic
//!
//! Part of the Civis 3D extension (feat/civis-3d-foundation).
//! See `docs/roadmap/civis-3d-extension.md` for the full design context.
//!
//! Functional requirements: FR-CIV-PLANET-*

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Marker version of this crate's public schema. Bumped on breaking changes
/// so replay (`.civreplay`) files can refuse to load mismatched versions.
pub const SCHEMA_VERSION: u32 = 0;

#[cfg(test)]
mod stub_tests {
    use super::*;

    /// FR-CIV-PLANET-000 — crate compiles and exposes a schema version.
    /// This is a placeholder until the first real FR test lands.
    #[test]
    fn schema_version_present() {
        assert_eq!(
            SCHEMA_VERSION, 0,
            "stub crate; bump when first real impl lands"
        );
    }
}
