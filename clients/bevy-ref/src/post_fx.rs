//! AAA post-processing + shadow stack for the Civis Bevy client.
//!
//! `PostFxPlugin` is self-contained: it finds the existing `Camera3d` entity
//! (spawned by `standalone.rs` / `camera.rs`) and inserts post-processing
//! components without requiring changes to those files.
//!
//! ## Components applied to `Camera3d`
//! - [`bevy::render::view::Hdr`] marker (enables HDR rendering on the camera)
//! - [`bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted`]
//! - [`bevy::post_process::bloom::Bloom`] (requires HDR)
//! - [`bevy::pbr::ScreenSpaceAmbientOcclusion`] (auto-requires `DepthPrepass` + `NormalPrepass`)
//! - [`bevy::pbr::ScreenSpaceReflections`] (baseline screen-space reflections)
//! - [`bevy::light::VolumetricFog`] (baseline camera-level volumetric fog)
//! - [`bevy::anti_alias::taa::TemporalAntiAliasing`] (auto-requires `MotionVectorPrepass` etc.)
//! - [`bevy::render::view::Msaa::Off`] (required by TAA)
//!
//! ## `DirectionalLight` tuning
//! `tune_sun_shadows` watches for newly-added `DirectionalLight` entities (the
//! sun spawned in `atmosphere.rs`) and patches them with 4-cascade CSM at 800 m.

#![cfg(feature = "bevy")]

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, DirectionalLight, VolumetricFog},
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceReflections},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::{ColorGrading, Hdr, Msaa},
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Configures which post-processing effects are enabled at startup.
/// All fields default to `true`.
#[derive(Resource, Debug, Clone)]
pub struct PostFxSettings {
    /// Enable `Tonemapping::AcesFitted`.
    pub aces: bool,
    /// Enable the baseline Bevy tonemapping pass.
    pub tonemapping: bool,
    /// Enable the baseline Bevy color grading component.
    pub color_grading: bool,
    /// Enable `Bloom` (requires HDR; automatically sets `Camera.hdr = true`).
    pub bloom: bool,
    /// Enable `ScreenSpaceAmbientOcclusion`.
    pub ssao: bool,
    /// Enable `ScreenSpaceReflections`.
    pub ssr: bool,
    /// Enable `VolumetricFog`.
    pub volumetric_fog: bool,
    /// Enable `TemporalAntiAliasing` (requires `Msaa::Off`).
    pub taa: bool,
}

impl Default for PostFxSettings {
    fn default() -> Self {
        Self {
            aces: true,
            tonemapping: true,
            color_grading: true,
            bloom: true,
            ssao: true,
            ssr: true,
            volumetric_fog: true,
            taa: true,
        }
    }
}

/// Marker component — prevents `apply_post_fx` from running more than once on
/// the same camera entity.
#[derive(Component)]
pub struct PostFxApplied;

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Self-contained post-processing plugin.
///
/// Insert `PostFxSettings` as a resource *before* adding this plugin to
/// override defaults:
/// ```rust,ignore
/// app.insert_resource(PostFxSettings { bloom: false, ..default() })
///    .add_plugins(PostFxPlugin);
/// ```
pub struct PostFxPlugin;

impl Plugin for PostFxPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: `ScreenSpaceAmbientOcclusionPlugin` is already registered by
        // `DefaultPlugins` (Bevy 0.18 PbrPlugin), so re-adding it panics with
        // "plugin was already added". We only insert the SSAO *component* on the
        // camera in `apply_post_fx`; the plugin itself is already present.
        app.init_resource::<PostFxSettings>()
            .add_systems(Update, apply_post_fx)
            .add_systems(Update, tune_sun_shadows);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Runs every frame until a `Camera3d` without `PostFxApplied` is found.
/// Inserts HDR, tonemapping, bloom, SSAO, SSR, and TAA onto that camera once.
///
/// FR-CIV-PBR-001 — when enabled, this adds Bevy's built-in SSAO component as
/// the baseline GI-lite ambient occlusion pass for the main 3D camera.
///
/// FR-CIV-PBR-002 — when enabled, this adds Bevy's built-in SSR component as
/// the baseline screen-space reflection pass for the main 3D camera.
///
/// FR-CIV-PBR-003 — when enabled, this adds Bevy's built-in VolumetricFog
/// component as the baseline volumetric fog/lighting pass for the main 3D
/// camera.
///
/// FR-CIV-PBR-004 — when enabled, this adds Bevy's built-in Tonemapping and
/// ColorGrading components as the baseline post-FX visual-parity closure for
/// the main 3D camera.
fn apply_post_fx(
    mut commands: Commands,
    settings: Res<PostFxSettings>,
    cameras: Query<Entity, (With<Camera3d>, Without<PostFxApplied>)>,
) {
    let Ok(cam_entity) = cameras.single() else {
        return;
    };

    let mut entity_cmd = commands.entity(cam_entity);

    // In Bevy 0.18 HDR is the `Hdr` marker component, not a `Camera` field.
    entity_cmd.insert((Hdr, Msaa::Off, PostFxApplied));

    if settings.tonemapping && settings.aces {
        entity_cmd.insert(Tonemapping::AcesFitted);
    }
    if settings.color_grading {
        entity_cmd.insert(ColorGrading::default());
    }
    if settings.bloom {
        entity_cmd.insert(Bloom::NATURAL);
    }
    if settings.ssao {
        // #[require] on ScreenSpaceAmbientOcclusion auto-inserts DepthPrepass + NormalPrepass.
        entity_cmd.insert(ScreenSpaceAmbientOcclusion::default());
    }
    if settings.ssr {
        entity_cmd.insert(ScreenSpaceReflections::default());
    }
    if settings.volumetric_fog {
        entity_cmd.insert(VolumetricFog::default());
    }
    if settings.taa {
        // #[require] on TemporalAntiAliasing auto-inserts DepthPrepass, MotionVectorPrepass,
        // TemporalJitter, and MipBias. Msaa::Off (inserted above) is also required.
        entity_cmd.insert(TemporalAntiAliasing::default());
    }
}

