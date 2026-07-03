//! CivLab Deterministic Simulation Engine
//!
//! Uses fixed-point arithmetic for deterministic simulation results.
//! Uses i64 with scaling for deterministic calculations.
//!
//! ## Modules
//!
//! - `engine` - Full ECS-based simulation with tick loop
//! - `step` - Simple step function for basic simulation
//! - `policy` - Policy/consumption calculations
//! - `metrics` - Tyranny/legitimacy metrics
//! - `io` - File I/O utilities

pub mod building_emergence;
pub mod technology;
pub mod command_queue;
pub mod conditions;
pub mod gameplay;
pub mod era;
pub mod godtools;
pub mod engine;
pub mod emergence;
pub mod disasters;
pub mod dormant_phases;
pub mod emergence_metrics;
pub mod hash_chain;
pub mod integrity;
pub mod invariants;
pub mod io;
pub mod lod;
pub mod metrics;
pub mod policy;
pub mod replay;
pub mod replay_format;
pub mod save_bundle;
pub mod scenario;
pub mod perf;
pub mod spawn;
pub mod spectator;
pub mod writing;

pub mod tutorial;


/// Fixed-point scaling factor (1 raw unit = SCALE joules). Engine energy
/// quantities are stored in fixed-point `i64` for determinism and converted
/// to `f64`/SI at the economy boundary using this constant.
pub const SCALE: i64 = 1_000;

pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics, DemographicsSnapshot,
};
// FR-AUDIO-wire: re-export the audio substrate's SFX trigger enum so
// downstream crates (civ-server JSON-RPC + WS bridge) can name it as
// `civ_engine::SfxTrigger` without taking a direct `civ-audio` dep.
pub use civ_audio::triggers::SfxTrigger;
pub use civ_mod_host::{load_manifest, ModBrowserEntry, ModGuestStateSave, ModType};
pub use civ_planet::Climate;
pub use civ_tactics::{DamageEvent, DoctrineLibrary};
pub use emergence::{
    CivAiDecision, EmergenceFeedEvent, EmergenceState,
};
pub use civ_emergence_metrics::branching::BranchingRegime;
pub use emergence_metrics::{EmergenceBranchingState, EmergenceSample};
pub use engine::{

    job_type_for_civilian_id, Building, BuildingType, Citizen, CombatDamagePulse, DiplomacyKind,
    EconomicFocus, EconomicFocusEvent, FactionRelationSnapshot, Fixed, InstitutionEvent, JobType,
    MilitaryUnit, Position, ResourceType, Resources, Sim, SimSeed, Simulation, SimulationSnapshot,
    PsycheDrivenBehavior, StratBand, StratificationEvent, StratificationEventKind,
    StratificationReport, TradeRoute, UnitType, WorldState,
};
pub use hash_chain::hash_hex;
pub use replay::ReplayLog;
pub use replay::ReplayError;
pub use replay_format::{decode_civreplay, encode_civreplay};
pub use save_bundle::{
    delete_slot, list_slots, load_from_slot, save_to_slot, CivSaveBundle, SaveSlotEntry,
};
pub use spawn::{
    grid_to_norm, spawn_airport_at, spawn_hangar_at, spawn_military_at, spawn_port_at,
    unit_type_label,
};
pub use spectator::SpectatorView;


// FR-CIV-ARCH: Emergent building layouts re-export so callers can use
// `civ_engine::EmergentLayout` and `civ_engine::LayoutStrategy` without
// directly depending on the private `building_layouts` module.
pub use building_layouts::{
    EmergentLayout, LayoutStrategy,
};
pub use era::{CivAge, CivEra, EraProgressionState, FactionEraSnapshot};
pub use history::{EraHistory, EraTransition};
pub use tech::{FactionEmergenceInputs, FactionTechState};

pub use tutorial::{TutorialMilestone, TutorialProgress};


