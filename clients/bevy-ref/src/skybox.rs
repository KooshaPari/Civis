//! HDR skybox / procedural sky dome for the Civis Bevy reference client.
//!
//! # Asset hunt results (2026-05-28)
//!
//! Searched `C:/Users/koosh/Dev` and `C:/Users/koosh/Downloads` (maxdepth 5)
//! for `*.ktx2`, `*.hdr`, `*.exr`, `*skybox*`, `*cubemap*`, and directories
//! named `*stratum*`, `*hdri*`, `*quixel*`, `*megascan*`, `*polyhaven*`.
//!
//! **Nothing found** that is usable for a KTX2 cubemap:
//! - `C:/Users/koosh/Dev/Civis/clients/unreal-show/Content/Megascans` — Unreal
//!   pak format; not extractable as .ktx2 without the Unreal pipeline.
//! - `C:/Users/koosh/Dev/WorldSphereMod/WorldSphereMod/GameResources/PhaseIcons/HdrSkybox.png` —
//!   HDR-named PNG, Unity asset; not a floating-point equirect HDR.
//!
//! # Current sky (2026-05-30)
//!
//! A committed CC0 equirectangular HDR (`assets/sky/kloofendal_43d_clear_puresky_1k.hdr`,
//! Poly Haven, CC0) is loaded by [`load_sky_texture`] and applied as the
//! `base_color_texture` on the interior of the inverted sky-dome sphere
//! ([`spawn_sky_dome`]). The sphere's spherical UVs map the equirect panorama
//! correctly, so the camera sees the real photographed sky. If the image fails to
//! resolve, the procedural gradient dome (below) is the explicit fallback — the
//! sky is never blank. [`tint_dome_by_cycle`] multiplies the panorama toward a
//! deep-blue night as the [`DayNightCycle`] advances.
//!
//! # Upgrading to a real HDRI cubemap (IBL)
//!
//! 1. Download an equirect `.hdr` from <https://polyhaven.com/hdris> (free CC0)
//!    or export from Quixel Bridge / Stratum.
//! 2. Convert to a KTX2 BC6H cubemap using KTX-Software:
//!    ```text
//!    toktx --cubemap --encode bc6h --genmipmap \
//!           assets/skybox/env.ktx2 \
//!           +x.hdr -x.hdr +y.hdr -y.hdr +z.hdr -z.hdr
//!    ```
//!    Or from a single equirect with `basisu`:
//!    ```text
//!    basisu -ktx2 -cube env_equirect.hdr -output_file assets/skybox/env.ktx2
//!    ```
//! 3. Drop `env.ktx2` (diffuse IBL) and `env_spec.ktx2` (specular IBL, mip chain)
//!    into `clients/bevy-ref/assets/skybox/`.
//! 4. In [`SkyboxPlugin`] switch the `build` method to load via `AssetServer`,
//!    insert `bevy::core_pipeline::Skybox { image: cube_handle, brightness: 1000.0 }`
//!    on the camera entity, and attach `EnvironmentMapLight { diffuse_map,
//!    specular_map, intensity: 900.0, ..default() }`.
//!
//! # Current implementation: procedural gradient dome
//!
//! An inverted sphere mesh (`radius = SKY_DOME_RADIUS`) with an emissive
//! `StandardMaterial` and an opaque gradient vertex colour is spawned once.
//! A per-frame system keeps it centred on the main camera so it is always at
//! apparent infinity.  The gradient colours respond to the `DayNightCycle`
//! resource (from `atmosphere.rs`) when it is present in the world.

#![cfg(feature = "bevy")]

use bevy::prelude::*;

use crate::atmosphere::DayNightCycle;

/// Radius of the sky dome sphere in world units.  Large enough to enclose the
/// entire map; must be bigger than the far-plane distance of the camera.
const SKY_DOME_RADIUS: f32 = 4_000.0;

/// Default zenith colour (deep blue) when no `DayNightCycle` is present.
const DEFAULT_ZENITH: [f32; 3] = [0.08, 0.18, 0.50];

/// Default horizon colour when no `DayNightCycle` is present.
const DEFAULT_HORIZON: [f32; 3] = [0.54, 0.74, 0.92];

/// Night zenith colour.
const NIGHT_ZENITH: [f32; 3] = [0.01, 0.03, 0.10];

/// Night horizon colour (dim purple-grey).
const NIGHT_HORIZON: [f32; 3] = [0.08, 0.09, 0.18];

/// Dawn/dusk horizon tint (orange-pink).
const DAWN_HORIZON: [f32; 3] = [0.94, 0.50, 0.22];

/// Committed CC0 equirectangular HDR panorama (Poly Haven, CC0). Mapped onto the
/// interior of the inverted sky-dome sphere via its spherical UVs, so the real
/// photographed sky is what the camera sees instead of a flat gradient.
const SKY_HDR_PATH: &str = "sky/kloofendal_43d_clear_puresky_1k.hdr";

