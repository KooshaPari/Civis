//! Emergence Oracle — programmatic FR verification for Civis emergence systems.
//!
//! Each [`FeatureOracle`] implementation maps to a specific FR-EMG-* requirement.
//! [`OracleRegistry::with_defaults`] wires all 8 domain oracles and [`OracleRegistry::run_all`]
//! batch-verifies them against a live [`Simulation`].

pub mod oracles;

use engine::Simulation;

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

    /// Create a registry pre-loaded with all 8 domain oracles.
    pub fn with_defaults() -> Self {
        use oracles::{
            architecture::ArchitectureOracle, creature::CreatureOracle,
            diplomacy::DiplomacyOracle, economy::EconomyOracle, language::LanguageOracle,
            legends::LegendsOracle, psyche::PsycheOracle, religion::ReligionOracle,
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
    fn oracle_registry_runs_all_eight() {
        let sim = Simulation::new();
        let registry = OracleRegistry::with_defaults();
        let verdicts = registry.run_all(&sim);
        assert_eq!(verdicts.len(), 8, "Expected 8 oracle verdicts");
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
}
