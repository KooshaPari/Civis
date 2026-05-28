use std::collections::HashMap;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use civ_bevy_ref::{
    agent_color_from_id, agent_label_stub, agent_scale_multiplier,
    bevy_render::{
        apply_chunk_material, mesh_buffer_to_bevy, spawn_default_scene, CHUNK_WIREFRAME_LINE_COLOR,
    },
    chunk_distance_from_camera, chunk_fade_complete, chunk_raycast_stub, chunk_to_minimap_uv,
    decode_chunk_id, focused_chunk_at_grid,
    gpu_features::GpuFeaturesPlugin,
    mesh_lod_level, minimap_uv_to_chunk_grid,
    native_backend::native_render_plugin,
    presentation_ambient_brightness, presentation_ambient_color_rgb, presentation_clear_color_rgb,
    presentation_day_factor_target, resolve_live_ws_url, should_render_chunk,
    ws_client::{WsClient, WsClientConfig},
    CameraTarget, CubicMesher, DebugRender, LiveHudSnapshot, MinimapBounds, AGENT_MARKER_DEPTH,
    AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH, VOXEL_CHUNK_EDGE,
};
use civ_protocol_3d::{agent_world_translation, AgentAppearanceFrame, Frame3d, VoxelDeltaFrame};
use civ_voxel::{ChunkId, ChunkView, LodLevel};

const CHUNK_EDGE: usize = 16;
const CHUNK_BASE_COLOR: [f32; 3] = [0.72, 0.69, 0.62];
const ORBIT_DRAG_SENSITIVITY: f32 = 0.005;
const ORBIT_SCROLL_SENSITIVITY: f32 = 2.0;
const ORBIT_KEYBOARD_DISTANCE_STEP: f32 = 4.0;
const ORBIT_PAN_SPEED: f32 = 12.0;
const MIN_ORBIT_ELEVATION: f32 = 0.05;
const MIN_ORBIT_DISTANCE: f32 = 8.0;
const MAX_ORBIT_DISTANCE: f32 = 200.0;
/// Small world-space id labels above agent markers (`Text2d` child entities).
const AGENT_NAME_LABELS: bool = true;
const AGENT_LABEL_FONT_SIZE: f32 = 10.0;
const AGENT_LABEL_Y_OFFSET: f32 = 1.05;
const MINIMAP_SIZE: f32 = 160.0;
const MINIMAP_DOT: f32 = 4.0;
const MINIMAP_INSET: f32 = 6.0;

/// Live orbit state derived from [`CameraTarget`]; updated by mouse drag and scroll.
#[derive(Resource, Debug, Clone, Copy)]
struct OrbitCamera {
    centre: [f32; 3],
    azimuth: f32,
    elevation: f32,
    distance: f32,
}

impl OrbitCamera {
    fn from_target(target: CameraTarget) -> Self {
        Self {
            centre: target.centre,
            azimuth: target.azimuth_rad,
            elevation: target.elevation_rad,
            distance: target.distance,
        }
    }

    fn as_target(&self) -> CameraTarget {
        CameraTarget {
            centre: self.centre,
            distance: self.distance,
            azimuth_rad: self.azimuth,
            elevation_rad: self.elevation,
        }
    }

    fn reset(&mut self) {
        *self = Self::from_target(CameraTarget::default());
    }

    fn adjust_distance(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(MIN_ORBIT_DISTANCE, MAX_ORBIT_DISTANCE);
    }

    /// Stub: pan orbit centre on the horizontal plane relative to current azimuth.
    fn pan_centre(&mut self, right: f32, forward: f32) {
        let sin = self.azimuth.sin();
        let cos = self.azimuth.cos();
        self.centre[0] += right * cos + forward * sin;
        self.centre[2] += -right * sin + forward * cos;
    }
}

#[derive(Resource)]
struct LiveBridge {
    client: WsClient,
}

