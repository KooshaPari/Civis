//! Real `.civsave` snapshot persistence for `Simulation`.

use std::fs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::Citizen;
use crate::{
    Building, CombatDamagePulse, DoctrineLibrary, Institution, InstitutionKind, Position,
    ReligiousProfile, ReplayLog,
    Simulation, WorldState,
};
use civ_agents::{ClusterMember, LodTier, Needs, Position3d, Tools, Wardrobe};
use crate::language::LanguageState;
use civ_needs::Health as LifeHealth;
use civ_voxel::{DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord};

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("io at {path}: {message}")]
    Io { path: String, message: String },
    #[error("bincode encode: {0}")]
    BincodeEncode(#[from] bincode_next::error::EncodeError),
    #[error("bincode decode: {0}")]
    BincodeDecode(#[from] bincode_next::error::DecodeError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SavedSimulation {
    state: WorldState,
    world: SavedWorld,
    replay_log: ReplayLog,
    mod_guest_state_json: String,
    voxel: SavedVoxelWorld,
    last_tick_voxel_events: Vec<DirtyChunkEvent>,
    last_tick_voxel_damage_count: usize,
    /// Cached last-settlement count and life-deaths tally so
    /// `Simulation::snapshot()` round-trips through save/load
    /// (these would otherwise default to 0 on the restored sim and
    /// `assert_eq!(loaded.snapshot(), sim.snapshot())` would fail).
    last_settlement_count: u32,
    last_life_deaths: u32,
    last_tick_combat_pulses: Vec<CombatDamagePulse>,
    religious_profiles: BTreeMap<u32, ReligiousProfile>,
    #[serde(default)]
    faction_languages: BTreeMap<u32, LanguageState>,
    settlements: BTreeMap<u32, u32>,
    institutions: BTreeMap<u32, Institution>,
    institution_levels_emitted: BTreeSet<(u32, InstitutionKind, u8)>,
    faction_doctrines: Vec<DoctrineLibrary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SavedWorld {
    entities: Vec<SavedEntity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct SavedEntity {
    citizen: Option<Citizen>,
    building: Option<Building>,
    military_unit: Option<MilitaryUnit>,
    agent: Option<civ_agents::Civilian>,
    wardrobe: Option<Wardrobe>,
    tools: Option<Tools>,
    needs: Option<Needs>,
    health: Option<LifeHealth>,
    position: Option<Position3d>,
    lod: Option<LodTier>,
    cluster_member: Option<ClusterMember>,
    position_2d: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SavedVoxelWorld {
    scale: i64,
    writes: Vec<(WorldCoord, MaterialId)>,
}

fn io_err(path: &Path, err: impl std::fmt::Display) -> SaveError {
    SaveError::Io {
        path: path.display().to_string(),
        message: err.to_string(),
    }
}

fn snapshot_world(world: &hecs::World) -> SavedWorld {
    let mut entities = Vec::new();

    for (entity, agent) in world.query::<&civ_agents::Civilian>().iter() {
        let mut record = SavedEntity {
            agent: Some(agent.clone()),
            ..SavedEntity::default()
        };
        if let Ok(value) = world.get::<&Citizen>(entity) {
            record.citizen = Some(*value);
        }
        if let Ok(value) = world.get::<&Wardrobe>(entity) {
            record.wardrobe = Some(*value);
        }
        if let Ok(value) = world.get::<&Tools>(entity) {
            record.tools = Some(*value);
        }
        if let Ok(value) = world.get::<&Needs>(entity) {
            record.needs = Some(*value);
        }
        if let Ok(value) = world.get::<&LifeHealth>(entity) {
            record.health = Some(*value);
        }
        if let Ok(value) = world.get::<&Position3d>(entity) {
            record.position = Some(*value);
        }
        if let Ok(value) = world.get::<&LodTier>(entity) {
            record.lod = Some(*value);
        }
        if let Ok(value) = world.get::<&ClusterMember>(entity) {
            record.cluster_member = Some(*value);
        }
        entities.push(record);
    }

    for (entity, building) in world.query::<&Building>().iter() {
        let mut record = SavedEntity {
            building: Some(*building),
            ..SavedEntity::default()
        };
        if let Ok(value) = world.get::<&Position>(entity) {
            record.position_2d = Some(*value);
        }
        entities.push(record);
    }

    for (entity, unit) in world.query::<&MilitaryUnit>().iter() {
        let mut record = SavedEntity {
            military_unit: Some(unit.clone()),
            ..SavedEntity::default()
        };
        if let Ok(value) = world.get::<&Position>(entity) {
            record.position_2d = Some(*value);
        }
        entities.push(record);
    }

    SavedWorld { entities }
}

fn restore_world(saved: &SavedWorld) -> hecs::World {
    let mut world = hecs::World::new();
    for entity in &saved.entities {
        let mut builder = hecs::EntityBuilder::new();
        if let Some(value) = entity.agent.clone() {
            builder.add(value);
        }
        if let Some(value) = entity.citizen {
            builder.add(value);
        }
        if let Some(value) = entity.building {
            builder.add(value);
        }
        if let Some(value) = entity.military_unit.clone() {
            builder.add(value);
        }
        if let Some(value) = entity.wardrobe {
            builder.add(value);
        }
        if let Some(value) = entity.tools {
            builder.add(value);
        }
        if let Some(value) = entity.needs {
            builder.add(value);
        }
        if let Some(value) = entity.health {
            builder.add(value);
        }
        if let Some(value) = entity.position {
            builder.add(value);
        }
        if let Some(value) = entity.position_2d {
            builder.add(value);
        }
        if let Some(value) = entity.lod {
            builder.add(value);
        }
        if let Some(value) = entity.cluster_member {
            builder.add(value);
        }
        if builder.has::<civ_agents::Civilian>()
            || builder.has::<Citizen>()
            || builder.has::<Building>()
            || builder.has::<MilitaryUnit>()
        {
            world.spawn(builder.build());
        }
    }
    world
}

fn snapshot_voxel(voxel: &VoxelWorld<MaterialId>) -> SavedVoxelWorld {
    // Sparse capture over the visible fixed-point range used by the test maps.
    let mut writes = Vec::new();
    for x in -64..=64 {
        for y in -16..=64 {
            for z in -64..=64 {
                let pos = WorldCoord {
                    x: i64::from(x) * crate::SCALE,
                    y: i64::from(y) * crate::SCALE,
                    z: i64::from(z) * crate::SCALE,
                };
                let value = voxel.read(pos);
                if value != MaterialId(0) {
                    writes.push((pos, value));
                }
            }
        }
    }
    SavedVoxelWorld {
        scale: crate::SCALE,
        writes,
    }
}

fn snapshot_sim(sim: &Simulation) -> SavedSimulation {
    let (settlements, institutions, institution_levels_emitted) = sim.saveable_institution_state();
    SavedSimulation {
        state: sim.state.clone(),
        world: snapshot_world(&sim.world),
        replay_log: sim.replay_log().clone(),
        mod_guest_state_json: sim
            .export_mod_guest_state()
            .to_json()
            .unwrap_or_else(|_| "{}".to_string()),
        voxel: snapshot_voxel(sim.voxel()),
        last_tick_voxel_events: sim.last_tick_voxel_events().to_vec(),
        last_tick_voxel_damage_count: sim.last_tick_voxel_damage_count(),
        last_tick_combat_pulses: sim.last_tick_combat_pulses().to_vec(),
        religious_profiles: sim.religious_profiles.clone(),
        faction_languages: sim.faction_languages().clone(),
        settlements,
        institutions,
        institution_levels_emitted,
        faction_doctrines: sim.saveable_faction_doctrines(),
        last_settlement_count: sim.last_settlement_count,
        last_life_deaths: sim.last_life_deaths,
    }
}

fn restore_sim(saved: SavedSimulation) -> Simulation {
    let mut sim = Simulation::with_seed(saved.state.rng_seed);
    sim.state = saved.state;
    sim.world = restore_world(&saved.world);
    *sim.replay_log_mut() = saved.replay_log;
    sim.religious_profiles = saved.religious_profiles;
    sim.restore_institution_state(
        saved.settlements,
        saved.institutions,
        saved.institution_levels_emitted,
    );
    sim.restore_faction_doctrines(saved.faction_doctrines);
    sim.set_faction_languages(saved.faction_languages);
    let _ = sim.restore_mod_guest_state(
        &civ_mod_host::ModGuestStateSave::from_json(&saved.mod_guest_state_json)
            .unwrap_or_default(),
    );

    sim.last_settlement_count = saved.last_settlement_count;
    sim.last_life_deaths = saved.last_life_deaths;

    // Rebuild voxel state through the public mutator.
    for (pos, value) in saved.voxel.writes {
        sim.voxel_mut().write(pos, value);
    }

    sim
}

pub fn save_game(sim: &Simulation, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let path = path.as_ref();
    let bytes = bincode_next::serde::encode_to_vec(
        &snapshot_sim(sim),
        bincode_next::config::standard(),
    )?;
    fs::write(path, bytes).map_err(|e| io_err(path, e))
}

pub fn load_game(path: impl AsRef<Path>) -> Result<Simulation, SaveError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|e| io_err(path, e))?;
    let saved: SavedSimulation = bincode_next::serde::decode_from_slice(
        &bytes,
        bincode_next::config::standard(),
    )?
    .0;
    Ok(restore_sim(saved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn save_and_load_round_trip_full_world_state() {
        let mut sim = Simulation::with_seed(17);
        sim.set_settlement_population(101, 2_000_000);
        sim.set_settlement_population(102, 1_500_000);
        sim.religious_profiles.insert(
            101,
            ReligiousProfile {
                monitoring: 0.42,
                mythic_coherence: 0.33,
                uncertainty_reduction: 0.27,
                age_ticks: sim.state.tick,
                population: 2_000_000,
                last_drift_seed: 1_234,
            },
        );
        sim.religious_profiles.insert(
            102,
            ReligiousProfile {
                monitoring: 0.21,
                mythic_coherence: 0.17,
                uncertainty_reduction: 0.55,
                age_ticks: sim.state.tick,
                population: 1_500_000,
                last_drift_seed: 2_468,
            },
        );

        for _ in 0..4 {
            sim.tick();
        }

        let (settlements, institutions, institution_levels_emitted) =
            sim.saveable_institution_state();
        let faction_doctrines = sim.saveable_faction_doctrines();

        let file = NamedTempFile::new().unwrap();
        save_game(&sim, file.path()).unwrap();
        let loaded = load_game(file.path()).unwrap();

        // Persisted round-trip contract is the full world-state surface:
        // `state` (including emergence/faction), terrain, entities,
        // doctrine libraries, and religion/institution maps.
        assert_eq!(loaded.state, sim.state);
        assert_eq!(snapshot_world(&sim.world), snapshot_world(&loaded.world));
        for x in -64..=64 {
            for y in -16..=64 {
                for z in -64..=64 {
                    let pos = civ_voxel::WorldCoord {
                        x: i64::from(x) * crate::SCALE,
                        y: i64::from(y) * crate::SCALE,
                        z: i64::from(z) * crate::SCALE,
                    };
                    assert_eq!(loaded.voxel().read(pos), sim.voxel().read(pos));
                }
            }
        }

        assert_eq!(loaded.saveable_faction_doctrines(), faction_doctrines);
        let restored_state = loaded.saveable_institution_state();
        assert_eq!(restored_state.0, settlements);
        assert_eq!(restored_state.1, institutions);
        assert_eq!(restored_state.2, institution_levels_emitted);
        assert_eq!(loaded.faction_languages(), sim.faction_languages());
        assert_eq!(loaded.religious_profiles, sim.religious_profiles);
        assert_eq!(loaded.last_settlement_count, sim.last_settlement_count);
        assert_eq!(loaded.last_life_deaths, sim.last_life_deaths);
        assert_eq!(*loaded.replay_log(), *sim.replay_log());
    }

    /// FR-CIV-014 — a civilian's 3D Y position survives save/load round-trip.
    #[test]
    fn save_and_load_preserves_position3d_y() {
        use civ_agents::{Alignment, Civilian, Position3d};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::with_seed(19);
        let original = Position3d {
            coord: WorldCoord {
                x: 42,
                y: 17,
                z: -9,
            },
        };
        sim.world.spawn((
            Civilian {
                id: 4242,
                alignment: Alignment::Faction(3),
                age: 27,
            },
            original,
        ));

        let file = NamedTempFile::new().unwrap();
        save_game(&sim, file.path()).unwrap();
        let loaded = load_game(file.path()).unwrap();

        let restored = loaded
            .world
            .query::<(&Civilian, &Position3d)>()
            .iter()
            .find(|(_, (civ, _))| civ.id == 4242)
            .map(|(_, (_, pos))| *pos)
            .expect("saved civilian position");

        assert_eq!(restored.coord.y, original.coord.y);
        assert_eq!(restored.coord.x, original.coord.x);
        assert_eq!(restored.coord.z, original.coord.z);
    }

    /// FR-CIV-014 — a civilian's 3D Y position survives replay round-trip.
    #[test]
    fn actor_y_persists_across_replay() {
        use civ_agents::{Civilian, Position3d};

        let mut sim = Simulation::with_seed(19);
        sim.tick();

        let (civilian_id, original_y) = sim
            .world
            .query::<(&Civilian, &Position3d)>()
            .iter()
            .next()
            .map(|(_, (civ, pos))| (civ.id, pos.coord.y))
            .expect("seeded civilian");

        let file = NamedTempFile::new().unwrap();
        sim.save_replay(file.path()).unwrap();
        let loaded = Simulation::load_replay_from_file(file.path()).unwrap();

        let restored_y = loaded
            .world
            .query::<(&Civilian, &Position3d)>()
            .iter()
            .find(|(_, (civ, _))| civ.id == civilian_id)
            .map(|(_, (_, pos))| pos.coord.y)
            .expect("replayed civilian position");

        assert_eq!(restored_y, original_y);
    }

    /// P5 / CIV-1000 §13 round-trip: write a `.civsave.zst` archive via
    /// [`crate::CivSaveBundle::save_archive`], reload it through
    /// [`crate::CivSaveBundle::load`], and assert that the persisted state
    /// surface (tick, macro population, settlement/life tallies, ECS entity
    /// counts, hash-chain root) survives intact. This is the contract that
    /// the load-on-launch path depends on.
    #[test]
    fn civsave_archive_round_trip_persists_state_surface() {
        use crate::CivSaveBundle;

        let mut sim = Simulation::with_seed(23);
        for _ in 0..5 {
            sim.tick();
        }
        let hash_before = sim.hash_chain_root();
        let citizen_count = sim.snapshot().citizen_count;
        let building_count = sim.snapshot().building_count;
        let pop_before = sim.state.population;
        let tick_before = sim.state.tick;

        let dir = tempfile::tempdir().expect("tempdir");
        let archive_path = dir.path().join("round-trip.civsave.zst");
        CivSaveBundle::save_archive(&archive_path, &sim).expect("save archive");

        let loaded = CivSaveBundle::load_archive(&archive_path).expect("load archive");
        assert_eq!(loaded.state.tick, tick_before);
        assert_eq!(loaded.state.population, pop_before);
        assert_eq!(loaded.snapshot().citizen_count, citizen_count);
        assert_eq!(loaded.snapshot().building_count, building_count);
        assert_eq!(loaded.hash_chain_root(), hash_before);
    }
}
