//! UI sound language — palette-role routing (FR-CIV-AUDIO-007).
//!
//! Audio-direction §1 (Tier 4 — UI sound language) defines a small
//! palette of UI sounds matching the visual language:
//!
//! | Role    | Color  | Sound family                       | `SfxKind`        |
//! |---------|--------|------------------------------------|------------------|
//! | Cyan    | cool   | soft click — UI feedback, info     | `UiClick`        |
//! |         |        |                                    | `UiHover`        |
//! | Gold    | warm   | confirm chime — civic / positive   | `UiConfirm`      |
//! | Cool    | muted  | cancel / dismiss — neutral         | `UiCancel`       |
//! | Acid    | hazard | low alert — hazard / warning       | `UiAlert`        |
//!
//! The "UI sound language" pillar is about *scarcity*: confirm /
//! alert sounds are reserved and rarely played, and a UISoundBudget
//! is used to clamp spammy UI triggers. This module provides:
//!
//! 1. [`UiSoundPalette`] — the role enum that maps to the project's
//!    visual palette (cyan / gold / cool / acid).
//! 2. [`SfxKind::palette_role`](crate::sfx::SfxKind::palette_role) —
//!    per-`SfxKind` lookup of its UI role.
//! 3. [`UiSoundBudget`] — a per-tick counter that drops excessive
//!    UI requests; the coalescer remains the source of truth, this
//!    is a fast *client-side* gate.
//!
//! Substrate only. The actual `UiSoundBudget::try_play` integration
//! happens in `clients/bevy-ref` (UI systems).

use serde::{Deserialize, Serialize};

use crate::sfx::SfxKind;

/// The UI sound palette roles (audio-direction §1 tier 4).
///
/// `Cyan` is the workhorse for routine UI feedback; `Gold` is
/// reserved for civic / positive confirms; `Cool` is the neutral
/// dismiss; `Acid` is the reserved hazard / alert role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiSoundPalette {
    /// Soft click / focus — UI feedback, info.
    Cyan,
    /// Confirm chime — civic / positive.
    Gold,
    /// Cancel / dismiss — neutral.
    Cool,
    /// Low alert — hazard / warning. Reserved, rarely played.
    Acid,
}

impl UiSoundPalette {
    /// Display label (palette color name).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            UiSoundPalette::Cyan => "cyan",
            UiSoundPalette::Gold => "gold",
            UiSoundPalette::Cool => "cool",
            UiSoundPalette::Acid => "acid",
        }
    }

    /// Priority weight used by [`UiSoundBudget`]. Higher = more
    /// likely to be kept under load.
    #[must_use]
    pub const fn priority(self) -> u8 {
        match self {
            UiSoundPalette::Cyan => 0,
            UiSoundPalette::Gold => 1,
            UiSoundPalette::Cool => 1,
            UiSoundPalette::Acid => 2,
        }
    }
}

/// Map a UI [`SfxKind`] to its [`UiSoundPalette`] role.
///
/// Non-UI kinds return `None` — they are not part of the UI sound
/// language. The substrate keeps the mapping exhaustive over the
/// `UiClick..UiAlert` slice so a future UI kind must be added here.
#[must_use]
pub fn ui_role(kind: SfxKind) -> Option<UiSoundPalette> {
    match kind {
        SfxKind::UiClick | SfxKind::UiHover => Some(UiSoundPalette::Cyan),
        SfxKind::UiConfirm => Some(UiSoundPalette::Gold),
        SfxKind::UiCancel => Some(UiSoundPalette::Cool),
        SfxKind::UiAlert => Some(UiSoundPalette::Acid),
        _ => None,
    }
}

/// Per-tick UI sound budget (audio-direction §1 tier 4: "UI sounds
/// are reserved and rarely played; UISoundBudget is used to clamp
/// spammy UI triggers").
///
/// Cheap to construct. The `try_play` method returns whether the
/// given `SfxKind` may play *this tick*, decrementing the budget.
/// `UiAlert` (acid role) is exempt from the budget — alerts must
/// always play through.
#[derive(Debug, Clone)]
pub struct UiSoundBudget {
    /// Remaining UI plays this tick. Decremented on `try_play`.
    remaining: u32,
    /// Max UI plays per tick. Default 4 (see `with_default`).
    max: u32,
}

impl Default for UiSoundBudget {
    fn default() -> Self {
        Self::with_default()
    }
}

impl UiSoundBudget {
    /// A `UiSoundBudget` with the default 4 plays-per-tick cap.
    #[must_use]
    pub const fn with_default() -> Self {
        Self { remaining: 4, max: 4 }
    }

    /// Custom budget (tests).
    #[must_use]
    pub const fn with_max(max: u32) -> Self {
        Self { remaining: max, max }
    }

    /// Remaining budget. Exposed for HUD readouts.
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Reset to `max` (call this at the top of each tick).
    pub fn refill(&mut self) {
        self.remaining = self.max;
    }

