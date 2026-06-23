use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::ui::widget::ImageNode;
use civ_agents::Civilian as AgentCivilian;
use civ_engine::{Building, Simulation};

use crate::sim_bridge::SimState;

/// Minimap side length in UI pixels.
pub const MINIMAP_SIZE: f32 = 200.0;
const MINIMAP_TEXTURE_SIZE: u32 = 512;
const MINIMAP_INSET: f32 = 8.0;
const MINIMAP_WORLD_MIN: f32 = 0.0;
const MINIMAP_WORLD_MAX: f32 = 256.0;
const MINIMAP_CAMERA_HEIGHT: f32 = 300.0;
const MINIMAP_CIVILIAN_DOT: f32 = 4.0;
const MINIMAP_BUILDING_DOT: f32 = 5.0;

#[derive(Resource)]
struct MinimapTexture(Handle<Image>);

#[derive(Component)]
struct MinimapRoot;

#[derive(Component)]
struct MinimapImage;

#[derive(Component)]
struct MinimapDot;

/// Plugin that renders a top-down minimap and lets the player click to teleport the main camera.
pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_minimap)
            .add_systems(Update, (sync_minimap_dots, teleport_camera_from_minimap));
    }
}

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: MINIMAP_TEXTURE_SIZE,
        height: MINIMAP_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[8, 16, 24, 255],
        TextureFormat::Rgba8UnormSrgb,
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image = images.add(image);

    commands.insert_resource(MinimapTexture(image.clone()));

    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 10,
            target: image.clone().into(),
            clear_color: ClearColorConfig::Custom(Color::srgb(0.03, 0.06, 0.09)),
            ..default()
        },
        Transform::from_xyz(128.0, MINIMAP_CAMERA_HEIGHT, 128.0)
            .looking_at(Vec3::new(128.0, 0.0, 128.0), Vec3::Z),
    ));

    let root = commands
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
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            ImageNode::new(image),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            MinimapImage,
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

fn world_position_for_civilian(civilian: &AgentCivilian, position: &civ_agents::Position3d) -> Vec3 {
    Vec3::new(position.coord.x as f32, 0.0, position.coord.z as f32)
}

fn world_position_for_building(building: &Building) -> Vec3 {
    Vec3::new(building.position.x as f32, 0.0, building.position.y as f32)
}

fn sync_minimap_dots(
    sim: Res<SimState>,
    mut commands: Commands,
    roots: Query<Entity, With<MinimapRoot>>,
    existing: Query<Entity, With<MinimapDot>>,
) {
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
        for (_, (civilian, position)) in sim.0.world.query::<(&AgentCivilian, &civ_agents::Position3d)>().iter()
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
    mut cameras: Query<&mut Transform, With<Camera3d>>,
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
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    camera.translation.x = world.x;
    camera.translation.z = world.z;
}
