use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use civ_bevy_ref::{
    bevy_render::{apply_chunk_material, spawn_default_scene, CHUNK_WIREFRAME_LINE_COLOR},
    chunk_fade_complete, chunk_raycast_terrain, chunk_to_minimap_uv, focused_chunk_at_grid,
    gpu_features::GpuFeaturesPlugin,
    live_focus::{
        compute_live_scene_focus, minimap_uv_to_world_xz, LiveSceneFocus, LIVE_FOCUS_LERP_SPEED,
    },
    live_minimap::{
        chunk_centre_world_xz, live_building_dot_color, minimap_bounds_from_keys,
        spawn_minimap_dot, world_minimap_uv, MinimapDotLayout, MinimapFocusRect,
        LIVE_MINIMAP_AGENT_COLOR, LIVE_MINIMAP_CAMERA_COLOR, LIVE_MINIMAP_CHUNK_FOCUSED_COLOR,
        LIVE_MINIMAP_CHUNK_LOADED_COLOR, LIVE_MINIMAP_DOT, LIVE_MINIMAP_GRAPH_DOT_SCALE,
    },
    faction_hud::{FactionHudPlugin, PlayerFactionId},
    god_panel::GodPanelPlugin,
    save_load_ui::SaveLoadUiPlugin,
    live_pick::{LivePickPlugin, LiveSelection},
    live_stream::{
        apply_agent_appearance_frame_with_labels_and_eye, apply_building_diff_frame,
        apply_civilian_state_frame, apply_climate_frame, apply_event_feed_frame,
        apply_faction_state_frame, apply_voxel_delta_frame, default_stream_meshes,
        format_event_feed_message, latest_climate, push_event_feed_to_hud_summary,
        sync_agent_labels_from_civilians, AgentLabelConfig, LiveAgentTag, LiveBuildingTag,
        LiveChunkFade, LiveChunkTag, LiveGraphParcelTag, LiveStreamMeshes, LiveStreamScene,
        StreamCulling,
        LIVE_CHUNK_BASE_COLOR, LIVE_CHUNK_EDGE,
    },
    minimap::MinimapRoot,
    minimap_uv_to_chunk_grid,
    native_backend::native_render_plugin,
    presentation_ambient_brightness, presentation_ambient_color_rgb, presentation_clear_color_rgb,
    presentation_day_factor_target, resolve_live_ws_url,
    event_feed::{EventFeed, EventFeedPlugin},
    emergence_dashboard::EmergenceDashboardPlugin,
    ws_client::{WsClient, WsClientConfig},
    CameraTarget, DebugRender, EmergenceHudData, LiveHudSnapshot, MinimapBounds,
    VOXEL_CHUNK_EDGE, WsConnectionState,
    post_fx::PostFxPlugin,
};
#[cfg(feature = "gi")]
use civ_bevy_ref::lighting_gi::SolariGiPlugin;
#[cfg(feature = "models")]
use civ_bevy_ref::animation::ActorAnimationPlugin;
#[cfg(feature = "models")]
use civ_bevy_ref::gltf_models::GltfModelsPlugin;
#[cfg(feature = "egui")]
use civ_bevy_ref::settings_ui::{GameSettings, KeyBinding, SettingsPlugin};
use civ_bevy_ref::diplomacy_ui::{DiplomacyBridge, DiplomacyUiPlugin};
use civ_protocol_3d::Frame3d;
use civ_voxel::ChunkId;
use serde_json;

const CHUNK_BASE_COLOR: [f32; 3] = LIVE_CHUNK_BASE_COLOR;
const ORBIT_DRAG_SENSITIVITY: f32 = 0.005;
const ORBIT_SCROLL_SENSITIVITY: f32 = 2.0;
const ORBIT_KEYBOARD_DISTANCE_STEP: f32 = 4.0;
const ORBIT_PAN_SPEED: f32 = 12.0;
const MIN_ORBIT_ELEVATION: f32 = 0.05;
const MIN_ORBIT_DISTANCE: f32 = 8.0;
const MAX_ORBIT_DISTANCE: f32 = 200.0;
const MINIMAP_SIZE: f32 = 160.0;
const MINIMAP_DOT: f32 = LIVE_MINIMAP_DOT;
const MINIMAP_INSET: f32 = 6.0;
const MINIMAP_HUD_LAYOUT: MinimapDotLayout = MinimapDotLayout::InsetHud {
    panel_size: MINIMAP_SIZE,
    inset: MINIMAP_INSET,
    plot_margin_dot: MINIMAP_DOT,
};

// FR-CIV-CLIENT-001
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    Connecting,
    InGame,
    ConnectionLost,
}

#[derive(Resource, Default)]
struct ConnectionOverlay {
    root: Option<Entity>,
    tick: u32,
}

#[derive(Component)]
struct SplashSpinner;
#[derive(Resource, Debug, Clone)]
struct ScenarioPanel {
    seed_index: usize,
    speed_index: usize,
    preset_index: usize,
}

impl Default for ScenarioPanel {
    fn default() -> Self {
        Self { seed_index: 0, speed_index: 0, preset_index: 0 }
    }
}


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

#[derive(Component)]
struct MinimapCameraDot;

#[derive(Resource)]
struct MinimapUi {
    dots: Entity,
    camera_dot: Entity,
}

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
    agent_count: usize,
    building_count: usize,
    graph_count: usize,
    bounds: Option<MinimapBounds>,
    focus: Option<LiveSceneFocus>,
    use_focus_bounds: bool,
}

