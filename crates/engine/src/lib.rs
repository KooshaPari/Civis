//! CivLab Deterministic Simulation Engine
//!
//! Uses fixed-point arithmetic for deterministic simulation results.
//! Uses i64 with scaling for deterministic calculations.
#![allow(
    dead_code,
    non_snake_case,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_parens,
    unused_variables,
    clippy::clone_on_copy,
    clippy::derivable_impls,
    clippy::double_must_use,
    clippy::for_kv_map,
    clippy::if_same_then_else,
    clippy::iter_cloned_collect,
    clippy::let_and_return,
    clippy::manual_clamp,
    clippy::needless_range_loop,
    clippy::ptr_arg,
    clippy::empty_line_after_doc_comments,
    clippy::inconsistent_digit_grouping,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_sort_by,
    clippy::unnecessary_cast,
    clippy::unwrap_or_default,
    clippy::useless_conversion,
    clippy::useless_format,
    clippy::wildcard_in_or_patterns
)]
// Staged simulation phases are intentionally compiled before integration wiring.
//!
//! ## Modules
//!
//! - `engine` - Full ECS-based simulation with tick loop
//! - `step` - Simple step function for basic simulation
//! - `policy` - Policy/consumption calculations

pub mod building_emergence;
pub mod climate;
pub mod command_queue;
pub mod conditions;
pub mod culture;
pub mod disasters;
pub mod diplomacy;
pub mod emergence;
pub mod emergence_metrics;
pub mod engine;
pub mod era;
pub mod economy_engine;
pub mod emergence_coupling;
pub mod faction_decisions;
pub mod gameplay;
pub mod godtools;
pub mod hash_chain;
pub mod integrity;
pub mod invariants;
pub mod io;
pub mod lod;
pub mod metrics;
pub mod perf;
pub mod policy;
pub mod replay;
pub mod replay_format;
pub mod save_bundle;
pub mod scenario;
pub mod spawn;
pub mod spectator;
pub mod tech;

// --- post-merge stubs ---------------------------------------------------------
// The following in-crate modules were removed by earlier cleanup lanes but
// downstream files (`lib.rs`, `engine.rs`) still hold `use crate::{X};`
// references. They are stubbed here so the rest of the crate compiles.
// The .rs files exist as TODO placeholders; a follow-up lane should either
// restore the originals or rewrite the callers to drop the imports.
pub mod building_layouts;
pub mod history;
pub mod language;
pub mod psyche_behavior;
pub mod religion;
pub mod writing;

pub mod tutorial;
pub mod fixed_math;

/// Fixed-point scaling factor (1 raw unit = SCALE joules). Engine energy
/// quantities are stored in fixed-point `i64` for determinism and converted
/// to `f64`/SI at the economy boundary using this constant.
pub const SCALE: i64 = 1_000;

// TODO(cleanup-surgeon): stub. `religion` is currently an empty `pub mod`
// placeholder; the real Belief/Religion implementations were removed by
// earlier lanes. Restore from git history, or rewrite callers.
// pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
// pub use demographics::{
//     carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics, DemographicsSnapshot,
// };
// FR-AUDIO-wire: re-export the audio substrate's SFX trigger enum so
// downstream crates (civ-server JSON-RPC + WS bridge) can name it as
// `civ_engine::SfxTrigger` without taking a direct `civ-audio` dep.
pub use building_emergence::{
    apply_emergence_facades, architecture_tile_sets, biome_style_tag,
    building_type_unlocked_at_era, culture_traits_for_cluster, emergence_demand_signals,
    emergent_style_key_for_sim, resource_stock_units, settlement_build_anchor,
};
pub use civ_audio::triggers::SfxTrigger;
pub use civ_build::{BiomeStyleTag, EmergentStyleKey};
pub use civ_emergence_metrics::branching::BranchingRegime;
pub use civ_mod_host::{load_manifest, ModBrowserEntry, ModGuestStateSave, ModType};
pub use emergence::{CivAiDecision, EmergenceFeedEvent, EmergenceState};
pub use emergence_metrics::{EmergenceBranchingState, EmergenceSample};
pub use engine::{
    cohesion_delta, diplomacy_conflict_threshold, diplomacy_peace_threshold,
    institution_belief_signal, institution_divergence_boost, job_type_for_civilian_id, Building,
    BuildingType, Citizen, ClusterStocks, CombatDamagePulse, EconomicFocus, EconomicFocusEvent,
    EmotionDrivenBehavior, InstitutionEvent, JobType,
    MilitaryUnit, MoodSnapshot, Position, PsycheDrivenBehavior, ResourceType, Resources, Sim,
    SimSeed, Simulation, SimulationSnapshot, StratBand, StratificationEvent,
    StratificationEventKind, StratificationReport, TradeRoute, UnitType, WorldState,
};
// Re-export diplomacy types from the extracted diplomacy module.
pub use diplomacy::{DiplomacyEvent, DiplomacyKind};
// Re-export climate types from the extracted climate module.
pub use climate::{CoastalColumn, WATER_MARKER_MATERIAL};
pub use hash_chain::hash_hex;
pub use replay::ReplayError;
pub use replay::ReplayLog;
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
// TODO(cleanup-surgeon): stub. The original `building_layouts` module is
// gone; downstream files that re-import these types need to be rewritten.
// pub use building_layouts::{
//     EmergentLayout, LayoutStrategy,
// };
pub use civ_institutions::InstitutionKind;
pub use civ_voxel::WorldCoord;
pub use era::{CivAge, CivEra, EraProgressionState, FactionEraSnapshot};
pub use psyche_behavior::behavior_from_psyche;
pub use religion::{
    apply_big_gods_response, last_religion_sample, ReligiousProfile, SubstrateGradients,
};
// TODO(cleanup-surgeon): `history`/`tech` modules are stubs — the real
// implementations need restoring. These re-exports were the cargo source of
// the E0432 cascade for era.rs / engine.rs.
// pub use history::{EraHistory, EraTransition};
// pub use tech::{FactionEmergenceInputs, FactionTechState};

