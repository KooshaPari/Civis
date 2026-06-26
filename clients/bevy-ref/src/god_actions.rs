#![cfg(all(feature = "bevy", feature = "egui"))]
//! Client-side god-tool effects for the live attach window (P1.2.1).
//!
//! The god panel posts `sim.god_action` over JSON-RPC; when the server has not
//! yet applied the effect (or voxel deltas are slow), this module applies an
//! immediate visible preview on the streamed [`LiveStreamScene`] chunk cache and
//! civilian/faction snapshots so players see terrain scars, fire, and population
//! shifts without waiting on the wire.

use std::collections::HashSet;

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use civ_protocol_3d::FactionTreasury3d;
use civ_voxel::material::{AIR, FIRE, LAVA, STONE};
use civ_voxel::{ChunkId, MaterialId};

use crate::bevy_render::CHUNK_WIREFRAME_LINE_COLOR;
use crate::game_ui::GodActionToast;
use crate::god_panel::GodPanelState;
use crate::live_focus::LiveSceneFocus;
use crate::live_ground::{live_ground_y, ChunkVoxelCache};
use crate::live_stream::{
    remesh_cached_chunks, LiveStreamScene, StreamCulling, LIVE_CHUNK_EDGE,
};
use crate::frame_budget::{scaled_cull_distance, GpuQualityMode};
use crate::menus::in_game;
use crate::terrain::{terrain_surface_y, WORLD_SIZE};
use crate::{decode_chunk_id, encode_chunk_id, DebugRender};

/// Legacy god-panel verb fired from egui.
#[derive(Message, Debug, Clone)]
pub struct GodActionRequest {
    /// Verb id (`smite`, `bless`, `earthquake`, `plague`, `miracle`).
    pub action: String,
    /// Normalised world X in `[0, 1]`.
    pub norm_x: f32,
    /// Normalised world Z in `[0, 1]` (wire field `y`).
    pub norm_y: f32,
    /// Faction target for bless/plague.
    pub target_faction: u32,
    /// Effect strength in `[0, 1]`.
    pub magnitude: f32,
}

/// Short-lived emissive marker so non-terrain verbs still pop visually.
#[derive(Component)]
struct GodEffectFlash {
    elapsed: f32,
    lifetime: f32,
    base_scale: f32,
}

/// Wires god-action request handling into the Bevy app.
pub struct GodActionsPlugin;

impl Plugin for GodActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GodActionRequest>()
            .init_resource::<GodEffectMeshes>()
            .init_resource::<GodActionToast>()
            .add_systems(
                Update,
                (apply_god_action_requests, tick_god_effect_flashes)
                    .chain()
                    .run_if(in_game),
            )
            .add_systems(
                Update,
                crate::game_ui::tick_god_action_toast.run_if(in_game),
            )
            .add_systems(
                EguiPrimaryContextPass,
                crate::game_ui::draw_god_action_toast_system.run_if(in_game),
            );
    }
}

#[derive(Resource)]
struct GodEffectMeshes {
    sphere: Handle<Mesh>,
}