#[derive(Resource, Default)]
struct MinimapPopup {
    /// Pending right-click tile coords; None when popup is closed.
    pending: Option<(i32, i32)>,
}

#[derive(Resource, Default)]
struct SimSpeedState {
    multiplier: u32,
}

#[derive(Resource)]
struct EmergencePollTimer(f32);
impl Default for EmergencePollTimer {
    fn default() -> Self {
        Self(0.0)
    }
}

#[derive(Component)]
struct ScenarioSeedLabel;

#[derive(Component)]
struct ScenarioSpeedLabel;

#[derive(Component)]
struct ScenarioPresetLabel;

#[derive(Component)]
struct ScenarioStartButton;

#[derive(Component)]
struct ScenarioStartButton;

#[derive(Resource, Default)]
struct MinimapPopup {
    /// Pending right-click tile coords; None when popup is closed.
    pending: Option<(i32, i32)>,
}

#[derive(Resource, Default)]
struct SimSpeedState {
    multiplier: u32,
}

#[derive(Resource, Default)]
struct EmergencePollTimer(f32);

fn main() {
    let mut app = App::new();
    app.add_plugins((
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
            PostFxPlugin,
            GpuFeaturesPlugin,
            LivePickPlugin,
            FactionHudPlugin,
            SaveLoadUiPlugin,
            TutorialPlugin,
            PerfHudPlugin,
            EguiPlugin::default(),
            EventFeedPlugin,
            EmergenceDashboardPlugin,
            civ_bevy_ref::AgentNeedsPlugin,
            DiplomacyUiPlugin,
            GodPanelPlugin,
        ))
        .init_resource::<LiveStreamScene>()
        .init_resource::<LiveSceneFocus>()
        .init_resource::<ConnectionOverlay>()
        .init_resource::<ScenarioPanel>()
        .init_resource::<MinimapPopup>()
        .init_resource::<SimSpeedState>()
        .init_resource::<EmergencePollTimer>()
        .init_resource::<EmergenceHudData>()
        .insert_resource(ScenePresentation::default())
        .insert_resource(DebugRender::default())
        .insert_resource(OrbitCamera::from_target(CameraTarget::default()))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(AppState::Connecting), spawn_connecting_overlay)
        .add_systems(OnExit(AppState::Connecting), despawn_connection_overlay)
        .add_systems(OnEnter(AppState::ConnectionLost), spawn_lost_overlay)
        .add_systems(OnExit(AppState::ConnectionLost), despawn_connection_overlay)
        .add_systems(Update, drive_app_state)
        .add_systems(Update, sync_perf_metrics.run_if(crate::menus::in_game))
        .add_systems(Update, animate_splash.run_if(in_state(AppState::Connecting)))
        .add_systems(Update, scenario_panel_input.run_if(in_state(AppState::Connecting)))
        .add_systems(
            Update,
            (
                debug_render_input,
                orbit_camera_input,
                minimap_click_focus,
                minimap_popup_ui,
                poll_emergence,
                viewport_chunk_raycast,
                update_orbit_camera_transform,
                apply_live_frames,
                sync_agent_labels_from_civilians.after(apply_live_frames),
                apply_spectator_meta,
                sync_live_hud_stats,
                sync_live_pick_detail,
                update_live_focus,
                follow_live_orbit_focus,
                sync_chunk_debug_render,
                update_chunk_fade,
                update_hud,
                update_minimap,
                // FR-CLIENT-render: climate frame → presentation.is_day.
                // Must run after `apply_live_frames` (which records the
                // climate snapshot) and before `update_presentation_lighting`
                // (which consumes the flag).
                sync_presentation_from_climate.after(apply_live_frames),
                update_presentation_lighting,
            ),
        )
        .run();

    #[cfg(feature = "egui")]
    {
        app.add_plugins(SettingsPlugin);
    }

    #[cfg(feature = "models")]
    {
        app.add_plugins((GltfModelsPlugin, ActorAnimationPlugin));
    }

    #[cfg(feature = "gi")]
    {
        app.add_plugins(SolariGiPlugin);
    }

    app.run();
}

