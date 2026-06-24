//! Top resource bar (CHROME E0, always-on).
//!
//! Per `docs/design/SAVELOAD_HUD_PLAN.md` §4.1 — 7 chips: POP, ERA, TREASURY,
//! YEAR, TIME, SPEED, AUTO. The `AUTO` chip is the **slot fingerprint**
//! pulled from `crates/save-db::last_n_for_session` (Phase 0) — the only
//! HUD signal of the autosave state.
//!
//! All chips are pure data — they take a snapshot and emit a render-ready
//! value. Host clients (Bevy/Godot/Unreal/web) map each `ChipValue` to its
//! concrete widget.

use serde::{Deserialize, Serialize};

use crate::tokens::TokenName;
use crate::HudEra;

/// Sim speed multiplier — the source of truth is `sim.snapshot.game_speed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimSpeed {
    Paused,
    Slow,
    Normal,
    Fast,
    Fastest,
    /// Single-step (the `G` keycap in §4.4 — not in the 8-key palette but
    /// listed as the per-step action).
    Step,
}

impl SimSpeed {
    /// Numeric multiplier for the `GameSpeed.multiplier` field on the
    /// snapshot. `0.0` when paused.
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            SimSpeed::Paused => 0.0,
            SimSpeed::Slow => 0.5,
            SimSpeed::Normal => 1.0,
            SimSpeed::Fast => 5.0,
            SimSpeed::Fastest => 10.0,
            SimSpeed::Step => 0.0,
        }
    }

    /// Short label rendered on the SPEED chip (`1×`, `5×`, `PAUSE`, `STEP`).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SimSpeed::Paused => "PAUSE",
            SimSpeed::Slow => "0.5×",
            SimSpeed::Normal => "1×",
            SimSpeed::Fast => "5×",
            SimSpeed::Fastest => "10×",
            SimSpeed::Step => "STEP",
        }
    }

    /// The token used to colour the SPEED chip dot per the design recipe:
    /// NEON when live, WARN when paused, TEXT_LOW when stepping.
    #[must_use]
    pub const fn dot_token(self) -> TokenName {
        match self {
            SimSpeed::Paused => TokenName::Warn,
            SimSpeed::Step => TokenName::TextLow,
            _ => TokenName::Neon,
        }
    }
}

impl Default for SimSpeed {
    fn default() -> Self {
        SimSpeed::Normal
    }
}

/// One chip on the top-bar — a label + measured value + token-driven colour.
///
/// The host client renders: `◆ POP 12.4K` where `POP` is `label_uppercase`,
/// `12.4K` is the formatted value, and `◆` is the chip's `glyph`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChipValue {
    /// Short uppercase label (`POP`, `ERA`, `TREASURY`, `YEAR`, `TIME`,
    /// `SPEED`, `AUTO`).
    pub label: &'static str,
    /// Single-character glyph rendered before the label (`◆ POP`, `⌖ ERA`).
    pub glyph: &'static str,
    /// Render-ready value (already formatted: `12.4K`, `IRON`, `14:32`).
    pub value: String,
    /// Token used for the label colour.
    pub label_token: TokenName,
    /// Token used for the value colour.
    pub value_token: TokenName,
}

/// Top-bar chip set. Always-on when the HUD is visible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopBarChips {
    pub pop: ChipValue,
    pub era: ChipValue,
    pub treasury: ChipValue,
    pub year: ChipValue,
    pub time: ChipValue,
    pub speed: ChipValue,
    pub auto: ChipValue,
}