#[derive(Resource, Default)]
struct LiveScene {
    chunks: HashMap<u64, Entity>,
    agents: HashMap<u64, Entity>,
    agent_materials: HashMap<u64, Handle<StandardMaterial>>,
}

#[derive(Resource)]
struct HudState {
    snapshot: LiveHudSnapshot,
    text: Entity,
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct MinimapPanel;

#[derive(Component)]
struct MinimapDots;

#[derive(Resource)]
struct MinimapUi {
    dots: Entity,
}

/// L5 presentation: day/night from `sim.snapshot` with smooth lighting ramp.
#[derive(Resource)]
struct ScenePresentation {
    is_day: bool,
    day_factor: f32,
}

impl Default for ScenePresentation {
    fn default() -> Self {
        Self {
            is_day: true,
            day_factor: 1.0,
        }
    }
}

#[derive(Resource, Default)]
struct MinimapCache {
    chunk_keys: Vec<u64>,
    bounds: Option<MinimapBounds>,
}

#[derive(Component)]
#[allow(dead_code)]
struct ChunkTag {
    id: ChunkId,
}

#[derive(Component)]
struct ChunkFade {
    elapsed: f32,
    base_rgb: [f32; 3],
}

impl ChunkFade {
    fn new() -> Self {
        Self {
            elapsed: 0.0,
            base_rgb: CHUNK_BASE_COLOR,
        }
    }
}

#[derive(Component)]
#[allow(dead_code)]
struct AgentTag {
    id: u64,
}

#[derive(Component)]
struct AgentLabel;

#[derive(Resource)]
struct AgentVisualAssets {
    mesh: Handle<Mesh>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Civis 3D — Bevy reference (live)".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(native_render_plugin()),
            WireframePlugin::default(),
            GpuFeaturesPlugin,
        ))
        .insert_resource(LiveScene::default())
        .insert_resource(ScenePresentation::default())
        .insert_resource(DebugRender::default())
        .insert_resource(OrbitCamera::from_target(CameraTarget::default()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                debug_render_input,
                orbit_camera_input,
                minimap_click_focus,
                viewport_chunk_raycast,
                update_orbit_camera_transform,
                apply_live_frames,
                apply_spectator_meta,
                sync_chunk_debug_render,
                update_chunk_fade,
                update_hud,
                update_minimap,
                update_presentation_lighting,
            ),
        )
        .run();
}

fn apply_spectator_meta(
    bridge: Res<LiveBridge>,
    mut presentation: ResMut<ScenePresentation>,
    mut hud: ResMut<HudState>,
) {
    for meta in bridge.client.poll_meta() {
        presentation.is_day = meta.is_day;
        if let Some(tick) = meta.tick {
            hud.snapshot.tick = Some(tick);
            hud.snapshot.connected = true;
        }
    }
}

