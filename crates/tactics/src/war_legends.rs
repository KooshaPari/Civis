//! FR-CIV-WARFARE-004 — Major battles and decisive victories promote war legends.

use legends::{
    ids::{Provenance, RegionId, SimRuntimeId, SourceCrate},
    model::{EventKind, RawSimEvent, Role},
};

/// Minimum battle magnitude (0..1) required to emit a legend event.
pub const LEGEND_BATTLE_MAGNITUDE_THRESHOLD: f32 = 0.4;
/// Magnitude used for events that qualify as decisive victories.
pub const DECISIVE_VICTORY_MAGNITUDE: f32 = 0.75;

/// Aggregated summary of a single battle outcome.
#[derive(Debug, Clone)]
pub struct BattleSummary {
    /// Tick the battle resolved on.
    pub tick: u64,
    /// Faction id of the attacking side.
    pub aggressor_faction: u32,
    /// Faction id of the defending side.
    pub defender_faction: u32,
    /// Combined casualties from both sides.
    pub total_casualties: u32,
    /// Casualties attributed to the aggressor.
    pub aggressor_casualties: u32,
    /// Optional spatial region where the battle occurred.
    pub region: Option<RegionId>,
}

impl BattleSummary {
    /// Normalized magnitude: 10 000 casualties = 1.0.
    pub fn magnitude(&self) -> f32 {
        (self.total_casualties as f32 / 10_000.0).min(1.0)
    }

    /// True when one side took ≥ 80% of total casualties.
    pub fn is_decisive_victory(&self) -> bool {
        if self.total_casualties == 0 {
            return false;
        }
        let defender_casualties = self.total_casualties.saturating_sub(self.aggressor_casualties);
        let ratio = defender_casualties as f32 / self.total_casualties as f32;
        ratio >= 0.8 || (1.0 - ratio) >= 0.8
    }
}

/// Convert a [`BattleSummary`] into a [`RawSimEvent`] ready for the legends bus.
///
/// Returns `None` when the battle magnitude is below
/// [`LEGEND_BATTLE_MAGNITUDE_THRESHOLD`] — minor skirmishes do not generate
/// legend entries.
pub fn battle_to_legend_event(summary: &BattleSummary) -> Option<RawSimEvent> {
    let mag = summary.magnitude();
    if mag < LEGEND_BATTLE_MAGNITUDE_THRESHOLD {
        return None;
    }
    let kind = if summary.is_decisive_victory() {
        EventKind::Battle
    } else {
        EventKind::Battle
    };
    let mut event = RawSimEvent::new(summary.tick, kind, SourceCrate::Tactics, mag);
    event = event.with_participant(
        SourceCrate::Tactics,
        SimRuntimeId(u64::from(summary.aggressor_faction)),
        Role::Aggressor,
    );
    event = event.with_participant(
        SourceCrate::Tactics,
        SimRuntimeId(u64::from(summary.defender_faction)),
        Role::Defender,
    );
    if let Some(region) = summary.region {
        event = event.with_region(region);
    }
    // Mark decisive victories with a higher provenance weight via raw_magnitude
    if summary.is_decisive_victory() {
        event.raw_magnitude = DECISIVE_VICTORY_MAGNITUDE.max(mag);
    }
    Some(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(total: u32, aggressor: u32) -> BattleSummary {
        BattleSummary {
            tick: 100,
            aggressor_faction: 1,
            defender_faction: 2,
            total_casualties: total,
            aggressor_casualties: aggressor,
            region: None,
        }
    }

    #[test]
    fn below_magnitude_threshold_returns_none() {
        // 100 casualties / 10_000 = 0.01 < 0.4
        let s = summary(100, 10);
        assert!(battle_to_legend_event(&s).is_none());
    }

    #[test]
    fn above_magnitude_threshold_returns_event() {
        // 5000 casualties / 10_000 = 0.5 >= 0.4
        let s = summary(5_000, 2_500);
        let ev = battle_to_legend_event(&s);
        assert!(ev.is_some());
        let ev = ev.unwrap();
        assert_eq!(ev.tick, 100);
        assert_eq!(ev.source, SourceCrate::Tactics);
        assert_eq!(ev.participants.len(), 2);
    }

    #[test]
    fn decisive_victory_sets_higher_magnitude() {
        // defender takes 90% → decisive
        let s = summary(10_000, 1_000);
        let ev = battle_to_legend_event(&s).unwrap();
        assert!(ev.raw_magnitude >= DECISIVE_VICTORY_MAGNITUDE);
    }

    #[test]
    fn zero_casualty_battle_not_decisive() {
        let s = summary(0, 0);
        assert!(!s.is_decisive_victory());
    }

    #[test]
    fn magnitude_caps_at_one() {
        let s = summary(100_000, 50_000);
        assert_eq!(s.magnitude(), 1.0);
    }

    #[test]
    fn region_propagated_to_event() {
        let mut s = summary(5_000, 2_500);
        s.region = Some(RegionId(42));
        let ev = battle_to_legend_event(&s).unwrap();
        assert_eq!(ev.region, Some(RegionId(42)));
    }
}
