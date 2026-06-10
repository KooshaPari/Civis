use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

use crate::terrain::{lerp, WATER_LEVEL};

const DAY_LENGTH_SECONDS: f32 = 10.0 * 60.0;
const STAR_COUNT: usize = 240;
const STAR_SHELL_RADIUS: f32 = 1_500.0;
const SUN_KEY_DIR: Vec3 = Vec3::new(-0.4, 0.8, -0.3);

#[derive(Resource, Clone, Copy)]
pub struct DayNightCycle {
    time_of_day: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        // Boot at solar noon. NOTE: this module's `update_lighting` derives the
        // sun direction as `sun_angle = t*TAU - PI/2`, whose elevation
        // (`sin(sun_angle)`) peaks at t = 0.5 — so 0.5 is the true sun-overhead
        // noon here, even though the design-doc keyframe table labels noon 0.75.
        // Booting at 0.0 = sun below horizon was the black-window root cause:
        // 200 lx key + the sky dome multiplied to deep-blue night → the world
        // rendered effectively black on launch.
        Self { time_of_day: 0.5 }
    }
}

impl DayNightCycle {
    /// Snap presentation phase from a live `sim.snapshot` `is_day` flag.
    pub fn set_from_is_day(&mut self, is_day: bool) {
        // Sun elevation here peaks at t=0.5 (see `Default`/`update_lighting`),
        // so map "day" to 0.5 for a true overhead noon and "night" to 0.0
        // (sun fully below horizon).
        self.time_of_day = if is_day { 0.5 } else { 0.0 };
    }

