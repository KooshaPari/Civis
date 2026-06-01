//! CC0 GLTF model loading for the Civis 3D reference client.
//!
//! Replaces the hand-rolled primitives that [`crate::sim_bridge`] currently
//! spawns (capsule civilians / cuboid buildings / cone trees) with real CC0
//! GLTF scenes loaded from `assets/models/*.glb`.
//!
//! # Gating & fallback
//!
//! Everything here is behind the `models` cargo feature (`--features
//! bevy,models`). When the feature is **off**, this module is not compiled and
//! callers keep using the procedural primitives. When the feature is **on** but
//! a `.glb` asset is absent, [`GameModels`] simply holds `None` for that slot
//! and the spawn helpers return [`ModelOrPrimitive::Primitive`] so callers fall
//! back to the existing mesh path. There is no silent half-state: a missing
//! handle is an explicit `None`, not a broken `SceneRoot`.
//!
//! # Expected asset files
//!
//! The asset-pipeline agent's `fetch-cc0-models` script populates these from
//! CC0 sources (Quaternius / Kenney — both public-domain / CC0):
//!
//! - `clients/bevy-ref/assets/models/civilian.glb`
//! - `clients/bevy-ref/assets/models/tree.glb`
//! - `clients/bevy-ref/assets/models/building.glb`
//!
//! See `clients/bevy-ref/assets/models/README.md` for provenance + the exact
//! source URLs the fetch script pulls. Bevy resolves these relative to the
//! crate's `assets/` directory via the default [`AssetServer`] root.
//!
//! # Closer wiring (DO NOT edit lib.rs / standalone.rs here — the closer does)
//!
//! This module is intentionally NOT yet referenced from the shared `lib.rs` /
//! `standalone.rs` (owned by other agents). The closer must add exactly:
//!
//! 1. In `clients/bevy-ref/src/lib.rs`, next to the other `#[cfg(feature =
//!    "bevy")] pub mod …;` declarations:
//!    ```ignore
//!    #[cfg(all(feature = "bevy", feature = "models"))]
//!    pub mod gltf_models;
//!    ```
//!
//! 2. In `clients/bevy-ref/src/bin/standalone.rs`, alongside the other
//!    `.add_plugins(...)` calls:
//!    ```ignore
//!    #[cfg(feature = "models")]
//!    app.add_plugins(civ_bevy_ref::gltf_models::GltfModelsPlugin);
//!    ```
//!
//! 3. (Optional, later) In `sim_bridge.rs`, replace the primitive spawn with
//!    `GameModels`-driven `SceneRoot` spawns using the helpers below — see the
//!    "sim_bridge integration point" section on [`civilian_scene`].

use bevy::prelude::*;
use civ_agents::ActorVisualKind;

/// Relative asset paths (under the crate `assets/` root) for each CC0 model.
///
/// Kept as constants so the fetch-cc0-models script and the loader agree on the
/// exact filenames.
pub mod asset_paths {
    /// Civilian / agent model (KayKit Knight — humanoid).
    pub const CIVILIAN: &str = "models/civilian.glb";
    /// Herd / fauna model (KayKit skeleton minion — unarmed creature).
    pub const HERD: &str = "models/creature_skeleton_minion.glb";
    /// Tree / vegetation decoration model.
    pub const TREE: &str = "models/tree.glb";
    /// Generic building model.
    pub const BUILDING: &str = "models/building.glb";
}

/// Loaded CC0 GLTF scene handles, populated at [`Startup`] by
/// [`load_game_models`].
///
/// Each field is `Option<Handle<Scene>>`: `Some` once the corresponding `.glb`
/// has been queued on the [`AssetServer`], `None` when the `models` feature
/// path could not resolve it. Callers treat `None` as "fall back to the
/// procedural primitive".
///
/// Note: a `Some(handle)` means the load was *requested*; the underlying scene
/// may still be loading or may fail to resolve if the file is absent. Spawning
/// a `SceneRoot` from a not-yet-loaded handle is fine in Bevy (the scene pops
/// in when ready); for a stricter gate, callers can poll
/// [`AssetServer::is_loaded`] before swapping out the primitive.
#[derive(Resource, Default, Clone)]
pub struct GameModels {
    /// Civilian / agent scene (replaces the capsule).
    pub civilian: Option<Handle<Scene>>,
    /// Herd / fauna scene (non-human, no sword).
    pub herd: Option<Handle<Scene>>,
    /// Tree / vegetation scene (replaces the cone decoration).
    pub tree: Option<Handle<Scene>>,
    /// Building scene (replaces the cuboid).
    pub building: Option<Handle<Scene>>,
}

