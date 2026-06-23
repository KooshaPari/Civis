//! Versioned scenario YAML loader (FR-API-001).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{Simulation, WorldState};
use crate::policy::policy_from_kind;
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

/// One entry in a scenario's weighted seed-mix (FR-CONTENT-SEEDMIX).
///
/// The `weight` is relative — only ratios matter, not magnitudes.
/// Must be > 0 and finite.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeedWeight {
    /// Which named-race archetype to sample.
    pub seed: civ_genetics::NamedSeed,
    /// Relative spawn weight (must be > 0 and finite).
    pub weight: f32,
}

/// Scenario-level starting-population parameters (FR-CONTENT-STARTCOND).
///
/// Controls how many civilian agents are spawned per faction, how many factions
/// are placed, and how far from each faction's capital they are scattered.
/// Faction capitals are arranged in a procedural ring so any count is supported.
///
/// All fields are `#[serde(default)]` so old YAML files without this block
/// continue to parse and reproduce the previous hardcoded behaviour (32/4/2500).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    /// Optional weighted race mix.  When non-empty, each spawned civilian's
    /// named-race archetype is sampled from this distribution instead of the
    /// default round-robin Ardani/Velthari/Grundak cycle.
    ///
    /// An empty `seed_mix` (the default) reproduces the pre-existing
    /// round-robin behaviour bit-identically.
    #[serde(default)]
    pub seed_mix: Vec<SeedWeight>,
}