fn scenario_panel_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut panel: ResMut<ScenarioPanel>,
    bridge: Res<LiveBridge>,
    mut seed_labels: Query<&mut Text, (With<ScenarioSeedLabel>, Without<ScenarioSpeedLabel>, Without<ScenarioStartButton>)>,
    mut speed_labels: Query<&mut Text, (With<ScenarioSpeedLabel>, Without<ScenarioSeedLabel>, Without<ScenarioStartButton>)>,
    mut preset_labels: Query<&mut Text, (With<ScenarioPresetLabel>, Without<ScenarioSeedLabel>, Without<ScenarioSpeedLabel>, Without<ScenarioStartButton>)>,
    start_buttons: Query<&Interaction, With<ScenarioStartButton>>,
) {
    // Keyboard shortcuts: Left/Right cycle seeds; Up/Down cycle speeds; Enter launches.
    if keys.just_pressed(KeyCode::ArrowRight) {
        panel.seed_index = (panel.seed_index + 1) % NAMED_SEEDS.len();
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        panel.seed_index = panel.seed_index.checked_sub(1).unwrap_or(NAMED_SEEDS.len() - 1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        panel.speed_index = (panel.speed_index + 1) % SPEED_OPTIONS.len();
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        panel.speed_index = panel.speed_index.checked_sub(1).unwrap_or(SPEED_OPTIONS.len() - 1);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        panel.preset_index = (panel.preset_index + 1) % PRESET_OPTIONS.len();
    }
    if keys.just_pressed(KeyCode::KeyO) {
        panel.preset_index = panel.preset_index.checked_sub(1).unwrap_or(PRESET_OPTIONS.len() - 1);
    }

    // Rebuild seed label
    if let Ok(mut text) = seed_labels.get_single_mut() {
        let label = NAMED_SEEDS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| if i == panel.seed_index { format!("[ {name} ]") } else { name.to_string() })
            .collect::<Vec<_>>()
            .join("  ");
        *text = Text::new(format!("Race: {label}"));
    }

    // Rebuild speed label
    if let Ok(mut text) = speed_labels.get_single_mut() {
        let label = SPEED_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| if i == panel.speed_index { format!("[ {name} ]") } else { name.to_string() })
            .collect::<Vec<_>>()
            .join("  ");
        *text = Text::new(format!("Speed: {label}"));
    }

    // Rebuild preset label
    if let Ok(mut text) = preset_labels.get_single_mut() {
        let label = PRESET_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, name)| if i == panel.preset_index { format!("[ {name} ]") } else { name.to_string() })
            .collect::<Vec<_>>()
            .join("  ");
        *text = Text::new(format!("Preset: {label}"));
    }

    // Launch on button click or Enter key
    let clicked = start_buttons
        .get_single()
        .map(|i| *i == Interaction::Pressed)
        .unwrap_or(false);
    if clicked || keys.just_pressed(KeyCode::Enter) {
        let (_, seed) = NAMED_SEEDS[panel.seed_index];
        let (_, speed) = SPEED_OPTIONS[panel.speed_index];
        let preset = PRESET_OPTIONS[panel.preset_index];
        bridge.client.send_rpc(
            "sim.load_scenario",
            serde_json::json!({ "preset": preset, "seed": seed }),
        );
        bridge.client.send_rpc("sim.set_speed", serde_json::json!({ "speed": speed }));
        info!("scenario launch: preset={preset} seed={seed} speed={speed}");
        bridge.client.send_rpc("sim.reset", serde_json::json!({ "seed": seed }));
        bridge.client.send_rpc("sim.set_speed", serde_json::json!({ "speed": speed }));
        info!("scenario launch: preset={preset} seed={seed} speed={speed}");
    }
}


fn drive_app_state(bridge: Res<LiveBridge>, current: Res<State<AppState>>, mut next: ResMut<NextState<AppState>>) {
    let ws = bridge.client.latest_connection_state();
    match (current.get(), ws) {
        (AppState::Connecting, WsConnectionState::Connected) => { next.set(AppState::InGame); }
        (AppState::InGame, WsConnectionState::Reconnecting) | (AppState::InGame, WsConnectionState::Disconnected) => { next.set(AppState::ConnectionLost); }
        (AppState::ConnectionLost, WsConnectionState::Connected) => { next.set(AppState::InGame); }
        _ => {}
    }
}

fn spawn_connecting_overlay(mut commands: Commands, mut overlay: ResMut<ConnectionOverlay>) {
    let root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.039, 0.047)),
        ZIndex(100),
    )).with_children(|p| {
        p.spawn((
            Text::new("CIVIS"),
            TextFont::from_font_size(72.0),
            TextColor(Color::srgb(0.494, 0.729, 0.710)),
        ));
        p.spawn((
            Text::new("An Emergent Civilization"),
            TextFont::from_font_size(20.0),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
        ));
        p.spawn((
            Text::new("⠋"),
            TextFont::from_font_size(28.0),
            TextColor(Color::srgb(0.494, 0.729, 0.710)),
            SplashSpinner,
        ));
        p.spawn((
            Text::new("Connecting to simulation server..."),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));
    }).id();
    overlay.root = Some(root);
    overlay.tick = 0;
}

fn spawn_lost_overlay(mut commands: Commands, mut overlay: ResMut<ConnectionOverlay>) {
    let root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0), left: Val::Px(0.0),
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.18, 0.02, 0.02, 0.92)),
        ZIndex(100),
    )).with_children(|p| {
        p.spawn((
            Text::new("CIVIS"),
            TextFont::from_font_size(72.0),
            TextColor(Color::srgb(0.494, 0.729, 0.710)),
        ));
        p.spawn((
            Text::new("Connection lost. Retrying..."),
            TextFont::from_font_size(22.0),
            TextColor(Color::srgb(1.0, 0.35, 0.35)),
        ));
        p.spawn((
            Text::new("The simulation will resume when the server is reachable."),
            TextFont::from_font_size(14.0),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        ));
    }).id();
    overlay.root = Some(root);
    overlay.tick = 0;
}