impl GameModels {
    /// True when every model slot has a requested handle.
    #[must_use]
    pub fn all_present(&self) -> bool {
        self.civilian.is_some()
            && self.herd.is_some()
            && self.tree.is_some()
            && self.building.is_some()
    }
}

/// Outcome of a model lookup: either a ready-to-spawn [`SceneRoot`] or a signal
/// that the caller should spawn its existing procedural primitive instead.
///
/// This keeps the fallback decision explicit at the call site rather than
/// hiding it inside the helper.
pub enum ModelOrPrimitive {
    /// A loaded GLTF scene the caller can attach via `commands.spawn(scene_root)`.
    Model(SceneRoot),
    /// No model available — caller spawns its procedural mesh/material instead.
    Primitive,
}

impl ModelOrPrimitive {
    /// Convenience: `true` when a real model scene is available.
    #[must_use]
    pub fn has_model(&self) -> bool {
        matches!(self, ModelOrPrimitive::Model(_))
    }
}

/// Bevy plugin that registers [`GameModels`] and loads the CC0 GLTF scenes at
/// startup.
///
/// Add it from `standalone.rs` (see the closer-wiring note in the module docs);
/// it is a no-op for rendering on its own — it only populates the
/// [`GameModels`] resource that `sim_bridge` (later) reads.
pub struct GltfModelsPlugin;

impl Plugin for GltfModelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameModels>()
            .add_systems(Startup, load_game_models);
    }
}

/// Startup system: queue each CC0 `.glb` on the [`AssetServer`] and store the
/// scene handle in [`GameModels`].
///
/// Uses [`GltfAssetLabel::Scene(0)`] — the first (default) scene in each glTF
/// document — which is the conventional single-scene export from Quaternius /
/// Kenney CC0 packs.
pub fn load_game_models(mut models: ResMut<GameModels>, asset_server: Res<AssetServer>) {
    models.civilian = Some(asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset_paths::CIVILIAN)));
    models.herd = Some(asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset_paths::HERD)));
    models.tree = Some(asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset_paths::TREE)));
    models.building =
        Some(asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset_paths::BUILDING)));
}

/// sim_bridge integration point — civilian.
///
/// Returns a [`ModelOrPrimitive`] for a civilian of the given `faction`. When a
/// civilian model is loaded, yields [`ModelOrPrimitive::Model`] wrapping a
/// [`SceneRoot`]; otherwise [`ModelOrPrimitive::Primitive`] so the caller
/// spawns the existing capsule + faction-coloured material.
///
/// The `faction` parameter is accepted now so the API is stable for later
/// per-faction tinting / model-variant selection (e.g. faction banners);
/// today it does not change which scene is returned.
///
/// Intended `sim_bridge.rs` usage (the closer / a later pass wires this — do
/// NOT edit sim_bridge from this module):
/// ```ignore
/// match civilian_scene(&models, civilian.faction) {
///     ModelOrPrimitive::Model(scene_root) => {
///         commands.spawn((SimCivilianMarker, scene_root, Transform::from_translation(world_pos)));
///     }
///     ModelOrPrimitive::Primitive => {
///         // existing capsule mesh + material spawn (current code path)
///     }
/// }
/// ```
#[must_use]
pub fn civilian_scene(models: &GameModels, _faction: u32) -> ModelOrPrimitive {
    scene_or_primitive(&models.civilian)
}

