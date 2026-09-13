use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, RenderTarget, ScalingMode};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::NoIndirectDrawing;
use bevy::ui::widget::ImageNode;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
#[cfg(test)]
use bevy::window::WindowResolution;
use civ_agents::{Alignment, Civilian as AgentCivilian};
use civ_engine::Building;

use crate::camera::CameraRig;
use crate::live_stream::ServerBridge;
use crate::sim_bridge::SimState;
use crate::spawn_tools::{select_action_binding, GameSettings};
use crate::terrain::WORLD_SIZE;
use crate::AttachMode;

/// Minimap side length in UI pixels.
pub const MINIMAP_SIZE: f32 = 200.0;
const MINIMAP_INSET: f32 = 8.0;
const MINIMAP_WORLD_MIN: f32 = 0.0;
const MINIMAP_WORLD_MAX: f32 = 256.0;
const MINIMAP_CIVILIAN_DOT: f32 = 4.0;
const MINIMAP_BUILDING_DOT: f32 = 5.0;
const MINIMAP_TEXTURE_SIZE: u32 = 256;
const MINIMAP_CAMERA_HEIGHT: f32 = 180.0;

#[derive(Resource, Clone)]
struct MinimapRenderTarget {
    image: Handle<Image>,
}

#[derive(Component)]
pub struct MinimapRoot;

#[derive(Component)]
pub struct MinimapDot;

#[derive(Component)]
pub struct MinimapCamera;

/// Marker for the rectangular viewport indicator overlay drawn on top of the
/// minimap terrain. Its `Node` rect is updated each frame to outline the area
/// visible to the main camera, projected to the world XZ plane.
#[derive(Component)]
pub struct MinimapViewport;

/// Vertical field-of-view (degrees) used to project the main camera's visible
/// rectangle onto the minimap. Matches the Bevy `PerspectiveProjection` default
/// and the value set in `setup_minimap_render_target`'s analogue for the main
/// camera (`Camera3d::default`).
const MINIMAP_VIEWPORT_FOV_DEG: f32 = 45.0;

/// Plugin that renders a top-down minimap and lets the player click to teleport the main camera.
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapSnapshotState>()
            .add_systems(
                Startup,
                (setup_minimap_render_target, setup_minimap).chain(),
            )
            .add_systems(
                Update,
                (
                    sync_minimap_dots,
                    update_minimap_viewport,
                    teleport_camera_from_minimap,
                ),
            )
            // Server-mode: keep the minimap terrain in sync with the live server
            // snapshot. The polling interval (5 s) is slower than the ws client's
            // own `sim.snapshot` poll (2 s) so this just enforces a fresh fetch
            // for the minimap texture; if the ws-client is disconnected the
            // periodic call still tries to fire (fire-and-forget) so when the
            // server comes back the minimap catches up immediately.
            .add_systems(Update, request_minimap_snapshot);
    }
}

/// Seconds between minimap-initiated `sim.snapshot` RPCs in server mode.
const MINIMAP_SNAPSHOT_INTERVAL_SECS: f32 = 5.0;
/// Resource tracking the last snapshot fire time so we can throttle the poll.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct MinimapSnapshotState {
    /// Seconds elapsed (game clock) when the last `sim.snapshot` was fired.
    /// `None` means we haven't fired yet.
    pub last_fired_secs: Option<f32>,
}

/// Periodically request a fresh `sim.snapshot` so the minimap terrain texture
/// stays in sync with the live server view. Runs only in server attach mode —
/// standalone mode reads from the in-process simulation directly.
fn request_minimap_snapshot(
    attach: Res<AttachMode>,
    bridge: Option<Res<ServerBridge>>,
    mut state: ResMut<MinimapSnapshotState>,
    time: Res<Time>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    let Some(ref bridge) = bridge else {
        return;
    };
    let now = time.elapsed_secs();
    let due = match state.last_fired_secs {
        None => true,
        Some(prev) => now - prev >= MINIMAP_SNAPSHOT_INTERVAL_SECS,
    };
    if due {
        bridge.send_rpc("sim.snapshot", serde_json::json!({}));
        state.last_fired_secs = Some(now);
    }
}

