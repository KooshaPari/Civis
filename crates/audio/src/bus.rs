//! The four-tier bus identity ([`BusId`]) and the linear-gain levels
//! ([`BusLevels`]) that get serialised into the player settings file.
//!
//! Audio-direction §1 specifies the four-tier mix tree
//! (Ambient / Score / Sfx / Ui under a single Master). This module
//! owns the bus *identity* (one `BusId` per tier, no allocation) and
//! the *levels* resource (linear gain per bus) — the cross-fade
//! math, tween constants, and the ducking rules live in `super::mix`
//! and the engine plugin.

use serde::{Deserialize, Serialize};

/// Identifier of a bus in the four-tier mix tree.
///
/// `Master` is the pre-fader bus sum; the four tier buses are mixed
/// into it. Ducking tweens in `super::mix` operate on tier buses
/// only; `Master` is the player volume slider.
///
/// The numeric tag is wire-stable — reordering variants is a breaking
/// change to the settings file format. Append new variants at the
/// end only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusId {
    /// Pre-fader sum of every tier. The player master volume slider.
    Master,
    /// Tier 1 — environmental ambient beds (looping, cross-faded).
    Ambient,
    /// Tier 2 — adaptive emergent score (4 stems, mood-driven remix).
    Score,
    /// Tier 3 — reactive one-shot world events (Birth/Death/Build/...).
    Sfx,
    /// Tier 4 — interface sound language (Click/Hover/Confirm/Alert).
    Ui,
}

impl BusId {
    /// All four tier buses, in mix-tree order.
    ///
    /// `Master` is intentionally excluded — it is the sum bus, not a
    /// content tier. Callers iterating for "all tiers" should use this.
    pub const TIERS: [BusId; 4] = [BusId::Ambient, BusId::Score, BusId::Sfx, BusId::Ui];

    /// `true` for tier buses (excludes `Master`).
    pub fn is_tier(self) -> bool {
        !matches!(self, BusId::Master)
    }

    /// Human-readable label, used in the settings UI labels.
    pub fn label(self) -> &'static str {
        match self {
            BusId::Master => "Master",
            BusId::Ambient => "Ambient",
            BusId::Score => "Score",
            BusId::Sfx => "SFX",
            BusId::Ui => "UI",
        }
    }
}

/// Linear-gain levels for the four tiers + the master sum.
///
/// Values are in `[0.0, 1.0]` linear (NOT dB). The kira backend applies
/// 20*log10 on its side; the substrate stays linear so designers can
/// reason about mix proportions in the same domain as `Volume`
/// tweens. `Master` defaults to `1.0` (unity pass-through); the tier
/// defaults follow audio-direction §3.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BusLevels {
    /// Master sum bus — the player slider.
    pub master: f32,
    /// Tier 1 — ambient beds.
    pub ambient: f32,
    /// Tier 2 — score stems.
    pub score: f32,
    /// Tier 3 — SFX.
    pub sfx: f32,
    /// Tier 4 — UI clicks.
    pub ui: f32,
}

impl BusLevels {
    /// Default mix per audio-direction §3.
    pub const DEFAULTS: BusLevels = BusLevels {
        master: 1.0,
        ambient: 0.35,
        score: 0.30,
        sfx: 0.70,
        ui: 0.55,
    };

    /// Read the gain for a given bus.
    pub fn get(self, bus: BusId) -> f32 {
        match bus {
            BusId::Master => self.master,
            BusId::Ambient => self.ambient,
            BusId::Score => self.score,
            BusId::Sfx => self.sfx,
            BusId::Ui => self.ui,
        }
    }

    /// Set the gain for a given bus, clamped to `[0.0, 1.0]`.
    ///
    /// Out-of-range inputs are clamped (not rejected) so a settings
    /// deserialiser fed stale data never panics. Returns the
    /// post-clamp value actually written.
    pub fn set(&mut self, bus: BusId, gain: f32) -> f32 {
        let clamped = gain.clamp(0.0, 1.0);
        match bus {
            BusId::Master => self.master = clamped,
            BusId::Ambient => self.ambient = clamped,
            BusId::Score => self.score = clamped,
            BusId::Sfx => self.sfx = clamped,
            BusId::Ui => self.ui = clamped,
        }
        clamped
    }

