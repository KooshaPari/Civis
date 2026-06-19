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
    /// Mod directory paths (repo-relative from `crates/engine`), e.g. `mods/example-policy`.
    /// MVP: manifests are loaded; WASM guests are not executed yet (CIV-0700).
    #[serde(default)]
    pub mods: Vec<String>,
    /// When set, enables faction fog-of-war in the war bridge (FR-CIV-TACTICS-045).
    #[serde(default)]
    pub fog_vision_radius: Option<u32>,
    /// Square fog grid edge when fog is enabled (default 64).
    #[serde(default = "default_fog_grid_size")]
    pub fog_grid_size: u32,
    /// Optional military cadence and combat tuning (FR-CIV-TACTICS-050).
    #[serde(default)]
    pub military: ScenarioMilitary,
<<<<<<< Updated upstream
=======
    /// Per-institution tax rates (FR-ECON-004). Applied each tick in `phase_economy`
    /// before the consumption drain, debiting `Macro(ACCOUNT_ENERGY_BUDGET)` and
    /// crediting each named institution.
    #[serde(default)]
    pub taxation: ScenarioTaxation,
}

/// Per-institution tax rates from scenario YAML (FR-ECON-004 partial).
/// `rates_bp` is institution_id → basis-points rate (0..=10_000). Use
/// [`INSTITUTION_TREASURY`] and [`INSTITUTION_MARKET`] as the canonical
/// institutions. `per_institution_cap` (if set) clamps any single tick's
/// collection per institution.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScenarioTaxation {
    /// institution_id (e.g. `INSTITUTION_TREASURY`) → basis-points rate.
    #[serde(default)]
    pub rates_bp: std::collections::BTreeMap<u32, u32>,
    /// Single-tick ceiling per institution (joules); `None` means uncapped.
    #[serde(default)]
    pub per_institution_cap: Option<i64>,
>>>>>>> Stashed changes
}

fn default_fog_grid_size() -> u32 {
    64
}