// FR-CIV-GOV-001/002/003 (civ-007 institutions epic). Re-exported so callers
// (server, clients, tests) can `use civ_engine::InstitutionKind` etc. without
// pulling the `civ-institutions` crate directly.
pub use civ_institutions::{
    Institution, InstitutionKind, GARRISON_UNLOCK_POPULATION,
    TEMPLE_UNLOCK_POPULATION,
};
pub use civ_planet::{BiomeKind, Climate, GeologyMap, MoonConfig, PlanetConfig, RegionBiome};
pub use civ_voxel::WorldCoord;
pub use civ_tactics::{
    apply_damage, bfs_next_step, evolve_doctrine, formation_offsets, grid_to_world_coord,
    line_of_sight, score_doctrine_fitness, tick_operational_movement, tick_war_bridge,
    CombatEngagement, DamageEvent, Doctrine, DoctrineLibrary, FactionEngagementStats,
    FormationKind, GridMove, MilitaryPhaseConfig, MilitaryUnitSample, NoopOperationalLayer,
    OperationalLayer, OperationalMovementConfig, WarBridgeConfig,
};

// FR-CIV-GOV-030 (civ-007 cohesion epic). Re-exported so callers
// (server, clients, tests) can name the cohesion types as `civ_engine::KinshipEdge`
// etc. without pulling the private `engine` module path.
pub use engine::{
    add_cohesion, add_trust, last_tick_cohesion, last_tick_cohesion_settlement,
    CohesionEvent, CohesionEventKind, CohesionSnapshot,
    FabricTier, KinshipEdge, KinshipKind,
};

// FR-CIV-UNREST-001 (civ-007 unrest sub-epic). Re-exported so callers
// can name the unrest types as `civ_engine::UnrestEvent` etc.
// without pulling the private `engine` module path.
pub use engine::{
    last_tick_unrest, last_tick_unrest_settlement, set_settlement_gini, unrest_level,
    UnrestEvent, UnrestLevel, UnrestSnapshot,
};
pub use metrics::{compute, compute_fixed, Metrics, MetricsFixed};
pub use policy::{
    effective_consumption, policy_from_kind, CapitalistPolicy, ControlSignals, NoopPolicy, Policy,
    PolicyInput, SubsistenceFirstPolicy, DEFAULT_ECONOMY_POLICY,
};
pub use integrity::{check_integrity, IntegrityError};
pub use invariants::{check_tick_invariants, InvariantError};
pub use lod::LodTier;
pub use lod::{
    aggregate_strategic, operational_hex_snapshot, project_zoom, should_tick_entity,
    should_tick_entity_with_policy, HexCellSnapshot, LodPolicy, ZoomLevel,
};
pub use metrics::{compute, compute_fixed, Metrics, MetricsFixed};
pub use policy::{
    effective_consumption, policy_from_kind, CapitalistPolicy, ControlSignals, NoopPolicy, Policy,
    PolicyInput, SubsistenceFirstPolicy, DEFAULT_ECONOMY_POLICY,
};
pub use replay::{ReplayError, ReplayEvent, ReplayLog};
pub use replay_format::{
    decode_civreplay, encode_civreplay, load_civreplay, save_civreplay, FOOTER_CHECKSUM_LEN,
    FORMAT_VERSION, MAGIC,
};
pub use save_bundle::{
    CivSaveBundle, CivSaveMetadata, SaveBundleError, CIVSAVE_FORMAT_VERSION, CIVSAVE_SPEC_ID,
};
pub use scenario::{
    baseline_scenario_path, load_scenario, Scenario, ScenarioError, ScenarioMilitary,
    SCENARIO_SCHEMA_VERSION,
};
pub use spectator::{BuildingPin, CivPin, Faction, JobLabel, SpectatorView};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Fixed-point type: i64 with 18 decimal places of precision
/// Stored as raw i64, divided by 10^18 for actual value
/// This ensures deterministic simulation across platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Fixed {
    /// Raw value scaled by 10^18
    pub raw: i64,
}

pub const SCALE: i64 = 1_000_000; // 10^6 (easier to work with)

impl Fixed {
    pub const ZERO: Fixed = Fixed { raw: 0 };
    pub const ONE: Fixed = Fixed { raw: SCALE };