/// L5 slice: day/night from `sim.snapshot` (`is_day`) on sun, ambient fill, and sky clear colour.
fn update_presentation_lighting(
    time: Res<Time>,
    mut presentation: ResMut<ScenePresentation>,
    mut lights: Query<&mut DirectionalLight>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: ResMut<ClearColor>,
) {
    let target = presentation_day_factor_target(presentation.is_day);
    let step = (time.delta_secs() * 2.5).clamp(0.0, 1.0);
    presentation.day_factor += (target - presentation.day_factor) * step;

    let day_factor = presentation.day_factor;
    for mut light in &mut lights {
        light.illuminance = 12_000.0 * day_factor;
    }

    let ambient_rgb = presentation_ambient_color_rgb(day_factor);
    ambient.color = Color::srgb(ambient_rgb[0], ambient_rgb[1], ambient_rgb[2]);
    ambient.brightness = presentation_ambient_brightness(day_factor);

    let clear_rgb = presentation_clear_color_rgb(day_factor);
    clear.0 = Color::srgb(clear_rgb[0], clear_rgb[1], clear_rgb[2]);
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    spawn_default_scene(&mut commands);
    commands.insert_resource(AgentVisualAssets {
        mesh: meshes.add(Cuboid::new(
            AGENT_MARKER_WIDTH,
            AGENT_MARKER_HEIGHT,
            AGENT_MARKER_DEPTH,
        )),
    });
    commands.insert_resource(LiveBridge {
        client: WsClient::spawn_with_config(resolve_live_ws_url(), WsClientConfig::default()),
    });

    let text = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            Text::new(LiveHudSnapshot::default().format_overlay()),
            TextFont::from_font_size(16.0),
            TextColor(Color::srgb(0.9, 0.92, 0.95)),
            HudText,
        ))
        .id();
    commands.insert_resource(HudState {
        snapshot: LiveHudSnapshot::default(),
        text,
    });

    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                width: Val::Px(MINIMAP_SIZE),
                height: Val::Px(MINIMAP_SIZE),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.06, 0.11, 0.88)),
            BorderColor::all(Color::srgba(0.35, 0.42, 0.52, 0.65)),
            MinimapPanel,
            Interaction::default(),
            RelativeCursorPosition::default(),
        ))
        .id();

    let dots = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                ..default()
            },
            MinimapDots,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(panel).add_child(dots);
    commands.insert_resource(MinimapUi { dots });
    commands.insert_resource(MinimapCache::default());
}

fn debug_render_input(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugRender>) {
    if keys.just_pressed(KeyCode::F3) {
        debug.toggle_wireframe();
    }
}

fn sync_chunk_debug_render(
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&ChunkFade>,
        ),
        With<ChunkTag>,
    >,
) {
    if !debug.is_changed() {
        return;
    }

    for (entity, material, fade) in &chunks {
        if debug.wireframe {
            commands.entity(entity).insert((
                Wireframe,
                WireframeColor {
                    color: CHUNK_WIREFRAME_LINE_COLOR,
                },
            ));
        } else {
            commands
                .entity(entity)
                .remove::<Wireframe>()
                .remove::<WireframeColor>();
        }

        if let Some(material) = materials.get_mut(&material.0) {
            let fade_elapsed = fade.map(|state| state.elapsed);
            apply_chunk_material(material, CHUNK_BASE_COLOR, debug.wireframe, fade_elapsed);
        }
    }
}

fn apply_live_frames(
    mut commands: Commands,
    bridge: Res<LiveBridge>,
    mut scene: ResMut<LiveScene>,
    mut hud: ResMut<HudState>,
    orbit: Res<OrbitCamera>,
    debug: Res<DebugRender>,
    assets: Res<AgentVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let frames = bridge.client.poll();
    if !frames.is_empty() {
        hud.snapshot.connected = true;
    }

    for frame in frames {
        hud.snapshot.tick = Some(frame.tick());
        match frame {
            Frame3d::VoxelDelta(delta) => apply_voxel_delta(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                &orbit,
                debug.as_ref(),
                delta,
            ),
            Frame3d::AgentAppearance(agents) => {
                apply_agent_appearance(&mut commands, &mut scene, &mut materials, &assets, agents)
            }
            Frame3d::BuildingDiff(_) => {}
        }
    }
}