fn despawn_connection_overlay(mut commands: Commands, mut overlay: ResMut<ConnectionOverlay>) {
    if let Some(root) = overlay.root.take() { commands.entity(root).despawn_recursive(); }
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn animate_splash(
    mut overlay: ResMut<ConnectionOverlay>,
    mut spinners: Query<&mut Text, With<SplashSpinner>>,
) {
    overlay.tick = overlay.tick.wrapping_add(1);
    if overlay.tick % 4 != 0 { return; }
    let frame = SPINNER_FRAMES[((overlay.tick / 4) as usize) % SPINNER_FRAMES.len()];
    for mut text in &mut spinners {
        **text = frame.to_string();
    }
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
    if let Some(rtt) = bridge.client.latest_rtt_ms() {
        hud.snapshot.ws_rtt_ms = Some(rtt);
    }
}

fn sync_live_pick_detail(
    selection: Res<LiveSelection>,
    scene: Res<LiveStreamScene>,
    mut hud: ResMut<HudState>,
) {
    hud.snapshot.pick_detail =
        civ_bevy_ref::live_stream::format_live_pick_hud_line(selection.0, &scene);
}

fn sync_live_hud_stats(
    bridge: Res<LiveBridge>,
    scene: Res<LiveStreamScene>,
    mut hud: ResMut<HudState>,
) {
    let civilians = civ_bevy_ref::live_stream::civilian_hud_count(&scene);
    let factions = civ_bevy_ref::live_stream::faction_hud_count(&scene);
    hud.snapshot.sync_scene_counts(
        scene.chunks.len(),
        scene.agents.len(),
        scene.buildings.len(),
        scene.graph_parcels.len(),
        civilians,
        factions,
    );
    if let Some(rtt) = bridge.client.latest_rtt_ms() {
        hud.snapshot.ws_rtt_ms = Some(rtt);
    }
}

fn live_stream_has_content(scene: &LiveStreamScene) -> bool {
    !scene.chunks.is_empty()
        || !scene.agents.is_empty()
        || !scene.buildings.is_empty()
        || !scene.graph_parcels.is_empty()
}

fn update_live_focus(
    hud: Res<HudState>,
    scene: Res<LiveStreamScene>,
    agents: Query<&Transform, With<LiveAgentTag>>,
    buildings: Query<&Transform, With<LiveBuildingTag>>,
    graph_parcels: Query<&Transform, With<LiveGraphParcelTag>>,
    mut focus: ResMut<LiveSceneFocus>,
) {
    if !hud.snapshot.connected {
        return;
    }

    let next = compute_live_scene_focus(&scene, &agents, &buildings, &graph_parcels);
    if next != *focus {
        *focus = next;
    }
}

fn follow_live_orbit_focus(
    hud: Res<HudState>,
    scene: Res<LiveStreamScene>,
    focus: Res<LiveSceneFocus>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if !hud.snapshot.connected || !live_stream_has_content(&scene) {
        return;
    }

    let alpha = (time.delta_secs() * LIVE_FOCUS_LERP_SPEED).clamp(0.0, 1.0);
    orbit.centre[0] += (focus.centre.x - orbit.centre[0]) * alpha;
    orbit.centre[2] += (focus.centre.z - orbit.centre[2]) * alpha;
}

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

/// FR-CLIENT-render: when the live scene has observed a climate frame, the
/// presentation's `is_day` flag is driven by the snapshot's `day_phase`
/// instead of being a static default. Falls through to the existing
/// `is_day=true` default when no climate frame has been observed yet so
/// pre-snapshot behaviour is preserved.
fn sync_presentation_from_climate(
    scene: Res<LiveStreamScene>,
    mut presentation: ResMut<ScenePresentation>,
) {
    let Some((snap, _)) = latest_climate(&scene) else {
        return;
    };
    presentation.is_day = snap.day_phase.is_day_band();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    spawn_default_scene(&mut commands);
    commands.insert_resource(default_stream_meshes(&mut meshes));
    let ws_client = WsClient::spawn_with_config(resolve_live_ws_url(), WsClientConfig::default());
    commands.insert_resource(DiplomacyBridge::new(ws_client.rpc_sender()));
    commands.insert_resource(LiveBridge { client: ws_client });

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
            MinimapRoot,
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

    let camera_dot = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(MINIMAP_DOT + 2.0),
                height: Val::Px(MINIMAP_DOT + 2.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(LIVE_MINIMAP_CAMERA_COLOR),
            FocusPolicy::Pass,
            MinimapCameraDot,
        ))
        .id();
    commands.entity(panel).add_child(camera_dot);

    commands.insert_resource(MinimapUi { dots, camera_dot });
    commands.insert_resource(MinimapCache::default());
}

fn debug_render_input(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugRender>) {
    if keys.just_pressed(KeyCode::F3) {
        debug.toggle_wireframe();
    }
}

fn speed_control_input(
    keys: Res<ButtonInput<KeyCode>>,
    bridge: Res<LiveBridge>,
    mut speed: ResMut<SimSpeedState>,
    mut hud: ResMut<HudState>,
) {
    let toggle_pause = keys.just_pressed(KeyCode::Space);
    let speed_up = keys.just_pressed(KeyCode::Period);
    let speed_down = keys.just_pressed(KeyCode::Comma);

    if !toggle_pause && !speed_up && !speed_down {
        return;
    }

    if toggle_pause {
        speed.paused = !speed.paused;
    } else if speed_up {
        speed.speed_idx = (speed.speed_idx + 1).min(SPEED_OPTIONS.len() - 1);
        speed.paused = false;
    } else {
        speed.speed_idx = speed.speed_idx.saturating_sub(1);
        speed.paused = false;
    }

    speed.multiplier = if speed.paused { 0 } else { SPEED_OPTIONS[speed.speed_idx] };
    let json = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"sim.set_speed","params":{{"multiplier":{}}}}}"#, speed.multiplier);
    bridge.client.send_rpc_raw(json);
    hud.snapshot.speed_multiplier = speed.multiplier;
}

