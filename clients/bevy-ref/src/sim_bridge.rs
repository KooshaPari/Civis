//! In-process simulation bridge for the standalone Bevy client.

use std::collections::HashMap;

use bevy::prelude::*;
use civ_agents::{spawn_civilian_at, ActorVisual, ActorVisualKind, Civilian};
use civ_engine::{spawn::spawn_airport_at, Building, BuildingType, Simulation};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
#[cfg(not(feature = "voxel"))]
use crate::terrain::{terrain_surface_y, WATER_LEVEL, WORLD_SIZE};
#[cfg(feature = "voxel")]
use crate::voxel_sim::{voxel_surface_y, VoxelSimState};
use crate::{live_attach::is_server_attach_mode, AttachMode};

/// Half-height of the civilian capsule (used to seat its base on terrain).
const CIVILIAN_HALF_HEIGHT: f32 = 1.6 + 1.4; // capsule half-length + radius
/// Half-height of the building cuboid (seats its base on terrain).
const BUILDING_HALF_HEIGHT: f32 = 7.0;

/// Maximum number of civilian entities the standalone Bevy client will render
/// concurrently. Matches the MVP cap noted in the P2 actors slice plan: the
/// sim may carry more, but the renderer drops the tail until the cap is
/// raised. Instanced meshes are the documented scale path beyond this cap.
const MAX_VISIBLE_CIVILIANS: usize = 512;

/// Approach speed for visual → sim-position interpolation (per second).
/// Higher = stiffer (snaps faster); lower = more visible drift. Picked so
/// the lerp covers ~5 sim ticks (0.5 s @ 10 Hz) inside one e-fold — a
/// civilian walking at the Hot LOD's `movement_speed_factor` ~0.002 moves
/// ~0.01 normalised units per tick, so 5 ticks ≈ 0.05 normalised units is
/// the typical step the interpolator has to absorb.
const CIVILIAN_LERP_SPEED: f32 = 8.0;

/// Latest sim-derived world position for a civilian visual entity. Written
/// by [`reconcile_civilian_lifecycle`], read by [`interpolate_civilian_transforms`].
/// Decoupling the write-rate (sim-tick cadence) from the read-rate
/// (frame cadence) is what lets the visual slide smoothly toward each new
/// sim position rather than snapping.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct CivilianVisualTarget {
    /// World-space target translation the entity is interpolating toward.
    pub translation: Vec3,
}

/// Uniform scale for the CC0 GLTF civilian (KayKit Knight) so it reads near the
/// gameplay capsule's height.
// Voxel world is [0, dims] world-space with 1 voxel = 1 world unit and terrain
// ~256 units tall. A ~1.8m glb at scale 1.7 reads as ~3 units = sub-pixel /
// invisible when the camera frames the whole world. Scale actors up so they
// read clearly against the voxel terrain (sub-pixel mesh-scale bug).
#[cfg(feature = "models")]
#[cfg(feature = "voxel")]
const CIVILIAN_MODEL_SCALE: f32 = 8.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const CIVILIAN_MODEL_SCALE: f32 = 3.0;
/// Scale for herd / fauna rigs (skeleton minion).
#[cfg(feature = "models")]
#[cfg(feature = "voxel")]
const HERD_MODEL_SCALE: f32 = 10.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const HERD_MODEL_SCALE: f32 = 2.4;
/// Uniform scale for the CC0 GLTF building (KayKit hexagon home).
#[cfg(feature = "models")]
#[cfg(feature = "voxel")]
const BUILDING_MODEL_SCALE: f32 = 4.0;
#[cfg(all(feature = "models", not(feature = "voxel")))]
const BUILDING_MODEL_SCALE: f32 = 6.0;

/// Faction tag for a model-backed civilian so a later material-tint pass can
/// colour the GLTF scene per faction. Primitives bake the colour in directly.
#[cfg(feature = "models")]
#[derive(Component)]
struct FactionTint(#[allow(dead_code)] u32);

#[cfg(feature = "models")]
#[derive(Component, Clone, Copy)]
struct PendingActorScene {
    kind: ActorVisualKind,
    faction: u32,
}
#[cfg(not(feature = "models"))]
#[derive(Component, Clone, Copy)]
struct PendingActorScene {
    kind: ActorVisualKind,
    faction: u32,
}

/// Resolve the loaded CC0 actor scene for `kind`, else `None` (capsule fallback).
#[cfg(feature = "models")]
fn actor_model_root(
    models: Option<&crate::gltf_models::GameModels>,
    kind: ActorVisualKind,
    faction: u32,
) -> Option<SceneRoot> {
    use crate::gltf_models::{actor_scene, ModelOrPrimitive};
    match models.map(|m| actor_scene(m, kind, faction)) {
        Some(ModelOrPrimitive::Model(root)) => Some(root),
        _ => None,
    }
}

#[cfg(feature = "models")]
fn actor_scene_if_loaded(
    models: Option<&crate::gltf_models::GameModels>,
    asset_server: &AssetServer,
    kind: ActorVisualKind,
    faction: u32,
) -> Option<SceneRoot> {
    use crate::gltf_models::{actor_scene, ModelOrPrimitive};
    let Some(model) = models else {
        return None;
    };
    match actor_scene(model, kind, faction) {
        ModelOrPrimitive::Model(SceneRoot(handle)) => asset_server
            .is_loaded_with_dependencies(&handle)
            .then_some(SceneRoot(handle)),
        ModelOrPrimitive::Primitive => None,
    }
}

#[cfg(feature = "models")]
fn model_scale_for(kind: ActorVisualKind) -> f32 {
    match kind {
        ActorVisualKind::Humanoid => CIVILIAN_MODEL_SCALE,
        ActorVisualKind::Herd => HERD_MODEL_SCALE,
    }
}

/// Resolve the loaded CC0 building scene to a spawnable `SceneRoot`, else `None`
/// to fall back to the procedural cuboid.
#[cfg(feature = "models")]
fn building_model_root(
    models: Option<&crate::gltf_models::GameModels>,
    building_type: BuildingType,
) -> Option<SceneRoot> {
    use crate::gltf_models::{building_scene_for, ModelOrPrimitive};
    match models.map(|m| building_scene_for(m, building_type)) {
        Some(ModelOrPrimitive::Model(root)) => Some(root),
        _ => None,
    }
}

/// Live simulation state shared by the minimap, HUD, and spawn tools.
#[derive(Resource)]
pub struct SimState(pub Simulation);

/// Spawn-once registries mapping a stable sim id to its rendered entity.
#[derive(Resource, Default)]
struct RenderedEntities {
    civilians: HashMap<u64, Entity>,
    buildings: HashMap<u64, Entity>,
}

/// Shared mesh/material handles so we spawn the assets once, not per entity.
#[derive(Resource)]
struct GameplayAssets {
    civilian_mesh: Handle<Mesh>,
    building_mesh: Handle<Mesh>,
}

/// In-process simulation marker for civilian entities rendered from `SimState`.
///
/// This tag is only used when not in server-attach mode, and should never be
/// attached by the live-stream `Frame3d` path.
#[derive(Component)]
pub struct SimCivilianMarker;

/// In-process simulation marker for building entities rendered from `SimState`.
///
/// This tag is only used when not in server-attach mode, and should never be
/// attached by the live-stream `Frame3d` path.
#[derive(Component)]
pub struct SimBuildingMarker;

// Public aliases for the scene-dump harness (machine-level verification).
pub use SimBuildingMarker as SimBuildingMarkerPublic;
pub use SimCivilianMarker as SimCivilianMarkerPublic;

impl Default for SimState {
    fn default() -> Self {
        Self(Simulation::default())
    }
}

/// In-process simulation tick interval for the standalone client.
const SIM_TICK_SECONDS: f32 = 0.1;

#[derive(Resource)]
struct SimTickTimer(Timer);

fn in_process_sim_active(mode: Res<AttachMode>) -> bool {
    !is_server_attach_mode(*mode)
}

/// Returns true only when the game is in the Playing state.
/// Used as a run condition to gate all sim systems.
#[cfg(feature = "egui")]
fn is_playing(mode: Res<crate::menus::GameUiMode>) -> bool {
    *mode == crate::menus::GameUiMode::Playing
}

/// Wires spawn-tool messages into the ECS simulation and optional HUD sync.
#[derive(Default)]
pub struct SimBridgePlugin;

impl Plugin for SimBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimState>()
            .init_resource::<RenderedEntities>()
            .init_resource::<DiplomacyStandings>()
            .add_systems(Startup, init_gameplay_assets)
            .insert_resource(SimTickTimer(Timer::from_seconds(
                SIM_TICK_SECONDS,
                TimerMode::Repeating,
            )));

        // All sim systems are gated: in_process_sim_active AND (egui-gated) is_playing.
        // Without egui the game has no menu system so we allow free running
        // (same behaviour as before this patch).
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            (
                maybe_reinit_sim_on_new_world,
                advance_simulation
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
                apply_spawn_civilian_requests
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
                apply_spawn_building_requests
                    .run_if(in_process_sim_active)
                    .run_if(is_playing),
            ),
        );

        #[cfg(not(feature = "egui"))]
        app.add_systems(
            Update,
            (
                advance_simulation.run_if(in_process_sim_active),
                apply_spawn_civilian_requests.run_if(in_process_sim_active),
                apply_spawn_building_requests.run_if(in_process_sim_active),
            ),
        );

        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            sync_game_ui_snapshot
                .run_if(in_process_sim_active)
                .run_if(is_playing),
        );

        // Diplomacy standings seam (civ-007 / P4). Polls the engine snapshot
        // and folds legacy `civ_engine::DiplomacyEvent`s into a local
        // `civ_diplomacy::DiplomacyState` to produce a sorted per-pair list
        // for the Diplomacy panel. Event-driven wiring is the upgrade path
        // (see module-level seam note on `DiplomacyStandings`).
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            sync_diplomacy_standings
                .run_if(in_process_sim_active)
                .run_if(is_playing),
        );

        // sync_visible_gameplay must also be gated: only render entities in Playing/Paused.
        // In Paused we keep entities visible (frozen) but don't tick; in menus we despawn all.
        // P2 visible-citizen-lifecycle: the reconcile pass runs every frame so births
        // and deaths surface the same frame the sim commits them (a tick-rate detection
        // shortcut would miss births that fire mid-`advance_simulation`). The interpolation
        // pass also runs every frame (sim ticks at 10 Hz; visuals must move at 60+ Hz to
        // feel like a walk, not a teleport).
        #[cfg(feature = "egui")]
        app.add_systems(
            Update,
            (
                reconcile_civilian_lifecycle
                    .run_if(in_process_sim_active)
                    .run_if(is_playing_or_paused),
                interpolate_civilian_transforms
                    .run_if(in_process_sim_active)
                    .run_if(is_playing_or_paused),
                #[cfg(feature = "models")]
                upgrade_pending_actor_scenes
                    .run_if(in_process_sim_active)
                    .run_if(is_playing_or_paused),
            ),
        );

        #[cfg(not(feature = "egui"))]
        app.add_systems(
            Update,
            (
                reconcile_civilian_lifecycle.run_if(in_process_sim_active),
                interpolate_civilian_transforms.run_if(in_process_sim_active),
                #[cfg(feature = "models")]
                upgrade_pending_actor_scenes.run_if(in_process_sim_active),
            ),
        );
    }
}