fn orbit_camera_input(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
    minimap: Query<&Interaction, With<MinimapPanel>>,
) {
    let minimap_active = minimap
        .single()
        .map(|interaction| *interaction != Interaction::None)
        .unwrap_or(false);

    if mouse_buttons.pressed(MouseButton::Left) && !minimap_active {
        for event in motion_events.read() {
            orbit.azimuth -= event.delta.x * ORBIT_DRAG_SENSITIVITY;
            orbit.elevation = (orbit.elevation - event.delta.y * ORBIT_DRAG_SENSITIVITY).clamp(
                MIN_ORBIT_ELEVATION,
                std::f32::consts::FRAC_PI_2 - MIN_ORBIT_ELEVATION,
            );
        }
    } else {
        motion_events.clear();
    }

    for event in scroll_events.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.05,
        };
        orbit.adjust_distance(-scroll * ORBIT_SCROLL_SENSITIVITY);
    }

    if keys.just_pressed(KeyCode::KeyR) {
        orbit.reset();
    }

    let zoom_in = keys.just_pressed(KeyCode::Equal)
        || keys.just_pressed(KeyCode::NumpadAdd)
        || keys.just_pressed(KeyCode::BracketLeft);
    let zoom_out = keys.just_pressed(KeyCode::Minus)
        || keys.just_pressed(KeyCode::NumpadSubtract)
        || keys.just_pressed(KeyCode::BracketRight);
    if zoom_in {
        orbit.adjust_distance(-ORBIT_KEYBOARD_DISTANCE_STEP);
    }
    if zoom_out {
        orbit.adjust_distance(ORBIT_KEYBOARD_DISTANCE_STEP);
    }

    let pan = ORBIT_PAN_SPEED * time.delta_secs();
    let mut right = 0.0;
    let mut forward = 0.0;
    if keys.pressed(KeyCode::KeyW) {
        forward += pan;
    }
    if keys.pressed(KeyCode::KeyS) {
        forward -= pan;
    }
    if keys.pressed(KeyCode::KeyA) {
        right -= pan;
    }
    if keys.pressed(KeyCode::KeyD) {
        right += pan;
    }
    if right != 0.0 || forward != 0.0 {
        orbit.pan_centre(right, forward);
    }
}

fn update_orbit_camera_transform(
    orbit: Res<OrbitCamera>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let target = orbit.as_target();
    let eye = target.orbit_position();
    let centre = Vec3::from_array(target.centre);

    for mut transform in &mut cameras {
        *transform = Transform::from_xyz(eye[0], eye[1], eye[2]).looking_at(centre, Vec3::Y);
    }
}

fn update_hud(
    time: Res<Time>,
    mut hud: ResMut<HudState>,
    mut text: Query<&mut Text, With<HudText>>,
) {
    let fps = 1.0 / time.delta_secs();
    hud.snapshot.fps = if hud.snapshot.fps <= 0.0 {
        fps
    } else {
        hud.snapshot.fps * 0.9 + fps * 0.1
    };

    let Ok(mut text) = text.get_mut(hud.text) else {
        return;
    };
    *text = Text::new(hud.snapshot.format_overlay());
}

fn minimap_bounds_from_keys(chunk_keys: &[u64]) -> Option<MinimapBounds> {
    let mut min_x = i32::MAX;
    let mut min_z = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_z = i32::MIN;
    for &raw in chunk_keys {
        let (cx, _cy, cz) = decode_chunk_id(ChunkId(raw));
        min_x = min_x.min(cx);
        min_z = min_z.min(cz);
        max_x = max_x.max(cx);
        max_z = max_z.max(cz);
    }
    if min_x == i32::MAX {
        None
    } else {
        Some((min_x, min_z, max_x, max_z))
    }
}