impl Default for ScenarioStartingConditions {
    fn default() -> Self {
        Self {
            civilians_per_faction: default_civilians_per_faction(),
            faction_count: default_faction_count(),
            quadrant_spread: default_quadrant_spread(),
            seed_mix: Vec::new(),
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
    /// Canonical seed-pack paths loaded for this scenario (content model: named
    /// races + raw-organism primitive). Empty when the scenario ships no seeds.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Active seed identity selected at load (e.g. `"raw_organism"`); `None`
    /// leaves seed selection to runtime defaults. FR-MATRIX scenario seeding.
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
    /// Per-institution tax rates (FR-ECON-004). Applied each tick in `phase_economy`
    /// before the consumption drain, debiting `Macro(ACCOUNT_ENERGY_BUDGET)` and
    /// crediting each named institution.
    #[serde(default)]
    pub taxation: ScenarioTaxation,
    /// Optional control-policy selection (FR-CORE-005). The `kind` string is
    /// resolved via [`crate::policy::policy_from_kind`]; unknown kinds fall
    /// back to the no-op policy. When the field is omitted the scenario
    /// defaults to the no-op policy as well.
    #[serde(default)]
    pub policy: ScenarioPolicy,
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
    pub rates_bp: std::collections::BTreeMap<i64, u32>,
    /// Single-tick ceiling per institution (joules); `None` means uncapped.
    #[serde(default)]
    pub per_institution_cap: Option<i64>,
}

/// Control-policy block in a scenario YAML file (FR-CORE-005).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioPolicy {
    /// Policy kind. One of `noop`, `capitalist`, `subsistence_first`.
    /// Unknown values are coerced to `noop` by [`crate::policy::policy_from_kind`].
    #[serde(default = "default_policy_kind")]
    pub kind: String,
}

impl Default for ScenarioPolicy {
    fn default() -> Self {
        Self {
            kind: default_policy_kind(),
        }
    }
}

fn default_policy_kind() -> String {
    "noop".to_string()
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
        sim.apply_scenario_taxation(&self.taxation);
        sim.register_mod_stubs(&self.mods);
        sim.set_policy(policy_from_kind(&self.policy.kind));
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

        for (i, sw) in self.starting_conditions.seed_mix.iter().enumerate() {
            if !sw.weight.is_finite() || sw.weight <= 0.0 {
                return Err(ScenarioError::Validation {
                    path,
                    field: "starting_conditions.seed_mix",
                    message: format!(
                        "weight at index {i} must be finite and > 0 (got {})",
                        sw.weight
                    ),
                });
            }
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

/// Path to a named preset scenario YAML under `scenarios/presets/`.
///
/// Presets are curated starting configurations that showcase the content knobs
/// (seed_mix, starting_conditions, divergence_override).  Use [`preset_names`]
/// to enumerate all available presets.
pub fn preset_scenario_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scenarios/presets/")
        .join(format!("{name}.yaml"))
}

/// The canonical set of curated scenario preset names.
///
/// Each name corresponds to a file at `scenarios/presets/<name>.yaml`.
pub fn preset_names() -> &'static [&'static str] {
    &[
        "single-race-ardani",
        "three-race-balanced",
        "ardani-dominant",
        "lush-frontier",
    ]
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
        assert_eq!(scenario.policy.kind, "noop");
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
            seeds: Vec::new(),
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
            taxation: ScenarioTaxation::default(),
            policy: ScenarioPolicy::default(),
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
            seeds: Vec::new(),
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
            policy: ScenarioPolicy::default(),
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
            seeds: Vec::new(),
            active_seed: None,
            divergence_override: None,
            starting_conditions: ScenarioStartingConditions::default(),
            taxation: ScenarioTaxation::default(),
            policy: ScenarioPolicy::default(),
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

    // ── FR-CONTENT-SEEDMIX: scenario seed_mix parsing & validation ───────────

    fn minimal_yaml_with_starting_conditions(extra: &str) -> String {
        format!(
            r#"version: 1
name: test
tick_start: 0
population: 1000000
base_consumption_joules: 5000000000
scarcity_multiplier: 1.0
fog_vision_radius: ~
fog_grid_size: 64
mods: []
starting_conditions:
{extra}
"#
        )
    }

    /// Absent seed_mix yields an empty Vec (serde default).
    #[test]
    fn scenario_seed_mix_absent_is_empty_default() {
        let yaml = minimal_yaml_with_starting_conditions("  civilians_per_faction: 4");
        let scenario = parse_yaml(&yaml).expect("valid scenario");
        assert!(
            scenario.starting_conditions.seed_mix.is_empty(),
            "absent seed_mix must default to empty Vec"
        );
    }

    /// A well-formed seed_mix block parses correctly.
    #[test]
    fn scenario_seed_mix_parses_and_validates() {
        let yaml = minimal_yaml_with_starting_conditions(
            r#"  civilians_per_faction: 4
  seed_mix:
    - seed: Ardani
      weight: 0.6
    - seed: Velthari
      weight: 0.3
    - seed: Grundak
      weight: 0.1"#,
        );
        let scenario = parse_yaml(&yaml).expect("valid seed_mix should parse");
        let mix = &scenario.starting_conditions.seed_mix;
        assert_eq!(mix.len(), 3);
        assert_eq!(mix[0].seed, civ_genetics::NamedSeed::Ardani);
        assert!((mix[0].weight - 0.6).abs() < 1e-6);
        assert_eq!(mix[2].seed, civ_genetics::NamedSeed::Grundak);
    }

    /// A weight of exactly 0.0 must fail validation.
    #[test]
    fn scenario_seed_mix_zero_weight_is_invalid() {
        let yaml = minimal_yaml_with_starting_conditions(
            r#"  seed_mix:
    - seed: Ardani
      weight: 0.0"#,
        );
        // parse_yaml calls validate internally
        let result = parse_yaml(&yaml);
        assert!(
            matches!(result, Err(ScenarioError::Validation { .. })),
            "weight=0 must yield a Validation error, got: {result:?}"
        );
    }

    /// A negative weight must fail validation.
    #[test]
    fn scenario_seed_mix_negative_weight_is_invalid() {
        let yaml = minimal_yaml_with_starting_conditions(
            r#"  seed_mix:
    - seed: Velthari
      weight: -1.0"#,
        );
        let result = parse_yaml(&yaml);
        assert!(
            matches!(result, Err(ScenarioError::Validation { .. })),
            "negative weight must yield a Validation error"
        );
    }

    // ── Preset YAML tests (FR-CONTENT-SEEDMIX / FR-CONTENT-STARTCOND) ────────

    /// Every preset in `preset_names()` must load, validate, and have a
    /// non-empty name and finite positive weights.
    #[test]
    fn all_presets_load_and_validate() {
        for name in preset_names() {
            let scenario = load_scenario(preset_scenario_path(name))
                .unwrap_or_else(|e| panic!("preset {name} failed to load: {e}"));
            assert!(!scenario.name.is_empty(), "preset {name} has empty name");
            for sw in &scenario.starting_conditions.seed_mix {
                assert!(
                    sw.weight > 0.0 && sw.weight.is_finite(),
                    "preset {name} has invalid weight {} for seed {:?}",
                    sw.weight,
                    sw.seed
                );
            }
        }
    }

    /// `three-race-balanced` must carry exactly 3 seeds: Ardani, Velthari, Grundak.
    #[test]
    fn preset_three_race_balanced_has_three_seeds() {
        let scenario = load_scenario(preset_scenario_path("three-race-balanced")).unwrap();
        let mix = &scenario.starting_conditions.seed_mix;
        assert_eq!(mix.len(), 3, "expected 3 seeds, got {}", mix.len());
        let seeds: Vec<civ_genetics::NamedSeed> = mix.iter().map(|sw| sw.seed).collect();
        assert!(seeds.contains(&civ_genetics::NamedSeed::Ardani), "missing Ardani");
        assert!(seeds.contains(&civ_genetics::NamedSeed::Velthari), "missing Velthari");
        assert!(seeds.contains(&civ_genetics::NamedSeed::Grundak), "missing Grundak");
    }

    /// `single-race-ardani` must have exactly 1 seed (Ardani) at weight 1.0.
    #[test]
    fn preset_single_race_is_monocultural() {
        let scenario = load_scenario(preset_scenario_path("single-race-ardani")).unwrap();
        let mix = &scenario.starting_conditions.seed_mix;
        assert_eq!(mix.len(), 1, "expected 1 seed, got {}", mix.len());
        assert_eq!(mix[0].seed, civ_genetics::NamedSeed::Ardani);
        assert!(
            (mix[0].weight - 1.0).abs() < f32::EPSILON,
            "weight should be 1.0, got {}",
            mix[0].weight
        );
    }

    // ============================================================================
    // FR-CORE-005 — Scenario policy wiring tests
    // ============================================================================

    /// FR-CORE-005 — scenario YAML with `policy: { kind: capitalist }` installs
    /// `CapitalistPolicy` on the simulation.
    #[test]
    fn scenario_policy_capitalist_installs_capitalist_policy() {
        let yaml = r#"
version: 1
name: capitalist-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
policy:
  kind: capitalist
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with policy");
        assert_eq!(scenario.policy.kind, "capitalist");
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.policy().name(), "capitalist");
    }