pub use tutorial::{TutorialMilestone, TutorialProgress};

// FR-CIV-GOV-001/002/003 (civ-007 institutions epic). Re-exported so callers
// (server, clients, tests) can `use civ_engine::InstitutionKind` etc. without
// pulling the `civ-institutions` crate directly.
// TODO(cleanup-surgeon): `civ-institutions` is missing from this crate's
//  Cargo.toml; the engine self-references it. Restore the dependency or
//  inline the types when the institutions module is re-introduced.
// pub use civ_institutions::{
//     Institution, InstitutionKind, GARRISON_UNLOCK_POPULATION,
//     TEMPLE_UNLOCK_POPULATION,
// };
pub use civ_planet::{BiomeKind, Climate, GeologyMap, MoonConfig, PlanetConfig, RegionBiome};
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
    add_cohesion, add_trust, last_tick_cohesion, last_tick_cohesion_settlement, CohesionEvent,
    CohesionEventKind, CohesionSnapshot, FabricTier, KinshipEdge, KinshipKind,
};

// FR-CIV-UNREST-001 (civ-007 unrest sub-epic). Re-exported so callers
// can name the unrest types as `civ_engine::UnrestEvent` etc.
// without pulling the private `engine` module path.
pub use engine::{
    last_tick_unrest, last_tick_unrest_settlement, set_settlement_gini, unrest_level, UnrestEvent,
    UnrestLevel, UnrestSnapshot,
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
// `metrics` + `policy` already exported above; previously redeclared here as a
// post-merge dup — removed. `replay`/`replay_format`/`save_bundle`/`scenario`/
// `spectator` re-exports live in the upper block (lines 70-80); the older
// broader exports here were colliding with them.
pub use replay_format::{
    load_civreplay, save_civreplay, FOOTER_CHECKSUM_LEN, FORMAT_VERSION, MAGIC,
};
pub use save_bundle::{CivSaveMetadata, SaveBundleError, CIVSAVE_FORMAT_VERSION, CIVSAVE_SPEC_ID};
pub use scenario::{
    baseline_scenario_path, load_scenario, Scenario, ScenarioError, ScenarioMilitary,
    SCENARIO_SCHEMA_VERSION,
};

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Fixed-point type re-exported from the `engine` submodule.
///
/// Historically this crate had its own `Fixed` struct at the crate root;
/// that duplicate caused E0308 errors (`expected engine::Fixed, found Fixed`)
/// at every disaster/spawn/lib call site that imported `crate::Fixed` while
/// the engine expected `crate::engine::Fixed`. Unifying on the submodule
/// definition (tuple `Fixed(i64)` with `FixedFromNum` trait) also fixes the
/// `i128: From<{float}>` E0277 errors, since floats now route through the
/// trait instead of `TryInto<i128>`.
pub use engine::Fixed;

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
    state.energy_budget_joules = if result.to_bits() < 0 {
        Fixed::ZERO
    } else {
        result
    };
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