impl Default for TopBarChips {
    fn default() -> Self {
        Self {
            pop: ChipValue {
                label: "POP",
                glyph: "◆",
                value: "0".to_owned(),
                label_token: TokenName::TextMid,
                value_token: TokenName::TextHi,
            },
            era: ChipValue {
                label: "ERA",
                glyph: "⌖",
                value: "—".to_owned(),
                label_token: TokenName::TextMid,
                value_token: TokenName::TextHi,
            },
            treasury: ChipValue {
                label: "TREAS",
                glyph: "⛁",
                value: "0".to_owned(),
                label_token: TokenName::TextMid,
                value_token: TokenName::TextHi,
            },
            year: ChipValue {
                label: "YEAR",
                glyph: "◷",
                value: "—".to_owned(),
                label_token: TokenName::TextLow,
                value_token: TokenName::TextMid,
            },
            time: ChipValue {
                label: "TIME",
                glyph: "☼",
                value: "00:00".to_owned(),
                label_token: TokenName::TextLow,
                value_token: TokenName::TextMid,
            },
            speed: ChipValue {
                label: "SPEED",
                glyph: "●",
                value: "1×".to_owned(),
                label_token: TokenName::TextLow,
                value_token: TokenName::Neon,
            },
            auto: ChipValue {
                label: "AUTO",
                glyph: "💾",
                value: "—".to_owned(),
                label_token: TokenName::TextLow,
                value_token: TokenName::TextLow,
            },
        }
    }
}

/// State of the AUTO (slot fingerprint) chip.
///
/// The chip's colour band is determined by the age of the most-recent
/// autosave (or the last failed autosave):
///
/// - **Fresh** (`≤ AUTO_FRESH_AGE_SECS`): `HOLO_GLOW` (teal, the "saved just
///   now" cue).
/// - **Medium** (`AUTO_FRESH..AUTO_STALE`): `TEXT_LOW` (quiet).
/// - **Stale** (`≥ AUTO_STALE_AGE_SECS`): `WARN` (amber, ≥ 30 min).
/// - **Failed**: `WARN` with the failure label — cleared on next successful
///   save.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AutoChipState {
    /// No autosave yet this session — chip shows `—`.
    None,
    /// Last successful autosave was ≤ fresh age.
    Fresh(u32),
    /// Last successful autosave was within the medium band.
    Medium(u32),
    /// Last successful autosave was ≥ stale age.
    Stale(u32),
    /// Last autosave attempt failed.
    Failed,
}

/// Default age bands per the plan (§4.1 row for AUTO + AC-HUD-2).
pub const AUTO_FRESH_AGE_SECS: u32 = 5 * 60;
pub const AUTO_STALE_AGE_SECS: u32 = 30 * 60;

impl AutoChipState {
    /// Compute the chip state from the age of the most recent successful
    /// autosave (or pass `None` for "never saved").
    #[must_use]
    pub fn from_age_seconds(age_secs: u32) -> Self {
        if age_secs <= AUTO_FRESH_AGE_SECS {
            AutoChipState::Fresh(age_secs)
        } else if age_secs < AUTO_STALE_AGE_SECS {
            AutoChipState::Medium(age_secs)
        } else {
            AutoChipState::Stale(age_secs)
        }
    }

    /// Token used to colour the AUTO chip's value text.
    #[must_use]
    pub const fn token(self) -> TokenName {
        match self {
            AutoChipState::None => TokenName::TextLow,
            AutoChipState::Fresh(_) => TokenName::HoloGlow,
            AutoChipState::Medium(_) => TokenName::TextLow,
            AutoChipState::Stale(_) | AutoChipState::Failed => TokenName::Warn,
        }
    }

    /// Render-ready value (`"14:32"`, `"stale"`, `"FAILED"`).
    #[must_use]
    pub fn display(self) -> &'static str {
        match self {
            AutoChipState::None => "—",
            AutoChipState::Fresh(_) => "fresh",
            AutoChipState::Medium(_) => "ok",
            AutoChipState::Stale(_) => "stale",
            AutoChipState::Failed => "FAILED",
        }
    }
}