    /// Try to play a UI sound this tick. Returns `true` if it
    /// should be forwarded to the coalescer, `false` if the budget
    /// is exhausted. `UiAlert` is exempt (always plays).
    pub fn try_play(&mut self, kind: SfxKind) -> bool {
        if matches!(kind, SfxKind::UiAlert) {
            return true;
        }
        if ui_role(kind).is_none() {
            return false;
        }
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_role_maps_ui_kinds_to_palette() {
        assert_eq!(ui_role(SfxKind::UiClick), Some(UiSoundPalette::Cyan));
        assert_eq!(ui_role(SfxKind::UiHover), Some(UiSoundPalette::Cyan));
        assert_eq!(ui_role(SfxKind::UiConfirm), Some(UiSoundPalette::Gold));
        assert_eq!(ui_role(SfxKind::UiCancel), Some(UiSoundPalette::Cool));
        assert_eq!(ui_role(SfxKind::UiAlert), Some(UiSoundPalette::Acid));
    }

    #[test]
    fn ui_role_returns_none_for_non_ui_kinds() {
        assert_eq!(ui_role(SfxKind::Birth), None);
        assert_eq!(ui_role(SfxKind::Death), None);
        assert_eq!(ui_role(SfxKind::Battle), None);
        assert_eq!(ui_role(SfxKind::Disaster), None);
        assert_eq!(ui_role(SfxKind::Meteor), None);
    }

    #[test]
    fn palette_label_matches_visual_palette() {
        assert_eq!(UiSoundPalette::Cyan.label(), "cyan");
        assert_eq!(UiSoundPalette::Gold.label(), "gold");
        assert_eq!(UiSoundPalette::Cool.label(), "cool");
        assert_eq!(UiSoundPalette::Acid.label(), "acid");
    }

    #[test]
    fn alert_priority_is_highest() {
        assert!(UiSoundPalette::Acid.priority() > UiSoundPalette::Cyan.priority());
        assert!(UiSoundPalette::Gold.priority() >= UiSoundPalette::Cyan.priority());
    }

    #[test]
    fn budget_starts_full() {
        let b = UiSoundBudget::with_default();
        assert_eq!(b.remaining(), 4);
    }

    #[test]
    fn budget_decrements_on_try_play() {
        let mut b = UiSoundBudget::with_default();
        assert!(b.try_play(SfxKind::UiClick));
        assert_eq!(b.remaining(), 3);
        assert!(b.try_play(SfxKind::UiHover));
        assert_eq!(b.remaining(), 2);
    }

    #[test]
    fn budget_exhaustion_blocks_routine_ui() {
        let mut b = UiSoundBudget::with_max(2);
        assert!(b.try_play(SfxKind::UiClick));
        assert!(b.try_play(SfxKind::UiConfirm));
        // Third routine play should be dropped.
        assert!(!b.try_play(SfxKind::UiClick));
    }

    #[test]
    fn budget_refill_restores_capacity() {
        let mut b = UiSoundBudget::with_max(2);
        let _ = b.try_play(SfxKind::UiClick);
        let _ = b.try_play(SfxKind::UiHover);
        assert!(!b.try_play(SfxKind::UiClick));
        b.refill();
        assert!(b.try_play(SfxKind::UiClick));
    }

    #[test]
    fn ui_alert_is_exempt_from_budget() {
        let mut b = UiSoundBudget::with_max(0);
        // Budget is zero…
        assert!(!b.try_play(SfxKind::UiClick));
        // …but alerts always play.
        assert!(b.try_play(SfxKind::UiAlert));
        assert!(b.try_play(SfxKind::UiAlert));
    }

    #[test]
    fn try_play_returns_false_for_non_ui_kinds() {
        let mut b = UiSoundBudget::with_default();
        assert!(!b.try_play(SfxKind::Birth));
        assert!(!b.try_play(SfxKind::Tech));
        // Non-UI plays do not consume budget.
        assert_eq!(b.remaining(), 4);
    }

    /// Covers FR-CIV-AUDIO-007 — UI sound language. The
    /// assertions here mirror audio-direction §1 tier 4: cyan
    /// is the routine feedback role, gold / cool are the confirm /
    /// cancel pair, and acid (alert) is the reserved hazard
    /// role that should rarely be played.
    #[test]
    fn fr_audio_007_ui_sound_language_maps_to_visual_palette() {
        // Cyan is the workhorse.
        assert_eq!(ui_role(SfxKind::UiClick), Some(UiSoundPalette::Cyan));
        assert_eq!(ui_role(SfxKind::UiHover), Some(UiSoundPalette::Cyan));
        // Gold is positive civic confirm.
        assert_eq!(ui_role(SfxKind::UiConfirm), Some(UiSoundPalette::Gold));
        // Cool is the neutral cancel.
        assert_eq!(ui_role(SfxKind::UiCancel), Some(UiSoundPalette::Cool));
        // Acid is the reserved alert role.
        assert_eq!(ui_role(SfxKind::UiAlert), Some(UiSoundPalette::Acid));
        // Acid has the highest priority and is exempt from the
        // budget — alerts must always play.
        assert!(UiSoundPalette::Acid.priority() >= UiSoundPalette::Gold.priority());
        let mut b = UiSoundBudget::with_max(0);
        assert!(b.try_play(SfxKind::UiAlert));
        // Non-UI kinds are not part of the UI sound language.
        assert_eq!(ui_role(SfxKind::Birth), None);
    }
}
