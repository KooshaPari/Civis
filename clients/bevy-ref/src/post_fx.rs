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
    light::{CascadeShadowConfigBuilder, DirectionalLight},
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::{Hdr, Msaa},
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Configures which post-processing effects are enabled at startup.
/// All fields default to `true`.
#[derive(Resource, Debug, Clone)]
pub struct PostFxSettings {
    /// Enable `Tonemapping::AcesFitted`.
    pub aces: bool,
    /// Enable `Bloom` (requires HDR; automatically sets `Camera.hdr = true`).
    pub bloom: bool,
    /// Enable `ScreenSpaceAmbientOcclusion`.
    pub ssao: bool,
    /// Enable `TemporalAntiAliasing` (requires `Msaa::Off`).
    pub taa: bool,
}

impl Default for PostFxSettings {
    fn default() -> Self {
        Self {
            aces: true,
            bloom: true,
            ssao: true,
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
/// Inserts HDR, tonemapping, bloom, SSAO, and TAA onto that camera once.
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

    if settings.aces {
        entity_cmd.insert(Tonemapping::AcesFitted);
    }
    if settings.bloom {
        entity_cmd.insert(Bloom {
            intensity: 0.15,
            ..default()
        });
    }
    if settings.ssao {
        // #[require] on ScreenSpaceAmbientOcclusion auto-inserts DepthPrepass + NormalPrepass.
        entity_cmd.insert(ScreenSpaceAmbientOcclusion::default());
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

        commands.entity(light_entity).insert((
            cascade_config,
            // Enable shadows on the DirectionalLight component itself.
            // atmosphere.rs spawns with shadows_enabled=false by default;
            // overwrite via a separate patch so we don't edit that file.
        ));

        // Patch shadows_enabled=true on the existing component.
        // Done via a targeted component insert — Bevy merges fields for
        // components already present on the entity.
        commands
            .entity(light_entity)
            .entry::<DirectionalLight>()
            .and_modify(|mut dl| dl.shadows_enabled = true);
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
        assert!(s.bloom, "bloom should default to true");
        assert!(s.ssao, "ssao should default to true");
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
        assert!(s.ssao);
        assert!(s.taa);
    }
}