impl TopBarChips {
    /// Refresh the ERA chip from a measured `HudEra`.
    pub fn set_era(&mut self, era: HudEra) {
        self.era.value = match era {
            HudEra::Unknown => "—".to_owned(),
            HudEra::Stone => "STONE".to_owned(),
            HudEra::Bronze => "BRONZE".to_owned(),
            HudEra::Iron => "IRON".to_owned(),
            HudEra::Steel => "STEEL".to_owned(),
            HudEra::Information => "INFO".to_owned(),
            HudEra::Atomic => "ATOMIC".to_owned(),
        };
        self.era.value_token = era.accent_token();
    }

    /// Refresh the SPEED chip from the measured `SimSpeed`.
    pub fn set_speed(&mut self, speed: SimSpeed) {
        self.speed.value = speed.label().to_owned();
        self.speed.value_token = speed.dot_token();
    }

    /// Refresh the AUTO chip from the measured autosave state.
    pub fn set_auto(&mut self, state: AutoChipState) {
        self.auto.value = state.display().to_owned();
        self.auto.value_token = state.token();
    }

    /// Format a pop count as `12.4K` / `1.2M`.
    #[must_use]
    pub fn format_pop(count: u64) -> String {
        format_count(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_speed_multiplier_and_label() {
        assert_eq!(SimSpeed::Paused.multiplier(), 0.0);
        assert_eq!(SimSpeed::Normal.multiplier(), 1.0);
        assert_eq!(SimSpeed::Fast.multiplier(), 5.0);
        assert_eq!(SimSpeed::Fastest.multiplier(), 10.0);
        assert_eq!(SimSpeed::Normal.label(), "1×");
        assert_eq!(SimSpeed::Paused.label(), "PAUSE");
    }

    #[test]
    fn sim_speed_dot_token() {
        assert_eq!(SimSpeed::Paused.dot_token(), TokenName::Warn);
        assert_eq!(SimSpeed::Normal.dot_token(), TokenName::Neon);
        assert_eq!(SimSpeed::Step.dot_token(), TokenName::TextLow);
    }

    #[test]
    fn auto_chip_state_bands() {
        assert!(matches!(
            AutoChipState::from_age_seconds(60),
            AutoChipState::Fresh(_)
        ));
        assert!(matches!(
            AutoChipState::from_age_seconds(900),
            AutoChipState::Medium(_)
        ));
        assert!(matches!(
            AutoChipState::from_age_seconds(3600),
            AutoChipState::Stale(_)
        ));
    }

    #[test]
    fn auto_chip_token_bands() {
        assert_eq!(AutoChipState::Fresh(60).token(), TokenName::HoloGlow);
        assert_eq!(AutoChipState::Medium(900).token(), TokenName::TextLow);
        assert_eq!(AutoChipState::Stale(3600).token(), TokenName::Warn);
        assert_eq!(AutoChipState::Failed.token(), TokenName::Warn);
    }

    #[test]
    fn format_pop_short_form() {
        assert_eq!(TopBarChips::format_pop(0), "0");
        assert_eq!(TopBarChips::format_pop(999), "999");
        assert_eq!(TopBarChips::format_pop(12_400), "12.4K");
        assert_eq!(TopBarChips::format_pop(1_200_000), "1.2M");
    }

    #[test]
    fn set_era_drives_value_and_token() {
        let mut chips = TopBarChips::default();
        chips.set_era(HudEra::Iron);
        assert_eq!(chips.era.value, "IRON");
        assert_eq!(chips.era.value_token, TokenName::Neon);

        chips.set_era(HudEra::Bronze);
        assert_eq!(chips.era.value, "BRONZE");
        assert_eq!(chips.era.value_token, TokenName::Amber);
    }
}

/// Internal: format a population count compactly.
fn format_count(count: u64) -> String {
    if count < 1_000 {
        count.to_string()
    } else if count < 1_000_000 {
        let k = count as f64 / 1_000.0;
        if k >= 100.0 {
            format!("{:.0}K", k)
        } else {
            format!("{:.1}K", k)
        }
    } else {
        let m = count as f64 / 1_000_000.0;
        format!("{:.1}M", m)
    }
}