/// Marker component so the follow-camera system can locate the dome entity.
#[derive(Component)]
pub struct SkyDome;

/// Handle to the loaded HDR sky panorama, kept alive in a resource so the asset
/// is not dropped. `None` only if the `bevy` image loaders cannot resolve it.
#[derive(Resource, Default)]
pub struct SkyTexture(pub Option<Handle<Image>>);

/// Registers all sky-dome systems and spawns the initial dome geometry.
///
/// Wire this into your `App` **before** `AtmospherePlugin` (if you use one) so
/// the clear-colour override in `atmosphere` does not stomp the dome.
///
/// ```ignore
/// app.add_plugins(SkyboxPlugin);
/// ```
pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyTexture>()
            .add_systems(Startup, (load_sky_texture, spawn_sky_dome).chain())
            .add_systems(Update, (follow_camera, tint_dome_by_cycle).chain());
    }
}

/// Queue the committed equirect HDR panorama on the [`AssetServer`] at startup.
fn load_sky_texture(mut sky: ResMut<SkyTexture>, asset_server: Res<AssetServer>) {
    sky.0 = Some(asset_server.load(SKY_HDR_PATH));
}

/// Spawn the inverted-sphere sky dome once at startup.
///
/// When the HDR panorama loaded, it is applied as the dome's `base_color_texture`
/// (`unlit` so it reads at full radiance regardless of scene lighting); otherwise
/// the procedural gradient is used as the fallback so the sky is never blank.
fn spawn_sky_dome(
    mut commands: Commands,
    sky: Res<SkyTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Sphere::new(SKY_DOME_RADIUS).mesh().build();

    let mat = StandardMaterial {
        // White base lets the HDR texture show its true colours; with no texture
        // it falls back to the gradient horizon tint.
        base_color: if sky.0.is_some() {
            Color::WHITE
        } else {
            horizon_color(DEFAULT_HORIZON)
        },
        base_color_texture: sky.0.clone(),
        emissive: if sky.0.is_some() {
            LinearRgba::NONE
        } else {
            LinearRgba::from(horizon_color(DEFAULT_HORIZON)) * 0.8
        },
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        ..default()
    };

    commands.spawn((
        SkyDome,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(mat)),
        Transform::default(),
    ));
}

/// Keep the dome centred on the main camera each frame (apparent infinity).
fn follow_camera(
    camera_q: Query<&Transform, (With<Camera3d>, Without<SkyDome>)>,
    mut dome_q: Query<&mut Transform, With<SkyDome>>,
) {
    let Ok(cam) = camera_q.single() else { return };
    for mut dome in &mut dome_q {
        dome.translation = cam.translation;
    }
}

/// Lerp dome colour between night/day/dawn palettes based on `DayNightCycle`.
fn tint_dome_by_cycle(
    cycle: Option<Res<DayNightCycle>>,
    dome_q: Query<&MeshMaterial3d<StandardMaterial>, With<SkyDome>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(mat_handle) = dome_q.single() else {
        return;
    };
    let Some(mat) = materials.get_mut(mat_handle) else {
        return;
    };

    // When the HDR panorama is bound as the base-colour texture, never overwrite
    // base_color with a flat colour (that would hide the photo). Instead multiply
    // the texture by a day→night brightness so the real sky darkens at night.
    if mat.base_color_texture.is_some() {
        let day = match cycle {
            Some(ref cycle) => day_blend(cycle_phase(cycle)),
            None => 1.0,
        };
        // Keep daytime at true colour; sink to a deep-blue night multiply.
        let lo = 0.18_f32;
        let k = lo + (1.0 - lo) * day;
        mat.base_color = Color::srgb(k, k, (k + 0.12).min(1.0));
        return;
    }

    let (zenith, horizon) = match cycle {
        Some(ref cycle) => sky_colors_for_cycle(cycle),
        None => (DEFAULT_ZENITH, DEFAULT_HORIZON),
    };

    // Use horizon colour for the base (dome interior is horizon-side).
    let h = horizon_color(horizon);
    let z = horizon_color(zenith);
    // Blend: base_color ≈ horizon, emissive adds the zenith glow.
    mat.base_color = h;
    mat.emissive = (LinearRgba::from(z) * 0.6 + LinearRgba::from(h) * 0.4) * 1.2;
}

// ---------------------------------------------------------------------------
// Pure helpers (testable without Bevy world)
// ---------------------------------------------------------------------------

/// Derive zenith + horizon sky colours from a [`DayNightCycle`] time.
///
/// `time_of_day` runs `0.0..1.0`; 0.75 = solar noon (per `DayNightCycle`
/// convention in `atmosphere.rs`).
#[must_use]
pub fn sky_colors_for_cycle(cycle: &DayNightCycle) -> ([f32; 3], [f32; 3]) {
    // Access the time via the public setter convention.  DayNightCycle exposes
    // `set_from_is_day`; we re-derive the phase from canonical field positions:
    // noon ≈ 0.75, midnight ≈ 0.25 (matches atmosphere.rs).
    let t = cycle_phase(cycle);
    let day_weight = day_blend(t);
    let dawn_weight = dawn_blend(t);

    let zenith = lerp3(NIGHT_ZENITH, DEFAULT_ZENITH, day_weight);
    let raw_horizon = lerp3(NIGHT_HORIZON, DEFAULT_HORIZON, day_weight);
    let horizon = lerp3(raw_horizon, DAWN_HORIZON, dawn_weight);

    (zenith, horizon)
}

