//! In-process simulation bridge for the standalone Bevy client.

use bevy::prelude::*;
use std::collections::HashMap;
use civ_agents::{
    infer_alignment_for_spawn, spawn_civilian_at, ActorVisual, ActorVisualKind, Alignment, Civilian,
};
use civ_engine::{
    spawn::{spawn_airport_at, spawn_hangar_at, spawn_port_at},
    Building, BuildingType, Simulation,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::spawn_tools::{SpawnBuildingRequest, SpawnCivilianRequest};
use crate::terrain::WORLD_SIZE;
use crate::{live_attach::is_server_attach_mode, AttachMode};
#[cfg(feature = "models")]
use civ_bevy_ref::gltf_models::{actor_scene, building_scene_for, ModelOrPrimitive};
#[cfg(feature = "models")]
type ModelResourceRef<'a> = Option<&'a Res<civ_bevy_ref::gltf_models::GameModels>>;
#[cfg(not(feature = "models"))]
type ModelResourceRef<'a> = Option<()>;

/// Live simulation state shared by the minimap, HUD, and spawn tools.
#[derive(Resource)]
pub struct SimState(pub Simulation);

#[derive(Component)]
struct SimCivilianMarker {
    id: u64,
    faction: u32,
    visual: ActorVisualKind,
}

#[derive(Component)]
struct SimBuildingMarker {
    building_type: BuildingType,
    position: civ_engine::Position,
}

impl Default for SimState {
    fn default() -> Self {
        Self(Simulation::default())
    }
}

/// In-process simulation tick interval for the standalone client.
const SIM_TICK_SECONDS: f32 = 0.1;

#[derive(Resource)]
struct SimTickAccumulator(f32);

#[derive(Resource)]
struct GameplayMarkerMeshes {
    civilian: Handle<Mesh>,
    building: Handle<Mesh>,
}

fn in_process_sim_active(mode: Res<AttachMode>) -> bool {
    !is_server_attach_mode(*mode)
}

/// Wires spawn-tool messages into the ECS simulation and optional HUD sync.
#[derive(Default)]
pub struct SimBridgePlugin;

impl Plugin for SimBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimState>()
            .insert_resource(SimTickAccumulator(0.0))
            .add_systems(Startup, setup_gameplay_marker_meshes)
            .add_systems(
                Update,
                (
                    advance_simulation.run_if(in_process_sim_active),
                    apply_spawn_civilian_requests.run_if(in_process_sim_active),
                    apply_spawn_building_requests.run_if(in_process_sim_active),
                ),
            );
        #[cfg(feature = "egui")]
        app.add_systems(Update, sync_game_ui_snapshot.run_if(in_process_sim_active));
        app.add_systems(Update, sync_visible_gameplay.run_if(in_process_sim_active));
    }
}

fn advance_simulation(
    time: Res<Time>,
    mut accumulator: ResMut<SimTickAccumulator>,
    mut sim: ResMut<SimState>,
    #[cfg(feature = "egui")] mode: Res<crate::menus::GameUiMode>,
    #[cfg(feature = "egui")] speed: Res<crate::game_ui::GameSpeed>,
    #[cfg(feature = "voxel")] voxel_state: Option<Res<VoxelSimState>>,
) {
    #[cfg(feature = "egui")]
    if *mode == crate::menus::GameUiMode::Paused || speed.multiplier <= 0.0 {
        return;
    }
    let factor = {
        #[cfg(feature = "egui")]
        {
            speed.factor()
        }
        #[cfg(not(feature = "egui"))]
        {
            1.0
        }
    };
    accumulator.0 += time.delta_secs() * factor;
    while accumulator.0 >= SIM_TICK_SECONDS {
        accumulator.0 -= SIM_TICK_SECONDS;
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

fn world_to_norm(position: Vec3) -> (f32, f32) {
    let wx = position.x + WORLD_SIZE * 0.5;
    let wz = position.z + WORLD_SIZE * 0.5;
    (
        (wx / WORLD_SIZE).clamp(0.0, 1.0),
        (wz / WORLD_SIZE).clamp(0.0, 1.0),
    )
}

fn apply_spawn_civilian_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnCivilianRequest>,
) {
    for request in requests.read() {
        let (nx, ny) = world_to_norm(request.position);
        let id = next_civilian_id(&sim.0);
        let seed = id.wrapping_add(nx.to_bits() as u64 ^ ny.to_bits() as u64);
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let alignment = infer_alignment_for_spawn(&sim.0.world, nx, ny);
        spawn_civilian_at(
            &mut sim.0.world,
            id,
            alignment,
            nx,
            ny,
            ActorVisualKind::Humanoid,
            &mut rng,
        );
    }
}

fn apply_spawn_building_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<SpawnBuildingRequest>,
) {
    for request in requests.read() {
        let (nx, ny) = world_to_norm(request.position);
        match request.kind {
            crate::spawn_tools::BuildingSpawnKind::CityCenter => {
                spawn_airport_at(&mut sim.0.world, nx, ny);
            }
            crate::spawn_tools::BuildingSpawnKind::Market => {
                spawn_port_at(&mut sim.0.world, nx, ny);
            }
            crate::spawn_tools::BuildingSpawnKind::Barracks => {
                spawn_hangar_at(&mut sim.0.world, nx, ny);
            }
        }
    }
}