fn setup_minimap_render_target(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let extent = Extent3d {
        width: MINIMAP_TEXTURE_SIZE,
        height: MINIMAP_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        extent,
        TextureDimension::D2,
        &[24, 32, 40, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    // A camera render target must advertise RENDER_ATTACHMENT; the default
    // texture usages (TEXTURE_BINDING | COPY_SRC | COPY_DST) are insufficient
    // and wgpu 27 rejects the color attachment otherwise.
    image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    let handle = images.add(image);
    commands.insert_resource(MinimapRenderTarget {
        image: handle.clone(),
    });

    commands.spawn((
        Camera3d::default(),
        // Keep the offscreen minimap on the same direct-draw compatibility
        // path as the main standalone camera.
        NoIndirectDrawing,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgba(0.05, 0.08, 0.12, 1.0)),
            ..default()
        },
        RenderTarget::Image(handle.into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WORLD_SIZE,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, MINIMAP_CAMERA_HEIGHT, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        MinimapCamera,
    ));
}

fn setup_minimap(mut commands: Commands, minimap_target: Res<MinimapRenderTarget>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(MINIMAP_INSET),
                bottom: Val::Px(MINIMAP_INSET),
                width: Val::Px(MINIMAP_SIZE),
                height: Val::Px(MINIMAP_SIZE),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.06, 0.94)),
            BorderColor::all(Color::srgba(0.35, 0.42, 0.50, 0.75)),
            Interaction::default(),
            RelativeCursorPosition::default(),
            FocusPolicy::Pass,
            MinimapRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(minimap_target.image.clone()),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
            // Viewport indicator overlay: a transparent rect whose border
            // outlines the main camera's visible area on the world XZ plane.
            // Sized/positioned by `update_minimap_viewport` each frame.
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(0.0),
                    height: Val::Px(0.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(1.0, 0.95, 0.55, 0.95)),
                BackgroundColor(Color::srgba(1.0, 0.95, 0.55, 0.10)),
                FocusPolicy::Pass,
                MinimapViewport,
            ));
        });
}

fn world_to_minimap_uv(position: Vec3) -> Vec2 {
    let u = ((position.x - MINIMAP_WORLD_MIN) / (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN))
        .clamp(0.0, 1.0);
    let v = ((position.z - MINIMAP_WORLD_MIN) / (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN))
        .clamp(0.0, 1.0);
    Vec2::new(u, 1.0 - v)
}

fn minimap_uv_to_world(uv: Vec2) -> Vec3 {
    let x = MINIMAP_WORLD_MIN + uv.x * (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN);
    let z = MINIMAP_WORLD_MIN + (1.0 - uv.y) * (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN);
    Vec3::new(x, 0.0, z)
}

fn civilian_faction_id(civilian: &AgentCivilian) -> Option<u32> {
    match civilian.alignment {
        Alignment::Faction(faction) => Some(faction),
        _ => None,
    }
}

fn civilian_color(civilian: &AgentCivilian) -> Color {
    let hue = civilian_faction_id(civilian).unwrap_or(0) as f32 * 85.0 % 360.0;
    Color::hsla(hue, 0.75, 0.58, 1.0)
}

fn world_position_for_civilian(
    _civilian: &AgentCivilian,
    position: &civ_agents::Position3d,
) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    let half = MINIMAP_WORLD_MAX * 0.5;
    Vec3::new(
        position.coord.x as f32 / scale * MINIMAP_WORLD_MAX - half,
        0.0,
        position.coord.z as f32 / scale * MINIMAP_WORLD_MAX - half,
    )
}

fn world_position_for_building(building: &Building) -> Vec3 {
    let half = MINIMAP_WORLD_MAX * 0.5;
    Vec3::new(
        ((building.position.x as f32 + 64.0) / 127.0).clamp(0.0, 1.0) * MINIMAP_WORLD_MAX - half,
        0.0,
        ((building.position.y as f32 + 64.0) / 127.0).clamp(0.0, 1.0) * MINIMAP_WORLD_MAX - half,
    )
}