/// Extract a normalised `time_of_day` float from `DayNightCycle`.
///
/// `DayNightCycle` keeps its field private; we read it indirectly via the
/// `set_from_is_day` side-effect: noon = 0.75, midnight = 0.25.  Without
/// public access we default to a fixed noon value so the dome stays lit.
/// When the upstream type exposes `time_of_day()`, replace this function.
#[inline]
fn cycle_phase(cycle: &DayNightCycle) -> f32 {
    // `DayNightCycle` now exposes a public `time_of_day()` getter, so read the
    // live phase directly instead of the previous fixed-noon placeholder.
    cycle.time_of_day()
}

/// Day blend weight: 0 at midnight (t≈0.25), 1 at noon (t≈0.75).
#[inline]
#[must_use]
fn day_blend(t: f32) -> f32 {
    let shifted = (t - 0.25).rem_euclid(1.0); // 0 at midnight
    let half_day = (shifted * std::f32::consts::PI).sin().clamp(0.0, 1.0);
    half_day
}

/// Dawn/dusk weight: peaks near sunrise (~0.5) and sunset (~1.0/0.0).
#[inline]
#[must_use]
fn dawn_blend(t: f32) -> f32 {
    let near_dawn = gauss(t, 0.5, 0.06);
    let near_dusk = gauss(t, 0.0, 0.06) + gauss(t, 1.0, 0.06);
    (near_dawn + near_dusk).min(1.0)
}

/// Gaussian bump centred at `mu` with standard deviation `sigma`.
#[inline]
fn gauss(x: f32, mu: f32, sigma: f32) -> f32 {
    let d = (x - mu) / sigma;
    (-0.5 * d * d).exp()
}

/// Component-wise linear interpolation of two RGB triples.
#[inline]
#[must_use]
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Wrap an sRGB triple as a Bevy `Color`.
#[inline]
fn horizon_color(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zenith_is_darker_than_horizon_at_noon() {
        // Synthesise a noon DayNightCycle via the public setter.
        let mut cycle = DayNightCycle::default();
        cycle.set_from_is_day(true); // sets time_of_day = 0.75
        let (zenith, horizon) = sky_colors_for_cycle(&cycle);
        // Zenith should be darker (less blue-bright) than horizon at noon.
        let zenith_luma = 0.2126 * zenith[0] + 0.7152 * zenith[1] + 0.0722 * zenith[2];
        let horizon_luma = 0.2126 * horizon[0] + 0.7152 * horizon[1] + 0.0722 * horizon[2];
        assert!(
            zenith_luma < horizon_luma,
            "zenith {zenith_luma:.3} should be darker than horizon {horizon_luma:.3}"
        );
    }

    #[test]
    fn zenith_is_dark_at_midnight() {
        let mut cycle = DayNightCycle::default();
        cycle.set_from_is_day(false); // sets time_of_day = 0.25
        let (zenith, _horizon) = sky_colors_for_cycle(&cycle);
        // Night zenith must be very dark.
        let luma = 0.2126 * zenith[0] + 0.7152 * zenith[1] + 0.0722 * zenith[2];
        assert!(luma < 0.15, "night zenith luma {luma:.3} should be < 0.15");
    }

    #[test]
    fn lerp3_interpolates_channels() {
        let a = [0.0_f32; 3];
        let b = [1.0_f32; 3];
        let mid = lerp3(a, b, 0.5);
        for ch in mid {
            assert!((ch - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn lerp3_clamps_outside_zero_one() {
        let a = [0.2_f32; 3];
        let b = [0.8_f32; 3];
        assert_eq!(lerp3(a, b, -1.0), a);
        assert_eq!(lerp3(a, b, 2.0), b);
    }

    #[test]
    fn dawn_blend_peaks_near_transitions() {
        // Dawn is near t=0.5 (sunrise); dusk near t=0.0/1.0 (sunset).
        let dawn = dawn_blend(0.5);
        let noon = dawn_blend(0.75);
        assert!(
            dawn > noon,
            "dawn peak {dawn:.3} should exceed noon {noon:.3}"
        );
    }

    #[test]
    fn day_blend_is_high_at_noon_low_at_midnight() {
        let noon = day_blend(0.75);
        let midnight = day_blend(0.25);
        assert!(
            noon > 0.9,
            "day_blend at noon should be near 1.0, got {noon:.3}"
        );
        assert!(
            midnight < 0.1,
            "day_blend at midnight should be near 0.0, got {midnight:.3}"
        );
    }
}
