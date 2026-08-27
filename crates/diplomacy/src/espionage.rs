//! Espionage system — covert spy-network deployment, execution, detection,
//! and counter-intelligence for the Civis diplomacy layer.
//!
//! This module provides a deterministic espionage engine that manages spy
//! networks between factions. Each [`SpyNetwork`] tracks its source/target,
//! operational strength, cover (stealth), and accumulated intelligence level.
//! The [`EspionageEngine`] owns all networks and drives per-tick simulation
//! (strength growth, cover decay, detection sweeps).
//!
//! # Detection formula
//!
//! ```text
//! detection_chance = base_detection_chance * (1.0 - cover) * action_risk_factor
//! ```
//!
//! Where `action_risk_factor` varies per [`EspionageAction`] variant —
//! low-risk actions like `GatherIntel` are harder to detect than high-risk
//! actions like `AssassinateLeader`.
//!
//! # Determinism
//!
//! All state transitions are pure functions of `(state, action, rng_seed)`.
//! The engine accepts an `rng: fn() -> f32` closure that must return a
//! value in `[0.0, 1.0)` — this keeps the interface RNG-agnostic and
//! testable.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// SpyNetwork
// ---------------------------------------------------------------------------

/// A single spy network operating between two factions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpyNetwork {
    /// Faction that deployed the network.
    pub source_faction: u32,
    /// Faction being spied on.
    pub target_faction: u32,
    /// Operational strength (0.0–1.0). Higher = more effective.
    pub strength: f32,
    /// Cover / stealth level (0.0–1.0). Higher = harder to detect.
    pub cover: f32,
    /// Accumulated intelligence (0 = none, 5 = maximum).
    pub intel_level: u8,
}

impl SpyNetwork {
    /// Create a new spy network with the given source, target, and initial
    /// strength. Cover starts at maximum (1.0) and intel at 0.
    pub fn new(source_faction: u32, target_faction: u32, strength: f32) -> Self {
        Self {
            source_faction,
            target_faction,
            strength: strength.clamp(0.0, 1.0),
            cover: 1.0,
            intel_level: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// EspionageAction
// ---------------------------------------------------------------------------

/// Actions a spy network can execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EspionageAction {
    /// Passively gather intelligence about the target faction.
    GatherIntel,
    /// Sabotage target infrastructure or resources.
    Sabotage,
    /// Steal a technology from the target faction.
    StealTechnology,
    /// Attempt to assassinate the target faction's leader.
    AssassinateLeader,
    /// Spread propaganda to destabilize or influence the target.
    SpreadPropaganda,
    /// Conduct counter-intelligence to identify enemy spies.
    CounterEspionage,
}

impl EspionageAction {
    /// Risk factor for detection — higher means easier to detect.
    pub fn risk_factor(self) -> f32 {
        match self {
            Self::GatherIntel => 0.3,
            Self::SpreadPropaganda => 0.4,
            Self::CounterEspionage => 0.5,
            Self::Sabotage => 0.7,
            Self::StealTechnology => 0.8,
            Self::AssassinateLeader => 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// SpyResult
// ---------------------------------------------------------------------------

/// Outcome of executing an espionage action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpyResult {
    /// The action completed fully without detection.
    Success,
    /// Partial success; the u32 carries a magnitude of effect (e.g. intel
    /// points stolen, damage dealt).
    Partial(u32),
    /// The spy was detected — network cover is shattered.
    Detected,
    /// The action failed (e.g. insufficient strength).
    Failed,
    /// The agent was lost (killed or captured) during the operation.
    AgentLost,
}

// ---------------------------------------------------------------------------
// EspionageConfig
// ---------------------------------------------------------------------------

/// Tunable parameters for the espionage engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EspionageConfig {
    /// Maximum number of spy networks allowed simultaneously.
    pub max_networks: u32,
    /// Base detection chance (0.0–1.0) before cover and risk modifiers.
    pub base_detection_chance: f32,
    /// Cover decay per second (subtract from cover each tick).
    pub cover_decay: f32,
    /// Strength growth per second when a network is active.
    pub strength_growth: f32,
}

impl Default for EspionageConfig {
    fn default() -> Self {
        Self {
            max_networks: 10,
            base_detection_chance: 0.15,
            cover_decay: 0.01,
            strength_growth: 0.02,
        }
    }
}

impl EspionageConfig {
    /// Sanity-check the configuration.
    pub fn validate(&self) -> Result<(), EspionageConfigError> {
        if self.max_networks == 0 {
            return Err(EspionageConfigError::ZeroMaxNetworks);
        }
        if !(0.0..=1.0).contains(&self.base_detection_chance) {
            return Err(EspionageConfigError::InvalidBaseDetectionChance(
                self.base_detection_chance,
            ));
        }
        if self.cover_decay < 0.0 {
            return Err(EspionageConfigError::NegativeCoverDecay(self.cover_decay));
        }
        if self.strength_growth < 0.0 {
            return Err(EspionageConfigError::NegativeStrengthGrowth(
                self.strength_growth,
            ));
        }
        Ok(())
    }
}

/// Configuration validation error.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EspionageConfigError {
    /// max_networks must be at least 1.
    #[error("max_networks must be >= 1")]
    ZeroMaxNetworks,
    /// base_detection_chance must be in [0.0, 1.0].
    #[error("base_detection_chance must be in [0.0, 1.0], got {0}")]
    InvalidBaseDetectionChance(f32),
    /// cover_decay must be non-negative.
    #[error("cover_decay must be >= 0.0, got {0}")]
    NegativeCoverDecay(f32),
    /// strength_growth must be non-negative.
    #[error("strength_growth must be >= 0.0, got {0}")]
    NegativeStrengthGrowth(f32),
}

// ---------------------------------------------------------------------------
// EspionageEngine
// ---------------------------------------------------------------------------

/// The espionage engine. Owns all spy networks and drives simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspionageEngine {
    /// Active spy networks.
    pub networks: Vec<SpyNetwork>,
    /// Engine configuration.
    pub config: EspionageConfig,
}

impl EspionageEngine {
    /// Create a new espionage engine with the given configuration.
    /// Validates the config and returns an error on invalid parameters.
    pub fn new(config: EspionageConfig) -> Result<Self, EspionageConfigError> {
        config.validate()?;
        Ok(Self {
            networks: Vec::new(),
            config,
        })
    }

