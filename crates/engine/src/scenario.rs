//! Versioned scenario YAML loader (FR-API-001).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{Simulation, WorldState};
use crate::policy::PolicyInput;

/// Supported scenario schema version.
pub const SCENARIO_SCHEMA_VERSION: u32 = 1;

fn default_version() -> u32 {
    SCENARIO_SCHEMA_VERSION
}

/// Parsed scenario configuration from YAML.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    /// Schema version; defaults to [`SCENARIO_SCHEMA_VERSION`] when omitted.
    #[serde(default = "default_version")]
    pub version: u32,
    pub name: String,
    pub tick_start: u64,
    pub population: u64,
    pub base_consumption_joules: u64,
    pub scarcity_multiplier: f64,
}

/// Errors while loading or validating a scenario file.
#[derive(Debug)]
pub enum ScenarioError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
    Validation {
        path: PathBuf,
        field: &'static str,
        message: String,
    },
    UnsupportedVersion {
        path: PathBuf,
        version: u32,
        supported: u32,
    },
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioError::Io { path, source } => {
                write!(f, "failed to read scenario {:?}: {}", path, source)
            }
            ScenarioError::Parse { path, message } => {
                write!(f, "failed to parse scenario {:?}: {}", path, message)
            }
            ScenarioError::Validation {
                path,
                field,
                message,
            } => {
                write!(
                    f,
                    "invalid scenario {:?} at field `{}`: {}",
                    path, field, message
                )
            }
            ScenarioError::UnsupportedVersion {
                path,
                version,
                supported,
            } => write!(
                f,
                "unsupported scenario version {} in {:?} (supported: {})",
                version, path, supported
            ),
        }
    }
}

impl std::error::Error for ScenarioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScenarioError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl Scenario {
    /// Policy parameters from this scenario.
    pub fn policy_input(&self) -> PolicyInput {
        PolicyInput {
            base_consumption_joules: self.base_consumption_joules as f64,
            scarcity_multiplier: self.scarcity_multiplier,
        }
    }

    /// Apply starting world-state fields from this scenario.
    pub fn apply_world_state(&self, state: &mut WorldState) {
        state.tick = self.tick_start;
        state.population = self.population;
    }

    /// Headless simulation seeded from scenario starting conditions.
    pub fn into_simulation(self, rng_seed: u64) -> Simulation {
        let mut sim = Simulation::with_seed(rng_seed);
        self.apply_world_state(&mut sim.state);
        sim.economy_policy = self.policy_input();
        sim
    }

    /// Validate field constraints after deserialization.
    pub fn validate(&self, path: &Path) -> Result<(), ScenarioError> {
        let path = path.to_path_buf();

        if self.name.trim().is_empty() {
            return Err(ScenarioError::Validation {
                path,
                field: "name",
                message: "must not be empty".into(),
            });
        }

        if self.population == 0 {
            return Err(ScenarioError::Validation {
                path,
                field: "population",
                message: "must be greater than 0".into(),
            });
        }

        if self.scarcity_multiplier < 0.0 {
            return Err(ScenarioError::Validation {
                path,
                field: "scarcity_multiplier",
                message: "must be non-negative".into(),
            });
        }

        Ok(())
    }
}