fn sync_minimap_dots(
    attach: Res<AttachMode>,
    sim: Option<Res<SimState>>,
    mut commands: Commands,
    roots: Query<Entity, With<MinimapRoot>>,
    existing: Query<Entity, With<MinimapDot>>,
    // Server-mode queries: read positions from live-streamed entities.
    live_agents: Query<
        (&crate::live_stream::LiveAgentTag, &Transform),
        With<crate::live_stream::LiveAgentTag>,
    >,
    live_buildings: Query<
        (&crate::live_stream::LiveBuildingTag, &Transform),
        With<crate::live_stream::LiveBuildingTag>,
    >,
) {
    // In server mode, always re-sync from live-streamed entity transforms.
    let is_server = *attach == AttachMode::Server;
    if !is_server
        && !sim
            .as_ref()
            .expect("standalone minimap requires SimState")
            .is_changed()
        && !attach.is_changed()
    {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Ok(root) = roots.single() else {
        return;
    };

    commands.entity(root).with_children(|parent| {
        if is_server {
            // Draw dots from live-streamed agent positions.
            for (_tag, transform) in live_agents.iter() {
                let pos = transform.translation;
                let uv = world_to_minimap_uv(pos);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_CIVILIAN_DOT * 0.5),
                        top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_CIVILIAN_DOT * 0.5),
                        width: Val::Px(MINIMAP_CIVILIAN_DOT),
                        height: Val::Px(MINIMAP_CIVILIAN_DOT),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::hsla(0.0, 0.75, 0.58, 1.0)),
                    MinimapDot,
                    FocusPolicy::Pass,
                ));
            }

            // Draw dots from live-streamed building positions.
            for (_tag, transform) in live_buildings.iter() {
                let pos = transform.translation;
                let uv = world_to_minimap_uv(pos);
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_BUILDING_DOT * 0.5),
                        top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_BUILDING_DOT * 0.5),
                        width: Val::Px(MINIMAP_BUILDING_DOT),
                        height: Val::Px(MINIMAP_BUILDING_DOT),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.92, 0.90, 0.86)),
                    MinimapDot,
                    FocusPolicy::Pass,
                ));
            }
        } else {
            // Standalone mode: read directly from the in-process simulation.
            let sim = sim.as_ref().expect("standalone minimap requires SimState");
            for (_, (civilian, position)) in sim
                .0
                .world
                .query::<(&AgentCivilian, &civ_agents::Position3d)>()
                .iter()
            {
                let uv = world_to_minimap_uv(world_position_for_civilian(civilian, position));
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_CIVILIAN_DOT * 0.5),
                        top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_CIVILIAN_DOT * 0.5),
                        width: Val::Px(MINIMAP_CIVILIAN_DOT),
                        height: Val::Px(MINIMAP_CIVILIAN_DOT),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(civilian_color(civilian)),
                    MinimapDot,
                    FocusPolicy::Pass,
                ));
            }

            for (_, building) in sim.0.world.query::<&Building>().iter() {
                let uv = world_to_minimap_uv(world_position_for_building(building));
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(uv.x * MINIMAP_SIZE - MINIMAP_BUILDING_DOT * 0.5),
                        top: Val::Px(uv.y * MINIMAP_SIZE - MINIMAP_BUILDING_DOT * 0.5),
                        width: Val::Px(MINIMAP_BUILDING_DOT),
                        height: Val::Px(MINIMAP_BUILDING_DOT),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    MinimapDot,
                    FocusPolicy::Pass,
                ));
            }
        }
    });
}

fn teleport_camera_from_minimap(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Option<Res<GameSettings>>,
    panel: Query<&RelativeCursorPosition, With<MinimapRoot>>,
    mut rig: ResMut<CameraRig>,
) {
    let select_pressed = select_action_binding(settings.as_deref()).is_just_pressed(&keys, &mouse);

    if !select_pressed {
        return;
    }

    let Ok(cursor) = panel.single() else {
        return;
    };
    let Some(normalized) = cursor.normalized else {
        return;
    };

    let world = minimap_uv_to_world(normalized);
    rig.target.x = world.x;
    rig.target.z = world.z;
}

