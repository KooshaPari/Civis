//! Chunk-residency limit contract — FR-CIV-SCALE-001 (MVP).
//!
//! Defines the **simple, area-based residency contract** the FR-CIV-SCALE-001
//! MVP commits to: the active working set is bounded at **≤ 0.5 mi² resident**
//! and must host **at least one 256³ CA chunk**. This module is the
//! data/contract surface; the streaming layer, the linter, and the perf HUD all
//! read it back through [`ResidencyLimits`] / [`ResidencyLimits::validate`]
//! to assert they fit the MVP.
//!
//! ## Why a separate module
//!
//! The chunk-count budget ([`crate::scale_budget::MvpResidentBudget`]) is the
//! "how many chunks" answer; this module is the **"what block of world"**
//! answer, expressed directly in real-world units (square miles) and CA-chunk
//! edge length (voxels). The two compose: the streaming layer satisfies the
//! chunk-count budget AND the area/chunk-size budget simultaneously.
//!
//! ## Determinism
//!
//! All fields are plain `f32` and `u32`. The validator is a pure function of
//! `(limits, current_area, active_chunk_size)`; two clients with identical
//! inputs always agree on `Ok`/`Err`. Validation is independent of the
//! simulation tick, so it is safe to invoke at module init and on every
//! residency change.
//!
//! ## Functional requirement
//!
//! [`ResidencyLimits::FR_ID`] = `"FR-CIV-SCALE-001"`. See
//! `FUNCTIONAL_REQUIREMENTS.md` §FR-CIV-SCALE-001.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Default MVP resident area: 0.5 square miles.
///
/// FR-CIV-SCALE-001: "MVP must support ~0.5 mi² resident".
pub const DEFAULT_MAX_RESIDENT_AREA_SQ_MI: f32 = 0.5;

/// Default MVP minimum active CA-chunk edge length (in voxels).
///
/// FR-CIV-SCALE-001: "with at least one 256³ CA chunk active".
pub const DEFAULT_MIN_ACTIVE_CA_CHUNK_SIZE: u32 = 256;

/// Chunk-residency limit contract for the FR-CIV-SCALE-001 MVP.
///
/// The two limits describe the MVP working set:
///
/// - [`Self::max_resident_area_sq_mi`] caps **how much of the world** the
///   streaming layer is allowed to keep resident in RAM at any time (≈ 0.5
///   mi² in the MVP).
/// - [`Self::min_active_ca_chunk_size`] is the **minimum CA chunk edge
///   length** in voxels the active working set must host at all times
///   (256 voxels per side in the MVP — i.e. at least one 256³ CA chunk is
///   present and ticking).
///
/// `validate_residency` (the free function and the inherent
/// [`Self::validate`] alias) checks a `(current_area, active_chunk_size)`
/// pair against these limits and reports the first violation as a
/// [`ResidencyError`].
///
/// **Note:** this struct is the contract surface for *what the FR says*;
/// the streaming-layer implementation that actually enforces the cap (LRU
/// eviction, hard budgets, prefetch cone management) is a separate, later
/// FR. This crate delegates enforcement to [`crate::stream::StreamingWorld`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResidencyLimits {
    /// Maximum resident area in square miles. Current state is invalid
    /// when `current_area_sq_mi > max_resident_area_sq_mi`. Default:
    /// `0.5` (the FR-CIV-SCALE-001 MVP figure).
    pub max_resident_area_sq_mi: f32,
    /// Minimum active CA-chunk edge length in voxels. Current state is
    /// invalid when `active_chunk_size < min_active_ca_chunk_size`.
    /// Default: `256` (the FR-CIV-SCALE-001 MVP figure).
    pub min_active_ca_chunk_size: u32,
}

impl ResidencyLimits {
    /// FR-CIV-SCALE-001 stable identifier. Quoted by `Covers FR` test
    /// annotations and the FR matrix.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-001";

    /// MVP defaults: 0.5 mi² resident, ≥ 256³ CA chunk active.
    pub const MVP: Self = Self {
        max_resident_area_sq_mi: DEFAULT_MAX_RESIDENT_AREA_SQ_MI,
        min_active_ca_chunk_size: DEFAULT_MIN_ACTIVE_CA_CHUNK_SIZE,
    };