/// Load and validate a scenario YAML file from `path`.
pub fn load_scenario(path: impl AsRef<Path>) -> Result<Scenario, ScenarioError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| ScenarioError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let scenario: Scenario = {
        let de = serde_yaml::Deserializer::from_str(&contents);
        serde_path_to_error::deserialize(de).map_err(|err| ScenarioError::Parse {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?
    };

    if scenario.version != SCENARIO_SCHEMA_VERSION {
        return Err(ScenarioError::UnsupportedVersion {
            path: path.to_path_buf(),
            version: scenario.version,
            supported: SCENARIO_SCHEMA_VERSION,
        });
    }

    scenario.validate(path)?;
    Ok(scenario)
}

/// Path to the repo `scenarios/baseline.yaml` from this crate's manifest dir.
pub fn baseline_scenario_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/baseline.yaml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_yaml_parses() {
        let scenario = load_scenario(baseline_scenario_path()).expect("baseline.yaml should load");

        assert_eq!(scenario.version, SCENARIO_SCHEMA_VERSION);
        assert_eq!(scenario.name, "baseline");
        assert_eq!(scenario.tick_start, 0);
        assert_eq!(scenario.population, 1_000_000);
        assert_eq!(scenario.base_consumption_joules, 5_000_000_000);
        assert!((scenario.scarcity_multiplier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn baseline_scenario_headless_smoke() {
        let scenario = load_scenario(baseline_scenario_path()).expect("baseline.yaml should load");

        assert_eq!(scenario.tick_start, 0);
        assert_eq!(scenario.population, 1_000_000);

        let mut sim = scenario.into_simulation(42);
        assert_eq!(sim.state.tick, 0);
        assert_eq!(sim.state.population, 1_000_000);

        for _ in 0..10 {
            sim.tick();
        }

        assert_eq!(sim.state.tick, 10);
    }

    /// Scenario YAML economy fields wire into `phase_economy` via `economy_policy`.
    #[test]
    fn scenario_economy_policy_affects_consumption() {
        use crate::Fixed;

        let base = Scenario {
            version: SCENARIO_SCHEMA_VERSION,
            name: "economy-test".into(),
            tick_start: 0,
            population: 1,
            base_consumption_joules: 1_000,
            scarcity_multiplier: 0.0,
        };

        let mut zero_scarcity = base.clone();
        zero_scarcity.scarcity_multiplier = 0.0;
        let mut high_scarcity = base;
        high_scarcity.scarcity_multiplier = 2.0;

        let mut sim_zero = zero_scarcity.into_simulation(7);
        let mut sim_high = high_scarcity.into_simulation(7);

        let budget_before = sim_zero.state.energy_budget_joules;
        sim_zero.tick();
        sim_high.tick();

        assert_eq!(sim_zero.state.energy_budget_joules, budget_before);
        assert_eq!(
            sim_high.state.energy_budget_joules,
            budget_before - Fixed::from_num(2_000i64)
        );
    }

    #[test]
    fn rejects_empty_name() {
        let yaml = r#"
version: 1
name: "   "
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let err = parse_yaml(yaml).expect_err("empty name");
        assert!(matches!(
            err,
            ScenarioError::Validation { field: "name", .. }
        ));
    }

    #[test]
    fn rejects_negative_scarcity() {
        let yaml = r#"
version: 1
name: test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: -0.5
"#;
        let err = parse_yaml(yaml).expect_err("negative scarcity");
        assert!(matches!(
            err,
            ScenarioError::Validation {
                field: "scarcity_multiplier",
                ..
            }
        ));
    }

    #[test]
    fn parse_error_includes_field_path() {
        let yaml = r#"
version: 1
name: test
tick_start: not_a_number
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let err = parse_yaml(yaml).expect_err("bad tick_start");
        let ScenarioError::Parse { message, .. } = err else {
            panic!("expected parse error, got {err:?}");
        };
        assert!(
            message.contains("tick_start"),
            "expected field path in message, got: {message}"
        );
    }

    fn parse_yaml(yaml: &str) -> Result<Scenario, ScenarioError> {
        let scenario: Scenario = {
            let de = serde_yaml::Deserializer::from_str(yaml);
            serde_path_to_error::deserialize(de).map_err(|err| ScenarioError::Parse {
                path: PathBuf::from("<test>"),
                message: err.to_string(),
            })?
        };

        if scenario.version != SCENARIO_SCHEMA_VERSION {
            return Err(ScenarioError::UnsupportedVersion {
                path: PathBuf::from("<test>"),
                version: scenario.version,
                supported: SCENARIO_SCHEMA_VERSION,
            });
        }

        scenario.validate(Path::new("<test>"))?;
        Ok(scenario)
    }
}
