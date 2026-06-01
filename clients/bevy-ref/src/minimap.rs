use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, RenderTarget, ScalingMode};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::ui::widget::ImageNode;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use civ_agents::Civilian as AgentCivilian;
use civ_engine::Building;
use std::collections::HashMap;

use crate::camera::CameraRig;
use crate::info_views::{cluster_color, InfoViewRegistry};
use crate::sim_bridge::SimState;
use crate::terrain::{color_for_height, terrain_height, WORLD_SIZE};
use crate::AttachMode;

/// Minimap side length in UI pixels.
pub const MINIMAP_SIZE: f32 = 200.0;
/// Minimap inset from the viewport edge (px).
pub const MINIMAP_INSET: f32 = 8.0;
const MINIMAP_WORLD_MIN: f32 = 0.0;
const MINIMAP_WORLD_MAX: f32 = 256.0;
const MINIMAP_CIVILIAN_DOT: f32 = 4.0;
const MINIMAP_BUILDING_DOT: f32 = 5.0;
const MINIMAP_CLUSTER_DOT: f32 = 11.0;
const MINIMAP_TEXTURE_SIZE: u32 = 256;
const MINIMAP_CAMERA_HEIGHT: f32 = 180.0;
/// Resolution of the painted top-down terrain base texture (square).
const TERRAIN_TEX_SIZE: u32 = 128;

// ---------------------------------------------------------------------------
// Theme — mirror `ui_theme` palette as Bevy colors so the minimap frame matches
// the rest of the HUD. `ui_theme` exposes `egui::Color32` constants which are
// not usable on Bevy UI nodes, so we re-express the same sRGB values here.
// ---------------------------------------------------------------------------

/// Glass panel fill (matches `ui_theme::DECK_GLASS`).
const THEME_PANEL: Color = Color::srgba(0.078, 0.102, 0.141, 0.68);
/// Inactive inner hairline (matches `ui_theme::DECK_BORDER`).
#[allow(dead_code)]
const THEME_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.11);
/// Holo-cyan rim + viewport accent (matches `ui_theme::HOLO_CYAN`).
const THEME_HOLO: Color = Color::srgb(0.357, 0.890, 1.0);

#[derive(Resource, Clone)]
struct MinimapRenderTarget {
    image: Handle<Image>,
}

#[derive(Component)]
pub struct MinimapRoot;

/// The painted terrain base layer (material colors); sits beneath the live
/// render-target image and the marker overlay.
#[derive(Component)]
pub struct MinimapTerrain;

#[derive(Component)]
pub struct MinimapDot;

/// Translucent full-panel tint that matches the active info-view overlay.
#[derive(Component)]
pub struct MinimapOverlayTint;

/// The camera-viewport rectangle drawn over the minimap.
#[derive(Component)]
pub struct MinimapViewport;

#[derive(Component)]
pub struct MinimapCamera;

/// Plugin that renders a top-down minimap and lets the player click to teleport the main camera.
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_minimap_render_target, setup_minimap).chain(),
        )
        .add_systems(
            Update,
            (
                sync_minimap_visibility,
                sync_minimap_dots,
                sync_minimap_viewport,
                sync_overlay_tint,
                teleport_camera_from_minimap,
            ),
        );
    }
}

/// Hide the minimap UI whenever the world is not in-game (main menu / setup /
/// loading) so the title screen renders clean. The minimap is Bevy UI (not
/// egui), so it is gated by toggling [`Visibility`] on [`MinimapRoot`] rather
/// than a `run_if` on a draw system.
fn sync_minimap_visibility(
    mode: Res<crate::menus::GameUiMode>,
    mut root: Query<&mut Visibility, With<MinimapRoot>>,
) {
    use crate::menus::GameUiMode;
    let want = if matches!(*mode, GameUiMode::Playing | GameUiMode::Paused) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut root {
        if *vis != want {
            *vis = want;
        }
    }
}