fn update_minimap(
    mut commands: Commands,
    scene: Res<LiveScene>,
    minimap: Res<MinimapUi>,
    orbit: Res<OrbitCamera>,
    hud: Res<HudState>,
    mut cache: ResMut<MinimapCache>,
    children: Query<&Children>,
) {
    let mut keys: Vec<u64> = scene.chunks.keys().copied().collect();
    keys.sort_unstable();
    if keys == cache.chunk_keys {
        return;
    }
    cache.chunk_keys = keys.clone();
    cache.bounds = minimap_bounds_from_keys(&keys);

    for child in children
        .get(minimap.dots)
        .into_iter()
        .flat_map(|c| c.iter())
    {
        commands.entity(child).despawn();
    }

    let Some(bounds) = cache.bounds else {
        return;
    };

    let plot = MINIMAP_SIZE - MINIMAP_INSET * 2.0 - MINIMAP_DOT;
    let focused = hud.snapshot.focused_chunk;
    for &raw in &keys {
        let [u, v] = chunk_to_minimap_uv(ChunkId(raw), bounds);
        let left = MINIMAP_INSET + u * plot;
        let top = MINIMAP_INSET + v * plot;
        let is_focused = focused.map(|id| id.0 == raw).unwrap_or(false);
        let dot_color = if is_focused {
            Color::srgb(0.95, 0.92, 0.45)
        } else {
            Color::srgb(0.72, 0.69, 0.62)
        };
        commands.entity(minimap.dots).with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(MINIMAP_DOT),
                    height: Val::Px(MINIMAP_DOT),
                    ..default()
                },
                BackgroundColor(dot_color),
                FocusPolicy::Pass,
            ));
        });
    }

    let cam_cx = (orbit.centre[0] / CHUNK_EDGE as f32).floor() as i32;
    let cam_cz = (orbit.centre[2] / CHUNK_EDGE as f32).floor() as i32;
    if let Some(cam_raw) = keys.iter().find(|&&raw| {
        let (cx, _cy, cz) = decode_chunk_id(ChunkId(raw));
        cx == cam_cx && cz == cam_cz
    }) {
        let [u, v] = chunk_to_minimap_uv(ChunkId(*cam_raw), bounds);
        let left = MINIMAP_INSET + u * plot - 1.0;
        let top = MINIMAP_INSET + v * plot - 1.0;
        commands.entity(minimap.dots).with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(left),
                    top: Val::Px(top),
                    width: Val::Px(MINIMAP_DOT + 2.0),
                    height: Val::Px(MINIMAP_DOT + 2.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.95, 0.98)),
                FocusPolicy::Pass,
            ));
        });
    }
}

fn minimap_click_focus(
    mouse: Res<ButtonInput<MouseButton>>,
    panels: Query<(&Interaction, &RelativeCursorPosition), With<MinimapPanel>>,
    cache: Res<MinimapCache>,
    scene: Res<LiveScene>,
    mut orbit: ResMut<OrbitCamera>,
    mut hud: ResMut<HudState>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((interaction, cursor)) = panels.single() else {
        return;
    };
    if *interaction == Interaction::None || cursor.normalized.is_none() {
        return;
    }

    let Some(bounds) = cache.bounds else {
        return;
    };
    let Some(normalized) = cursor.normalized else {
        return;
    };

    let (cx, cz) = minimap_uv_to_chunk_grid([normalized.x, normalized.y], bounds);
    orbit.centre[0] = (cx as f32 + 0.5) * CHUNK_EDGE as f32;
    orbit.centre[2] = (cz as f32 + 0.5) * CHUNK_EDGE as f32;

    let preferred_cy = (orbit.centre[1] / CHUNK_EDGE as f32).floor() as i32;
    let loaded: Vec<u64> = scene.chunks.keys().copied().collect();
    hud.snapshot.focused_chunk = Some(focused_chunk_at_grid(cx, cz, preferred_cy, &loaded));
}

