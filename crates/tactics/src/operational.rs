//! Operational-layer hook (Phase 4 war bridge extension point, FR-CIV-TACTICS-030).

use crate::war_bridge::CombatEngagement;

/// Phase-4 operational telemetry sink. Hosts (engine, watch) may register a hook
/// to fan out engagements without coupling `civ-tactics` to ECS.
pub trait OperationalLayer {
    /// Called after engagements resolve on a tactics cadence tick.
    fn on_combat_engagements(&mut self, tick: u64, engagements: &[CombatEngagement]);
}

/// Default no-op operational layer.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopOperationalLayer;

impl OperationalLayer for NoopOperationalLayer {
    fn on_combat_engagements(&mut self, _tick: u64, _engagements: &[CombatEngagement]) {}
}
