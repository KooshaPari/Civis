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

fn default_civilians_per_faction() -> u32 {
    32
}
fn default_faction_count() -> u32 {
    4
}
fn default_quadrant_spread() -> i32 {
    2500
}

/// Scenario-level starting-population parameters (FR-CONTENT-STARTCOND).
///
/// Controls how many civilian agents are spawned per faction, how many factions
/// are placed, and how far from each faction's capital they are scattered.
/// Faction capitals are arranged in a procedural ring so any count is supported.
///
/// All fields are `#[serde(default)]` so old YAML files without this block
/// continue to parse and reproduce the previous hardcoded behaviour (32/4/2500).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScenarioStartingConditions {
    /// Civilians spawned around each faction capital (default: 32).
    #[serde(default = "default_civilians_per_faction")]
    pub civilians_per_faction: u32,
    /// Number of faction capitals placed on the ring (default: 4, max: 64).
    #[serde(default = "default_faction_count")]
    pub faction_count: u32,
    /// Half-width of the random jitter box around each capital in grid units
    /// (default: 2500, must be > 0).
    #[serde(default = "default_quadrant_spread")]
    pub quadrant_spread: i32,
}

impl Default for ScenarioStartingConditions {
    fn default() -> Self {
        Self {
            civilians_per_faction: default_civilians_per_faction(),
            faction_count: default_faction_count(),
            quadrant_spread: default_quadrant_spread(),
        }
    }
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
    /// Optional content-seed wiring (FR-CONTENT-MODEL / CIV-008).
    ///
    /// * `seeds` is a list of RON files (paths relative to the repo root,
    ///   resolved against the engine's manifest dir at load time) that are
    ///   parsed into [`civ_genetics::SeedSet`]s and merged into the
    ///   simulation's [`civ_genetics::SeedLibrary`].
    /// * `active_seed` is the id of the seed used for spawn-time DNA (None
    ///   means raw drift with no seed reference; the example seed set's
    ///   `raw_organism` is loaded by default regardless).
    #[serde(default)]
    pub seeds: Vec<String>,
    #[serde(default)]
    pub active_seed: Option<String>,
    /// Scenario-level divergence override (0..1). When set, overrides the
    /// active seed's own `divergence` dial at spawn time.
    ///
    /// * `0.0` → all new agents receive an exact clone of the seed genome.
    /// * `1.0` → full class mutation rate (free drift).
    /// * Absent / `None` → the seed's own `divergence` field is used.
    #[serde(default)]
    pub divergence_override: Option<f32>,
    /// Starting-population parameters: civilians per faction, faction count,
    /// and spatial spread. Defaults reproduce the pre-config hardcoded values
    /// (32 civilians × 4 factions, 2500 grid-unit spread).
    #[serde(default)]
    pub starting_conditions: ScenarioStartingConditions,
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
        let mut sim = Simulation::with_seed_and_starting_conditions(
            rng_seed,
            self.starting_conditions,
        );
        self.apply_world_state(&mut sim.state);
        sim.economy_policy = self.policy_input();
        sim.configure_military_fog(self.fog_vision_radius, self.fog_grid_size);
        sim.apply_scenario_military(&self.military);
        sim.register_mod_stubs(&self.mods);
        // Content seeds: load any referenced RON files into the library and
        // pin the active seed id (FR-CONTENT-MODEL / CIV-008). Unknown ids
        // are reported via the emergence feed and otherwise ignored.
        for seed_path in &self.seeds {
            sim.register_seed_file(seed_path);
        }
        sim.set_active_seed(self.active_seed);
        sim.set_divergence_override(self.divergence_override);
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