/// Run condition: allow entity sync while Playing or Paused (freeze-in-place).
#[cfg(feature = "egui")]
fn is_playing_or_paused(mode: Res<crate::menus::GameUiMode>) -> bool {
    matches!(
        *mode,
        crate::menus::GameUiMode::Playing | crate::menus::GameUiMode::Paused
    )
}

/// Watches for the transition into `Playing` and, when detected, reinitialises
/// the simulation from `WorldSetupParams.seed` and clears all previously-rendered
/// entities so a New World always starts clean.
///
/// Uses a `Local<Option<GameUiMode>>` to track the previous frame's mode so the
/// reinit fires exactly once per MainMenu→Playing (or Loading→Playing) edge.
#[cfg(feature = "egui")]
fn maybe_reinit_sim_on_new_world(
    mut commands: Commands,
    mode: Res<crate::menus::GameUiMode>,
    params: Res<crate::menus::WorldSetupParams>,
    mut sim: ResMut<SimState>,
    mut rendered: ResMut<RenderedEntities>,
    mut prev_mode: Local<Option<crate::menus::GameUiMode>>,
) {
    let current = *mode;
    let previous = *prev_mode;
    *prev_mode = Some(current);

    // Only act on the frame we transition INTO Playing from a non-Playing state.
    let just_entered_playing = current == crate::menus::GameUiMode::Playing
        && previous != Some(crate::menus::GameUiMode::Playing);

    if !just_entered_playing {
        return;
    }

    // `WorldSetupParams.seed` is already a parsed u64 (the menus agent keeps it
    // in sync via `commit_text()`).  Use it directly; no string parsing needed.
    let seed: u64 = params.seed;

    info!(
        "[sim_bridge] New World: reinitialising simulation with seed={}",
        seed
    );

    // Despawn every previously-rendered civilian entity.
    for (_, entity) in rendered.civilians.drain() {
        commands.entity(entity).despawn();
    }
    // Despawn every previously-rendered building entity.
    for (_, entity) in rendered.buildings.drain() {
        commands.entity(entity).despawn();
    }

    // Replace the simulation with a fresh one seeded from WorldSetupParams.
    sim.0 = Simulation::with_seed(seed);
}

fn advance_simulation(
    time: Res<Time>,
    mut timer: ResMut<SimTickTimer>,
    mut sim: ResMut<SimState>,
    #[cfg(feature = "egui")] speed: Res<crate::game_ui::GameSpeed>,
    #[cfg(feature = "voxel")] voxel_state: Option<Res<VoxelSimState>>,
) {
    // Paused guard: the system is already excluded from Paused by run conditions,
    // but keep the speed-multiplier zero-check for game-speed=0 edge case.
    #[cfg(feature = "egui")]
    if speed.multiplier == 0 {
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        let prior_sample_tick = sim.0.last_emergence_sample().map(|sample| sample.tick);
        #[cfg(feature = "voxel")]
        match voxel_state.as_deref() {
            Some(voxel_state) => sim.0.tick_with_emergence_source(Some(&voxel_state.grid)),
            None => sim.0.tick_with_emergence_source(None),
        }
        #[cfg(not(feature = "voxel"))]
        sim.0.tick();
        let latest_sample_tick = sim.0.last_emergence_sample().map(|sample| sample.tick);
        if latest_sample_tick != Some(sim.0.state.tick) && latest_sample_tick == prior_sample_tick {
            sim.0.sample_emergence();
        }
    }
}

#[cfg(not(feature = "voxel"))]
fn world_to_norm(position: Vec3) -> (f32, f32) {
    let wx = position.x + WORLD_SIZE * 0.5;
    let wz = position.z + WORLD_SIZE * 0.5;
    (
        (wx / WORLD_SIZE).clamp(0.0, 1.0),
        (wz / WORLD_SIZE).clamp(0.0, 1.0),
    )
}