    /// Deploy a new spy network from `source` to `target` with the given
    /// initial `strength`. Returns the network index on success, or an
    /// error if the maximum network count is reached.
    pub fn deploy(
        &mut self,
        source: u32,
        target: u32,
        strength: f32,
    ) -> Result<usize, EspionageError> {
        if self.networks.len() as u32 >= self.config.max_networks {
            return Err(EspionageError::MaxNetworksReached);
        }
        if source == target {
            return Err(EspionageError::SameFaction);
        }
        let idx = self.networks.len();
        self.networks
            .push(SpyNetwork::new(source, target, strength));
        Ok(idx)
    }

    /// Execute an espionage action on the network at `network_id`.
    ///
    /// The `rng` closure must return a value in `[0.0, 1.0)`. The engine
    /// uses it to resolve detection and success probability.
    ///
    /// Detection formula:
    /// ```text
    /// detection_chance = base * (1.0 - cover) * action_risk_factor
    /// ```
    pub fn execute(
        &mut self,
        action: EspionageAction,
        network_id: usize,
        rng: impl FnOnce() -> f32,
    ) -> Result<SpyResult, EspionageError> {
        if network_id >= self.networks.len() {
            return Err(EspionageError::InvalidNetwork(network_id));
        }

        let network = &mut self.networks[network_id];

        // Calculate detection chance.
        let risk = action.risk_factor();
        let detection_chance = self.config.base_detection_chance * (1.0 - network.cover) * risk;
        let detection_roll = rng();

        // Check detection.
        if detection_roll < detection_chance {
            // Agent is detected — zero cover and return Detected.
            network.cover = 0.0;
            return Ok(SpyResult::Detected);
        }

        // Resolve action outcome based on strength.
        match action {
            EspionageAction::GatherIntel => {
                // Success chance scales with strength.
                if network.strength >= 0.3 {
                    if network.intel_level < 5 {
                        network.intel_level += 1;
                    }
                    Ok(SpyResult::Success)
                } else {
                    Ok(SpyResult::Partial(network.intel_level as u32))
                }
            }
            EspionageAction::Sabotage => {
                if network.strength >= 0.5 {
                    let damage = (network.strength * 100.0) as u32;
                    Ok(SpyResult::Partial(damage))
                } else {
                    Ok(SpyResult::Failed)
                }
            }
            EspionageAction::StealTechnology => {
                if network.strength >= 0.6 && network.intel_level >= 2 {
                    Ok(SpyResult::Success)
                } else {
                    Ok(SpyResult::Failed)
                }
            }
            EspionageAction::AssassinateLeader => {
                // Highest-risk action; requires max intel and high strength.
                if network.strength >= 0.8 && network.intel_level >= 4 {
                    Ok(SpyResult::Success)
                } else if network.strength >= 0.6 {
                    Ok(SpyResult::Partial(0))
                } else {
                    // Low strength on assassination = agent likely lost.
                    Ok(SpyResult::AgentLost)
                }
            }
            EspionageAction::SpreadPropaganda => {
                if network.strength >= 0.2 {
                    let influence = (network.strength * 50.0) as u32;
                    Ok(SpyResult::Partial(influence))
                } else {
                    Ok(SpyResult::Failed)
                }
            }
            EspionageAction::CounterEspionage => {
                // Counter-intel costs cover (the agent exposes itself slightly).
                network.cover = (network.cover - 0.05).max(0.0);
                if network.strength >= 0.4 {
                    Ok(SpyResult::Success)
                } else {
                    Ok(SpyResult::Failed)
                }
            }
        }
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Each tick:
    /// - Strength grows by `config.strength_growth * dt` (capped at 1.0).
    /// - Cover decays by `config.cover_decay * dt` (floored at 0.0).
    /// - Networks with cover == 0.0 are flagged for removal (they remain in
    ///   the list until explicitly dismantled or cleaned up).
    pub fn tick(&mut self, dt: f32) {
        for network in &mut self.networks {
            network.strength = (network.strength + self.config.strength_growth * dt).min(1.0);
            network.cover = (network.cover - self.config.cover_decay * dt).max(0.0);
        }
    }

    /// Return indices of all networks whose cover has decayed to 0.0
    /// (i.e. they are detected / compromised).
    pub fn detected_networks(&self) -> Vec<usize> {
        self.networks
            .iter()
            .enumerate()
            .filter(|(_, n)| n.cover <= 0.0)
            .map(|(i, _)| i)
            .collect()
    }

    /// Dismantle (remove) a spy network by index. Returns `true` if the
    /// network was found and removed, `false` if the index was invalid.
    pub fn dismantle(&mut self, network_id: usize) -> bool {
        if network_id < self.networks.len() {
            self.networks.remove(network_id);
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// EspionageError
// ---------------------------------------------------------------------------

/// Errors produced by espionage operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EspionageError {
    /// Maximum number of spy networks has been reached.
    #[error("maximum number of spy networks reached")]
    MaxNetworksReached,
    /// Cannot spy on yourself.
    #[error("cannot deploy spy network to own faction")]
    SameFaction,
    /// Invalid network index.
    #[error("invalid network index: {0}")]
    InvalidNetwork(usize),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create an engine with default config.
    fn engine() -> EspionageEngine {
        EspionageEngine::new(EspionageConfig::default()).expect("valid config")
    }

    /// Helper: deterministic rng that always returns the given value.
    fn fixed_rng(value: f32) -> impl FnOnce() -> f32 {
        move || value
    }

    // ---- 1. Deployment: successful deploy returns correct index ----------

    #[test]
    fn deploy_returns_sequential_indices() {
        let mut eng = engine();
        let i0 = eng.deploy(1, 2, 0.5).expect("deploy 0");
        let i1 = eng.deploy(3, 4, 0.7).expect("deploy 1");
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(eng.networks.len(), 2);
    }

    // ---- 2. Deployment: same-faction rejected ----------------------------

    #[test]
    fn deploy_same_faction_rejected() {
        let mut eng = engine();
        let result = eng.deploy(1, 1, 0.5);
        assert!(matches!(result, Err(EspionageError::SameFaction)));
        assert_eq!(eng.networks.len(), 0);
    }

    // ---- 3. Deployment: max networks limit enforced ----------------------

    #[test]
    fn deploy_max_networks_enforced() {
        let config = EspionageConfig {
            max_networks: 2,
            ..Default::default()
        };
        let mut eng = EspionageEngine::new(config).expect("valid config");
        eng.deploy(1, 2, 0.5).expect("deploy 0");
        eng.deploy(3, 4, 0.5).expect("deploy 1");
        assert!(matches!(
            eng.deploy(5, 6, 0.5),
            Err(EspionageError::MaxNetworksReached)
        ));
    }

    // ---- 4. Execution: GatherIntel succeeds and bumps intel level --------

    #[test]
    fn gather_intel_increases_intel_level() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.5).expect("deploy");

        // No detection (rng returns 1.0 > detection_chance).
        let result = eng
            .execute(EspionageAction::GatherIntel, 0, fixed_rng(1.0))
            .expect("execute");
        assert_eq!(result, SpyResult::Success);
        assert_eq!(eng.networks[0].intel_level, 1);
    }

    // ---- 5. Execution: detection when rng < detection_chance ------------

    #[test]
    fn execute_detected_when_rng_below_threshold() {
        let mut eng = engine();
        // Deploy with low cover so detection_chance > 0.
        let mut net = SpyNetwork::new(1, 2, 0.5);
        net.cover = 0.5; // 50% cover => (1 - 0.5) = 0.5
        eng.networks.push(net);

        // detection_chance = 0.15 * 0.5 * 0.3 (GatherIntel) = 0.0225
        // rng returns 0.01 < 0.0225 => detected
        let result = eng
            .execute(EspionageAction::GatherIntel, 0, fixed_rng(0.01))
            .expect("execute");
        assert_eq!(result, SpyResult::Detected);
        assert_eq!(eng.networks[0].cover, 0.0, "cover zeroed on detection");
    }

    // ---- 6. Execution: Sabotage returns Partial with damage value --------

    #[test]
    fn sabotage_returns_partial_damage() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.6).expect("deploy");