/// Project the main camera's ground-plane footprint onto the minimap and update
/// the `MinimapViewport` rect to outline that area. Uses the camera's `target`
/// + `distance` + `pitch` + `yaw` from `CameraRig`, and the window's logical
/// aspect ratio for the horizontal extent. The indicator stays inside the
/// minimap even when the camera is over-zoomed or the rig is pointed past
/// the world bounds.
fn update_minimap_viewport(
    rig: Res<CameraRig>,
    windows: Query<&Window>,
    mut indicators: Query<&mut Node, With<MinimapViewport>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut node) = indicators.single_mut() else {
        return;
    };

    let aspect = if window.height() > 0.0 {
        window.width() / window.height()
    } else {
        1.0
    };

    // Camera distance is measured along the view direction; the distance from
    // the rig target to the ground plane along that direction is
    // `distance * cos(pitch)`. The visible half-height on the ground plane is
    // that distance times tan(fov/2); visible half-width is half-height * aspect.
    let ground_distance = (rig.distance * rig.pitch.cos()).max(1.0);
    let half_height = ground_distance * (MINIMAP_VIEWPORT_FOV_DEG * 0.5).to_radians().tan();
    let half_width = half_height * aspect;
    let world_height = half_height * 2.0;
    let world_width = half_width * 2.0;

    // Convert the world-space footprint to minimap-pixel position. The viewport
    // box is centered on `rig.target` projected via the same UV mapping used by
    // the dots, then we clamp the pixel rect to the minimap bounds so the
    // indicator never spills outside when the camera sees beyond the world edge.
    let uv_center = world_to_minimap_uv(rig.target);
    let center_px = Vec2::new(uv_center.x, uv_center.y) * MINIMAP_SIZE;
    // UV -> pixel scale: `world_size / UV_UNITS_PER_WORLD`. The UV mapping uses
    // the world span 0..MINIMAP_WORLD_MAX and renders at MINIMAP_SIZE pixels,
    // so 1 world unit = (MINIMAP_SIZE / MINIMAP_WORLD_MAX) pixels.
    let px_per_world = MINIMAP_SIZE / (MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN).max(1.0);
    let width_px = (world_width * px_per_world).clamp(8.0, MINIMAP_SIZE);
    let height_px = (world_height * px_per_world).clamp(8.0, MINIMAP_SIZE);

    let left = (center_px.x - width_px * 0.5).clamp(0.0, MINIMAP_SIZE - width_px);
    let top = (center_px.y - height_px * 0.5).clamp(0.0, MINIMAP_SIZE - height_px);
    node.left = Val::Px(left);
    node.top = Val::Px(top);
    node.width = Val::Px(width_px);
    node.height = Val::Px(height_px);
}

#[cfg(test)]
mod attach_mode_tests {
    use super::*;
    use crate::live_stream::{LiveAgentTag, LiveBuildingTag};

    #[test]
    fn server_minimap_uses_only_streamed_entities_without_local_state() {
        let mut app = App::new();
        app.insert_resource(AttachMode::Server)
            .add_systems(Update, sync_minimap_dots);
        app.world_mut().spawn((Node::default(), MinimapRoot));
        let agent = app
            .world_mut()
            .spawn((
                LiveAgentTag { id: 777 },
                Transform::from_xyz(32.0, 8.0, -16.0),
            ))
            .id();
        app.world_mut().spawn((
            LiveBuildingTag { id: 888 },
            Transform::from_xyz(-32.0, 0.0, 16.0),
        ));
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<MinimapDot>>()
                .iter(app.world())
                .count(),
            2
        );
        assert!(!app.world().contains_resource::<SimState>());

        // An incidental local resource must never add its default population.
        app.insert_resource(SimState::default());
        app.world_mut().despawn(agent);
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<MinimapDot>>()
                .iter(app.world())
                .count(),
            1
        );
    }

    #[test]
    fn standalone_minimap_keeps_local_population() {
        let mut app = App::new();
        let sim = SimState::default();
        let expected = sim
            .0
            .world
            .query::<(&AgentCivilian, &civ_agents::Position3d)>()
            .iter()
            .count()
            + sim.0.world.query::<&Building>().iter().count();
        assert!(expected > 0);
        app.insert_resource(AttachMode::Standalone)
            .insert_resource(sim)
            .add_systems(Update, sync_minimap_dots);
        app.world_mut().spawn((Node::default(), MinimapRoot));
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<MinimapDot>>()
                .iter(app.world())
                .count(),
            expected
        );
    }
}

#[cfg(test)]
mod viewport_indicator_tests {
    use super::*;