        if let Some(v) = self.divergence_override {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) {
                return Err(ScenarioError::Validation {
                    path,
                    field: "divergence_override",
                    message: format!(
                        "must be in [0, 1] and finite (got {v})"
                    ),
                });
            }
        }

        if self.starting_conditions.faction_count == 0
            || self.starting_conditions.faction_count > 64
        {
            return Err(ScenarioError::Validation {
                path,
                field: "starting_conditions.faction_count",
                message: "must be in 1..=64".into(),
            });
        }
        if self.starting_conditions.civilians_per_faction > 100_000 {
            return Err(ScenarioError::Validation {
                path,
                field: "starting_conditions.civilians_per_faction",
                message: "must be <= 100_000".into(),
            });
        }
        if self.starting_conditions.quadrant_spread <= 0 {
            return Err(ScenarioError::Validation {
                path,
                field: "starting_conditions.quadrant_spread",
                message: "must be > 0".into(),
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

    /// Covers FR-API-001.
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

    /// Covers FR-CIV-TACTICS-045.
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
            seeds: vec![],
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
        };
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.military_phase_config().war.fog_vision_radius, Some(6));
        assert_eq!(sim.military_phase_config().war.fog_grid_size, 32);
    }

    /// Covers FR-CIV-TACTICS-050 and FR-CIV-TACTICS-035.
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
            seeds: vec![],
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
        };
        let sim = scenario.into_simulation(1);
        let cfg = sim.military_phase_config();
        assert_eq!(cfg.movement.cadence_ticks, 8);
        assert_eq!(cfg.movement_pulses_per_cadence, 3);
        assert_eq!(cfg.war.cadence_ticks, 32);
        assert_eq!(cfg.war.engage_range_grid, 12);
    }

    /// Covers FR-MOD-004.
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
            seeds: vec![],
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
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

    /// Content seeds from scenario YAML wire into the simulation's seed
    /// library and active seed id (FR-CONTENT-MODEL / CIV-008).
    #[test]
    fn scenario_seeds_wire_into_seed_library() {
        let yaml = r#"
version: 1
name: seeds-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
seeds:
  - scenarios/canonical_seeds.ron
active_seed: human_baseline
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with seeds");
        let sim = scenario.into_simulation(11);
        // The example set is pre-loaded (raw_organism + human_baseline +
        // deep_one) and canonical_seeds.ron adds no new ids.
        assert!(sim.seed_library().get("raw_organism").is_some());
        assert!(sim.seed_library().get("human_baseline").is_some());
        assert!(sim.seed_library().get("deep_one").is_some());
        // Active seed id is honoured.
        assert_eq!(sim.active_seed_id(), Some("human_baseline"));
        // No feed events for successful load (the example feed is empty
        // until a tick runs, but seed_loaded entries may have been pushed
        // by register_seed_file; just confirm no error feed was raised).
        for ev in sim.emergence_feed() {
            assert_ne!(ev.kind, "seed_load_failed");
            assert_ne!(ev.kind, "seed_unknown");
        }
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

    /// `divergence_override` parses from YAML and validates its 0..1 range.
    #[test]
    fn scenario_divergence_override_parses_and_validates() {
        // Valid value: 0.5 should parse and round-trip.
        let yaml_valid = r#"
version: 1
name: div-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
divergence_override: 0.5
"#;
        let scenario = parse_yaml(yaml_valid).expect("divergence_override: 0.5 must parse");
        assert!(
            (scenario.divergence_override.unwrap() - 0.5).abs() < f32::EPSILON,
            "divergence_override must round-trip as Some(0.5)"
        );

        // Out-of-range value: 1.5 must fail validation.
        let yaml_invalid = r#"
version: 1
name: div-test-bad
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
divergence_override: 1.5
"#;
        let err = parse_yaml(yaml_invalid).expect_err("divergence_override: 1.5 must fail");
        assert!(
            matches!(
                err,
                ScenarioError::Validation {
                    field: "divergence_override",
                    ..
                }
            ),
            "expected validation error for divergence_override, got {err:?}"
        );

        // Absent field: must default to None (backward-compatible).
        let yaml_absent = r#"
version: 1
name: div-test-absent
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let scenario_absent = parse_yaml(yaml_absent).expect("absent divergence_override must parse");
        assert_eq!(
            scenario_absent.divergence_override, None,
            "absent divergence_override must default to None"
        );
    }

    // ---- starting_conditions tests (FR-CONTENT-STARTCOND) ----

    #[test]
    fn scenario_starting_conditions_defaults() {
        let yaml = r#"
version: 1
name: sc-default-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let s = parse_yaml(yaml).unwrap();
        assert_eq!(s.starting_conditions, ScenarioStartingConditions::default());
    }

    #[test]
    fn scenario_starting_conditions_parses() {
        let yaml = r#"
version: 1
name: sc-parse-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
starting_conditions:
  civilians_per_faction: 10
  faction_count: 6
  quadrant_spread: 1000
"#;
        let s = parse_yaml(yaml).unwrap();
        assert_eq!(s.starting_conditions.civilians_per_faction, 10);
        assert_eq!(s.starting_conditions.faction_count, 6);
        assert_eq!(s.starting_conditions.quadrant_spread, 1000);
    }

    #[test]
    fn scenario_starting_conditions_validates_faction_count_zero() {
        let yaml = r#"
version: 1
name: sc-zero-factions
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
starting_conditions:
  faction_count: 0
"#;
        let s: Scenario = {
            let de = serde_yaml::Deserializer::from_str(yaml);
            serde_path_to_error::deserialize(de).map_err(|err| ScenarioError::Parse {
                path: PathBuf::from("<test>"),
                message: err.to_string(),
            })
        }
        .unwrap();
        assert!(s.validate(Path::new("<test>")).is_err());
    }

    #[test]
    fn scenario_starting_conditions_validates_faction_count_too_high() {
        let yaml = r#"
version: 1
name: sc-too-many-factions
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
starting_conditions:
  faction_count: 9999
"#;
        let s: Scenario = {
            let de = serde_yaml::Deserializer::from_str(yaml);
            serde_path_to_error::deserialize(de).map_err(|err| ScenarioError::Parse {
                path: PathBuf::from("<test>"),
                message: err.to_string(),
            })
        }
        .unwrap();
        assert!(s.validate(Path::new("<test>")).is_err());
    }

    #[test]
    fn starting_conditions_spawns_expected_count() {
        // 2 civilians per faction × 3 factions = 6 civilians
        let yaml = r#"
version: 1
name: sc-spawn-count
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
starting_conditions:
  civilians_per_faction: 2
  faction_count: 3
  quadrant_spread: 2500
"#;
        let s = parse_yaml(yaml).unwrap();
        let sim = s.into_simulation(42);
        let count = civ_agents::count_civilians(&sim.world);
        assert_eq!(count, 6, "expected 2 civilians × 3 factions = 6");
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