    /// Validate a `(current_area_sq_mi, active_chunk_size)` pair against
    /// these limits. Returns `Ok(())` if the current state is within the
    /// residency envelope, otherwise the first [`ResidencyError`]
    /// violation.
    ///
    /// This is a thin alias over the free function
    /// [`validate_residency`] kept for ergonomic call sites that already
    /// hold a [`ResidencyLimits`] value:
    ///
    /// ```ignore
    /// let limits = ResidencyLimits::MVP;
    /// limits.validate(0.5, 256)?; // Ok
    /// limits.validate(0.5, 128)?; // Err(TooSmall { .. })
    /// ```
    pub fn validate(
        &self,
        current_area_sq_mi: f32,
        active_chunk_size: u32,
    ) -> Result<(), ResidencyError> {
        validate_residency_impl(*self, current_area_sq_mi, active_chunk_size)
    }
}

impl Default for ResidencyLimits {
    fn default() -> Self {
        Self::MVP
    }
}

/// Error type returned by [`validate_residency`] when the current
/// residency state violates one of the [`ResidencyLimits`].
///
/// Ordered: `ExceedsArea` is checked first (the streaming-layer cap), then
/// `TooSmall` (the CA-chunk-size floor). One error per call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResidencyError {
    /// Current resident area exceeds `max_resident_area_sq_mi`.
    ExceedsArea {
        /// The resident area the validator was given.
        current_area_sq_mi_bits: u32,
        /// The area cap from the limits.
        max_resident_area_sq_mi_bits: u32,
    },
    /// Active CA-chunk edge length is below `min_active_ca_chunk_size`.
    TooSmall {
        /// The chunk-size the validator was given.
        active_chunk_size: u32,
        /// The minimum CA-chunk edge length from the limits.
        min_active_ca_chunk_size: u32,
    },
}

impl core::fmt::Display for ResidencyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ExceedsArea {
                current_area_sq_mi_bits,
                max_resident_area_sq_mi_bits,
            } => {
                let current = f32::from_bits(*current_area_sq_mi_bits);
                let max = f32::from_bits(*max_resident_area_sq_mi_bits);
                write!(
                    f,
                    "residency: current area {current} mi^2 exceeds limit {max} mi^2 (FR-CIV-SCALE-001)"
                )
            }
            Self::TooSmall {
                active_chunk_size,
                min_active_ca_chunk_size,
            } => write!(
                f,
                "residency: active chunk size {active_chunk_size} < minimum {min_active_ca_chunk_size} CA-chunk edge (FR-CIV-SCALE-001)"
            ),
        }
    }
}

impl std::error::Error for ResidencyError {}

/// Validate the current residency state against the FR-CIV-SCALE-001 MVP
/// residency contract.
///
/// Args:
/// - `current_area_sq_mi`: how much of the world is currently resident
///   (in square miles). Must be a non-negative finite `f32`. Negative
///   values are treated as zero (the validator does **not** panic on
///   negative input — it would be unhelpful to crash the streaming
///   boot path on a malformed measurement).
/// - `active_chunk_size`: edge length, in voxels, of the active CA
///   chunk. Zero means "no CA chunk active".
///
/// Returns `Ok(())` iff **both** invariants hold:
///
/// 1. `current_area_sq_mi <= limits.max_resident_area_sq_mi`, AND
/// 2. `active_chunk_size >= limits.min_active_ca_chunk_size`.
///
/// The boundary is **inclusive on both ends**: exactly 0.5 mi² with
/// exactly a 256³ chunk passes (this is the FR-CIV-SCALE-001 "supported"
/// assertion). Going one ULP over 0.5 mi², or reducing the chunk size
/// by one voxel, fails.
///
/// `NaN` / `inf` resident areas always fail with `ExceedsArea` (the FR
/// caps the resident area, and `NaN` comparisons are never `true`, so
/// they fall into the "exceeds" branch by the validator's design).
pub fn validate_residency(
    limits: ResidencyLimits,
    current_area_sq_mi: f32,
    active_chunk_size: u32,
) -> Result<(), ResidencyError> {
    validate_residency_impl(limits, current_area_sq_mi, active_chunk_size)
}

