//! Bus ducking scheduler (FR-CIV-AUDIO-008).
//!
//! Audio-direction §1 (Tier 5 — Bus ducking) specifies:
//!
//! > When a disaster hits, a milestone lands, a UI modal opens, or
//! > a combat swell begins, the engine ducks the bed and score
//! > buses so the foreground event reads clearly. Tweens land in
//! > 150–400 ms; the moment the foreground ends, the buses
//! > restore.
//!
//! This module is a **deterministic, substrate-level** model of that
//! behavior. It exposes a [`DuckingScheduler`] that knows:
//!
//! 1. *How long* the tween lasts (per reason).
//! 2. *Where* the bus was before the duck (so `restore` returns
//!    cleanly).
//! 3. *Whether* the scheduler can layer concurrent ducks (it can:
//!    each reason is independent and the loudest one wins).
//!
//! The substrate does not call into any audio backend — it just
//! produces the level curve so the client can drive its DSP. Tests
//! are pure logic assertions on the curve.
//!
//! ## Tween band
//!
//! Audio-direction §1 sets the tween band at 150–400 ms. The
//! default reasons in [`DuckingTween::for_reason`] all sit in that
//! band:
//!
//! - `CombatSwell`: 200 ms (short — the swell is its own contrast).
//! - `UiModal`: 300 ms (default — between Duck-Trans).
//! - `Milestone`: 350 ms (civic-positive — gentle but firm).
//! - `Disaster`: 250 ms (foreground must read clearly).
//!
//! All values are configurable through [`DuckingTween::custom`].

use serde::{Deserialize, Serialize};

/// Why a bus duck is being requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuckingReason {
    /// A disaster started (audio-direction §1 tier 5).
    Disaster,
    /// A civic / cultural milestone landed.
    Milestone,
    /// A modal opened (e.g. console, trade-panel, scenario dialog).
    UiModal,
    /// Combat is escalating — score should yield to battle SFX.
    CombatSwell,
}

impl DuckingReason {
    /// Every reason in a stable order. Used by iterators / asserts.
    pub const ALL: [DuckingReason; 4] = [
        DuckingReason::Disaster,
        DuckingReason::Milestone,
        DuckingReason::UiModal,
        DuckingReason::CombatSwell,
    ];

    /// Display label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            DuckingReason::Disaster => "disaster",
            DuckingReason::Milestone => "milestone",
            DuckingReason::UiModal => "ui_modal",
            DuckingReason::CombatSwell => "combat_swell",
        }
    }
}

/// The tween curve parameters (milliseconds).
///
/// Tween length sits inside the audio-direction §1 band of
/// 150–400 ms. `target_level` is the relative gain the bus is
/// *ducked to* (`0.0` = mute, `1.0` = no duck).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DuckingTween {
    /// How long the tween takes, in milliseconds. Must be in
    /// `[150, 400]` per audio-direction §1.
    pub tween_ms: u32,
    /// The relative gain to duck to. `0.0` = full duck, `1.0` = no
    /// duck. The substrate uses `0.4` for the default reasons —
    /// enough contrast for the foreground to read.
    pub target_level: f32,
}

impl DuckingTween {
    /// Default tween for a reason. All defaults sit in the
    /// 150–400 ms band (audio-direction §1 tier 5).
    #[must_use]
    pub const fn for_reason(reason: DuckingReason) -> Self {
        match reason {
            DuckingReason::Disaster => Self {
                tween_ms: 250,
                target_level: 0.4,
            },
            DuckingReason::Milestone => Self {
                tween_ms: 350,
                target_level: 0.4,
            },
            DuckingReason::UiModal => Self {
                tween_ms: 300,
                target_level: 0.4,
            },
            DuckingReason::CombatSwell => Self {
                tween_ms: 200,
                target_level: 0.4,
            },
        }
    }

    /// Custom tween. Saturates `target_level` to `[0, 1]` and
    /// clamps `tween_ms` to the 150–400 ms band.
    #[must_use]
    pub const fn custom(tween_ms: u32, target_level: f32) -> Self {
        let ms = if tween_ms < 150 {
            150
        } else if tween_ms > 400 {
            400
        } else {
            tween_ms
        };
        let lvl = if target_level < 0.0 {
            0.0
        } else if target_level > 1.0 {
            1.0
        } else {
            target_level
        };
        Self {
            tween_ms: ms,
            target_level: lvl,
        }
    }
}

