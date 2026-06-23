//! Unified military-phase cadence and per-tick work budget (FR-CIV-TACTICS-035).

use crate::movement::OperationalMovementConfig;
use crate::war_bridge::WarBridgeConfig;

/// How much tactical work runs inside each engine tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MilitaryPhaseConfig {
    /// Operational movement cadence and multi-step pulses.
    pub movement: OperationalMovementConfig,
    /// War-bridge engagement cadence and combat parameters.
    pub war: WarBridgeConfig,
    /// Extra operational movement pulses on cadence boundaries (1 = legacy single step).
    pub movement_pulses_per_cadence: u8,
}

impl Default for MilitaryPhaseConfig {
    fn default() -> Self {
        Self {
            movement: OperationalMovementConfig::default(),
            war: WarBridgeConfig::default(),
            movement_pulses_per_cadence: 2,
        }
    }
}