/// Internal validator shared by the free function and
/// [`ResidencyLimits::validate`]. The two entry points only differ in
/// how the limits are passed in (by value vs `&self`).
fn validate_residency_impl(
    limits: ResidencyLimits,
    current_area_sq_mi: f32,
    active_chunk_size: u32,
) -> Result<(), ResidencyError> {
    // Treat negative as zero — the streaming layer never produces
    // negative area, but a misbehaving caller can't crash the validator.
    let current = if current_area_sq_mi.is_sign_negative() {
        0.0
    } else {
        current_area_sq_mi
    };
    // The FR boundary is INCLUSIVE: exactly the MVP figure must pass.
    // `NaN` comparisons are always `false`, so a NaN resident area falls
    // into the "exceeds" branch (it is neither ≤ nor >, so we must
    // explicitly reject it).
    if current.is_nan() || current.is_infinite() || current > limits.max_resident_area_sq_mi {
        return Err(ResidencyError::ExceedsArea {
            current_area_sq_mi_bits: current.to_bits(),
            max_resident_area_sq_mi_bits: limits.max_resident_area_sq_mi.to_bits(),
        });
    }
    if active_chunk_size < limits.min_active_ca_chunk_size {
        return Err(ResidencyError::TooSmall {
            active_chunk_size,
            min_active_ca_chunk_size: limits.min_active_ca_chunk_size,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! FR-CIV-SCALE-001 unit tests for the residency-limit contract.
    //!
    //! Test names are `fr_civ_scale_001_residency_*` so the matrix
    //! scanner (`docs/audits/_gather_ids.py`) can link them back to the
    //! FR without ambiguity.
    use super::*;

    /// Helper: the FR-CIV-SCALE-001 defaults.
    fn mvp() -> ResidencyLimits {
        ResidencyLimits::MVP
    }

    /// FR-CIV-SCALE-001 — the MVP defaults are exactly what the FR
    /// asks for: 0.5 mi² resident, 256³ CA chunk.
    #[test]
    fn fr_civ_scale_001_residency_mvp_defaults_match_fr() {
        let limits = ResidencyLimits::MVP;
        assert_eq!(limits.max_resident_area_sq_mi, 0.5);
        assert_eq!(limits.min_active_ca_chunk_size, 256);
        assert_eq!(DEFAULT_MAX_RESIDENT_AREA_SQ_MI, 0.5);
        assert_eq!(DEFAULT_MIN_ACTIVE_CA_CHUNK_SIZE, 256);
        // `Default` matches `MVP` (callers can `.default()` and get the
        // same answer).
        assert_eq!(ResidencyLimits::default(), ResidencyLimits::MVP);
        assert_eq!(ResidencyLimits::FR_ID, "FR-CIV-SCALE-001");
    }

    /// FR-CIV-SCALE-001 — the validator passes at **exactly** the MVP
    /// figure (0.5 mi², 256³ chunk), since the boundary is inclusive.
    #[test]
    fn fr_civ_scale_001_residency_passes_at_exact_mvp_boundary() {
        let limits = mvp();
        // Both signatures — free function and inherent method.
        assert!(
            validate_residency(limits, 0.5, 256).is_ok(),
            "0.5 mi^2 + 256 chunk must satisfy the MVP"
        );
        assert!(
            limits.validate(0.5, 256).is_ok(),
            "limits.validate(0.5, 256) must also satisfy the MVP"
        );
    }

    /// FR-CIV-SCALE-001 — the validator rejects an area that exceeds
    /// the limit (the include-the-limit boundary + 1 ULP fails).
    #[test]
    fn fr_civ_scale_001_residency_rejects_area_over_limit() {
        let limits = mvp();
        // 0.500_001 mi² > 0.5 → ExceedsArea.
        let err = validate_residency(limits, 0.500_001, 256).unwrap_err();
        match err {
            ResidencyError::ExceedsArea {
                current_area_sq_mi_bits,
                max_resident_area_sq_mi_bits,
            } => {
                assert_eq!(f32::from_bits(current_area_sq_mi_bits), 0.500_001);
                assert_eq!(f32::from_bits(max_resident_area_sq_mi_bits), 0.5);
            }
            other => panic!("expected ExceedsArea, got {other:?}"),
        }
        // Far over the limit: also rejected.
        assert!(matches!(
            validate_residency(limits, 5.0, 256),
            Err(ResidencyError::ExceedsArea { .. })
        ));
        // Area-over check takes precedence over chunk-size check
        // (a single call returns one error, and `ExceedsArea` is
        // checked first).
        assert!(matches!(
            validate_residency(limits, 1.0, 64),
            Err(ResidencyError::ExceedsArea { .. })
        ));
    }

    /// FR-CIV-SCALE-001 — the validator rejects chunks smaller than the
    /// floor (255 < 256 → `TooSmall`), including the degenerate 0-chunk
    /// case ("no CA chunk active").
    #[test]
    fn fr_civ_scale_001_residency_rejects_chunk_below_minimum() {
        let limits = mvp();
        // 255 < 256 → TooSmall.
        let err = validate_residency(limits, 0.5, 255).unwrap_err();
        match err {
            ResidencyError::TooSmall {
                active_chunk_size,
                min_active_ca_chunk_size,
            } => {
                assert_eq!(active_chunk_size, 255);
                assert_eq!(min_active_ca_chunk_size, 256);
            }
            other => panic!("expected TooSmall, got {other:?}"),
        }
        // 0 chunks (no CA active) → TooSmall.
        assert!(matches!(
            validate_residency(limits, 0.0, 0),
            Err(ResidencyError::TooSmall { .. })
        ));
        // Way under: 32 < 256 → TooSmall.
        assert!(matches!(
            validate_residency(limits, 0.0, 32),
            Err(ResidencyError::TooSmall { .. })
        ));
    }

    /// FR-CIV-SCALE-001 — the error type's `Display` impl mentions the
    /// FR ID and is grep-friendly for diagnostic dumps.
    #[test]
    fn fr_civ_scale_001_residency_error_display_mentions_fr() {
        let over = ResidencyError::ExceedsArea {
            current_area_sq_mi_bits: 0.6_f32.to_bits(),
            max_resident_area_sq_mi_bits: 0.5_f32.to_bits(),
        };
        let msg = over.to_string();
        assert!(msg.contains("FR-CIV-SCALE-001"), "{msg}");
        assert!(msg.contains("0.6"), "{msg}");
        let too_small = ResidencyError::TooSmall {
            active_chunk_size: 128,
            min_active_ca_chunk_size: 256,
        };
        let msg = too_small.to_string();
        assert!(msg.contains("FR-CIV-SCALE-001"), "{msg}");
        assert!(msg.contains("128") && msg.contains("256"), "{msg}");
    }

    /// FR-CIV-SCALE-001 — `NaN` / `inf` resident areas are rejected
    /// (NaN comparisons are always false so they fall into
    /// `ExceedsArea`).
    #[test]
    fn fr_civ_scale_001_residency_nan_and_inf_rejected() {
        let limits = mvp();
        assert!(matches!(
            validate_residency(limits, f32::NAN, 256),
            Err(ResidencyError::ExceedsArea { .. })
        ));
        assert!(matches!(
            validate_residency(limits, f32::INFINITY, 256),
            Err(ResidencyError::ExceedsArea { .. })
        ));
    }

    /// FR-CIV-SCALE-001 — negative resident areas are clamped to 0 and
    /// pass the area check (only the chunk-size check fires when the
    /// area is "negative-but-clamped").
    #[test]
    fn fr_civ_scale_001_residency_negative_area_clamps_and_passes_area_check() {
        let limits = mvp();
        // Negative area + valid chunk → Ok (negative becomes 0).
        assert!(validate_residency(limits, -1.0, 256).is_ok());
        // Negative area + too-small chunk → TooSmall (area clamped to
        // 0, area check passes, chunk check fires).
        assert!(matches!(
            validate_residency(limits, -1.0, 128),
            Err(ResidencyError::TooSmall { .. })
        ));
    }

    /// FR-CIV-SCALE-001 — a tuned `ResidencyLimits` (smaller cap,
    /// larger minimum chunk) still validates with explicitly-tightened
    /// (`current_area`, `chunk_size`) pairs but rejects anything that
    /// would have passed the MVP defaults.
    #[test]
    fn fr_civ_scale_001_residency_tuned_limits_round_trip() {
        let tight = ResidencyLimits {
            max_resident_area_sq_mi: 0.25,
            min_active_ca_chunk_size: 512,
        };
        // Within tightened limits: both pass.
        assert!(tight.validate(0.25, 512).is_ok());
        // MVP-default area exceeds tightened cap.
        assert!(matches!(
            tight.validate(0.5, 512),
            Err(ResidencyError::ExceedsArea { .. })
        ));
        // MVP-default chunk is below tightened floor.
        assert!(matches!(
            tight.validate(0.1, 256),
            Err(ResidencyError::TooSmall { .. })
        ));
    }
}