/// State of a single active duck per reason.
#[derive(Debug, Clone, Copy, PartialEq)]
struct DuckState {
    /// The tween we are currently applying.
    tween: DuckingTween,
    /// The bus level *before* the duck started. Used to restore.
    pre_duck_level: f32,
}

/// Deterministic ducking scheduler.
///
/// Owns per-reason duck state. Calling `apply` records the current
/// bus level as `pre_duck_level` and returns the tween curve.
/// Calling `restore` looks up the `pre_duck_level` and returns the
/// restore tween (always 200 ms — the foreground is over, no
/// need for a slow ramp). Concurrent ducks are layered: the
/// loudest active `target_level` wins; releasing a duck recomputes
/// the next-loudest active target.
///
/// The substrate is **frame-rate-independent**: it returns a curve
/// the client advances with its own `dt`. Tests advance with a
/// controlled `dt` so the math is exact.
#[derive(Debug, Clone, Default)]
pub struct DuckingScheduler {
    /// Active ducks per reason. `None` means "not currently
    /// ducking for this reason".
    active: [Option<DuckState>; 4],
}

impl DuckingScheduler {
    /// Construct an empty scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a duck is currently active for `reason`.
    #[must_use]
    pub fn is_ducking(&self, reason: DuckingReason) -> bool {
        self.active[reason as usize].is_some()
    }

    /// Apply a duck for `reason`. Records the pre-duck level and
    /// returns the tween to apply.
    ///
    /// If a duck for the same reason is already active, this is a
    /// no-op (the original tween keeps running) — duck stacking is
    /// by reason, not by call count. This matches the audio
    /// pillar: "the foreground reads clearly" but never ramp-clips.
    pub fn apply(&mut self, reason: DuckingReason, pre_duck_level: f32) -> DuckingTween {
        if self.active[reason as usize].is_none() {
            self.active[reason as usize] = Some(DuckState {
                tween: DuckingTween::for_reason(reason),
                pre_duck_level,
            });
        }
        DuckingTween::for_reason(reason)
    }

    /// Override the default tween for `reason` before applying.
    /// Useful for tests and for clients that want a longer / shorter
    /// ramp on a specific moment.
    pub fn apply_with_tween(
        &mut self,
        reason: DuckingReason,
        pre_duck_level: f32,
        tween: DuckingTween,
    ) -> DuckingTween {
        self.active[reason as usize] = Some(DuckState {
            tween,
            pre_duck_level,
        });
        tween
    }

    /// Release the duck for `reason` and return the *effective*
    /// tween the client should restore to. The restore tween is
    /// 200 ms by default — short, so the foreground ends cleanly.
    /// If no duck was active for `reason`, returns `None`.
    pub fn restore(&mut self, reason: DuckingReason) -> Option<(f32, DuckingTween)> {
        self.active[reason as usize].take().map(|state| {
            // The pre-duck level is the restore target.
            (
                state.pre_duck_level,
                DuckingTween {
                    tween_ms: 200,
                    target_level: state.pre_duck_level.clamp(0.0, 1.0),
                },
            )
        })
    }

    /// The currently loudest active duck target — the bus should
    /// be at `min(target_level for active reasons)`. Returns
    /// `None` if no ducks are active.
    #[must_use]
    pub fn effective_target(&self) -> Option<f32> {
        self.active
            .iter()
            .flatten()
            .map(|s| s.tween.target_level)
            .fold(None, |acc, lvl| Some(acc.map_or(lvl, |a: f32| a.min(lvl))))
    }