/// Paint a CPU top-down texture of terrain material colors (water/sand/grass/
/// rock/snow via `terrain::color_for_height`). Used as the minimap base layer so
/// the map reads as terrain even before the live render camera produces a frame.
fn build_terrain_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = TERRAIN_TEX_SIZE;
    let mut data = vec![0u8; (size * size * 4) as usize];
    let span = MINIMAP_WORLD_MAX - MINIMAP_WORLD_MIN;
    for py in 0..size {
        for px in 0..size {
            // Texture row 0 is the top (north); world Z grows downward (south).
            let wx = MINIMAP_WORLD_MIN + (px as f32 + 0.5) / size as f32 * span;
            let wz = MINIMAP_WORLD_MIN + (py as f32 + 0.5) / size as f32 * span;
            let h = terrain_height(wx, wz);
            let c = color_for_height(h);
            let i = ((py * size + px) * 4) as usize;
            // BGRA channel order for Bgra8UnormSrgb.
            data[i] = (c[2] * 255.0) as u8;
            data[i + 1] = (c[1] * 255.0) as u8;
            data[i + 2] = (c[0] * 255.0) as u8;
            data[i + 3] = 255;
        }
    }
    let extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let image = Image::new(
        extent,
        TextureDimension::D2,
        data,
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    images.add(image)
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

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    minimap_target: Res<MinimapRenderTarget>,
) {
    let terrain_tex = build_terrain_texture(&mut images);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(MINIMAP_INSET),
                bottom: Val::Px(MINIMAP_INSET),
                width: Val::Px(MINIMAP_SIZE),
                height: Val::Px(MINIMAP_SIZE),
                border: UiRect::all(Val::Px(1.5)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(THEME_PANEL),
            BorderColor::all(THEME_HOLO),
            Interaction::default(),
            RelativeCursorPosition::default(),
            FocusPolicy::Pass,
            MinimapRoot,
        ))
        .with_children(|parent| {
            // Base layer: painted terrain material colors.
            parent.spawn((
                ImageNode::new(terrain_tex),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                MinimapTerrain,
                FocusPolicy::Pass,
            ));
            // Live layer: the orthographic render-camera frame, blended on top.
            parent.spawn((
                ImageNode::new(minimap_target.image.clone())
                    .with_color(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                FocusPolicy::Pass,
            ));
            // Overlay tint: matches the active info-view (hidden when off).
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                MinimapOverlayTint,
                FocusPolicy::Pass,
            ));
            // Camera viewport rectangle (positioned each frame).
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(THEME_HOLO),
                MinimapViewport,
                FocusPolicy::Pass,
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

fn civilian_color(civilian: &AgentCivilian) -> Color {
    let hue = (civilian.faction as f32 * 85.0) % 360.0;
    Color::hsla(hue, 0.75, 0.58, 1.0)
}

fn world_position_for_civilian(
    _civilian: &AgentCivilian,
    position: &civ_agents::Position3d,
) -> Vec3 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    Vec3::new(
        position.coord.x as f32 / scale,
        0.0,
        position.coord.z as f32 / scale,
    )
}

fn world_position_for_building(building: &Building) -> Vec3 {
    Vec3::new(building.position.x as f32, 0.0, building.position.y as f32)
}

fn sync_minimap_dots(
    attach: Res<AttachMode>,
    sim: Res<SimState>,
    mut commands: Commands,
    roots: Query<Entity, With<MinimapRoot>>,
    existing: Query<Entity, With<MinimapDot>>,
) {
    if *attach == AttachMode::Server {
        return;
    }
    if !sim.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let Ok(root) = roots.single() else {
        return;
    };

    // Accumulate per-faction centroids while we lay down civilian dots so we can
    // draw a cluster / settlement marker at each faction's centre of mass.
    let mut cluster_acc: HashMap<u32, (Vec2, u32)> = HashMap::new();

    commands.entity(root).with_children(|parent| {
        for (_, (civilian, position)) in sim
            .0
            .world
            .query::<(&AgentCivilian, &civ_agents::Position3d)>()
            .iter()
        {
            let uv = world_to_minimap_uv(world_position_for_civilian(civilian, position));
            let entry = cluster_acc.entry(civilian.faction).or_insert((Vec2::ZERO, 0));
            entry.0 += uv;
            entry.1 += 1;
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

        // Settlement / cluster markers: a ringed dot at each faction centroid,
        // tinted with the shared cluster palette so it matches the Territory
        // info-view.
        for (faction, (sum, count)) in cluster_acc {
            if count < 3 {
                continue; // ignore lone wanderers; only mark real clusters.
            }
            let centroid = sum / count as f32;
            let rgb = cluster_color(faction);
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(centroid.x * MINIMAP_SIZE - MINIMAP_CLUSTER_DOT * 0.5),
                    top: Val::Px(centroid.y * MINIMAP_SIZE - MINIMAP_CLUSTER_DOT * 0.5),
                    width: Val::Px(MINIMAP_CLUSTER_DOT),
                    height: Val::Px(MINIMAP_CLUSTER_DOT),
                    border: UiRect::all(Val::Px(1.5)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(rgb[0], rgb[1], rgb[2], 0.45)),
                BorderColor::all(Color::srgb(rgb[0], rgb[1], rgb[2])),
                MinimapDot,
                FocusPolicy::Pass,
            ));
        }
    });
}