fn viewport_chunk_raycast(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    minimap: Query<&Interaction, With<MinimapPanel>>,
    orbit: Res<OrbitCamera>,
    mut hud: ResMut<HudState>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let minimap_active = minimap
        .single()
        .map(|interaction| *interaction != Interaction::None)
        .unwrap_or(false);
    if minimap_active {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(transform, cursor) else {
        return;
    };

    let origin = ray.origin.to_array();
    let direction = ray.direction.to_array();
    if let Some(chunk) = chunk_raycast_stub(origin, direction, orbit.centre[1], VOXEL_CHUNK_EDGE) {
        hud.snapshot.focused_chunk = Some(chunk);
    }
}

fn apply_voxel_delta(
    commands: &mut Commands,
    scene: &mut LiveScene,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    orbit: &OrbitCamera,
    debug: &DebugRender,
    delta: VoxelDeltaFrame,
) {
    let target = orbit.as_target();
    let eye = target.orbit_position();
    let max_dist = orbit.distance;

    for chunk in delta.deltas {
        let chunk_id = chunk.event.chunk_id;
        if !should_render_chunk(chunk_id, eye, max_dist) {
            if let Some(entity) = scene.chunks.remove(&chunk_id.0) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if chunk.voxels.len() != CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE {
            continue;
        }

        let chunk_view = ChunkView {
            id: chunk.event.chunk_id,
            voxels: &chunk.voxels,
        };
        let distance = chunk_distance_from_camera(chunk.event.chunk_id, eye, CHUNK_EDGE as f32);
        let lod = LodLevel(mesh_lod_level(distance));
        let Ok(mesh_buffer) = CubicMesher::mesh_cubic(chunk_view, lod) else {
            continue;
        };
        let mesh = meshes.add(mesh_buffer_to_bevy(&mesh_buffer));
        let mut material = StandardMaterial {
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        };
        apply_chunk_material(&mut material, CHUNK_BASE_COLOR, debug.wireframe, Some(0.0));
        let material_handle = materials.add(material);
        let transform = chunk_transform(chunk.event.chunk_id);

        let entity = *scene
            .chunks
            .entry(chunk.event.chunk_id.0)
            .or_insert_with(|| {
                commands
                    .spawn((
                        ChunkTag {
                            id: chunk.event.chunk_id,
                        },
                        Transform::default(),
                    ))
                    .id()
            });
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            ChunkFade::new(),
        ));
        if debug.wireframe {
            commands.entity(entity).insert((
                Wireframe,
                WireframeColor {
                    color: CHUNK_WIREFRAME_LINE_COLOR,
                },
            ));
        } else {
            commands
                .entity(entity)
                .remove::<Wireframe>()
                .remove::<WireframeColor>();
        }
    }
}

fn update_chunk_fade(
    time: Res<Time>,
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut fades: Query<(Entity, &mut ChunkFade, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if debug.wireframe {
        return;
    }

    for (entity, mut fade, material) in fades.iter_mut() {
        fade.elapsed += time.delta_secs();
        if let Some(material) = materials.get_mut(&material.0) {
            apply_chunk_material(material, fade.base_rgb, false, Some(fade.elapsed));
        }
        if chunk_fade_complete(fade.elapsed) {
            commands.entity(entity).remove::<ChunkFade>();
        }
    }
}

fn apply_agent_appearance(
    commands: &mut Commands,
    scene: &mut LiveScene,
    materials: &mut Assets<StandardMaterial>,
    assets: &AgentVisualAssets,
    agents: AgentAppearanceFrame,
) {
    for update in agents.updates {
        let rgb = agent_color_from_id(update.agent_id);
        let scale = agent_scale_multiplier(update.scale);
        let (x, y, z) = agent_world_translation(&update, 0.8);
        let transform = Transform::from_xyz(x, y, z).with_scale(Vec3::splat(scale));

        let material_handle = scene
            .agent_materials
            .entry(update.agent_id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }

        let entity = *scene.agents.entry(update.agent_id).or_insert_with(|| {
            let entity = commands
                .spawn(AgentTag {
                    id: update.agent_id,
                })
                .id();
            if AGENT_NAME_LABELS {
                let label = agent_label_stub(update.agent_id, None);
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text2d::new(label),
                        TextFont::from_font_size(AGENT_LABEL_FONT_SIZE),
                        TextColor(Color::srgba(0.95, 0.96, 0.98, 0.92)),
                        Transform::from_xyz(0.0, AGENT_LABEL_Y_OFFSET, 0.0),
                        AgentLabel,
                    ));
                });
            }
            entity
        });

        commands.entity(entity).insert((
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

fn chunk_transform(id: ChunkId) -> Transform {
    let (x, y, z) = decode_chunk_id(id);
    Transform::from_xyz(
        x as f32 * CHUNK_EDGE as f32,
        y as f32 * CHUNK_EDGE as f32,
        z as f32 * CHUNK_EDGE as f32,
    )
}
