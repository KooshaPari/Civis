//! Doctrine fitness from engagement outcomes (FR-CIV-TACTICS-023).

use crate::Doctrine;

/// Per-faction combat stats accumulated during the last war-bridge cadence.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FactionEngagementStats {
    /// Engagements where this faction had the shooter role.
    pub engagements_as_shooter: u32,
    /// Engagements where this faction was targeted.
    pub engagements_as_target: u32,
    /// Voxels removed by this faction's queued damage events.
    pub voxels_removed: u32,
}

impl FactionEngagementStats {
    /// Net engagement pressure (shooter minus target).
    pub fn net_pressure(&self) -> i32 {
        self.engagements_as_shooter as i32 - self.engagements_as_target as i32
    }
}

/// Re-score a doctrine from composition balance plus live engagement stats.
///
/// Deterministic for fixed inputs; used immediately before [`crate::evolve_doctrine`].
pub fn score_doctrine_fitness(doctrine: &Doctrine, stats: &FactionEngagementStats) -> f32 {
    let composition_sum: u32 = doctrine
        .unit_composition
        .iter()
        .map(|&c| u32::from(c))
        .sum();
    let composition_balance =
        composition_sum as f32 / doctrine.unit_composition.len().max(1) as f32;
    let battle = stats.net_pressure() as f32 * 3.0
        + stats.engagements_as_shooter as f32 * 1.5
        + stats.voxels_removed as f32 * 0.25;
    composition_balance + battle
}