/// Watches for newly-spawned `DirectionalLight` entities (e.g. the sun from
/// `atmosphere.rs`) and configures 4-cascade shadow maps covering 800 m.
///
/// Using `Added<DirectionalLight>` means this fires exactly once per light
/// without requiring changes to `atmosphere.rs`.
fn tune_sun_shadows(mut commands: Commands, new_lights: Query<Entity, Added<DirectionalLight>>) {
    for light_entity in &new_lights {
        let cascade_config = CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 800.0,
            ..default()
        }
        .build();

        commands.entity(light_entity).insert(cascade_config);
        commands.entity(light_entity).insert(DirectionalLight {
            shadows_enabled: true,
            ..default()
        });
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_fx_settings_default_all_true() {
        let s = PostFxSettings::default();
        assert!(s.aces, "aces should default to true");
        assert!(s.tonemapping, "tonemapping should default to true");
        assert!(s.color_grading, "color_grading should default to true");
        assert!(s.bloom, "bloom should default to true");
        assert!(s.ssao, "ssao should default to true");
        assert!(s.ssr, "ssr should default to true");
        assert!(s.volumetric_fog, "volumetric_fog should default to true");
        assert!(s.taa, "taa should default to true");
    }

    #[test]
    fn post_fx_settings_partial_override() {
        let s = PostFxSettings {
            bloom: false,
            ..default()
        };
        assert!(!s.bloom);
        assert!(s.aces);
        assert!(s.tonemapping);
        assert!(s.color_grading);
        assert!(s.ssao);
        assert!(s.ssr);
        assert!(s.volumetric_fog);
        assert!(s.taa);
    }

    // ─── Per-flag independence & matrix coverage ────────────────────────────────
    // Every flag in `PostFxSettings` should be toggleable independently. These
    // tests walk the full toggle matrix in a way that catches accidental
    // coupled resets (e.g., a refactor that turns ssao off also disables ssr).

    fn check_flag(name: &'static str, off: PostFxSettings) {
        assert!(off.is_off(name), "{name} should be off when explicitly disabled");
        for other in [
            "aces",
            "tonemapping",
            "color_grading",
            "bloom",
            "ssao",
            "ssr",
            "volumetric_fog",
            "taa",
        ] {
            if other == name {
                continue;
            }
            assert!(
                !off.is_off(other),
                "flag {other} should remain on when only {name} is disabled"
            );
        }
    }

    #[test]
    fn post_fx_settings_each_flag_can_be_off_independently() {
        let _ = PostFxSettings::default();
        check_flag("aces", PostFxSettings { aces: false, ..PostFxSettings::default() });
        check_flag(
            "tonemapping",
            PostFxSettings { tonemapping: false, ..PostFxSettings::default() },
        );
        check_flag(
            "color_grading",
            PostFxSettings { color_grading: false, ..PostFxSettings::default() },
        );
        check_flag(
            "bloom",
            PostFxSettings { bloom: false, ..PostFxSettings::default() },
        );
        check_flag("ssao", PostFxSettings { ssao: false, ..PostFxSettings::default() });
        check_flag("ssr", PostFxSettings { ssr: false, ..PostFxSettings::default() });
        check_flag(
            "volumetric_fog",
            PostFxSettings {
                volumetric_fog: false,
                ..PostFxSettings::default()
            },
        );
        check_flag("taa", PostFxSettings { taa: false, ..PostFxSettings::default() });
    }

    #[test]
    fn post_fx_settings_all_off_round_trips() {
        let s = PostFxSettings {
            aces: false,
            tonemapping: false,
            color_grading: false,
            bloom: false,
            ssao: false,
            ssr: false,
            volumetric_fog: false,
            taa: false,
        };
        assert!(s.is_off("aces"));
        assert!(s.is_off("tonemapping"));
        assert!(s.is_off("color_grading"));
        assert!(s.is_off("bloom"));
        assert!(s.is_off("ssao"));
        assert!(s.is_off("ssr"));
        assert!(s.is_off("volumetric_fog"));
        assert!(s.is_off("taa"));
    }
}

// ── Test-local trait (defined below the test module so it cannot leak into
// ── Test-local trait (defined below the test module so it cannot leak into
// the public API surface). ────────────────────────────────────────────────────

#[allow(dead_code)] // exercised by cfg(test) PostFxProbe tests
trait PostFxProbe {
    fn is_off(&self, name: &str) -> bool;
}
impl PostFxProbe for PostFxSettings {
    fn is_off(&self, name: &str) -> bool {
        match name {
            "aces" => !self.aces,
            "tonemapping" => !self.tonemapping,
            "color_grading" => !self.color_grading,
            "bloom" => !self.bloom,
            "ssao" => !self.ssao,
            "ssr" => !self.ssr,
            "volumetric_fog" => !self.volumetric_fog,
            "taa" => !self.taa,
            _ => panic!("unknown post-fx flag: {name}"),
        }
    }
}