#[cfg(feature = "voxel")]
fn world_to_norm(position: Vec3, voxel: Option<&VoxelSimState>) -> (f32, f32) {
    let (Some(voxel),) = (voxel,) else {
        return (0.5, 0.5);
    };
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return (0.5, 0.5);
    }
    (
        (position.x / dims[0] as f32).clamp(0.0, 1.0),
        (position.z / dims[2] as f32).clamp(0.0, 1.0),
    )
}

fn apply_spawn_civilian_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnCivilianRequest>,
    #[cfg(feature = "voxel")] voxel_state: Option<Res<VoxelSimState>>,
) {
    for request in requests.read() {
        let (nx, ny) = {
            #[cfg(not(feature = "voxel"))]
            {
                world_to_norm(request.position)
            }
            #[cfg(feature = "voxel")]
            {
                world_to_norm(request.position, voxel_state.as_deref())
            }
        };
        let id = next_civilian_id(&sim.0);
        let seed = id.wrapping_add(nx.to_bits() as u64 ^ ny.to_bits() as u64);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        spawn_civilian_at(
            &mut sim.0.world,
            id,
            civ_agents::Alignment::None,
            nx,
            ny,
            request.model_kind,
            &mut rng,
        );
    }
}

fn apply_spawn_building_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnBuildingRequest>,
    #[cfg(feature = "voxel")] voxel_state: Option<Res<VoxelSimState>>,
) {
    for request in requests.read() {
        let (nx, ny) = {
            #[cfg(not(feature = "voxel"))]
            {
                world_to_norm(request.position)
            }
            #[cfg(feature = "voxel")]
            {
                world_to_norm(request.position, voxel_state.as_deref())
            }
        };
        spawn_airport_at(&mut sim.0.world, nx, ny);
    }
}

/// Build the shared civilian/building meshes once at startup.
fn init_gameplay_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let civilian_mesh = meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(1.4, 3.2)));
    let building_mesh = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
        7.0, 14.0, 7.0,
    )));
    commands.insert_resource(GameplayAssets {
        civilian_mesh,
        building_mesh,
    });
}

/// Reconcile rendered civilians/buildings with the simulation.
///
/// P2 visible-citizen-lifecycle split: this system owns the lifecycle
/// (spawn on birth, despawn on death, write the sim-derived target
/// position into [`CivilianVisualTarget`]) and runs every frame. The
/// per-frame transform smoothing lives in [`interpolate_civilian_transforms`].
///
/// **Why every frame and not gated on `sim.is_changed()`:** the sim uses
/// in-place mutation of `hecs::World`, which means a hecs despawn in
/// `phase_life` is invisible to Bevy's change-detection plumbing. We pay
/// the O(N) query cost each frame so births and deaths surface the same
/// frame the sim commits them, instead of being deferred until the next
/// user-initiated `sim.snapshot` or replay event.
fn reconcile_civilian_lifecycle(
    mut commands: Commands,
    sim: Res<SimState>,
    assets: Option<Res<GameplayAssets>>,
    #[cfg(feature = "voxel")] voxel_state: Option<Res<VoxelSimState>>,
    #[cfg(feature = "models")] models: Option<Res<crate::gltf_models::GameModels>>,
    #[cfg(feature = "models")] asset_server: Res<AssetServer>,
    mut rendered: ResMut<RenderedEntities>,
    mut transforms: Query<&mut Transform>,
    mut targets: Query<&mut CivilianVisualTarget>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(assets) = assets else {
        return;
    };

    let mut civ_count = 0_usize;
    let mut first_world_pos: Option<Vec3> = None;
    let mut seen_civilians: Vec<u64> = Vec::new();
    let mut skipped_cap = 0_usize;

    for (_, (civilian, position, visual)) in sim
        .0
        .world
        .query::<(&Civilian, &civ_agents::Position3d, Option<&ActorVisual>)>()
        .iter()
    {
        if civ_count >= MAX_VISIBLE_CIVILIANS {
            // Cap reached: track the dropped count for the log line below.
            skipped_cap += 1;
            continue;
        }
        let visual_kind = visual.map(|v| v.0).unwrap_or(ActorVisualKind::Humanoid);
        civ_count += 1;
        seen_civilians.push(civilian.id);
        let world_pos = {
            #[cfg(not(feature = "voxel"))]
            {
                sim_position_to_world(position) + Vec3::Y * CIVILIAN_HALF_HEIGHT
            }
            #[cfg(feature = "voxel")]
            {
                sim_position_to_world(position, voxel_state.as_deref())
                    + Vec3::Y * CIVILIAN_HALF_HEIGHT
            }
        };
        if first_world_pos.is_none() {
            first_world_pos = Some(world_pos);
        }

        if let Some(&entity) = rendered.civilians.get(&civilian.id) {
            // Already known: refresh the sim-derived *target* so the
            // interpolation pass can slide the visual toward the latest
            // sim position. We deliberately do NOT write `transform.translation`
            // here — that would snap the visual every sim tick (10 Hz) and
            // defeat the whole point of `interpolate_civilian_transforms`.
            // The first-frame spawn already sets Transform to world_pos, so
            // there is no visual discontinuity.
            if let Ok(mut target) = targets.get_mut(entity) {
                target.translation = world_pos;
            }
        } else {
            #[cfg(feature = "models")]
            let scene_root = actor_scene_if_loaded(
                models.as_deref(),
                &asset_server,
                visual_kind,
                civilian_faction_id(civilian),
            );
            #[cfg(not(feature = "models"))]
            let scene_root: Option<SceneRoot> = None;

            let entity = if let Some(scene_root) = scene_root {
                #[cfg(feature = "models")]
                {
                    commands
                        .spawn((
                            SimCivilianMarker,
                            FactionTint(civilian_faction_id(civilian)),
                            scene_root,
                            CivilianVisualTarget {
                                translation: world_pos,
                            },
                            Transform::from_translation(world_pos - Vec3::Y * CIVILIAN_HALF_HEIGHT)
                                .with_scale(Vec3::splat(model_scale_for(visual_kind))),
                        ))
                        .id()
                }
                #[cfg(not(feature = "models"))]
                {
                    let _ = scene_root;
                    unreachable!()
                }
            } else {
                let faction = civilian_faction_id(civilian);
                let color = faction_color(faction);
                let material = materials.add(StandardMaterial {
                    base_color: color,
                    // Solid lit object, not a glowing pill: kill the emissive bloom.
                    emissive: LinearRgba::BLACK,
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                });
                commands
                    .spawn((
                        #[cfg(feature = "models")]
                        SimCivilianMarker,
                        #[cfg(feature = "models")]
                        PendingActorScene {
                            kind: visual_kind,
                            faction,
                        },
                        #[cfg(not(feature = "models"))]
                        SimCivilianMarker,
                        CivilianVisualTarget {
                            translation: world_pos,
                        },
                        Mesh3d(assets.civilian_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(world_pos),
                    ))
                    .id()
            };
            rendered.civilians.insert(civilian.id, entity);
        }
    }

    // Despawn civilians that no longer exist in the simulation (death path).
    // This is the visibility half of the lifecycle: every frame we walk the
    // map and drop entities whose sim id is gone, so the world stays clean
    // even when the sim despawns civilians faster than the next hecs query
    // re-runs.
    rendered.civilians.retain(|id, &mut entity| {
        if seen_civilians.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });

    let mut seen_buildings: Vec<u64> = Vec::new();
    let mut building_idx = 0_u64;
    for (_, building) in sim.0.world.query::<&Building>().iter() {
        let id = building_idx;
        building_idx += 1;
        seen_buildings.push(id);
        let world_pos = {
            #[cfg(not(feature = "voxel"))]
            {
                building_world_position(building) + Vec3::Y * BUILDING_HALF_HEIGHT
            }
            #[cfg(feature = "voxel")]
            {
                building_world_position(building, voxel_state.as_deref())
                    + Vec3::Y * BUILDING_HALF_HEIGHT
            }
        };

        if let Some(&entity) = rendered.buildings.get(&id) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = world_pos;
            }
        } else {
            #[cfg(feature = "models")]
            let scene_root = building_model_root(models.as_deref(), building.building_type);
            #[cfg(not(feature = "models"))]
            let scene_root: Option<SceneRoot> = None;

            let entity = if let Some(scene_root) = scene_root {
                #[cfg(feature = "models")]
                {
                    commands
                        .spawn((
                            SimBuildingMarker,
                            scene_root,
                            Transform::from_translation(world_pos - Vec3::Y * BUILDING_HALF_HEIGHT)
                                .with_scale(Vec3::splat(BUILDING_MODEL_SCALE)),
                        ))
                        .id()
                }
                #[cfg(not(feature = "models"))]
                {
                    let _ = scene_root;
                    unreachable!()
                }
            } else {
                let color = building_color(building.building_type);
                let material = materials.add(StandardMaterial {
                    base_color: color,
                    // No additive/neon glow: read as a solid lit structure.
                    emissive: LinearRgba::BLACK,
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                });
                commands
                    .spawn((
                        SimBuildingMarker,
                        Mesh3d(assets.building_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(world_pos),
                    ))
                    .id()
            };
            rendered.buildings.insert(id, entity);
        }
    }
    rendered.buildings.retain(|id, &mut entity| {
        if seen_buildings.contains(id) {
            true
        } else {
            commands.entity(entity).despawn();
            false
        }
    });

    if skipped_cap > 0 {
        info!(
            "[sim_bridge] civilians visible={} (cap={}, dropped {} above cap) buildings={} civilian[0] world={:?}",
            civ_count,
            MAX_VISIBLE_CIVILIANS,
            skipped_cap,
            seen_buildings.len(),
            first_world_pos
        );
    } else {
        info!(
            "[sim_bridge] civilians={} buildings={} civilian[0] world={:?}",
            civ_count,
            seen_buildings.len(),
            first_world_pos
        );
    }
}