    /// Build an app with the viewport system, a tiny camera distance, and a
    /// 16:9 window. Returns the indicator's `Node` after one update.
    fn run_indicator(rig: CameraRig, window_size: (f32, f32)) -> Node {
        let mut app = App::new();
        app.insert_resource(rig);
        // Bevy Window resource needs concrete physical / logical size before
        // `Window::width()/height()` work. Provide a minimal one.
        let (w, h) = window_size;
        app.world_mut().spawn(Window {
            resolution: WindowResolution::new(w as u32, h as u32),
            ..default()
        });
        app.add_systems(Update, update_minimap_viewport);
        app.world_mut().spawn((Node::default(), MinimapViewport));
        app.update();
        let mut q = app
            .world_mut()
            .query_filtered::<&Node, With<MinimapViewport>>();
        let node = q.single(app.world()).expect("viewport indicator");
        node.clone()
    }

    #[test]
    fn indicator_keeps_camera_centered_at_default_target() {
        let rig = CameraRig::default();
        let node = run_indicator(rig, (1600.0, 900.0));
        // The `world_to_minimap_uv` mapping uses world span 0..MINIMAP_WORLD_MAX,
        // so the rig's default target of (0,0,0) projects to UV (0,1) which in
        // flipped-V UI space lands at the bottom-left of the minimap. The
        // indicator's centre should land there, NOT the geometric centre.
        let left = match node.left {
            Val::Px(v) => v,
            _ => panic!("expected pixel left"),
        };
        let top = match node.top {
            Val::Px(v) => v,
            _ => panic!("expected pixel top"),
        };
        let width = match node.width {
            Val::Px(v) => v,
            _ => panic!("expected pixel width"),
        };
        let height = match node.height {
            Val::Px(v) => v,
            _ => panic!("expected pixel height"),
        };
        let cx = left + width * 0.5;
        let cy = top + height * 0.5;
        // Bottom-left of the minimap: cx should be a small positive value
        // (close to the indicator's half-width), cy at MINIMAP_SIZE - half-height.
        assert!(
            cx < width + 2.0,
            "indicator cx={} should be at the bottom-left (~width/2), got > width+2",
            cx,
        );
        assert!(
            cy > MINIMAP_SIZE - height - 2.0,
            "indicator cy={} should be near the bottom of the minimap (MINIMAP_SIZE - height/2 = {}), got < {}",
            cy,
            MINIMAP_SIZE - height,
            MINIMAP_SIZE - height - 2.0,
        );
    }

    #[test]
    fn indicator_stays_within_minimap_bounds_when_zoomed_out() {
        let mut rig = CameraRig::default();
        rig.distance = 900.0; // pushed to the max zoom
        let node = run_indicator(rig, (1600.0, 900.0));
        // Far zoom produces a wide world footprint → the clamped pixel size
        // should be at MINIMAP_SIZE — confirming the safety clamp works.
        let width = match node.width {
            Val::Px(v) => v,
            _ => panic!("expected pixel width"),
        };
        let height = match node.height {
            Val::Px(v) => v,
            _ => panic!("expected pixel height"),
        };
        assert!(
            width <= MINIMAP_SIZE + f32::EPSILON,
            "indicator width={} should not exceed minimap size {}",
            width,
            MINIMAP_SIZE,
        );
        assert!(
            height <= MINIMAP_SIZE + f32::EPSILON,
            "indicator height={} should not exceed minimap size {}",
            height,
            MINIMAP_SIZE,
        );
    }

    #[test]
    fn indicator_pixels_grow_when_aspect_widens() {
        let mut rig = CameraRig::default();
        rig.distance = 60.0;
        let wide = run_indicator(rig, (2560.0, 900.0));
        let tall = run_indicator(rig, (900.0, 900.0));
        let wide_w = match wide.width {
            Val::Px(v) => v,
            _ => panic!("expected pixel width"),
        };
        let tall_w = match tall.width {
            Val::Px(v) => v,
            _ => panic!("expected pixel width"),
        };
        let wide_h = match wide.height {
            Val::Px(v) => v,
            _ => panic!("expected pixel height"),
        };
        let tall_h = match tall.height {
            Val::Px(v) => v,
            _ => panic!("expected pixel height"),
        };
        // Wider aspect -> strictly wider indicator; vertical extent identical.
        assert!(
            wide_w > tall_w,
            "wide aspect indicator width={} should exceed square aspect width={}",
            wide_w,
            tall_w,
        );
        assert!(
            (wide_h - tall_h).abs() < 0.5,
            "vertical extent must track FOV alone (wide_h={}, tall_h={})",
            wide_h,
            tall_h,
        );
    }
}
