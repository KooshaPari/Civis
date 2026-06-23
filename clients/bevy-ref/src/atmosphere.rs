use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

use crate::terrain::{lerp, WATER_LEVEL};

const DAY_LENGTH_SECONDS: f32 = 10.0 * 60.0;
const STAR_COUNT: usize = 240;
const STAR_SHELL_RADIUS: f32 = 1_500.0;

#[derive(Resource, Clone, Copy)]
pub struct DayNightCycle {
    time_of_day: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self { time_of_day: 0.0 }
    }
}

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct MoonLight;

#[derive(Component)]
pub struct StarField;

#[derive(Component)]
pub struct WaterSurface;

pub fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::srgba(0.54, 0.74, 0.92, 1.0)));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        affects_lightmapped_meshes: true,
    });
    commands.insert_resource(DayNightCycle::default());

    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 8.0, 0.0)),
    ));

    commands.spawn((
        MoonLight,
        Visibility::Hidden,
        DirectionalLight {
            illuminance: 500.0,
            shadows_enabled: false,
            color: Color::srgb(0.35, 0.45, 0.75).into(),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PI / 4.0, -PI / 8.0, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 2.0 }))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.05, 0.05),
            ..default()
        })),
        Transform::from_xyz(128.0, 20.0, 128.0),
    ));

    commands.spawn((
        WaterSurface,
        Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Plane3d::default().mesh().size(256.0, 256.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.16, 0.34, 0.55, 0.78),
            perceptual_roughness: 0.12,
            reflectance: 0.08,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0.0, WATER_LEVEL - 0.8, 0.0),
    ));

    let star_mesh = meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: 1.0 }));
    let star_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE.into(),
        unlit: true,
        ..default()
    });
    commands.spawn((StarField, Visibility::Hidden)).with_children(|parent| {
        for i in 0..STAR_COUNT {
            let (theta, phi) = star_angles(i as u32);
            let dir = Vec3::new(
                theta.cos() * phi.sin(),
                phi.cos(),
                theta.sin() * phi.sin(),
            );
            parent.spawn((
                Mesh3d(star_mesh.clone()),
                MeshMaterial3d(star_material.clone()),
                Transform::from_translation(dir * STAR_SHELL_RADIUS).with_scale(Vec3::splat(0.75)),
            ));
        }
    });
}

pub fn animate_water(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<WaterSurface>>,
    mut cycle: ResMut<DayNightCycle>,
) {
    let delta = time.delta_secs() / DAY_LENGTH_SECONDS;
    cycle.time_of_day = (cycle.time_of_day + delta).fract();
    for mut transform in &mut query {
        transform.translation.y = WATER_LEVEL - 0.8 + (time.elapsed_secs() * 1.6).sin() * 0.06;
    }
}

pub fn update_lighting(
    cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
    mut sun_query: Query<&mut DirectionalLight, With<SunLight>>,
    mut sun_transform_query: Query<&mut Transform, (With<SunLight>, Without<MoonLight>)>,
    mut moon_query: Query<(&mut DirectionalLight, &mut Transform, &mut Visibility), With<MoonLight>>,
    mut star_query: Query<&mut Visibility, With<StarField>>,
) {
    let t = cycle.time_of_day;
    let sun_angle = t * TAU - PI * 0.5;
    let sun_dir = Vec3::new(sun_angle.cos(), sun_angle.sin(), 0.35).normalize();
    let moon_dir = -sun_dir;
    let daylight = ((sun_dir.y + 0.15) / 1.15).clamp(0.0, 1.0);
    let sun_color = lerp_color(Color::srgb(1.0, 0.42, 0.18), Color::WHITE, daylight);
    let dusk_color = Color::srgb(0.85, 0.16, 0.12);
    let dawn_weight = smoothstep(0.0, 0.16, t) * (1.0 - smoothstep(0.34, 0.5, t));
    let dusk_weight = smoothstep(0.5, 0.66, t) * (1.0 - smoothstep(0.84, 1.0, t));
    let night_blue = Color::srgb(0.03, 0.06, 0.16);
    let sky_day = Color::srgb(0.53, 0.76, 0.95);
    let sky_dusk = Color::srgb(0.78, 0.37, 0.23);
    let sky_dawn = Color::srgb(0.94, 0.61, 0.34);
    let sky = blend_colors(
        blend_colors(night_blue, sky_dawn, dawn_weight),
        sky_dusk,
        dusk_weight,
    );
    clear_color.0 = blend_colors(sky, sky_day, daylight);

    if let Ok(mut sun_light) = sun_query.single_mut() {
        sun_light.color = if daylight > 0.55 {
            Color::WHITE.into()
        } else if dawn_weight > dusk_weight {
            sun_color.into()
        } else if dusk_weight > 0.0 {
            dusk_color.into()
        } else {
            Color::srgb(0.4, 0.5, 0.8).into()
        };
        sun_light.illuminance = if daylight > 0.1 { 15_000.0 * daylight.max(0.15) } else { 200.0 };
    }
    if let Ok(mut sun_transform) = sun_transform_query.single_mut() {
        *sun_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, sun_dir));
    }

    if let Ok((mut moon_light, mut moon_transform, mut moon_visibility)) = moon_query.single_mut() {
        let is_night = daylight < 0.1;
        *moon_visibility = if is_night { Visibility::Visible } else { Visibility::Hidden };
        moon_light.color = Color::srgb(0.35, 0.45, 0.75).into();
        moon_light.illuminance = if is_night { 500.0 } else { 0.0 };
        *moon_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, moon_dir));
    }

    if let Ok(mut stars_visibility) = star_query.single_mut() {
        *stars_visibility = if daylight < 0.1 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn blend_colors(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
        lerp(a.alpha, b.alpha, t),
    )
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    blend_colors(a, b, t.clamp(0.0, 1.0))
}

fn star_angles(seed: u32) -> (f32, f32) {
    let x = seed as f32 + 1.0;
    let u = crate::terrain::hash(x * 1.13, x * 0.37);
    let v = crate::terrain::hash(x * 0.73, x * 1.91);
    let theta = u * TAU;
    let phi = (v * 2.0 - 1.0).acos();
    (theta, phi)
}