impl FromWorld for GodEffectMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            sphere: meshes.add(Mesh::from(Sphere::new(1.0))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerrainVerb {
    Meteor,
    Quake,
}

impl TerrainVerb {
    fn radius(self, magnitude: f32) -> f32 {
        let base = match self {
            TerrainVerb::Meteor => 7.0,
            TerrainVerb::Quake => 8.0,
        };
        base * (0.45 + magnitude.clamp(0.0, 1.0))
    }
}

fn norm_to_world(cache: &ChunkVoxelCache, nx: f32, nz: f32) -> Vec3 {
    let wx = nx * WORLD_SIZE - WORLD_SIZE * 0.5;
    let wz = nz * WORLD_SIZE - WORLD_SIZE * 0.5;
    let wy = live_ground_y(cache, wx, wz, 0.0);
    Vec3::new(wx, wy, wz)
}

fn seed_chunk_from_terrain(cache: &mut ChunkVoxelCache, chunk_id: ChunkId) {
    let (cx, cy, cz) = decode_chunk_id(chunk_id);
    let edge = LIVE_CHUNK_EDGE as i32;
    let half = WORLD_SIZE * 0.5;
    let mut voxels = vec![AIR; LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE];
    for lz in 0..LIVE_CHUNK_EDGE {
        for lx in 0..LIVE_CHUNK_EDGE {
            let wx = cx * edge + lx as i32;
            let wz = cz * edge + lz as i32;
            let surface = terrain_surface_y(wx as f32 + half, wz as f32 + half);
            for ly in 0..LIVE_CHUNK_EDGE {
                let wy = cy * edge + ly as i32;
                if (wy as f32) < surface {
                    let idx = lx + ly * LIVE_CHUNK_EDGE + lz * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE;
                    voxels[idx] = STONE;
                }
            }
        }
    }
    cache.insert(chunk_id, voxels);
}

fn ensure_chunk_ready(cache: &mut ChunkVoxelCache, chunk_id: ChunkId) {
    if cache.get_chunk(chunk_id).is_none() {
        seed_chunk_from_terrain(cache, chunk_id);
    }
}

fn voxel_index(ix: usize, iy: usize, iz: usize) -> usize {
    ix + iy * LIVE_CHUNK_EDGE + iz * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE
}

fn disaster_cell_material(
    verb: TerrainVerb,
    dx: i64,
    dy: i64,
    dist2: f32,
    r2: f32,
    current: MaterialId,
    above_air: bool,
) -> Option<MaterialId> {
    if dist2 > r2 {
        return None;
    }
    match verb {
        TerrainVerb::Meteor => {
            if dy == 0 && above_air && current != AIR && dist2 > r2 * 0.35 {
                return Some(FIRE);
            }
            if dy > -1 {
                Some(AIR)
            } else if dist2 > r2 * 0.5 {
                Some(LAVA)
            } else {
                Some(AIR)
            }
        }
        TerrainVerb::Quake => {
            if dy >= 0 && current != AIR {
                Some(AIR)
            } else {
                None
            }
        }
    }
}

fn apply_terrain_verb(
    cache: &mut ChunkVoxelCache,
    center: Vec3,
    verb: TerrainVerb,
    magnitude: f32,
) -> (usize, HashSet<ChunkId>) {
    let r = verb.radius(magnitude);
    let ri = r.ceil() as i64;
    let r2 = r * r;
    let (cx, cy, cz) = (
        center.x.round() as i64,
        center.y.round() as i64,
        center.z.round() as i64,
    );
    let mut changed = 0usize;
    let mut dirty = HashSet::new();
    let edge = LIVE_CHUNK_EDGE as i32;
    for dz in -ri..=ri {
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let dist2 = (dx * dx + dy * dy + dz * dz) as f32;
                let wx = cx + dx;
                let wy = cy + dy;
                let wz = cz + dz;
                if wx < 0 || wy < 0 || wz < 0 {
                    continue;
                }
                let chunk_id = encode_chunk_id(
                    wx.div_euclid(edge),
                    wy.div_euclid(edge),
                    wz.div_euclid(edge),
                );
                ensure_chunk_ready(cache, chunk_id);
                let lx = wx.rem_euclid(edge) as usize;
                let ly = wy.rem_euclid(edge) as usize;
                let lz = wz.rem_euclid(edge) as usize;
                let idx = voxel_index(lx, ly, lz);
                let voxels = cache.ensure_chunk(chunk_id);
                let current = voxels[idx];
                let above_air = ly + 1 < LIVE_CHUNK_EDGE
                    && voxels[voxel_index(lx, ly + 1, lz)] == AIR;
                let Some(mat) = disaster_cell_material(verb, dx, dy, dist2, r2, current, above_air)
                else {
                    continue;
                };
                if voxels[idx] != mat {
                    voxels[idx] = mat;
                    changed += 1;
                    dirty.insert(chunk_id);
                }
            }
        }
    }
    (changed, dirty)
}