    /// Returns `true` if every bus sits in `[0.0, 1.0]`. Used by the
    /// settings round-trip guard to fail loud on corrupt files.
    pub fn is_well_formed(self) -> bool {
        self.master.is_finite()
            && self.ambient.is_finite()
            && self.score.is_finite()
            && self.sfx.is_finite()
            && self.ui.is_finite()
            && (0.0..=1.0).contains(&self.master)
            && (0.0..=1.0).contains(&self.ambient)
            && (0.0..=1.0).contains(&self.score)
            && (0.0..=1.0).contains(&self.sfx)
            && (0.0..=1.0).contains(&self.ui)
    }
}

impl Default for BusLevels {
    fn default() -> Self {
        Self::DEFAULTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_iteration_excludes_master() {
        let tiers = BusId::TIERS;
        assert_eq!(tiers.len(), 4);
        assert!(tiers.iter().all(|b| b.is_tier()));
        assert!(!BusId::Master.is_tier());
    }

    #[test]
    fn defaults_follow_audio_direction_table() {
        let d = BusLevels::DEFAULTS;
        // Audio-direction §3: Ambient 0.35, Score 0.30, Sfx 0.70, Ui 0.55, Master 1.0.
        assert!((d.ambient - 0.35).abs() < f32::EPSILON);
        assert!((d.score - 0.30).abs() < f32::EPSILON);
        assert!((d.sfx - 0.70).abs() < f32::EPSILON);
        assert!((d.ui - 0.55).abs() < f32::EPSILON);
        assert!((d.master - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_clamps_out_of_range_to_unit_range() {
        let mut lv = BusLevels::default();
        let written = lv.set(BusId::Sfx, 5.0);
        assert!((written - 1.0).abs() < f32::EPSILON);
        assert!((lv.get(BusId::Sfx) - 1.0).abs() < f32::EPSILON);

        let written = lv.set(BusId::Ui, -0.25);
        assert!(written.abs() < f32::EPSILON);
        assert!(lv.get(BusId::Ui).abs() < f32::EPSILON);
    }

    #[test]
    fn is_well_formed_rejects_nan_and_out_of_range() {
        let mut bad = BusLevels::default();
        assert!(bad.is_well_formed());

        bad.ambient = f32::NAN;
        assert!(!bad.is_well_formed());

        let bad = BusLevels {
            sfx: 1.5,
            ..Default::default()
        };
        assert!(!bad.is_well_formed());
    }

    #[test]
    fn serde_round_trip_is_bit_identical() {
        let original = BusLevels {
            master: 0.9,
            ambient: 0.4,
            score: 0.25,
            sfx: 0.65,
            ui: 0.5,
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: BusLevels = serde_json::from_str(&json).unwrap();
        assert_eq!(back.master.to_bits(), original.master.to_bits());
        assert_eq!(back.ambient.to_bits(), original.ambient.to_bits());
        assert_eq!(back.score.to_bits(), original.score.to_bits());
        assert_eq!(back.sfx.to_bits(), original.sfx.to_bits());
        assert_eq!(back.ui.to_bits(), original.ui.to_bits());
    }

    /// Covers FR-CIV-AUDIO-001 — five player sliders + master mute; each tier
    /// independently mutable. We check that the public API supports
    /// per-tier get/set and that tiers cannot alias `Master`.
    #[test]
    fn fr_audio_001_per_tier_mix_is_independent() {
        let mut lv = BusLevels::default();
        // Per-tier edits do not bleed across buses.
        lv.set(BusId::Ambient, 0.10);
        lv.set(BusId::Sfx, 0.80);
        assert!((lv.get(BusId::Ambient) - 0.10).abs() < f32::EPSILON);
        assert!((lv.get(BusId::Sfx) - 0.80).abs() < f32::EPSILON);
        // Master stays untouched.
        assert!((lv.get(BusId::Master) - 1.0).abs() < f32::EPSILON);
        // Score/Ui untouched.
        assert!((lv.get(BusId::Score) - 0.30).abs() < f32::EPSILON);
        assert!((lv.get(BusId::Ui) - 0.55).abs() < f32::EPSILON);
    }
}