    /// Number of active ducks (for HUD / debug).
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.iter().filter(|s| s.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tweens_sit_in_150_400ms_band() {
        for r in DuckingReason::ALL {
            let t = DuckingTween::for_reason(r);
            assert!(
                t.tween_ms >= 150 && t.tween_ms <= 400,
                "{:?} tween_ms={} out of band",
                r,
                t.tween_ms
            );
        }
    }

    #[test]
    fn custom_clamps_tween_ms_to_band() {
        let lo = DuckingTween::custom(50, 0.5);
        assert_eq!(lo.tween_ms, 150);
        let hi = DuckingTween::custom(800, 0.5);
        assert_eq!(hi.tween_ms, 400);
        let mid = DuckingTween::custom(275, 0.5);
        assert_eq!(mid.tween_ms, 275);
    }

    #[test]
    fn custom_clamps_target_level() {
        assert_eq!(DuckingTween::custom(200, -0.1).target_level, 0.0);
        assert_eq!(DuckingTween::custom(200, 1.5).target_level, 1.0);
        assert!((DuckingTween::custom(200, 0.42).target_level - 0.42).abs() < 1e-5);
    }

    #[test]
    fn apply_records_duck() {
        let mut s = DuckingScheduler::new();
        let t = s.apply(DuckingReason::Disaster, 0.8);
        assert_eq!(t.tween_ms, 250);
        assert!(s.is_ducking(DuckingReason::Disaster));
        assert_eq!(s.active_count(), 1);
    }

    #[test]
    fn apply_is_idempotent_for_same_reason() {
        let mut s = DuckingScheduler::new();
        s.apply(DuckingReason::UiModal, 1.0);
        // Second apply for the same reason does not stack; the
        // first tween continues.
        let t = s.apply(DuckingReason::UiModal, 0.0);
        assert_eq!(t.tween_ms, 300);
        assert_eq!(s.active_count(), 1);
    }

    #[test]
    fn restore_returns_pre_duck_level_and_drops_duck() {
        let mut s = DuckingScheduler::new();
        s.apply(DuckingReason::CombatSwell, 0.85);
        let restored = s.restore(DuckingReason::CombatSwell);
        assert!(restored.is_some());
        let (level, tween) = restored.unwrap();
        assert!((level - 0.85).abs() < 1e-5);
        assert!(!s.is_ducking(DuckingReason::CombatSwell));
        // Restore tween is the 200 ms gentle ramp.
        assert_eq!(tween.tween_ms, 200);
    }

    #[test]
    fn restore_with_no_active_duck_returns_none() {
        let mut s = DuckingScheduler::new();
        assert!(s.restore(DuckingReason::Disaster).is_none());
    }

    #[test]
    fn concurrent_ducks_layer_and_loudest_wins() {
        let mut s = DuckingScheduler::new();
        s.apply(DuckingReason::Disaster, 1.0);
        s.apply_with_tween(DuckingReason::UiModal, 1.0, DuckingTween::custom(200, 0.2));
        // The loudest (lowest target_level) wins.
        assert!((s.effective_target().unwrap() - 0.2).abs() < 1e-5);
        // Restoring UiModal leaves Disaster active.
        s.restore(DuckingReason::UiModal);
        assert!(s.is_ducking(DuckingReason::Disaster));
        assert!(!s.is_ducking(DuckingReason::UiModal));
        // Now Disaster is the only one.
        assert!((s.effective_target().unwrap() - 0.4).abs() < 1e-5);
    }

    #[test]
    fn restore_clamps_pre_duck_level() {
        let mut s = DuckingScheduler::new();
        s.apply_with_tween(
            DuckingReason::Milestone,
            1.7,
            DuckingTween::for_reason(DuckingReason::Milestone),
        );
        let (_, tween) = s.restore(DuckingReason::Milestone).unwrap();
        assert!(tween.target_level <= 1.0);
    }

    /// Covers FR-CIV-AUDIO-008 — bus ducking. Asserts the
    /// tween-band (150–400 ms), the layer-then-restore model,
    /// and that Disaster / Milestone / UiModal / CombatSwell all
    /// participate with their default reasons.
    #[test]
    fn fr_audio_008_bus_ducking_lives_in_audio_direction_band() {
        let mut s = DuckingScheduler::new();
        for r in DuckingReason::ALL {
            let t = s.apply(r, 0.9);
            assert!(
                t.tween_ms >= 150 && t.tween_ms <= 400,
                "{:?} tween_ms={} out of band",
                r,
                t.tween_ms
            );
            // target_level is in [0, 1] (default 0.4).
            assert!(t.target_level >= 0.0 && t.target_level <= 1.0);
        }
        // All four reasons are active.
        assert_eq!(s.active_count(), 4);
        // Loudest target is the lowest (default 0.4 for all).
        assert!((s.effective_target().unwrap() - 0.4).abs() < 1e-5);
        // Restore each in order — Disaster, Milestone, UiModal, CombatSwell.
        for r in DuckingReason::ALL {
            assert!(s.restore(r).is_some());
        }
        assert_eq!(s.active_count(), 0);
        assert!(s.effective_target().is_none());
    }
}