fn apply_bless(scene: &mut LiveStreamScene, faction: u32, magnitude: f32) -> String {
    let boost = (magnitude.max(0.1) * 1000.0) as f64;
    let mut healed = 0usize;
    for entry in scene.civilian_entries.values_mut() {
        if entry.faction_id == faction {
            entry.health = (entry.health + 0.25 * magnitude).min(1.0);
            healed += 1;
        }
    }
    if let Some(faction_entry) = scene.faction_entries.iter_mut().find(|f| f.id == faction) {
        faction_entry.treasury.amount += boost;
    } else {
        scene.faction_entries.push(civ_protocol_3d::FactionStateEntry {
            id: faction,
            era: 0,
            government: civ_protocol_3d::Government3d::Republic,
            treasury: FactionTreasury3d { amount: boost, ..Default::default() },
        });
        scene.factions.insert(faction);
    }
    format!(
        "Bless: faction {faction} +{boost:.0} treasury, healed {healed} civilians"
    )
}

fn apply_plague(scene: &mut LiveStreamScene, faction: u32, magnitude: f32) -> String {
    let debit = (magnitude.max(0.1) * 500.0) as f64;
    let mut sickened = 0usize;
    for entry in scene.civilian_entries.values_mut() {
        if entry.faction_id == faction {
            entry.health = (entry.health - 0.35 * magnitude).max(0.05);
            sickened += 1;
        }
    }
    if let Some(faction_entry) = scene.faction_entries.iter_mut().find(|f| f.id == faction) {
        faction_entry.treasury.amount = (faction_entry.treasury.amount - debit).max(0.0);
    }
    format!(
        "Plague: faction {faction} -{debit:.0} treasury, sickened {sickened} civilians"
    )
}

fn apply_miracle(scene: &mut LiveStreamScene, magnitude: f32) -> String {
    let boost = (magnitude.max(0.1) * 250.0) as f64;
    let mut healed = 0usize;
    for entry in scene.civilian_entries.values_mut() {
        entry.health = (entry.health + 0.15 * magnitude).min(1.0);
        healed += 1;
    }
    for faction_entry in &mut scene.faction_entries {
        faction_entry.treasury.amount += boost;
    }
    format!("Miracle: all factions +{boost:.0} treasury, healed {healed} civilians")
}

fn spawn_effect_flash(
    commands: &mut Commands,
    meshes: &GodEffectMeshes,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
    color: Color,
    scale: f32,
) {
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.into(),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.sphere.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(center).with_scale(Vec3::splat(scale)),
        GodEffectFlash {
            elapsed: 0.0,
            lifetime: 1.8,
            base_scale: scale,
        },
    ));
}