    /// Normalised time-of-day phase in `0.0..1.0` (noon ≈ 0.75, midnight ≈ 0.25).
    ///
    /// Exposed so presentation layers (e.g. the HDR sky dome in [`crate::skybox`])
    /// can darken/brighten with the cycle instead of guessing a fixed phase.
    #[must_use]
    pub fn time_of_day(&self) -> f32 {
        self.time_of_day
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
    // Ambient floor (cool navy #0D1628 @ low brightness) per
    // docs/design/lighting-biomes-art.md §1.2 — just lifts black crevices
    // toward the brand-cool shadow family; the warm sun is the key, the cool
    // sky dome is the fill. The old `WHITE @ 500` neutral-on-neutral rig was
    // why the world read flat.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.051, 0.086, 0.157),
        brightness: 280.0,
        affects_lightmapped_meshes: true,
    });
    commands.insert_resource(DayNightCycle::default());

    commands.spawn((
        SunLight,
        DirectionalLight {
            // Warm key #FFF4E0 (≈5400K) @ 32 000 lx noon target (§1.2).
            color: Color::srgb(1.000, 0.957, 0.878).into(),
            illuminance: 32_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, SUN_KEY_DIR)),
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

    // (Removed a stray red debug sphere that floated at world-centre.)

    // Flat water plane is the HEIGHTMAP fallback only — under the `voxel`
    // feature the volumetric voxel world owns hydrology, so the plane would
    // be a stray flat "billboard" over the real water. Gate it out.
    #[cfg(not(feature = "voxel"))]
    commands.spawn((
        WaterSurface,
        Mesh3d(
            meshes.add(Mesh::from(
                bevy::math::primitives::Plane3d::default()
                    .mesh()
                    .size(256.0, 256.0),
            )),
        ),
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
    commands
        .spawn((StarField, Transform::default(), Visibility::Hidden))
        .with_children(|parent| {
            for i in 0..STAR_COUNT {
                let (theta, phi) = star_angles(i as u32);
                let dir = Vec3::new(theta.cos() * phi.sin(), phi.cos(), theta.sin() * phi.sin());
                parent.spawn((
                    Mesh3d(star_mesh.clone()),
                    MeshMaterial3d(star_material.clone()),
                    Transform::from_translation(dir * STAR_SHELL_RADIUS)
                        .with_scale(Vec3::splat(0.75)),
                ));
            }
        });
}

pub fn animate_water(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<WaterSurface>>,
    mut cycle: ResMut<DayNightCycle>,
    live: Option<Res<crate::live_attach::LiveAttachState>>,
) {
    let live_connected = live.map(|state| state.connected).unwrap_or(false);
    if !live_connected {
        let delta = time.delta_secs() / DAY_LENGTH_SECONDS;
        cycle.time_of_day = (cycle.time_of_day + delta).fract();
    }
    for mut transform in &mut query {
        transform.translation.y = WATER_LEVEL - 0.8 + (time.elapsed_secs() * 1.6).sin() * 0.06;
    }
}

pub fn update_lighting(
    cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun_query: Query<&mut DirectionalLight, (With<SunLight>, Without<MoonLight>)>,
    mut sun_transform_query: Query<&mut Transform, (With<SunLight>, Without<MoonLight>)>,
    mut moon_query: Query<
        (&mut DirectionalLight, &mut Transform, &mut Visibility),
        (With<MoonLight>, Without<SunLight>, Without<StarField>),
    >,
    mut star_query: Query<&mut Visibility, (With<StarField>, Without<MoonLight>)>,
) {
    let t = cycle.time_of_day;
    let sun_angle = t * TAU - PI * 0.5;
    let sun_dir = Vec3::new(
        sun_angle.cos().mul_add(0.4, SUN_KEY_DIR.x),
        (sun_angle.sin() * 0.2 + SUN_KEY_DIR.y).clamp(-1.0, 1.0),
        sun_angle.cos().mul_add(0.3, SUN_KEY_DIR.z),
    )
    .normalize();
    let moon_dir = -sun_dir;
    let daylight = ((sun_dir.y + 0.15) / 1.15).clamp(0.0, 1.0);
    // Warm key per docs/design/lighting-biomes-art.md §1.2/§3: the sun warms to
    // ember (#F56839) near the horizon and rolls to warm-white (#FFF4E0) at
    // noon — never neutral white, so lit faces stay gold and shadows cool.
    let key_warm = Color::srgb(1.000, 0.957, 0.878); // #FFF4E0 noon
    let key_ember = Color::srgb(0.961, 0.408, 0.227); // #F56839 low sun
    let sun_color = lerp_color(key_ember, key_warm, smoothstep(0.0, 0.55, daylight));
    let dusk_color = Color::srgb(0.961, 0.408, 0.227);
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
            key_warm.into()
        } else if dawn_weight > dusk_weight {
            sun_color.into()
        } else if dusk_weight > 0.0 {
            dusk_color.into()
        } else {
            // Cool moonlit key (#3A4D80) below the horizon (§3 midnight row).
            Color::srgb(0.227, 0.302, 0.502).into()
        };
        // Soft dawn/dusk ramp from 200 lx night floor to the 32 000 lx noon key
        // (§1.2 / §3); smoothstep keeps the transition filmic, not linear-harsh.
        sun_light.illuminance = lerp(200.0, 32_000.0, smoothstep(0.0, 0.25, daylight));
    }
    // Ambient (sky-dome fill) rides the cycle. The key is the warm overhead sun;
    // ambient fills the faces the sun rakes at a grazing angle (vertical walls,
    // shaded sides) so they read instead of crushing to black. A near-black navy
    // @120 lx was far too weak a fill against a straight-down noon sun — every
    // vertical voxel face went black. Lift the daytime fill and warm/neutralize
    // the color so shaded faces stay legible while still cooler than the key.
    ambient.brightness = lerp(220.0, 420.0, daylight);
    ambient.color = lerp_color(
        Color::srgb(0.07, 0.10, 0.18), // cool navy night fill
        Color::srgb(0.55, 0.62, 0.74), // bright cool-neutral day sky fill
        daylight,
    );
    if let Ok(mut sun_transform) = sun_transform_query.single_mut() {
        // `sun_dir` points FROM the ground TO the sun (up at noon). A directional
        // light's forward (-Z) is the direction photons travel, which must point
        // DOWN toward the terrain. Aim the light along `-sun_dir`, otherwise the
        // sun shines up into the sky and every top face renders unlit/black.
        *sun_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, -sun_dir));
    }

    if let Ok((mut moon_light, mut moon_transform, mut moon_visibility)) = moon_query.single_mut() {
        let is_night = daylight < 0.1;
        *moon_visibility = if is_night {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        // Moonlight locked to #3A4D80 (cool) for palette coherence (§3).
        moon_light.color = Color::srgb(0.227, 0.302, 0.502).into();
        moon_light.illuminance = if is_night { 500.0 } else { 0.0 };
        // Same convention as the sun: `moon_dir` points up at the moon, so aim
        // the light's forward along `-moon_dir` to shine down on the terrain.
        *moon_transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, -moon_dir));
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