/// Position + size the camera viewport rectangle from the main camera rig.
///
/// Approximates the on-ground footprint of the orbit camera as a box centred on
/// `rig.target`, sized from the orbit distance, then projects it to minimap UV.
fn sync_minimap_viewport(
    rig: Res<CameraRig>,
    mut viewport: Query<&mut Node, With<MinimapViewport>>,
) {
    let Ok(mut node) = viewport.single_mut() else {
        return;
    };
    // Footprint half-extent grows with stand-off distance (rough heuristic).
    let half = (rig.distance * 0.30).clamp(12.0, MINIMAP_WORLD_MAX * 0.5);
    let center = Vec3::new(rig.target.x, 0.0, rig.target.z);
    let min_uv = world_to_minimap_uv(center - Vec3::new(half, 0.0, half));
    let max_uv = world_to_minimap_uv(center + Vec3::new(half, 0.0, half));
    let left = min_uv.x.min(max_uv.x) * MINIMAP_SIZE;
    let top = min_uv.y.min(max_uv.y) * MINIMAP_SIZE;
    let w = (min_uv.x.max(max_uv.x) - min_uv.x.min(max_uv.x)) * MINIMAP_SIZE;
    let h = (min_uv.y.max(max_uv.y) - min_uv.y.min(max_uv.y)) * MINIMAP_SIZE;
    node.left = Val::Px(left);
    node.top = Val::Px(top);
    node.width = Val::Px(w.max(2.0));
    node.height = Val::Px(h.max(2.0));
}

/// Tint the whole minimap to match the active info-view overlay (or clear it).
///
/// Reads the active overlay's legend mid-stop colour as a representative tint.
/// Degrades gracefully: if no overlay is active (or the resource is absent) the
/// tint is fully transparent.
fn sync_overlay_tint(
    registry: Option<Res<InfoViewRegistry>>,
    mut tint: Query<&mut BackgroundColor, With<MinimapOverlayTint>>,
) {
    let Ok(mut bg) = tint.single_mut() else {
        return;
    };
    let color = registry
        .as_deref()
        .and_then(InfoViewRegistry::active_overlay)
        .map(|overlay| {
            // Use the legend midpoint as the representative overlay colour; the
            // Territory overlay has an empty legend, so fall back to accent.
            let rgb = overlay
                .legend
                .get(overlay.legend.len() / 2)
                .map(|s| s.rgb)
                .unwrap_or([0.337, 0.800, 0.949]);
            Color::srgba(rgb[0], rgb[1], rgb[2], 0.22)
        })
        .unwrap_or(Color::NONE);
    bg.0 = color;
}

fn teleport_camera_from_minimap(
    mouse: Res<ButtonInput<MouseButton>>,
    panel: Query<&RelativeCursorPosition, With<MinimapRoot>>,
    mut rig: ResMut<CameraRig>,
    #[cfg(feature = "egui")] mut egui_ctx: bevy_egui::EguiContexts,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // If egui owns the pointer (e.g. a HUD button was clicked), do not teleport.
    #[cfg(feature = "egui")]
    if let Ok(ctx) = egui_ctx.ctx_mut() {
        if ctx.wants_pointer_input() {
            return;
        }
    }

    let Ok(cursor) = panel.single() else {
        return;
    };

    // `normalized` is Some only when the cursor is over the node, but its
    // values can still be outside [0,1] if the node has overflow:visible.
    // Guard explicitly so clicks near (but outside) the bezel don't fire.
    let Some(normalized) = cursor.normalized else {
        return;
    };
    if normalized.x < 0.0 || normalized.x > 1.0 || normalized.y < 0.0 || normalized.y > 1.0 {
        return;
    }

    let world = minimap_uv_to_world(normalized);
    rig.target.x = world.x.clamp(MINIMAP_WORLD_MIN, MINIMAP_WORLD_MAX);
    rig.target.z = world.z.clamp(MINIMAP_WORLD_MIN, MINIMAP_WORLD_MAX);
}