    /// FR-CORE-005 — scenario YAML with `policy: { kind: subsistence_first }`
    /// installs `SubsistenceFirstPolicy` on the simulation.
    #[test]
    fn scenario_policy_subsistence_first_installs_subsistence_first_policy() {
        let yaml = r#"
version: 1
name: subsistence-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
policy:
  kind: subsistence_first
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with policy");
        assert_eq!(scenario.policy.kind, "subsistence_first");
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.policy().name(), "subsistence_first");
    }

    /// FR-CORE-005 — scenario YAML without a `policy` field defaults to
    /// `NoopPolicy`.
    #[test]
    fn scenario_without_policy_field_defaults_to_noop() {
        let yaml = r#"
version: 1
name: noop-default-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario without policy");
        assert_eq!(scenario.policy.kind, "noop");
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — scenario YAML with an unknown policy kind falls back to
    /// `NoopPolicy` (defensive: we never fail a scenario load on a typo).
    #[test]
    fn scenario_unknown_policy_kind_falls_back_to_noop() {
        let yaml = r#"
version: 1
name: unknown-policy-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
policy:
  kind: libertarian_anarchism
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with unknown policy");
        assert_eq!(scenario.policy.kind, "libertarian_anarchism");
        let sim = scenario.into_simulation(1);
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — the policy installed by a scenario produces
    /// `last_control_signals` after a tick.
    #[test]
    fn scenario_policy_signals_propagate_through_tick() {
        let yaml = r#"
version: 1
name: policy-tick-test
tick_start: 0
population: 100
base_consumption_joules: 1
scarcity_multiplier: 1.0
policy:
  kind: capitalist
"#;
        let scenario = parse_yaml(yaml).expect("parse scenario with policy");
        let mut sim = scenario.into_simulation(1);
        sim.tick();
        // Default CapitalistPolicy is a no-op, so signals are empty.
        assert_eq!(sim.last_control_signals(), &crate::ControlSignals::default());

    }
}
