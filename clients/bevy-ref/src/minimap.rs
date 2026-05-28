use bevy::asset::RenderAssetUsages;
use bevy::camera::{ClearColorConfig, RenderTarget, ScalingMode};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::widget::ImageNode;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use civ_agents::Civilian as AgentCivilian;
use civ_engine::Building;

use crate::sim_bridge::SimState;
use crate::camera::CameraRig;
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

/// Plugin that renders a top-down minimap and lets the player click to teleport the main camera.
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_minimap_render_target, setup_minimap).chain(),
        )
        .add_systems(Update, (sync_minimap_dots, teleport_camera_from_minimap));
    }
}

fn setup_minimap_render_target(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let extent = Extent3d {
        width: MINIMAP_TEXTURE_SIZE,
        height: MINIMAP_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let image = Image::new_fill(
        extent,
        TextureDimension::D2,
        &[24, 32, 40, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
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

fn world_position_for_civilian(_civilian: &AgentCivilian, position: &civ_agents::Position3d) -> Vec3 {
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

    commands.entity(root).with_children(|parent| {
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
    });
}

fn teleport_camera_from_minimap(
    mouse: Res<ButtonInput<MouseButton>>,
    panel: Query<&RelativeCursorPosition, With<MinimapRoot>>,
    mut rig: ResMut<CameraRig>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
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