fn apply_god_action_requests(
    mut commands: Commands,
    mut requests: MessageReader<GodActionRequest>,
    mut scene: ResMut<LiveStreamScene>,
    mut panel: ResMut<GodPanelState>,
    mut toast: Option<ResMut<GodActionToast>>,
    focus: Res<LiveSceneFocus>,
    debug: Res<DebugRender>,
    effect_meshes: Res<GodEffectMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gpu_quality: Option<Res<GpuQualityMode>>,
) {
    for req in requests.read() {
        let center = norm_to_world(&scene.chunk_voxels, req.norm_x, req.norm_y);
        let mag = req.magnitude.clamp(0.0, 1.0);
        let status = match req.action.as_str() {
            "smite" => {
                let (cells, dirty) = apply_terrain_verb(
                    &mut scene.chunk_voxels,
                    center,
                    TerrainVerb::Meteor,
                    mag,
                );
                remesh_dirty_chunks(
                    &mut commands,
                    &mut scene,
                    &focus,
                    &debug,
                    &effect_meshes,
                    &mut meshes,
                    &mut materials,
                    &dirty,
                    gpu_quality.as_deref().copied().unwrap_or_default(),
                );
                spawn_effect_flash(
                    &mut commands,
                    &effect_meshes,
                    &mut materials,
                    center + Vec3::Y * 2.0,
                    Color::srgb(1.0, 0.35, 0.1),
                    4.0 + mag * 6.0,
                );
                format!("Smite: meteor scar at ({:.2},{:.2}) — {cells} voxels", req.norm_x, req.norm_y)
            }
            "earthquake" => {
                let (cells, dirty) = apply_terrain_verb(
                    &mut scene.chunk_voxels,
                    center,
                    TerrainVerb::Quake,
                    mag,
                );
                remesh_dirty_chunks(
                    &mut commands,
                    &mut scene,
                    &focus,
                    &debug,
                    &effect_meshes,
                    &mut meshes,
                    &mut materials,
                    &dirty,
                    gpu_quality.as_deref().copied().unwrap_or_default(),
                );
                spawn_effect_flash(
                    &mut commands,
                    &effect_meshes,
                    &mut materials,
                    center,
                    Color::srgb(0.55, 0.45, 0.35),
                    6.0 + mag * 8.0,
                );
                format!(
                    "Earthquake: rubble at ({:.2},{:.2}) — {cells} voxels collapsed",
                    req.norm_x, req.norm_y
                )
            }
            "bless" => {
                let msg = apply_bless(&mut scene, req.target_faction, mag);
                spawn_effect_flash(
                    &mut commands,
                    &effect_meshes,
                    &mut materials,
                    center + Vec3::Y * 3.0,
                    Color::srgb(0.4, 0.95, 0.55),
                    5.0,
                );
                msg
            }
            "plague" => {
                let msg = apply_plague(&mut scene, req.target_faction, mag);
                spawn_effect_flash(
                    &mut commands,
                    &effect_meshes,
                    &mut materials,
                    center + Vec3::Y * 2.0,
                    Color::srgb(0.45, 0.2, 0.65),
                    5.0,
                );
                msg
            }
            "miracle" => {
                let msg = apply_miracle(&mut scene, mag);
                spawn_effect_flash(
                    &mut commands,
                    &effect_meshes,
                    &mut materials,
                    Vec3::new(0.0, 12.0, 0.0),
                    Color::srgb(1.0, 0.92, 0.45),
                    14.0,
                );
                msg
            }
            other => format!("Unknown god action: {other}"),
        };
        panel.status = Some(status.clone());
        if let Some(toast) = toast.as_mut() {
            toast.show(status);
        }
    }
}

fn remesh_dirty_chunks(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    focus: &LiveSceneFocus,
    debug: &DebugRender,
    _effect_meshes: &GodEffectMeshes,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    dirty: &HashSet<ChunkId>,
    gpu_quality: GpuQualityMode,
) {
    if dirty.is_empty() {
        return;
    }
    let base_distance = focus.half_extent * 4.0 + 256.0;
    let culling = StreamCulling {
        eye: [focus.centre.x, 64.0, focus.centre.z],
        max_distance: scaled_cull_distance(base_distance, gpu_quality),
        gpu_quality,
    };
    let wire = debug.wireframe.then_some(CHUNK_WIREFRAME_LINE_COLOR);
    let ids: Vec<ChunkId> = dirty.iter().copied().collect();
    remesh_cached_chunks(
        commands,
        scene,
        meshes,
        materials,
        culling,
        debug,
        &ids,
        wire,
    );
}

fn tick_god_effect_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GodEffectFlash, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mat_handles: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut transform) in &mut query {
        flash.elapsed += dt;
        if flash.elapsed >= flash.lifetime {
            commands.entity(entity).despawn();
            continue;
        }
        let t = flash.elapsed / flash.lifetime;
        let pulse = 1.0 + t * 0.6;
        transform.scale = Vec3::splat(flash.base_scale * pulse);
        if let Ok(handle) = mat_handles.get(entity) {
            if let Some(mat) = materials.get_mut(&handle.0) {
                let alpha = (1.0 - t).max(0.0);
                mat.base_color = mat.base_color.with_alpha(alpha * 0.85);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smite_mutates_chunk_cache() {
        let mut cache = ChunkVoxelCache::new();
        let chunk = encode_chunk_id(0, 0, 0);
        seed_chunk_from_terrain(&mut cache, chunk);
        let center = Vec3::new(8.0, 10.0, 8.0);
        let (changed, dirty) = apply_terrain_verb(&mut cache, center, TerrainVerb::Meteor, 0.8);
        assert!(changed > 0, "smite should modify voxels");
        assert!(!dirty.is_empty(), "smite should mark chunks dirty");
    }
}