/// Smoothly slide every visible civilian's transform toward its latest sim
/// position. Pairs with [`reconcile_civilian_lifecycle`], which writes
/// [`CivilianVisualTarget`] once per sim tick. This system runs every frame
/// so civilians move at the renderer's cadence (60+ Hz) rather than the
/// sim's (10 Hz), making the walk read as a walk instead of a teleport.
///
/// Uses an exponential approach (`1 - exp(-k·dt)`) so the lerp factor is
/// frame-rate independent: doubling the dt roughly doubles the closure
/// fraction. `CIVILIAN_LERP_SPEED` is the picked stiffness.
fn interpolate_civilian_transforms(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &CivilianVisualTarget), With<SimCivilianMarker>>,
) {
    let dt = time.delta_secs();
    // Clamp dt so a long pause (debugger break, alt-tab, slow CI) doesn't
    // catapult the visual past the target in a single frame — same
    // robustness rule Bevy's `Slerp` interpolators use.
    let dt = dt.min(0.1);
    let alpha = 1.0 - (-CIVILIAN_LERP_SPEED * dt).exp();
    for (mut transform, target) in &mut query {
        transform.translation = transform.translation.lerp(target.translation, alpha);
    }
}

fn civilian_faction_id(civilian: &Civilian) -> u32 {
    match civilian.alignment {
        civ_agents::Alignment::Faction(faction) => faction,
        _ => 0,
    }
}

#[cfg(feature = "models")]
fn upgrade_pending_actor_scenes(
    mut commands: Commands,
    models: Option<Res<crate::gltf_models::GameModels>>,
    asset_server: Res<AssetServer>,
    mut rendered: ResMut<RenderedEntities>,
    query: Query<(Entity, &Transform, &PendingActorScene), (With<SimCivilianMarker>, With<Mesh3d>)>,
) {
    let Some(models) = models.as_deref() else {
        return;
    };

    let mut upgrades: Vec<(u64, Entity)> = Vec::new();
    for (&sim_id, &entity) in rendered.civilians.iter() {
        let Ok((entity, transform, pending)) = query.get(entity) else {
            continue;
        };

        let Some(scene_root) =
            actor_scene_if_loaded(Some(models), &asset_server, pending.kind, pending.faction)
        else {
            continue;
        };

        let mut next_transform = *transform;
        next_transform.translation -= Vec3::Y * CIVILIAN_HALF_HEIGHT;
        next_transform.scale = Vec3::splat(model_scale_for(pending.kind));

        let new_entity = commands
            .spawn((
                SimCivilianMarker,
                FactionTint(pending.faction),
                scene_root,
                next_transform,
            ))
            .id();
        commands.entity(entity).despawn();

        upgrades.push((sim_id, new_entity));
    }

    for (sim_id, new_entity) in upgrades {
        rendered.civilians.insert(sim_id, new_entity);
    }
}

/// Map a normalised sim agent coordinate to centred, terrain-seated world XZ.
#[cfg(not(feature = "voxel"))]
fn sim_position_to_world(position: &civ_agents::Position3d) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    let nx = position.coord.x as f32 / scale;
    let nz = position.coord.z as f32 / scale;
    let mesh_x = nx * WORLD_SIZE;
    let mesh_z = nz * WORLD_SIZE;
    // Seat on the surface, but never below sea level (no underwater civilians).
    let y = terrain_surface_y(mesh_x, mesh_z).max(WATER_LEVEL);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}

#[cfg(feature = "voxel")]
fn sim_position_to_world(position: &civ_agents::Position3d, voxel: Option<&VoxelSimState>) -> Vec3 {
    // Mapping choice A: use voxel grid dims as the single world extent.
    // Sim-normalized [0,1] maps directly to [0,dims] so 0.5 lands at center and
    // corners at 0 and 1 match voxel box corners.
    let (Some(voxel),) = (voxel,) else {
        return Vec3::ZERO;
    };
    let scale = civ_voxel::FIXED_SCALE as f32;
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return Vec3::ZERO;
    }
    let nx = (position.coord.x as f32 / scale).clamp(0.0, 1.0);
    let nz = (position.coord.z as f32 / scale).clamp(0.0, 1.0);
    let mesh_x = nx * dims[0] as f32;
    let mesh_z = nz * dims[2] as f32;
    let y = voxel_surface_y(&voxel.grid, mesh_x, mesh_z);
    Vec3::new(mesh_x, y, mesh_z)
}