/// Herd / fauna scene (skeleton minion). See [`civilian_scene`] for usage.
#[must_use]
pub fn herd_scene(models: &GameModels) -> ModelOrPrimitive {
    scene_or_primitive(&models.herd)
}

/// Pick the actor scene for a sim agent's visual kind.
#[must_use]
pub fn actor_scene(models: &GameModels, kind: ActorVisualKind, faction: u32) -> ModelOrPrimitive {
    match kind {
        ActorVisualKind::Humanoid => civilian_scene(models, faction),
        ActorVisualKind::Herd => herd_scene(models),
    }
}

/// sim_bridge integration point — building. See [`civilian_scene`] for the
/// match/fallback pattern.
#[must_use]
pub fn building_scene(models: &GameModels) -> ModelOrPrimitive {
    scene_or_primitive(&models.building)
}

/// decorations integration point — tree. See [`civilian_scene`] for the
/// match/fallback pattern.
#[must_use]
pub fn tree_scene(models: &GameModels) -> ModelOrPrimitive {
    scene_or_primitive(&models.tree)
}

fn scene_or_primitive(handle: &Option<Handle<Scene>>) -> ModelOrPrimitive {
    match handle {
        Some(handle) => ModelOrPrimitive::Model(SceneRoot(handle.clone())),
        None => ModelOrPrimitive::Primitive,
    }
}

// =============================================================================
// PART A — bevy_atmosphere note (real sky)
// =============================================================================
//
// Status (checked 2026-05-29 via crates.io): the latest published
// `bevy_atmosphere` is 0.13.0 (2025-05-06), which depends on `bevy ^0.16`.
// There is NO release compatible with Bevy 0.18 yet, so the real-sky swap is
// deferred and the procedural skybox dome (`src/skybox.rs` +
// `src/atmosphere.rs`) remains the sky. The `atmosphere` cargo feature and the
// commented `bevy_atmosphere` dep in Cargo.toml are placeholders.
//
// When an 0.18-compatible release ships (expected ~0.14):
//   1. Uncomment the dep in Cargo.toml and flip the feature to
//      `atmosphere = ["bevy", "dep:bevy_atmosphere"]`.
//   2. From standalone.rs (closer-owned), add:
//        #[cfg(feature = "atmosphere")]
//        app.add_plugins(bevy_atmosphere::plugin::AtmospherePlugin);
//      and attach `bevy_atmosphere::prelude::AtmosphereCamera::default()` to the
//      camera entity.
//   3. Retire the procedural dome in skybox.rs (skybox-owner agent).
//
// The thin wrapper below documents the add without pulling the dep, so callers
// have a single stable symbol to reference once the feature is enabled.

/// Placeholder wrapper for the future `bevy_atmosphere::plugin::AtmospherePlugin`.
///
/// No 0.18-compatible `bevy_atmosphere` exists yet (see the note above), so this
/// is a documentation anchor only. Once the dep is enabled, the closer should
/// add the real `AtmospherePlugin` (not this stub) in `standalone.rs`.
#[cfg(feature = "atmosphere")]
#[derive(Default)]
pub struct AtmospherePlugin2;

#[cfg(feature = "atmosphere")]
impl Plugin for AtmospherePlugin2 {
    fn build(&self, _app: &mut App) {
        // Intentionally empty: real bevy_atmosphere is not yet 0.18-compatible.
        // Keeps the procedural skybox dome as the active sky. Replace with
        // `app.add_plugins(bevy_atmosphere::plugin::AtmospherePlugin)` once the
        // dep is uncommented in Cargo.toml.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_models_fall_back_to_primitive() {
        let models = GameModels::default();
        assert!(!models.all_present());
        assert!(!civilian_scene(&models, 0).has_model());
        assert!(!building_scene(&models).has_model());
        assert!(!tree_scene(&models).has_model());
    }

    #[test]
    fn asset_paths_point_at_models_dir() {
        assert!(asset_paths::CIVILIAN.starts_with("models/"));
        assert!(asset_paths::TREE.ends_with(".glb"));
        assert!(asset_paths::BUILDING.ends_with(".glb"));
    }
}