fn setup_gameplay_marker_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let civilian = meshes.add(Mesh::from(bevy::math::primitives::Capsule3d::new(0.45, 1.1)));
    let building = meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(2.0, 2.5, 2.0)));
    commands.insert_resource(GameplayMarkerMeshes { civilian, building });
}

fn sync_visible_gameplay(
    mut commands: Commands,
    sim: Res<SimState>,
    existing_civilians: Query<(Entity, &SimCivilianMarker)>,
    existing_buildings: Query<(Entity, &SimBuildingMarker)>,
    marker_meshes: Res<GameplayMarkerMeshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "models")]
    models: Option<Res<civ_bevy_ref::gltf_models::GameModels>>,
) {
    if !sim.is_changed() {
        return;
    }

    let mut civilian_entities: HashMap<u64, (Entity, u32, ActorVisualKind)> = existing_civilians
        .iter()
        .map(|(entity, marker)| (marker.id, (entity, marker.faction, marker.visual)))
        .collect();
    let mut building_entities: HashMap<(i32, i32, BuildingType), Entity> = existing_buildings
        .iter()
        .map(|(entity, marker)| ((marker.position.x, marker.position.y, marker.building_type), entity))
        .collect();

    #[cfg(feature = "models")]
    let model_resource = models.as_ref();
    #[cfg(not(feature = "models"))]
    let model_resource: ModelResourceRef = None;

    for (_, (civilian, position, actor_visual)) in sim
        .0
        .world
        .query::<(&Civilian, &civ_agents::Position3d, Option<&ActorVisual>)>()
        .iter()
    {
        let world_pos = sim_position_to_world(position);
        let faction_id = faction_id(&civilian.alignment);
        let visual = actor_visual_kind(actor_visual);
        let entity = match civilian_entities.remove(&civilian.id) {
            Some((entity, old_faction, old_visual))
                if old_faction == faction_id && old_visual == visual =>
            {
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(
                        world_pos
                            + if matches!(visual, ActorVisualKind::Humanoid) {
                                Vec3::Y * 0.8
                            } else {
                                Vec3::Y * 0.25
                            },
                    ));
                entity
            }
            Some((entity, _, _)) => {
                commands.entity(entity).despawn();
                spawn_civilian_visual(
                    &mut commands,
                    model_resource,
                    &marker_meshes.civilian,
                    &mut materials,
                    civilian.id,
                    faction_id,
                    visual,
                    &world_pos,
                )
            }
            None => spawn_civilian_visual(
                &mut commands,
                model_resource,
                &marker_meshes.civilian,
                &mut materials,
                civilian.id,
                faction_id,
                visual,
                &world_pos,
            ),
        };
        let _ = entity;
    }

    for (_, building) in sim.0.world.query::<&Building>().iter() {
        let world_pos = building_world_position(building);
        let key = (building.position.x, building.position.y, building.building_type);
        let entity = match building_entities.remove(&key) {
            Some(entity) => {
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(world_pos + Vec3::Y * 1.25));
                entity
            }
            None => spawn_building_visual(
                &mut commands,
                model_resource,
                &marker_meshes.building,
                &mut materials,
                building.building_type,
                building.position,
                &world_pos,
            ),
        };
        let _ = entity;
    }

    for entity in civilian_entities.into_values().map(|(entity, _, _)| entity) {
        commands.entity(entity).despawn();
    }
    for entity in building_entities.into_values() {
        commands.entity(entity).despawn();
    }
}

fn sim_position_to_world(position: &civ_agents::Position3d) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    let half = WORLD_SIZE * 0.5;
    Vec3::new(
        position.coord.x as f32 / scale * WORLD_SIZE - half,
        0.0,
        position.coord.z as f32 / scale * WORLD_SIZE - half,
    )
}

fn building_world_position(building: &Building) -> Vec3 {
    let half = WORLD_SIZE * 0.5;
    Vec3::new(
        ((building.position.x as f32 + 64.0) / 127.0).clamp(0.0, 1.0) * WORLD_SIZE - half,
        0.0,
        ((building.position.y as f32 + 64.0) / 127.0).clamp(0.0, 1.0) * WORLD_SIZE - half,
    )
}

fn faction_color(alignment: &Alignment) -> Color {
    let hue = (faction_id(alignment) as f32 * 85.0) % 360.0;
    Color::hsla(hue, 0.75, 0.55, 1.0)
}

fn faction_id(alignment: &Alignment) -> u32 {
    match alignment {
        Alignment::Faction(faction) => *faction,
        Alignment::OtherEntity(_) | Alignment::None => 0,
    }
}