/// Map an integer building grid tile to centred, terrain-seated world XZ.
///
/// `Building::position` is a centred grid coordinate in `[-64, 63]` (0,0 is
/// map centre). We normalise it to `0..1` (matching `engine::grid_to_norm`),
/// scale onto the mesh extent, then sample the surface. If that surface is
/// below sea level the base is clamped to [`WATER_LEVEL`] so the building rests
/// at the shoreline instead of being submerged or floating on the water plane.
#[cfg(not(feature = "voxel"))]
fn building_world_position(building: &Building) -> Vec3 {
    let nx = ((building.position.x + 64) as f32 / 127.0).clamp(0.0, 1.0);
    let nz = ((building.position.y + 64) as f32 / 127.0).clamp(0.0, 1.0);
    let mesh_x = nx * WORLD_SIZE;
    let mesh_z = nz * WORLD_SIZE;
    let y = terrain_surface_y(mesh_x, mesh_z).max(WATER_LEVEL);
    Vec3::new(mesh_x - WORLD_SIZE * 0.5, y, mesh_z - WORLD_SIZE * 0.5)
}
#[cfg(feature = "voxel")]
fn building_world_position(building: &Building, voxel: Option<&VoxelSimState>) -> Vec3 {
    let (Some(voxel),) = (voxel,) else {
        return Vec3::ZERO;
    };
    let dims = voxel.grid.dims;
    if dims[0] == 0 || dims[2] == 0 {
        return Vec3::ZERO;
    }
    let x_span = (dims[0] as f32 - 1.0).max(1.0);
    let z_span = (dims[2] as f32 - 1.0).max(1.0);
    let nx = ((building.position.x as f32 + x_span * 0.5) / x_span).clamp(0.0, 1.0);
    let nz = ((building.position.y as f32 + z_span * 0.5) / z_span).clamp(0.0, 1.0);
    let mesh_x = nx * dims[0] as f32;
    let mesh_z = nz * dims[2] as f32;
    let y = voxel_surface_y(&voxel.grid, mesh_x, mesh_z);
    Vec3::new(mesh_x, y, mesh_z)
}

fn faction_color(faction: u32) -> Color {
    let hue = (faction as f32 * 85.0) % 360.0;
    // Normal saturated faction colour (not over-bright) for a lit PBR body.
    Color::hsla(hue, 0.6, 0.45, 1.0)
}

fn building_color(building_type: BuildingType) -> Color {
    match building_type {
        BuildingType::Farm => Color::srgb(0.55, 0.75, 0.35),
        BuildingType::Mine => Color::srgb(0.52, 0.48, 0.42),
        BuildingType::Barracks => Color::srgb(0.72, 0.34, 0.34),
        BuildingType::Temple => Color::srgb(0.72, 0.62, 0.88),
        BuildingType::Market => Color::srgb(0.88, 0.67, 0.25),
        BuildingType::House => Color::srgb(0.79, 0.59, 0.40),
        BuildingType::CityCenter => Color::srgb(0.38, 0.58, 0.86),
    }
}

fn next_civilian_id(sim: &Simulation) -> u64 {
    sim.world
        .query::<&Civilian>()
        .iter()
        .map(|(_, civilian)| civilian.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn tick_to_era_label(tick: u64) -> String {
    let era = if tick < 100 {
        "Prehistoric"
    } else if tick < 500 {
        "Stone Age"
    } else if tick < 1500 {
        "Bronze Age"
    } else if tick < 3000 {
        "Iron Age"
    } else if tick < 6000 {
        "Classical"
    } else if tick < 12000 {
        "Medieval"
    } else if tick < 25000 {
        "Renaissance"
    } else if tick < 50000 {
        "Industrial"
    } else if tick < 100000 {
        "Modern"
    } else {
        "Information"
    };
    era.to_string()
}

#[cfg(feature = "egui")]
fn sync_game_ui_snapshot(
    sim: Res<SimState>,
    speed: Res<crate::game_ui::GameSpeed>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
    mut resources: ResMut<crate::game_ui::WorldResources>,
    mut roster: ResMut<crate::game_ui::FactionRoster>,
) {
    if !sim.is_changed() {
        return;
    }
    let sim = &sim.0;

    // Pull the authoritative per-tick snapshot for stocks + vital stats.
    let snap = sim.snapshot();
    let population = snap.citizen_count as u64;

    // Emergent clusters (settlements) drive both the faction count chip and the
    // left-panel roster — these are NOT hardcoded factions.
    let roster_rows = build_faction_roster(sim);
    let faction_count = roster_rows.len() as u32;
    roster.factions = roster_rows;

    snapshot.set_sim_state(
        snap.tick,
        population,
        faction_count,
        tick_to_era_label(snap.tick),
        speed.multiplier.max(1),
    );

    // World resource strip: global economy stocks + the pooled settlement
    // commons as a stand-in treasury, plus births/deaths this tick.
    // TODO(FR-CIV-ECON): resources are seeded, not yet produced by agent labor at the resource-read site.
    let food = snap.resources.food.to_f64();
    let materials = snap.resources.wood.to_f64() + snap.resources.metal.to_f64();
    let energy = snap.resources.energy.to_f64();
    let treasury: f64 = sim
        .cluster_stocks()
        .values()
        .map(|stock| stock.total() as f64)
        .sum();
    resources.update_stocks(
        food,
        materials,
        energy,
        treasury,
        snap.births_this_tick,
        snap.deaths_this_tick,
    );
}

/// Build the left-panel roster from emergent clusters.
///
/// Counts civilians per [`civ_agents::ClusterMember`] cluster id; civilians with
/// no membership component are pooled under the id-0 "Unaffiliated" bucket so
/// the panel always reflects the full live population. Rows are ordered by
/// descending size so the largest settlements lead.
#[cfg(feature = "egui")]
fn build_faction_roster(sim: &Simulation) -> Vec<crate::game_ui::FactionInfo> {
    use std::collections::BTreeMap;
    let mut sizes: BTreeMap<u64, u64> = BTreeMap::new();
    for (_, (_civilian, member)) in sim
        .world
        .query::<(&Civilian, Option<&civ_agents::ClusterMember>)>()
        .iter()
    {
        let id = member.map(|m| m.cluster.0).unwrap_or(0);
        *sizes.entry(id).or_insert(0) += 1;
    }
    let mut rows: Vec<_> = sizes
        .into_iter()
        .map(|(id, count)| crate::game_ui::FactionInfo::from_cluster(id, count))
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then(a.name.cmp(&b.name)));
    rows
}

// ---------------------------------------------------------------------------
// Diplomacy Standings seam (civ-007 / P4 diplomacy substrate).
// ---------------------------------------------------------------------------

/// One row in the Diplomacy panel's Standings list — a single pairwise
/// relation between two actor ids, projected from the live substrate into a
/// UI-friendly scalar + coarse stance label.
///
/// # Seam
///
/// This struct is the **client-side projection** of a
/// [`civ_diplomacy::Relation`]. The bevy-ref client does not own the substrate
/// (the simulation does); it mirrors the engine's pair list by polling the
/// snapshot each tick and folding legacy `civ_engine::DiplomacyEvent`s into a
/// local [`civ_diplomacy::DiplomacyState`] for projection.
///
/// **Upgrade path:** when the simulation exposes a Bevy message stream
/// (`Messages<DiplomacyTickEvent>`) directly, replace `sync_diplomacy_standings`
/// with a message reader and drop the local substrate instance. The shape of
/// [`StandingRow`] is the contract — keep it stable.
#[cfg(feature = "egui")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingRow {
    /// Lower actor id (canonical pair key).
    pub a: u32,
    /// Higher actor id.
    pub b: u32,
    /// Signed scalar standing in `[-standing_max, +standing_max]`.
    pub standing: i32,
    /// Coarse stance label, derived from `standing` + the substrate config's
    /// thresholds. One of `"Hostile" | "Neutral" | "Allied"`.
    pub stance: StandingStance,
    /// Tick of the last update (bump or decay step that actually changed the
    /// value). Useful for tooltip / replay debugging.
    pub last_updated_tick: u64,
}

/// Coarse stance surfaced to the UI. Mirrors `civ_diplomacy::Stance` but as a
/// `Copy` enum so the UI can pattern-match without importing the substrate.
#[cfg(feature = "egui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandingStance {
    /// Standing ≤ `hostile_threshold`.
    Hostile,
    /// Standing between the two thresholds.
    Neutral,
    /// Standing ≥ `allied_threshold`.
    Allied,
}

