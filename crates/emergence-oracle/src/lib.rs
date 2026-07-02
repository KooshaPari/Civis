//! Emergence Oracle — programmatic FR verification for Civis emergence systems.
//!
//! Each [`FeatureOracle`] implementation maps to a specific FR-EMG-* requirement.
//! [`OracleRegistry::with_defaults`] wires all 24 domain oracles and [`OracleRegistry::run_all`]
//! batch-verifies them against a live [`Simulation`].

pub mod oracles;

use civ_engine::Simulation;

/// Result of a single oracle check against a running simulation.
#[derive(Debug, Clone)]
pub struct OracleVerdict {
    /// Feature requirement identifier (e.g. "FR-EMG-001").
    pub fr_id: String,
    /// Whether the measured value meets the threshold.
    pub passed: bool,
    /// Measured value extracted from simulation state.
    pub measured: f64,
    /// Acceptance threshold the measured value is compared against.
    pub threshold: f64,
    /// Human-readable detail string for diagnostics.
    pub detail: String,
}

/// A domain-specific oracle that validates one FR against simulation state.
pub trait FeatureOracle: Send + Sync {
    /// The FR identifier this oracle validates.
    fn fr_id(&self) -> &str;
    /// Run the check and return a verdict.
    fn check(&self, sim: &Simulation) -> OracleVerdict;
}

/// Registry that holds and runs a set of [`FeatureOracle`] instances.
pub struct OracleRegistry {
    oracles: Vec<Box<dyn FeatureOracle>>,
}

impl OracleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            oracles: Vec::new(),
        }
    }

    /// Register a single oracle.
    pub fn register(&mut self, oracle: Box<dyn FeatureOracle>) {
        self.oracles.push(oracle);
    }

    /// Create a registry pre-loaded with all 24 domain oracles.
    pub fn with_defaults() -> Self {
        use oracles::{
            architecture::ArchitectureOracle, coastal_settlement::CoastalSettlementOracle, creature::CreatureOracle,
            desert_caravan::DesertCaravanOracle, diplomacy::DiplomacyOracle, disaster::DisasterOracle, economy::EconomyOracle,
            epidemic::EpidemicOracle, expansion::ExpansionOracle, festival::FestivalOracle, genetics::GeneticsOracle, i18n::I18nOracle, language::LanguageOracle,
            legends::LegendsOracle, migration::MigrationOracle, migration_flow::MigrationFlowOracle, mood::MoodOracle, mountain_pass::MountainPassOracle, psyche::PsycheOracle,
            religion::ReligionOracle, trade::TradeOracle, stratification::StratificationOracle,
            religious_conflict::ReligiousConflictOracle, river_trade::RiverTradeOracle,
        };
        let mut registry = Self::new();
        registry.register(Box::new(ReligionOracle));
        registry.register(Box::new(LanguageOracle));
        registry.register(Box::new(EconomyOracle));
        registry.register(Box::new(LegendsOracle));
        registry.register(Box::new(DiplomacyOracle));
        registry.register(Box::new(PsycheOracle));
        registry.register(Box::new(ArchitectureOracle));
        registry.register(Box::new(CreatureOracle));
        registry.register(Box::new(MigrationOracle));
        registry.register(Box::new(EpidemicOracle));
        registry.register(Box::new(TradeOracle));
        registry.register(Box::new(FestivalOracle));
        registry.register(Box::new(DisasterOracle));
        registry.register(Box::new(MoodOracle));
        registry.register(Box::new(StratificationOracle));
        registry.register(Box::new(ReligiousConflictOracle));
        registry.register(Box::new(ExpansionOracle));
        registry.register(Box::new(I18nOracle));
        registry.register(Box::new(MigrationFlowOracle));
        registry.register(Box::new(CoastalSettlementOracle));
        registry.register(Box::new(RiverTradeOracle));
        registry.register(Box::new(MountainPassOracle));
        registry.register(Box::new(DesertCaravanOracle));
        registry.register(Box::new(GeneticsOracle));
        registry
    }

    /// Run every registered oracle against the given simulation and return all verdicts.
    pub fn run_all(&self, sim: &Simulation) -> Vec<OracleVerdict> {
        self.oracles.iter().map(|o| o.check(sim)).collect()
    }
}

impl Default for OracleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_registry_runs_all_twenty_four() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        assert_eq!(verdicts.len(), 24, "Expected 24 oracle verdicts");
    }

    #[test]
    fn all_verdicts_have_fr_ids() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        for v in &verdicts {
            assert!(!v.fr_id.is_empty(), "fr_id must not be empty");
        }
    }

    #[test]
    fn all_verdicts_have_non_empty_detail() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        for v in &verdicts {
            assert!(!v.detail.is_empty(), "detail must not be empty for {}", v.fr_id);
        }
    }

    /// After 300 ticks the architecture oracle must pass unconditionally: the
    /// initial world spawns 6 buildings (1 CityCenter + 5 Farms) which exceeds
    /// the threshold of 3.
    #[test]
    fn architecture_oracle_passes_after_300_ticks() {
        let mut sim = Simulation::new();
        for _ in 0..300 {
            sim.tick();
        }
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        let arch = verdicts
            .iter()
            .find(|v| v.fr_id == "FR-EMG-007")
            .expect("FR-EMG-007 must be present");
        assert!(
            arch.passed,
            "Architecture oracle must pass after 300 ticks; detail: {}",
            arch.detail
        );
    }

    /// At tick 0 every oracle must pass (no emergence has had time to run).
    #[test]
    fn all_oracles_pass_at_tick_zero() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        for v in &verdicts {
            assert!(
                v.passed,
                "Oracle {} must pass at tick 0; detail: {}",
                v.fr_id, v.detail
            );
        }
    }

    /// Verdicts must expose numeric thresholds (not NaN or negative infinity).
    #[test]
    fn verdicts_have_finite_thresholds() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        for v in &verdicts {
            assert!(
                v.threshold.is_finite(),
                "threshold for {} must be finite, got {}",
                v.fr_id,
                v.threshold
            );
            assert!(
                v.measured.is_finite(),
                "measured for {} must be finite, got {}",
                v.fr_id,
                v.measured
            );
        }
    }
}