fn actor_visual_kind(actor_visual: Option<&ActorVisual>) -> ActorVisualKind {
    actor_visual.map(|actor| actor.0).unwrap_or(ActorVisualKind::Humanoid)
}

#[cfg(feature = "models")]
fn spawn_civilian_visual(
    commands: &mut Commands<'_>,
    models: ModelResourceRef,
    civilian_mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    civilian_id: u64,
    faction: u32,
    visual: ActorVisualKind,
    world_pos: &Vec3,
) -> Entity {
    let Some(models) = models else {
        let color = faction_color(&Alignment::with_faction(faction));
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: color.into(),
            perceptual_roughness: 0.55,
            ..default()
        });
        return commands
            .spawn((
                SimCivilianMarker {
                    id: civilian_id,
                    faction,
                    visual,
                },
                Mesh3d(civilian_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(*world_pos + Vec3::Y * 0.8),
            ))
            .id();
    };
    match actor_scene(models, visual, faction) {
        ModelOrPrimitive::Model(scene_root) => {
            commands
                .spawn((
                    SimCivilianMarker {
                        id: civilian_id,
                        faction,
                        visual,
                    },
                    scene_root,
                    Transform::from_translation(*world_pos + Vec3::Y * 0.25),
                ))
                .id()
        }
        ModelOrPrimitive::Primitive => {
            let color = faction_color(&Alignment::with_faction(faction));
            let material = materials.add(StandardMaterial {
                base_color: color,
                emissive: color.into(),
                perceptual_roughness: 0.55,
                ..default()
            });
            commands
                .spawn((
                    SimCivilianMarker {
                        id: civilian_id,
                        faction,
                        visual,
                    },
                    Mesh3d(civilian_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(*world_pos + Vec3::Y * 0.8),
                ))
                .id()
        }
    }
}

#[cfg(feature = "models")]
fn spawn_building_visual(
    commands: &mut Commands<'_>,
    models: ModelResourceRef,
    building_mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    building_type: BuildingType,
    position: civ_engine::Position,
    world_pos: &Vec3,
) -> Entity {
    let Some(models) = models else {
        return spawn_building_primitive(
            commands,
            building_mesh,
            materials,
            building_type,
            world_pos,
            position,
        );
    };
    match building_scene_for(models, building_type) {
        ModelOrPrimitive::Model(scene_root) => {
            commands
                .spawn((
                    SimBuildingMarker {
                        building_type,
                        position,
                    },
                    scene_root,
                    Transform::from_translation(*world_pos),
                ))
                .id()
        }
        ModelOrPrimitive::Primitive => {
            spawn_building_primitive(
                commands,
                building_mesh,
                materials,
                building_type,
                world_pos,
                position,
            )
        }
    }
}

#[cfg(not(feature = "models"))]
fn spawn_civilian_visual(
    commands: &mut Commands<'_, '_>,
    _models: ModelResourceRef,
    civilian_mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    civilian_id: u64,
    faction: u32,
    visual: ActorVisualKind,
    world_pos: &Vec3,
) -> Entity {
    let color = faction_color(&Alignment::with_faction(faction));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        perceptual_roughness: 0.55,
        ..default()
    });
    commands
        .spawn((
            SimCivilianMarker {
                id: civilian_id,
                faction,
                visual,
            },
            Mesh3d(civilian_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(*world_pos + Vec3::Y * 0.8),
        ))
        .id()
}

#[cfg(not(feature = "models"))]
fn spawn_building_visual(
    commands: &mut Commands<'_, '_>,
    _models: ModelResourceRef,
    building_mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    building_type: BuildingType,
    position: civ_engine::Position,
    world_pos: &Vec3,
) -> Entity {
    spawn_building_primitive(commands, building_mesh, materials, building_type, world_pos, position)
}

fn spawn_building_primitive(
    commands: &mut Commands<'_, '_>,
    building_mesh: &Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    building_type: BuildingType,
    world_pos: &Vec3,
    position: civ_engine::Position,
) -> Entity {
    let color = building_color(building_type);
    let material = materials.add(StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        ..default()
    });
    commands
        .spawn((
            SimBuildingMarker {
                building_type,
                position,
            },
            Mesh3d(building_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(*world_pos + Vec3::Y * 1.25),
        ))
        .id()
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

#[cfg(feature = "egui")]
fn sync_game_ui_snapshot(
    sim: Res<SimState>,
    speed: Res<crate::game_ui::GameSpeed>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
) {
    if !sim.is_changed() && !speed.is_changed() {
        return;
    }

    let population = sim.0.world.query::<&Civilian>().iter().count() as u64;
    let mut factions = std::collections::HashSet::new();
    for (_, civilian) in sim.0.world.query::<&Civilian>().iter() {
        let Some(faction) = (match civilian.alignment {
            Alignment::Faction(faction) => Some(faction),
            _ => None,
        }) else {
            continue;
        };
        factions.insert(faction);
    }
    let factions = factions.len() as u32;
    let tick = sim.0.state.tick;
    snapshot.set_sim_state(tick, population, factions, tick.to_string(), speed.multiplier);
}