/// Diplomacy Standings resource read by `crate::diplomacy_ui`.
///
/// Updated each frame the sim ticks (gated by `is_playing`). Rows are kept
/// sorted descending by absolute standing so the most intense relations sit at
/// the top — players scan for "who is at war" and "who is allied", not the
/// middle. The substrate's BTreeMap iteration is stable but ascending; we
/// resort on write to keep the UI work in O(n log n) and zero per-frame
/// allocation.
#[cfg(feature = "egui")]
#[derive(Resource, Default, Debug, Clone)]
pub struct DiplomacyStandings {
    /// Sorted rows (descending |standing|). Empty until the first sample.
    pub rows: Vec<StandingRow>,
    /// Tick the rows were last refreshed from the sim. `0` means "no sample
    /// yet" so the UI can show a "waiting for sim" placeholder.
    pub last_refresh_tick: u64,
    /// Whether the substrate has ever received any input. Mirrors the
    /// `live` flag in `crate::diplomacy_ui::DiplomacyState`.
    pub live: bool,
}

#[cfg(feature = "egui")]
impl DiplomacyStandings {
    /// Project a signed standing value to a [`StandingStance`] using the same
    /// thresholds as the substrate (`hostile_threshold` ≤ 0, `allied_threshold`
    /// ≥ 0). Matches `civ_diplomacy::stance_for` semantically.
    pub fn stance_for(standing: i32, hostile_threshold: i32, allied_threshold: i32) -> StandingStance {
        if standing <= hostile_threshold {
            StandingStance::Hostile
        } else if standing >= allied_threshold {
            StandingStance::Allied
        } else {
            StandingStance::Neutral
        }
    }
}

/// Build the `DiploamcyStandings` rows from a sim snapshot, folding legacy
/// `civ_engine::DiplomacyEvent`s through a local substrate instance.
///
/// This is the **seam** between the engine's event surface and the new
/// `civ-diplomacy` substrate. The substrate is the authoritative scorer; the
/// engine's `DiplomacyEvent` list is the input we already have. When the
/// substrate is wired directly into the engine (civ-007 Phase 2), this
/// function should be replaced by a reader on the live event stream — the
/// output shape (`Vec<StandingRow>` sorted by |standing|) is the contract.
#[cfg(feature = "egui")]
fn build_standings_rows(
    sim: &Simulation,
    substrate: &mut civ_diplomacy::DiplomacyState,
    last_event_tick: &mut u64,
) -> Vec<StandingRow> {
    let snap = sim.snapshot();

    // Fold any new diplomacy events through the substrate. The substrate
    // emits threshold-crossing events on its own `pending_events` queue; we
    // drain them after the fold so the substrate state stays in sync (decay
    // is not applied here — we mirror the engine's pacing, not the substrate's
    // intrinsic decay clock, because the panel reflects "what the sim knows",
    // not "what the substrate believes after a fixed decay interval").
    for ev in &snap.diplomacy_events {
        if ev.tick <= *last_event_tick {
            continue;
        }
        let interaction = legacy_event_to_interaction(ev);
        substrate.ingest(&[interaction]);
        *last_event_tick = (*last_event_tick).max(ev.tick);
    }
    // Drain substrate-emitted events so they don't accumulate; we don't render
    // them here, but a future event-feed could subscribe to them.
    let _ = substrate.drain_events();

    let cfg = &substrate.config;
    let mut rows: Vec<StandingRow> = substrate
        .relations()
        .map(|rel| StandingRow {
            a: rel.pair.lo.0,
            b: rel.pair.hi.0,
            standing: rel.standing,
            stance: DiplomacyStandings::stance_for(rel.standing, cfg.hostile_threshold, cfg.allied_threshold),
            last_updated_tick: rel.last_updated_tick,
        })
        .collect();
    // Sort: most extreme (positive or negative) standing first. Ties broken
    // by the canonical (lo, hi) id pair so the order is deterministic across
    // frames and across runs (replay-safe).
    rows.sort_by(|x, y| {
        y.standing
            .unsigned_abs()
            .cmp(&x.standing.unsigned_abs())
            .then((x.a, x.b).cmp(&(y.a, y.b)))
    });
    rows
}

/// Translate a legacy `civ_engine::DiplomacyEvent` into a substrate
/// `InteractionEvent::Gesture` with a coarse delta. The magnitudes are
/// chosen to map the engine's three coarse kinds onto the substrate's
/// scalar standing so a single TradeAgreement nudges a pair toward Allied
/// and a single Conflict crosses the Hostile threshold given the default
/// `hostile_threshold = -100`.
#[cfg(feature = "egui")]
fn legacy_event_to_interaction(
    ev: &civ_engine::DiplomacyEvent,
) -> civ_diplomacy::InteractionEvent {
    use civ_diplomacy::{ActorId, InteractionEvent};
    let delta: i32 = match ev.kind {
        civ_engine::DiplomacyKind::TradeAgreement => 60,
        civ_engine::DiplomacyKind::Peace => 25,
        civ_engine::DiplomacyKind::Conflict => -120,
    };
    InteractionEvent::Gesture {
        from: ActorId(ev.faction_a),
        to: ActorId(ev.faction_b),
        delta,
        tick: ev.tick,
    }
}