fn action_pressed(
    #[cfg(feature = "egui")] settings: Option<&GameSettings>,
    action: &str,
    default: KeyBinding,
    keys: &ButtonInput<KeyCode>,
    mouse_buttons: &ButtonInput<MouseButton>,
) -> bool {
    #[cfg(feature = "egui")]
    {
        settings
            .and_then(|s| s.key_for(action))
            .unwrap_or(default)
            .is_pressed(keys, mouse_buttons)
    }
    #[cfg(not(feature = "egui"))]
    {
        default.is_pressed(keys, mouse_buttons)
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
            Option<&LiveChunkFade>,
        ),
        With<LiveChunkTag>,
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
    mut scene: ResMut<LiveStreamScene>,
    mut hud: ResMut<HudState>,
    orbit: Res<OrbitCamera>,
    debug: Res<DebugRender>,
    assets: Res<LiveStreamMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut feed: ResMut<EventFeed>,
) {
    let frames = bridge.client.poll();
    if !frames.is_empty() {
        hud.snapshot.connected = true;
    }

    let target = orbit.as_target();
    let eye = target.orbit_position();
    let culling = StreamCulling {
        eye,
        max_distance: orbit.distance,
    };
    let wireframe_color = debug.wireframe.then_some(CHUNK_WIREFRAME_LINE_COLOR);

    for frame in frames {
        hud.snapshot.tick = Some(frame.tick());
        match frame {
            Frame3d::VoxelDelta(delta) => apply_voxel_delta_frame(
                &mut commands,
                &mut scene,
                &mut meshes,
                &mut materials,
                culling,
                debug.as_ref(),
                delta,
                wireframe_color,
            ),
            Frame3d::AgentAppearance(agents) => {
                // FR-CLIENT-render: pass the camera eye so far-away agents
                // get a distance-LOD scale attenuation (otherwise they punch
                // through the terrain near the horizon).
                apply_agent_appearance_frame_with_labels_and_eye(
                    &mut commands,
                    &mut scene,
                    &mut materials,
                    assets.as_ref(),
                    agents,
                    AgentLabelConfig { enabled: true },
                    Some(eye),
                );
            }
            Frame3d::BuildingDiff(building) => apply_building_diff_frame(
                &mut commands,
                &mut scene,
                &mut materials,
                assets.as_ref(),
                building,
            ),
            Frame3d::CivilianState(civilian) => apply_civilian_state_frame(&mut scene, civilian),
            Frame3d::FactionState(faction) => apply_faction_state_frame(&mut scene, faction),
            Frame3d::EventFeed(ref event_frame) => {
                for msg in &event_frame.events {
                    info!(
                        "event feed (tick {}): {}",
                        event_frame.tick,
                        format_event_feed_message(msg),
                    );
                }
                push_event_feed_to_hud_summary(&mut hud.snapshot, event_frame);
                apply_event_feed_frame(&mut feed, event_frame.clone());
            }
            // FR-CLIENT-render: route the streamed climate frame into the
            // live scene (was previously dropped). Downstream sky/sun/
            // ambient systems consume it via `latest_climate(&scene)`.
            Frame3d::Climate(climate) => apply_climate_frame(&mut scene, climate),
        }
    }
}