/// Optional military-phase overrides from scenario YAML (FR-CIV-TACTICS-050).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScenarioMilitary {
    /// Override [`OperationalMovementConfig::cadence_ticks`].
    #[serde(default)]
    pub movement_cadence_ticks: Option<u64>,
    /// Override [`MilitaryPhaseConfig::movement_pulses_per_cadence`].
    #[serde(default)]
    pub movement_pulses_per_cadence: Option<u8>,
    /// Override [`WarBridgeConfig::cadence_ticks`].
    #[serde(default)]
    pub war_cadence_ticks: Option<u64>,
    /// Override [`WarBridgeConfig::engage_range_grid`].
    #[serde(default)]
    pub engage_range_grid: Option<i32>,
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
        sim.configure_military_fog(self.fog_vision_radius, self.fog_grid_size);
        sim.apply_scenario_military(&self.military);
        sim.register_mod_stubs(&self.mods);
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

        if let Some(v) = self.military.movement_cadence_ticks {
            if v == 0 {
                return Err(ScenarioError::Validation {
                    path,
                    field: "military.movement_cadence_ticks",
                    message: "must be greater than 0".into(),
                });
            }
        }
        if let Some(v) = self.military.war_cadence_ticks {
            if v == 0 {
                return Err(ScenarioError::Validation {
                    path,
                    field: "military.war_cadence_ticks",
                    message: "must be greater than 0".into(),
                });
            }
        }
        if let Some(v) = self.military.engage_range_grid {
            if v < 1 {
                return Err(ScenarioError::Validation {
                    path,
                    field: "military.engage_range_grid",
                    message: "must be at least 1".into(),
                });
            }
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
        assert_eq!(scenario.fog_vision_radius, Some(8));
        assert_eq!(scenario.fog_grid_size, 64);
        assert_eq!(scenario.military.war_cadence_ticks, Some(16));
        assert_eq!(scenario.military.engage_range_grid, Some(10));
        assert_eq!(
            scenario.mods,
            vec!["mods/example-policy", "mods/example-economic"]
        );
    }

    #[test]
    fn scenario_fog_wires_military_phase() {
        let scenario = Scenario {
            version: SCENARIO_SCHEMA_VERSION,
            name: "fog".into(),
            tick_start: 0,
            population: 100,
            base_consumption_joules: 1,
            scarcity_multiplier: 1.0,
            mods: vec![],
            fog_vision_radius: Some(6),
            fog_grid_size: 32,
            military: ScenarioMilitary::default(),
        };
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.military_phase_config().war.fog_vision_radius, Some(6));
        assert_eq!(sim.military_phase_config().war.fog_grid_size, 32);
    }

    #[test]
    fn scenario_military_wires_military_phase() {
        let scenario = Scenario {
            version: SCENARIO_SCHEMA_VERSION,
            name: "mil".into(),
            tick_start: 0,
            population: 100,
            base_consumption_joules: 1,
            scarcity_multiplier: 1.0,
            mods: vec![],
            fog_vision_radius: None,
            fog_grid_size: default_fog_grid_size(),
            military: ScenarioMilitary {
                movement_cadence_ticks: Some(8),
                movement_pulses_per_cadence: Some(3),
                war_cadence_ticks: Some(32),
                engage_range_grid: Some(12),
            },
            taxation: ScenarioTaxation::default(),
        };
        let sim = scenario.into_simulation(1);
        let cfg = sim.military_phase_config();
        assert_eq!(cfg.movement.cadence_ticks, 8);
        assert_eq!(cfg.movement_pulses_per_cadence, 3);
        assert_eq!(cfg.war.cadence_ticks, 32);
        assert_eq!(cfg.war.engage_range_grid, 12);
    }

    #[test]
    fn scenario_mods_loads_example_policy() {
        let yaml = r#"
version: 1
name: mod-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
mods:
  - mods/example-policy
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with mods");
        let sim = scenario.into_simulation(99);
        assert_eq!(sim.mod_host().mods().len(), 1);
        assert_eq!(sim.mod_host().mods()[0].manifest.meta.id, "example-policy");
        assert!(
            sim.replay_log()
                .events
                .iter()
                .any(|e| matches!(e, crate::ReplayEvent::ModLoaded { mod_id, .. } if mod_id == "example-policy"))
        );
        let bus = sim.replay_log().mod_loaded_bus_events();
        assert_eq!(bus.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&bus[0]).expect("mod.loaded bus json");
        assert_eq!(v["event"], "mod.loaded.v1");
        assert_eq!(v["mod_id"], "example-policy");
        assert!(v.get("mod_name").is_some());
        assert!(v.get("version").is_some());
        assert!(v.get("tick").is_some());
    }

    #[test]
    fn mod_guest_state_exports_after_baseline_load() {
        let scenario = load_scenario(baseline_scenario_path()).expect("baseline");
        let mut sim = scenario.clone().into_simulation(1);
        assert!(sim.mod_browser_entries().len() >= 2);
        sim.mod_host_mut()
            .restore_guest_memory("example-policy", vec![1, 2]);
        let save = sim.export_mod_guest_state();
        let json = save.to_json().expect("json");
        let mut sim2 = scenario.into_simulation(2);
        sim2.restore_mod_guest_state(&crate::ModGuestStateSave::from_json(&json).expect("parse"))
            .expect("restore");
        assert_eq!(
            sim2.mod_host().guest_memory_snapshot("example-policy"),
            vec![1, 2]
        );
    }

    #[test]
    fn baseline_scenario_headless_smoke() {
        let scenario = load_scenario(baseline_scenario_path()).expect("baseline.yaml should load");

        assert_eq!(scenario.tick_start, 0);
        assert_eq!(scenario.population, 1_000_000);

        let mut sim = scenario.into_simulation(42);
        assert_eq!(sim.state.tick, 0);
        assert_eq!(sim.state.population, 1_000_000);
        assert_eq!(sim.mod_host().mods().len(), 2);
        let ids: Vec<_> = sim
            .mod_host()
            .mods()
            .iter()
            .map(|m| m.manifest.meta.id.as_str())
            .collect();
        assert!(ids.contains(&"example-policy"));
        assert!(ids.contains(&"example-economic"));

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
            mods: vec![],
            fog_vision_radius: None,
            fog_grid_size: default_fog_grid_size(),
            military: ScenarioMilitary::default(),
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

    /// FR-ECON-004: scenario YAML `taxation` block must round-trip through the
/// loader and reach `into_simulation`. Verifies the BTreeMap<i64, u32>
/// (institution_id → basis points) shape and the optional per-institution cap.
    #[test]
    fn scenario_taxation_round_trips_from_yaml() {
        use civ_economy::{INSTITUTION_MARKET, INSTITUTION_TREASURY};
        let yaml = r#"
version: 1
name: tax-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
taxation:
  rates_bp:
    0: 500   # treasury at 5%
    1: 250   # market at 2.5%
  per_institution_cap: 10000
"#;
        let scenario = parse_yaml(yaml).expect("parse");
        assert_eq!(scenario.taxation.rates_bp.get(&INSTITUTION_TREASURY), Some(&500));
        assert_eq!(scenario.taxation.rates_bp.get(&INSTITUTION_MARKET), Some(&250));
        assert_eq!(scenario.taxation.per_institution_cap, Some(10_000));

        let sim = scenario.into_simulation(1);
        assert_eq!(
            sim.taxation.rates_bp.get(&INSTITUTION_TREASURY),
            Some(&500)
        );
        assert_eq!(sim.taxation.per_institution_cap, Some(10_000));
    }

    /// FR-ECON-004: when `taxation` is omitted from YAML, the scenario loads
    /// with an empty `rates_bp` (no tax collection) — no breakage for
    /// scenarios that pre-date the feature.
    #[test]
    fn scenario_taxation_defaults_to_empty_when_absent() {
        let yaml = r#"
version: 1
name: no-tax
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let scenario = parse_yaml(yaml).expect("parse");
        assert!(scenario.taxation.rates_bp.is_empty());
        assert_eq!(scenario.taxation.per_institution_cap, None);
        let sim = scenario.into_simulation(1);
        assert!(sim.taxation.rates_bp.is_empty());
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