        let result = eng
            .execute(EspionageAction::Sabotage, 0, fixed_rng(1.0))
            .expect("execute");
        // strength 0.6 >= 0.5 => Partial(60)
        assert_eq!(result, SpyResult::Partial(60));
    }

    // ---- 7. Execution: AssassinateLeader with high stats = Success -------

    #[test]
    fn assassinate_leader_high_stats_succeeds() {
        let mut eng = engine();
        let mut net = SpyNetwork::new(1, 2, 0.9);
        net.intel_level = 5;
        eng.networks.push(net);

        let result = eng
            .execute(EspionageAction::AssassinateLeader, 0, fixed_rng(1.0))
            .expect("execute");
        assert_eq!(result, SpyResult::Success);
    }

    // ---- 8. Execution: AssassinateLeader with low strength = AgentLost ---

    #[test]
    fn assassinate_leader_low_strength_loses_agent() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.3).expect("deploy");

        let result = eng
            .execute(EspionageAction::AssassinateLeader, 0, fixed_rng(1.0))
            .expect("execute");
        assert_eq!(result, SpyResult::AgentLost);
    }

    // ---- 9. Execution: CounterEspionage reduces cover --------------------

    #[test]
    fn counter_espionage_reduces_cover() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.5).expect("deploy");
        let cover_before = eng.networks[0].cover;
        assert_eq!(cover_before, 1.0);

        let _ = eng
            .execute(EspionageAction::CounterEspionage, 0, fixed_rng(1.0))
            .expect("execute");

        // cover should be reduced by 0.05.
        assert!(
            (eng.networks[0].cover - 0.95).abs() < f32::EPSILON,
            "cover expected ~0.95, got {}",
            eng.networks[0].cover
        );
    }

    // ---- 10. Tick: strength grows and cover decays -----------------------

    #[test]
    fn tick_grows_strength_and_decays_cover() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.5).expect("deploy");

        eng.tick(1.0); // 1 second

        let net = &eng.networks[0];
        // strength: 0.5 + 0.02 = 0.52
        assert!(
            (net.strength - 0.52).abs() < 1e-6,
            "strength expected 0.52, got {}",
            net.strength
        );
        // cover: 1.0 - 0.01 = 0.99
        assert!(
            (net.cover - 0.99).abs() < 1e-6,
            "cover expected 0.99, got {}",
            net.cover
        );
    }

    // ---- 11. Detected networks + dismantle -------------------------------

    #[test]
    fn detected_networks_and_dismantle() {
        let mut eng = engine();
        eng.deploy(1, 2, 0.5).expect("deploy 0");
        eng.deploy(3, 4, 0.5).expect("deploy 1");
        eng.deploy(5, 6, 0.5).expect("deploy 2");

        // Force cover to zero on the middle network.
        eng.networks[1].cover = 0.0;

        let detected = eng.detected_networks();
        assert_eq!(detected, vec![1]);

        // Dismantle the detected network.
        assert!(eng.dismantle(1));
        assert_eq!(eng.networks.len(), 2);
        // Verify remaining networks are correct.
        assert_eq!(eng.networks[0].source_faction, 1);
        assert_eq!(eng.networks[1].source_faction, 5);

        // Dismantle with invalid index returns false.
        assert!(!eng.dismantle(99));
    }

    // ---- 12. Execute on invalid network index returns error --------------

    #[test]
    fn execute_invalid_network_index_errors() {
        let mut eng = engine();
        let result = eng.execute(EspionageAction::GatherIntel, 0, fixed_rng(1.0));
        assert!(matches!(result, Err(EspionageError::InvalidNetwork(0))));
    }

    // ---- Config validation ------------------------------------------------

    #[test]
    fn config_validation_catches_bad_values() {
        let bad_max = EspionageConfig {
            max_networks: 0,
            ..Default::default()
        };
        assert!(matches!(
            bad_max.validate(),
            Err(EspionageConfigError::ZeroMaxNetworks)
        ));

        let bad_chance = EspionageConfig {
            base_detection_chance: 1.5,
            ..Default::default()
        };
        assert!(matches!(
            bad_chance.validate(),
            Err(EspionageConfigError::InvalidBaseDetectionChance(1.5))
        ));

        let bad_decay = EspionageConfig {
            cover_decay: -0.1,
            ..Default::default()
        };
        assert!(matches!(
            bad_decay.validate(),
            Err(EspionageConfigError::NegativeCoverDecay(-0.1))
        ));

        let bad_growth = EspionageConfig {
            strength_growth: -0.05,
            ..Default::default()
        };
        assert!(matches!(
            bad_growth.validate(),
            Err(EspionageConfigError::NegativeStrengthGrowth(-0.05))
        ));
    }

    // ---- SpyNetwork::new clamps strength ----------------------------------

    #[test]
    fn spy_network_new_clamps_strength() {
        let net = SpyNetwork::new(1, 2, 2.0);
        assert_eq!(net.strength, 1.0, "strength clamped to 1.0");

        let net = SpyNetwork::new(1, 2, -0.5);
        assert_eq!(net.strength, 0.0, "strength clamped to 0.0");
    }

    // ---- EspionageAction risk factors -------------------------------------

    #[test]
    fn action_risk_factors_are_ordered() {
        assert!(
            EspionageAction::GatherIntel.risk_factor() < EspionageAction::Sabotage.risk_factor()
        );
        assert!(
            EspionageAction::Sabotage.risk_factor()
                < EspionageAction::AssassinateLeader.risk_factor()
        );
        assert_eq!(EspionageAction::AssassinateLeader.risk_factor(), 1.0);
    }
}