fn orbit_camera_input(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    #[cfg(feature = "egui")]
    settings: Option<Res<GameSettings>>,
    mut orbit: ResMut<OrbitCamera>,
    minimap: Query<&Interaction, With<MinimapPanel>>,
) {
    let minimap_active = minimap
        .single()
        .map(|interaction| *interaction != Interaction::None)
        .unwrap_or(false);

    let rotate_pressed = action_pressed(
        #[cfg(feature = "egui")]
        settings.as_deref(),
        civ_bevy_ref::settings_ui::ACTION_CAMERA_ROTATE,
        KeyBinding::Mouse(MouseButton::Left),
        &keys,
        &mouse_buttons,
    );

    if rotate_pressed && !minimap_active {
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

    let reset_pressed = {
        #[cfg(feature = "egui")]
        {
            settings
                .as_ref()
                .and_then(|s| s.key_for(civ_bevy_ref::settings_ui::ACTION_CAMERA_RESET))
                .unwrap_or(KeyBinding::Key(KeyCode::KeyR))
                .is_just_pressed(&keys, &mouse_buttons)
        }
        #[cfg(not(feature = "egui"))]
        {
            keys.just_pressed(KeyCode::KeyR)
        }
    };
    if reset_pressed {
        orbit.reset();
    }

    let zoom_in = {
        #[cfg(feature = "egui")]
        {
            settings
                .as_ref()
                .and_then(|s| s.key_for(civ_bevy_ref::settings_ui::ACTION_CAMERA_ZOOM_IN))
                .unwrap_or(KeyBinding::Key(KeyCode::Equal))
                .is_just_pressed(&keys, &mouse_buttons)
        }
        #[cfg(not(feature = "egui"))]
        {
            keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) || keys.just_pressed(KeyCode::BracketLeft)
        }
    };
    let zoom_out = {
        #[cfg(feature = "egui")]
        {
            settings
                .as_ref()
                .and_then(|s| s.key_for(civ_bevy_ref::settings_ui::ACTION_CAMERA_ZOOM_OUT))
                .unwrap_or(KeyBinding::Key(KeyCode::Minus))
                .is_just_pressed(&keys, &mouse_buttons)
        }
        #[cfg(not(feature = "egui"))]
        {
            keys.just_pressed(KeyCode::Minus)
                || keys.just_pressed(KeyCode::NumpadSubtract)
                || keys.just_pressed(KeyCode::BracketRight)
        }
    };
    if zoom_in {
        orbit.adjust_distance(-ORBIT_KEYBOARD_DISTANCE_STEP);
    }
    if zoom_out {
        orbit.adjust_distance(ORBIT_KEYBOARD_DISTANCE_STEP);
    }

    let pan = ORBIT_PAN_SPEED * time.delta_secs();
    let mut right = 0.0;
    let mut forward = 0.0;
    let pan_pressed = |action: &str, fallback: KeyCode| -> bool {
        #[cfg(feature = "egui")]
        {
            settings
                .as_ref()
                .and_then(|s| s.key_for(action))
                .unwrap_or(KeyBinding::Key(fallback))
                .is_pressed(&keys, &mouse_buttons)
        }
        #[cfg(not(feature = "egui"))]
        {
            match fallback {
                KeyCode::KeyW => keys.pressed(KeyCode::KeyW),
                KeyCode::KeyS => keys.pressed(KeyCode::KeyS),
                KeyCode::KeyA => keys.pressed(KeyCode::KeyA),
                KeyCode::KeyD => keys.pressed(KeyCode::KeyD),
                _ => false,
            }
        }
    };

    if pan_pressed(civ_bevy_ref::settings_ui::ACTION_CAMERA_MOVE_FORWARD, KeyCode::KeyW) {
        forward += pan;
    }
    if pan_pressed(civ_bevy_ref::settings_ui::ACTION_CAMERA_MOVE_BACKWARD, KeyCode::KeyS) {
        forward -= pan;
    }
    if pan_pressed(civ_bevy_ref::settings_ui::ACTION_CAMERA_MOVE_LEFT, KeyCode::KeyA) {
        right -= pan;
    }
    if pan_pressed(civ_bevy_ref::settings_ui::ACTION_CAMERA_MOVE_RIGHT, KeyCode::KeyD) {
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
    selection: Res<LiveSelection>,
    mut hud: ResMut<HudState>,
    mut text: Query<&mut Text, With<HudText>>,
) {
    let fps = 1.0 / time.delta_secs();
    hud.snapshot.fps = if hud.snapshot.fps <= 0.0 {
        fps
    } else {
        hud.snapshot.fps * 0.9 + fps * 0.1
    };
    hud.snapshot.selected_live = selection.0;

    let Ok(mut text) = text.get_mut(hud.text) else {
        return;
    };
    *text = Text::new(hud.snapshot.format_overlay());
}

fn inset_minimap_uv_from_cursor(normalized: Vec2) -> [f32; 2] {
    let px = normalized.x * MINIMAP_SIZE;
    let py = normalized.y * MINIMAP_SIZE;
    let plot_origin = MINIMAP_INSET;
    let plot_size = MINIMAP_SIZE - MINIMAP_INSET * 2.0 - MINIMAP_DOT;
    let span = plot_size.max(f32::EPSILON);
    [
        ((px - plot_origin) / span).clamp(0.0, 1.0),
        ((py - plot_origin) / span).clamp(0.0, 1.0),
    ]
}

fn update_minimap(
    mut commands: Commands,
    scene: Res<LiveStreamScene>,
    focus: Res<LiveSceneFocus>,
    minimap: Res<MinimapUi>,
    orbit: Res<OrbitCamera>,
    hud: Res<HudState>,
    mut cache: ResMut<MinimapCache>,
    children: Query<&Children>,
    mut camera_dot: Query<&mut Node, With<MinimapCameraDot>>,
    agents: Query<&Transform, With<LiveAgentTag>>,
    agents_changed: Query<&Transform, (With<LiveAgentTag>, Changed<Transform>)>,
    buildings: Query<&Transform, With<LiveBuildingTag>>,
    buildings_changed: Query<&Transform, (With<LiveBuildingTag>, Changed<Transform>)>,
    graph_parcels: Query<&Transform, With<LiveGraphParcelTag>>,
    graph_parcels_changed: Query<&Transform, (With<LiveGraphParcelTag>, Changed<Transform>)>,
) {
    let mut keys: Vec<u64> = scene.chunks.keys().copied().collect();
    keys.sort_unstable();
    let agent_count = scene.agents.len();
    let building_count = scene.buildings.len();
    let graph_count = scene.graph_parcels.len();
    let transforms_changed = !agents_changed.is_empty()
        || !buildings_changed.is_empty()
        || !graph_parcels_changed.is_empty();
    let use_focus_bounds = hud.snapshot.connected && live_stream_has_content(&scene);
    let focus_snapshot = use_focus_bounds.then_some(*focus);
    let new_bounds = minimap_bounds_from_keys(&keys);

    let camera_uv = if let Some(focus) = focus_snapshot {
        Some(MinimapFocusRect {
            centre_x: focus.centre.x,
            centre_z: focus.centre.z,
            half_extent: focus.half_extent,
        })
        .map(|focus_rect| focus_rect.world_to_uv(orbit.centre[0], orbit.centre[2]))
    } else {
        new_bounds.map(|bounds| world_minimap_uv(orbit.centre[0], orbit.centre[2], bounds))
    };

    if let Ok(mut node) = camera_dot.get_mut(minimap.camera_dot) {
        if let Some(uv) = camera_uv {
            let (left, top) = MINIMAP_HUD_LAYOUT.dot_origin(uv, MINIMAP_DOT + 2.0);
            node.left = Val::Px(left);
            node.top = Val::Px(top);
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }

    if keys == cache.chunk_keys
        && agent_count == cache.agent_count
        && building_count == cache.building_count
        && graph_count == cache.graph_count
        && use_focus_bounds == cache.use_focus_bounds
        && focus_snapshot == cache.focus
        && !transforms_changed
    {
        return;
    }

    cache.chunk_keys = keys.clone();
    cache.agent_count = agent_count;
    cache.building_count = building_count;
    cache.graph_count = graph_count;
    cache.use_focus_bounds = use_focus_bounds;
    cache.focus = focus_snapshot;
    cache.bounds = new_bounds;

    for child in children
        .get(minimap.dots)
        .into_iter()
        .flat_map(|c| c.iter())
    {
        commands.entity(child).despawn();
    }

    let building_dot = live_building_dot_color(scene.building_provenance);
    let focused = hud.snapshot.focused_chunk;

    commands.entity(minimap.dots).with_children(|parent| {
        if let Some(focus) = focus_snapshot {
            let focus_rect = MinimapFocusRect {
                centre_x: focus.centre.x,
                centre_z: focus.centre.z,
                half_extent: focus.half_extent,
            };

            for &raw in &keys {
                let (x, z) = chunk_centre_world_xz(ChunkId(raw), LIVE_CHUNK_EDGE);
                let uv = focus_rect.world_to_uv(x, z);
                let dot_color = if focused.map(|id| id.0 == raw).unwrap_or(false) {
                    LIVE_MINIMAP_CHUNK_FOCUSED_COLOR
                } else {
                    LIVE_MINIMAP_CHUNK_LOADED_COLOR
                };
                spawn_minimap_dot(
                    parent,
                    MINIMAP_HUD_LAYOUT,
                    uv,
                    MINIMAP_DOT,
                    dot_color,
                    false,
                );
            }

            for transform in &agents {
                let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
                spawn_minimap_dot(
                    parent,
                    MINIMAP_HUD_LAYOUT,
                    uv,
                    LIVE_MINIMAP_DOT,
                    LIVE_MINIMAP_AGENT_COLOR,
                    false,
                );
            }

            for transform in &buildings {
                let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
                spawn_minimap_dot(
                    parent,
                    MINIMAP_HUD_LAYOUT,
                    uv,
                    LIVE_MINIMAP_DOT,
                    building_dot,
                    false,
                );
            }

            for transform in &graph_parcels {
                let uv = focus_rect.world_to_uv(transform.translation.x, transform.translation.z);
                spawn_minimap_dot(
                    parent,
                    MINIMAP_HUD_LAYOUT,
                    uv,
                    LIVE_MINIMAP_DOT * LIVE_MINIMAP_GRAPH_DOT_SCALE,
                    building_dot,
                    false,
                );
            }

            return;
        }

        let Some(bounds) = cache.bounds else {
            return;
        };

        for &raw in &keys {
            let uv = chunk_to_minimap_uv(ChunkId(raw), bounds);
            let dot_color = if focused.map(|id| id.0 == raw).unwrap_or(false) {
                LIVE_MINIMAP_CHUNK_FOCUSED_COLOR
            } else {
                LIVE_MINIMAP_CHUNK_LOADED_COLOR
            };
            spawn_minimap_dot(
                parent,
                MINIMAP_HUD_LAYOUT,
                uv,
                MINIMAP_DOT,
                dot_color,
                false,
            );
        }

        for transform in &agents {
            let uv = world_minimap_uv(transform.translation.x, transform.translation.z, bounds);
            spawn_minimap_dot(
                parent,
                MINIMAP_HUD_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT,
                LIVE_MINIMAP_AGENT_COLOR,
                false,
            );
        }

        for transform in &buildings {
            let uv = world_minimap_uv(transform.translation.x, transform.translation.z, bounds);
            spawn_minimap_dot(
                parent,
                MINIMAP_HUD_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT,
                building_dot,
                false,
            );
        }

        for transform in &graph_parcels {
            let uv = world_minimap_uv(transform.translation.x, transform.translation.z, bounds);
            spawn_minimap_dot(
                parent,
                MINIMAP_HUD_LAYOUT,
                uv,
                LIVE_MINIMAP_DOT * LIVE_MINIMAP_GRAPH_DOT_SCALE,
                building_dot,
                false,
            );
        }

    });
}

fn minimap_click_focus(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    #[cfg(feature = "egui")]
    settings: Option<Res<GameSettings>>,
    panels: Query<(&Interaction, &RelativeCursorPosition), With<MinimapPanel>>,
    cache: Res<MinimapCache>,
    scene: Res<LiveStreamScene>,
    mut orbit: ResMut<OrbitCamera>,
    mut hud: ResMut<HudState>,
    mut popup: ResMut<MinimapPopup>,
    bridge: Res<LiveBridge>,
) {
    let select_pressed = action_pressed(
        #[cfg(feature = "egui")]
        settings.as_deref(),
        civ_bevy_ref::settings_ui::ACTION_SELECT_OR_PICK,
        KeyBinding::Mouse(MouseButton::Left),
        &keys,
        &mouse,
    );
    if !select_pressed {
        // Right-click: open inspect popup
        if mouse.just_pressed(MouseButton::Right) {
            if let Ok((interaction, cursor)) = panels.single() {
                if *interaction != Interaction::None {
                    if let Some(normalized) = cursor.normalized {
                        let uv = inset_minimap_uv_from_cursor(normalized);
                        let (tx, ty) = if cache.use_focus_bounds {
                            if let Some(focus) = cache.focus {
                                let (x, z) = minimap_uv_to_world_xz(Vec2::new(uv[0], uv[1]), focus);
                                (x as i32, z as i32)
                            } else {
                                (0, 0)
                            }
                        } else if let Some(bounds) = cache.bounds {
                            let (cx, cz) = minimap_uv_to_chunk_grid(inset_minimap_uv_from_cursor(normalized), bounds);
                            (cx, cz)
                        } else {
                            (0, 0)
                        };
                        popup.pending = Some((tx, ty));
                    }
                }
            }
            return;
        }
        // Suppress unused warning — bridge is available for future left-click RPCs.
        let _ = &bridge;

        if !mouse.just_pressed(MouseButton::Left) {
            return;
        }
    }

    let Ok((interaction, cursor)) = panels.single() else {
        return;
    };
    if *interaction == Interaction::None || cursor.normalized.is_none() {
        return;
    }

    let Some(normalized) = cursor.normalized else {
        return;
    };

    if cache.use_focus_bounds {
        let Some(focus) = cache.focus else {
            return;
        };
        let uv = inset_minimap_uv_from_cursor(normalized);
        let (x, z) = minimap_uv_to_world_xz(Vec2::new(uv[0], uv[1]), focus);
        orbit.centre[0] = x;
        orbit.centre[2] = z;

        let cx = (x / LIVE_CHUNK_EDGE as f32).floor() as i32;
        let cz = (z / LIVE_CHUNK_EDGE as f32).floor() as i32;
        let preferred_cy = (orbit.centre[1] / LIVE_CHUNK_EDGE as f32).floor() as i32;
        let loaded: Vec<u64> = scene.chunks.keys().copied().collect();
        hud.snapshot.focused_chunk = Some(focused_chunk_at_grid(cx, cz, preferred_cy, &loaded));
        return;
    }

    let Some(bounds) = cache.bounds else {
        return;
    };

    let (cx, cz) = minimap_uv_to_chunk_grid(inset_minimap_uv_from_cursor(normalized), bounds);
    orbit.centre[0] = (cx as f32 + 0.5) * LIVE_CHUNK_EDGE as f32;
    orbit.centre[2] = (cz as f32 + 0.5) * LIVE_CHUNK_EDGE as f32;

    let preferred_cy = (orbit.centre[1] / LIVE_CHUNK_EDGE as f32).floor() as i32;
    let loaded: Vec<u64> = scene.chunks.keys().copied().collect();
    hud.snapshot.focused_chunk = Some(focused_chunk_at_grid(cx, cz, preferred_cy, &loaded));
}

fn viewport_chunk_raycast(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    #[cfg(feature = "egui")]
    settings: Option<Res<GameSettings>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    minimap: Query<&Interaction, With<MinimapPanel>>,
    _orbit: Res<OrbitCamera>,
    mut hud: ResMut<HudState>,
) {
    let select_pressed = action_pressed(
        #[cfg(feature = "egui")]
        settings.as_deref(),
        civ_bevy_ref::settings_ui::ACTION_SELECT_OR_PICK,
        KeyBinding::Mouse(MouseButton::Left),
        &keys,
        &mouse,
    );
    if !select_pressed {
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
    if let Some(chunk) = chunk_raycast_terrain(origin, direction, VOXEL_CHUNK_EDGE) {
        hud.snapshot.focused_chunk = Some(chunk);
    }
}

fn update_chunk_fade(
    time: Res<Time>,
    debug: Res<DebugRender>,
    mut commands: Commands,
    mut fades: Query<(
        Entity,
        &mut LiveChunkFade,
        &MeshMaterial3d<StandardMaterial>,
    )>,
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
            commands.entity(entity).remove::<LiveChunkFade>();
        }
    }
}

fn minimap_popup_ui(
    mut contexts: EguiContexts,
    mut popup: ResMut<MinimapPopup>,
    bridge: Res<LiveBridge>,
    mut orbit: ResMut<OrbitCamera>,
    mut hud: ResMut<HudState>,
    scene: Res<LiveStreamScene>,
) {
    let Some((tx, ty)) = popup.pending else {
        return;
    };
    egui::Window::new("Tile Actions")
        .collapsible(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Tile ({tx}, {ty})"));
            if ui.button("Inspect tile").clicked() {
                let json = format!(
                    r#"{{"jsonrpc":"2.0","id":1,"method":"sim.inspect_tile","params":{{"x":{tx},"y":{ty}}}}}"#
                );
                bridge.client.send_rpc_raw(json);
                popup.pending = None;
            }
            if ui.button("Center camera").clicked() {
                orbit.centre[0] = tx as f32;
                orbit.centre[2] = ty as f32;
                let preferred_cy = (orbit.centre[1] / LIVE_CHUNK_EDGE as f32).floor() as i32;
                let loaded: Vec<u64> = scene.chunks.keys().copied().collect();
                hud.snapshot.focused_chunk = Some(focused_chunk_at_grid(
                    tx / LIVE_CHUNK_EDGE as i32,
                    ty / LIVE_CHUNK_EDGE as i32,
                    preferred_cy,
                    &loaded,
                ));
                popup.pending = None;
            }
            if ui.button("Cancel").clicked() {
                popup.pending = None;
            }
        });
}

fn poll_emergence(
    time: Res<Time>,
    bridge: Res<LiveBridge>,
    mut timer: ResMut<EmergencePollTimer>,
    mut hud: ResMut<HudState>,
    speed: Res<SimSpeedState>,
    mut emergence_res: ResMut<EmergenceHudData>,
) {
    hud.snapshot.speed_multiplier = speed.multiplier;
    // Apply any parsed emergence responses received from the server.
    for em in bridge.client.poll_emergence() {
        hud.snapshot.emergence = Some(em.clone());
        *emergence_res = em;
        hud.snapshot.emergence = Some(em);
    }
    timer.0 += time.delta_secs();
    if timer.0 < 10.0 {
        return;
    }
    timer.0 = 0.0;
    let json = r#"{"jsonrpc":"2.0","id":2,"method":"sim.emergence","params":null}"#.to_string();
    bridge.client.send_rpc_raw(json);
}