/// System: pull diplomacy events out of the sim snapshot and refresh the
/// `DiplomacyStandings` resource.
///
/// Uses a `Local` to keep the substrate instance + dedup cursor on the
/// system itself rather than the resource, so the resource stays a pure
/// view (cheap to clone for the UI; no leaked substrate state).
#[cfg(feature = "egui")]
fn sync_diplomacy_standings(
    sim: Res<SimState>,
    mut standings: ResMut<DiplomacyStandings>,
    // Local: persistent across frames but not serialized.
    mut local: Local<Option<(civ_diplomacy::DiplomacyState, u64)>>,
) {
    if !sim.is_changed() {
        return;
    }
    let (substrate, last_event_tick) = local.get_or_insert_with(|| {
        // Default substrate config matches the substrate crate's `Default`
        // (standing_max 1000, decay 1, hostile -100, allied 100). We never
        // call `decay()` from this seam — the engine's pacing wins.
        let cfg = civ_diplomacy::DiplomacyConfig::default();
        let state = civ_diplomacy::DiplomacyState::new(cfg)
            .expect("civ-diplomacy default config validates");
        (state, 0)
    });
    let rows = build_standings_rows(&sim.0, substrate, last_event_tick);
    let tick = sim.0.state.tick;
    standings.last_refresh_tick = tick;
    standings.live = !rows.is_empty() || *last_event_tick > 0;
    standings.rows = rows;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "models")]
    #[test]
    fn model_scale_matches_actor_kind() {
        // Guards the sub-pixel actor-scale fix: humanoids and herd use their
        // respective constants (and herd reads larger than a civilian).
        assert_eq!(
            model_scale_for(ActorVisualKind::Humanoid),
            CIVILIAN_MODEL_SCALE
        );
        assert_eq!(model_scale_for(ActorVisualKind::Herd), HERD_MODEL_SCALE);
        assert!(HERD_MODEL_SCALE >= CIVILIAN_MODEL_SCALE);
    }

    #[test]
    fn faction_color_is_deterministic_and_hue_varies() {
        // Same faction -> same colour across calls; different factions differ.
        assert_eq!(faction_color(0), faction_color(0));
        assert_ne!(faction_color(0), faction_color(1));
        assert_ne!(faction_color(1), faction_color(2));
    }

    #[test]
    fn building_color_is_distinct_per_type() {
        use std::collections::HashSet;
        let types = [
            BuildingType::Farm,
            BuildingType::Mine,
            BuildingType::Barracks,
            BuildingType::Temple,
            BuildingType::Market,
            BuildingType::House,
            BuildingType::CityCenter,
        ];
        // Encode each colour's sRGB bytes so distinct building types read as
        // visually distinct swatches (no two share a colour).
        let mut seen: HashSet<[u8; 3]> = HashSet::new();
        for t in types {
            let c = building_color(t).to_srgba();
            let key = [
                (c.red * 255.0) as u8,
                (c.green * 255.0) as u8,
                (c.blue * 255.0) as u8,
            ];
            assert!(seen.insert(key), "duplicate building colour for {t:?}");
        }
    }

    #[test]
    fn tick_to_era_label_uses_expected_boundaries() {
        assert_eq!(tick_to_era_label(0), "Prehistoric");
        assert_eq!(tick_to_era_label(50000), "Modern");
    }

    // -- Diplomacy standings seam -------------------------------------------

    #[cfg(feature = "egui")]
    #[test]
    fn standings_stance_for_matches_substrate_thresholds() {
        let (h, a) = (-100, 100);
        // Boundaries: ≤ hostile → Hostile, ≥ allied → Allied, else Neutral.
        assert_eq!(
            DiplomacyStandings::stance_for(-101, h, a),
            StandingStance::Hostile
        );
        assert_eq!(
            DiplomacyStandings::stance_for(-100, h, a),
            StandingStance::Hostile
        );
        assert_eq!(
            DiplomacyStandings::stance_for(-99, h, a),
            StandingStance::Neutral
        );
        assert_eq!(
            DiplomacyStandings::stance_for(0, h, a),
            StandingStance::Neutral
        );
        assert_eq!(
            DiplomacyStandings::stance_for(99, h, a),
            StandingStance::Neutral
        );
        assert_eq!(
            DiplomacyStandings::stance_for(100, h, a),
            StandingStance::Allied
        );
    }

    #[cfg(feature = "egui")]
    #[test]
    fn legacy_event_to_interaction_maps_kinds_to_signed_deltas() {
        use civ_diplomacy::InteractionEvent;
        let trade = legacy_event_to_interaction(&civ_engine::DiplomacyEvent {
            tick: 5,
            faction_a: 1,
            faction_b: 2,
            kind: civ_engine::DiplomacyKind::TradeAgreement,
        });
        let peace = legacy_event_to_interaction(&civ_engine::DiplomacyEvent {
            tick: 5,
            faction_a: 1,
            faction_b: 2,
            kind: civ_engine::DiplomacyKind::Peace,
        });
        let conflict = legacy_event_to_interaction(&civ_engine::DiplomacyEvent {
            tick: 5,
            faction_a: 1,
            faction_b: 2,
            kind: civ_engine::DiplomacyKind::Conflict,
        });
        // TradeAgreement and Peace produce positive deltas; Conflict negative.
        let (from, to, delta, tick) = match trade {
            InteractionEvent::Gesture { from, to, delta, tick } => (from, to, delta, tick),
            _ => panic!("trade is a Gesture"),
        };
        assert_eq!((from.0, to.0, tick), (1, 2, 5));
        assert!(delta > 0);
        let conflict_delta = match conflict {
            InteractionEvent::Gesture { delta, .. } => delta,
            _ => panic!("conflict is a Gesture"),
        };
        assert!(conflict_delta < 0);
        let peace_delta = match peace {
            InteractionEvent::Gesture { delta, .. } => delta,
            _ => panic!("peace is a Gesture"),
        };
        assert!(peace_delta > 0);
        // Sanity: a single Conflict (-120) crosses the default hostile
        // threshold of -100, so the projection should land the pair in the
        // Hostile band after a single event.
        assert!(conflict_delta <= -100);
    }

    #[cfg(feature = "egui")]
    #[test]
    fn build_standings_rows_folds_legacy_events_and_sorts_by_abs() {
        use civ_diplomacy::{ActorId, InteractionEvent};
        // Fresh sim has no diplomacy events → no rows.
        let sim = Simulation::default();
        let cfg = civ_diplomacy::DiplomacyConfig::default();
        let mut substrate =
            civ_diplomacy::DiplomacyState::new(cfg).expect("default config validates");
        let mut last_event_tick: u64 = 0;
        let rows = build_standings_rows(&sim, &mut substrate, &mut last_event_tick);
        assert!(rows.is_empty(), "no events -> no rows");
        assert_eq!(last_event_tick, 0);

        // Seed two relations directly into the substrate and verify the
        // projection + sort order: most extreme |standing| first.
        substrate.ingest(&[InteractionEvent::Gesture {
            from: ActorId(0),
            to: ActorId(2),
            delta: 60, // +60 (positive, but mid-band)
            tick: 1,
        }]);
        substrate.ingest(&[InteractionEvent::Gesture {
            from: ActorId(0),
            to: ActorId(1),
            delta: -150, // -150 → Hostile
            tick: 1,
        }]);
        substrate.ingest(&[InteractionEvent::Gesture {
            from: ActorId(1),
            to: ActorId(2),
            delta: 120, // +120 → Allied
            tick: 1,
        }]);

        let rows = build_standings_rows(&sim, &mut substrate, &mut last_event_tick);
        assert_eq!(rows.len(), 3, "three pairwise relations");
        // (0,1) is at -150, (1,2) is at +120, (0,2) is at +60 → |standings|
        // are 150, 120, 60. Sort descending by |standing| yields that order.
        let order: Vec<(u32, u32, i32)> = rows
            .iter()
            .map(|r| (r.a, r.b, r.standing))
            .collect();
        assert_eq!(order, vec![(0, 1, -150), (1, 2, 120), (0, 2, 60)]);
        // Stance labels project correctly given the default thresholds.
        assert!(matches!(rows[0].stance, StandingStance::Hostile));
        assert!(matches!(rows[1].stance, StandingStance::Allied));
        assert!(matches!(rows[2].stance, StandingStance::Neutral));
        // Dedup cursor must advance when the sim has no events (we just
        // seeded the substrate directly, so the cursor stays at 0).
        assert_eq!(last_event_tick, 0);
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn actor_seats_feet_on_voxel_surface() {
        use civ_agents::Position3d;
        use civ_voxel::fluid_ca::CaGrid;
        use civ_voxel::material::DIRT;
        use civ_voxel::WorldCoord;

        // 8x8x8 world, solid DIRT floor 3 voxels tall under every column.
        let dims = [8usize, 8, 8];
        let mut grid = CaGrid::new(dims);
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                for y in 0..3 {
                    grid.set(x, y, z, DIRT);
                }
            }
        }
        let mut voxel = VoxelSimState::default();
        voxel.grid = grid;

        // A civilian at normalised map centre (0.5, 0.5).
        let scale = civ_voxel::FIXED_SCALE as f32;
        let pos = Position3d {
            coord: WorldCoord {
                x: (0.5 * scale) as i64,
                y: 0,
                z: (0.5 * scale) as i64,
            },
        };
        let world = sim_position_to_world(&pos, Some(&voxel));

        // The mapped XZ must land inside the grid extent, and Y must equal the
        // voxel surface so the model's feet rest on the terrain. A 3-tall floor
        // (solid at y=0,1,2) has its surface at y=3.0 — voxel_surface_y returns
        // `highest_solid_y + 1` (top face of voxel index 2).
        let expected_y = voxel_surface_y(&voxel.grid, world.x, world.z);
        assert!((world.y - expected_y).abs() < f32::EPSILON);
        assert!(
            (world.y - 3.0).abs() < f32::EPSILON,
            "surface should be 3.0, got {}",
            world.y
        );
        assert!(world.x >= 0.0 && world.x <= dims[0] as f32);
        assert!(world.z >= 0.0 && world.z <= dims[2] as f32);
    }

    // -------------------------------------------------------------------
    // P2 visible-citizen-lifecycle — reconcile + interpolate unit tests.
    //
    // These build a minimal Bevy `App` with the sim-bridge resources and
    // systems, drive the hecs sim directly (spawn/despawn/tick), then
    // assert the resulting entity population + transforms. The mesh /
    // material handles are dummy — the tests never render, they only
    // assert the (de)spawn + transform pipeline.
    // -------------------------------------------------------------------

    #[cfg(feature = "bevy")]
    fn build_bridge_app() -> App {
        let mut app = App::new();
        // MinimalPlugins gives us `Time` (the interpolator's clock). We add
        // `AssetPlugin` + `ImagePlugin` separately so `Assets<Mesh>` /
        // `Assets<StandardMaterial>` exist (the `from_world`-init that
        // installs them lives in `bevy::prelude::PipelinedRenderingPlugin`).
        // Headless (no GPU) — we never render, only build the (de)spawn +
        // transform pipeline.
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(bevy::image::ImagePlugin::default())
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>()
            .init_resource::<SimState>()
            .init_resource::<RenderedEntities>()
            .add_systems(Startup, |mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>| {
                let civilian_mesh = meshes
                    .add(Mesh::from(bevy::math::primitives::Capsule3d::new(1.4, 3.2)));
                let building_mesh = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
                    7.0, 14.0, 7.0,
                )));
                commands.insert_resource(GameplayAssets {
                    civilian_mesh,
                    building_mesh,
                });
            })
            .add_systems(Update, (reconcile_civilian_lifecycle, interpolate_civilian_transforms));
        app
    }

    /// P2 visible-citizen-lifecycle — when a sim civilian exists, a Bevy
    /// `SimCivilianMarker` entity is spawned carrying the sim-derived
    /// world position in `CivilianVisualTarget`. Guards the **birth**
    /// half of the lifecycle slice.
    #[cfg(feature = "bevy")]
    #[test]
    fn reconcile_spawns_entity_per_sim_civilian() {
        use civ_agents::Civilian;
        let mut app = build_bridge_app();

        // Sanity: a fresh sim already carries a population.
        let sim_count = {
            let sim = app.world().resource::<SimState>();
            sim.0.world.query::<&Civilian>().iter().count()
        };
        assert!(sim_count > 0, "default sim should have at least one civilian");

        // First reconcile: produces one entity per sim civilian.
        app.update();

        let bevy_count = app
            .world_mut()
            .query_filtered::<Entity, With<SimCivilianMarker>>()
            .iter(app.world())
            .count();
        assert_eq!(
            bevy_count, sim_count,
            "reconcile must spawn one Bevy entity per sim civilian"
        );

        // Every spawned entity must carry a CivilianVisualTarget with a
        // finite translation (proves the sim → world position math wired
        // through the bridge, not just the marker).
        let mut bad = 0_usize;
        let mut q = app
            .world_mut()
            .query::<(&Transform, &CivilianVisualTarget)>();
        for (t, target) in q.iter(app.world()) {
            if !t.translation.is_finite() || !target.translation.is_finite() {
                bad += 1;
            }
            // First-spawn frame: target and transform must agree (the
            // interpolator hasn't run yet, so the transform == target).
            if (t.translation - target.translation).length() > f32::EPSILON {
                bad += 1;
            }
        }
        assert_eq!(bad, 0, "every visual must have finite, aligned transform+target");
    }

    /// P2 visible-citizen-lifecycle — when a sim civilian is despawned
    /// from the hecs world (death path), the next reconcile drops the
    /// matching Bevy entity. Guards the **death** half of the slice.
    #[cfg(feature = "bevy")]
    #[test]
    fn reconcile_despawns_entity_when_sim_citizen_dies() {
        use civ_agents::Civilian;
        let mut app = build_bridge_app();

        // Spawn everything.
        app.update();
        let before = app
            .world_mut()
            .query_filtered::<Entity, With<SimCivilianMarker>>()
            .iter(app.world())
            .count();
        assert!(before > 0);

        // Kill every civilian in the hecs world. Hold a single
        // `&mut SimState` borrow for the whole despawn pass so the
        // hecs query borrows don't outlive `sim`.
        let killed: usize = {
            let mut sim = app.world_mut().resource_mut::<SimState>();
            let to_kill: Vec<hecs::Entity> = sim
                .0
                .world
                .query::<&Civilian>()
                .iter()
                .map(|(e, _)| e)
                .collect();
            for e in to_kill {
                let _ = sim.0.world.despawn(e);
            }
            // Sanity: sim is empty after the pass.
            let remaining = sim.0.world.query::<&Civilian>().iter().count();
            assert_eq!(remaining, 0);
            before
        };
        assert!(killed > 0);

        // One more reconcile → all Bevy marker entities must be gone.
        app.update();
        let after = app
            .world_mut()
            .query_filtered::<Entity, With<SimCivilianMarker>>()
            .iter(app.world())
            .count();
        assert_eq!(
            after, 0,
            "every despawned sim civilian must drop its Bevy entity on the next reconcile"
        );
    }

    /// P2 visible-citizen-lifecycle — the interpolator slides the visual
    /// transform toward the sim target each frame, so an entity whose
    /// transform starts off-target moves part-way toward the target
    /// (not the full distance) in a single update. This is the
    /// smoothness contract the slice promises: civilians walk, they
    /// don't teleport.
    #[cfg(feature = "bevy")]
    #[test]
    fn interpolation_moves_visual_toward_target() {
        let mut app = build_bridge_app();

        // First reconcile spawns the entities at their sim positions.
        app.update();

        // Pick any spawned entity, capture its (sim-aligned) target, and
        // yank its Transform back to origin. The reconciler has nothing
        // to change about the target (sim hasn't moved), so on the
        // *next* update the interpolator will lerp Transform from
        // origin toward the still-unchanged target. The reconciler
        // runs *before* the interpolator in the schedule tuple, so we
        // don't fight a stale target write.
        let (entity, target_translation) = {
            let mut q = app
                .world_mut()
                .query::<(Entity, &Transform, &CivilianVisualTarget)>();
            q.iter(app.world())
                .map(|(e, _, t)| (e, t.translation))
                .next()
                .expect("at least one civilian visual")
        };
        {
            let mut q = app.world_mut().query::<&mut Transform>();
            let Ok(mut t) = q.get_mut(app.world_mut(), entity) else {
                panic!("entity must have a Transform")
            };
            t.translation = Vec3::ZERO;
        }

        // Advance the clock by 0.1 s and run an update. With
        // CIVILIAN_LERP_SPEED = 8.0, the per-frame alpha is
        // 1 - exp(-0.8) ≈ 0.55, so the visual should move ~55% of
        // the gap (not the full distance) in a single update.
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f64(0.1));
        }
        app.update();

        let after = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .unwrap()
            .translation;
        let traveled = (after - Vec3::ZERO).length();
        let full = (target_translation - Vec3::ZERO).length();
        assert!(
            traveled > 0.0,
            "interpolator should move visual toward target: after={after:?}"
        );
        assert!(
            traveled < full,
            "interpolator should not snap full distance in one frame: traveled={traveled} full={full}"
        );
    }
}