    pub fn from_num<T: TryInto<i128>>(n: T) -> Self {
        let scaled = n.try_into().unwrap_or(0) * SCALE as i128;
        Fixed { raw: scaled as i64 }
    }

    pub fn from_raw(raw: i64) -> Self {
        Fixed { raw }
    }

    pub fn to_f64(self) -> f64 {
        self.raw as f64 / SCALE as f64
    }

    pub fn saturating_add(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.saturating_add(other.raw),
        }
    }

    pub fn saturating_sub(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.saturating_sub(other.raw),
        }
    }

    pub fn clamp(self, min: Fixed, max: Fixed) -> Fixed {
        Fixed {
            raw: self.raw.clamp(min.raw, max.raw),
        }
    }
}

impl std::ops::Add for Fixed {
    type Output = Fixed;
    fn add(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw + other.raw,
        }
    }
}

impl std::ops::Sub for Fixed {
    type Output = Fixed;
    fn sub(self, other: Fixed) -> Fixed {
        Fixed {
            raw: self.raw - other.raw,
        }
    }
}

impl std::ops::Mul for Fixed {
    type Output = Fixed;
    fn mul(self, other: Fixed) -> Fixed {
        // Multiply and divide by scale to maintain precision
        let result = (self.raw as i128) * (other.raw as i128) / SCALE as i128;
        Fixed { raw: result as i64 }
    }
}

impl std::ops::Div for Fixed {
    type Output = Fixed;
    fn div(self, other: Fixed) -> Fixed {
        let result = (self.raw as i128 * SCALE as i128) / (other.raw.max(1) as i128);
        Fixed { raw: result as i64 }
    }
}

impl std::ops::AddAssign for Fixed {
    fn add_assign(&mut self, other: Fixed) {
        self.raw += other.raw;
    }
}

impl std::ops::SubAssign for Fixed {
    fn sub_assign(&mut self, other: Fixed) {
        self.raw -= other.raw;
    }
}

impl serde::Serialize for Fixed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(self.to_f64())
    }
}

impl<'de> serde::Deserialize<'de> for Fixed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let f = f64::deserialize(deserializer)?;
        Ok(Fixed::from_num((f * SCALE as f64) as i64))
    }
}

/// Seeded RNG for deterministic simulation
pub type SimRng = ChaCha8Rng;

/// Create a seeded RNG from world state
pub fn create_rng(seed: u64) -> SimRng {
    SimRng::seed_from_u64(seed)
}

/// Advance simulation by one tick (simple API)
pub fn step(mut state: WorldState, consumption_joules: Fixed) -> WorldState {
    state.tick += 1;
    let result = state
        .energy_budget_joules
        .saturating_sub(consumption_joules);
    state.energy_budget_joules = if result.raw < 0 { Fixed::ZERO } else { result };
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn step_advances_tick() {
        let s = WorldState::default();
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.tick, 1);
    }

    #[test]
    fn step_decreases_energy() {
        let s = WorldState::default();
        // Initial energy is 1_000_000_000_000, subtract 1000 = 999_999_999_000
        let expected = Fixed::from_num(1_000_000_000_000i64) - Fixed::from_num(1000i64);
        let n = step(s, Fixed::from_num(1000));
        assert_eq!(n.energy_budget_joules, expected);
    }

    #[test]
    fn step_energy_floor_at_zero() {
        let s = WorldState {
            energy_budget_joules: Fixed::from_num(50),
            ..WorldState::default()
        };
        let n = step(s, Fixed::from_num(100));
        assert_eq!(n.energy_budget_joules, Fixed::ZERO);
    }

    #[test]
    fn determinism_same_seed_same_output() {
        let s1 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
            ..WorldState::default()
        };
        let s2 = WorldState {
            tick: 0,
            population: 100,
            energy_budget_joules: Fixed::from_num(1000),
            rng_seed: 12345,
            factions: HashMap::new(),
            faction_treasury: HashMap::new(),
            ..WorldState::default()
        };

        let r1 = step(s1, Fixed::from_num(10));
        let r2 = step(s2, Fixed::from_num(10));

        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }
}
