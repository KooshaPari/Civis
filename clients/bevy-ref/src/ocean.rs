//! Native ocean surface plugin for the Civis Bevy reference client.
//!
//! Wraps [`bevy_water`] 0.18 to spawn a procedural wave surface at the
//! simulation's canonical sea level ([`WATER_LEVEL`]) without hand-rolling any
//! shader or mesh code.
//!
//! # Integration — heightmap (terrain) path
//!
//! When the `voxel` feature is **not** active (pure heightmap world),
//! `OceanPlugin` is the sole water provider:
//!
//! ```rust,ignore
//! app.add_plugins(OceanPlugin::default());
//! ```
//!
//! # Integration — voxel path (alongside VoxelSimPlugin)
//!
//! `VoxelSimPlugin` owns world-gen, `WaterSettings`, and the water-plane
//! spawn.  Use `OceanPlugin::water_plugin_only()` to register the
//! `bevy_water::WaterPlugin` shader/material infrastructure without adding
//! a second spawn system or clobbering `WaterSettings`:
//!
//! ```rust,ignore
//! app.add_plugins(OceanPlugin::water_plugin_only());
//! ```
//!
//! # Quality flag
//!
//! Insert [`OceanQuality`] as a resource **before** `OceanPlugin::build`:
//!
//! ```rust,ignore
//! app.insert_resource(OceanQuality::Low)
//!    .add_plugins(OceanPlugin::default());
//! ```
//!
//! `Low` uses coarser wave subdivisions and `WaterQuality::Low`; `High`
//! (default) enables full FFT-based waves and reflections via
//! `WaterQuality::High`.

use bevy::light::NotShadowCaster;
use bevy::mesh::PlaneMeshBuilder;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy_water::water::{
    material::{StandardWaterMaterial, WaterMaterial},
    WaterQuality, WaterSettings, WaterTile,
};
use bevy_water::WaterPlugin;

use crate::terrain::{HEIGHT_SCALE, WATER_LEVEL, WORLD_SIZE};

// ---------------------------------------------------------------------------
// Public surface
// ---------------------------------------------------------------------------

/// Quality tier for the ocean surface.
///
/// Stored as a Bevy [`Resource`] so it can be configured before the plugin
/// builds, e.g. in tests or headless CI where GPU resources are limited.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OceanQuality {
    /// Full FFT-based waves + reflections (`WaterQuality::High`).
    #[default]
    High,
    /// Reduced subdivisions + `WaterQuality::Low`.  Suitable for CI / low-end
    /// hardware that is still GPU-capable but where saving memory is preferred.
    Low,
}

/// Marker component placed on the spawned ocean [`Entity`] so other systems
/// (e.g. info-views, disaster tools) can query it reliably.
#[derive(Component)]
pub struct OceanSurface;

/// Self-contained ocean plugin.
///
/// Two construction modes:
///
/// - [`OceanPlugin::default()`] — full mode: registers `WaterPlugin`, inserts
///   [`WaterSettings`] at [`WATER_LEVEL`], and spawns the wave-plane at
///   `Startup`.  Use this in the **heightmap** path or when no other plugin
///   owns water.
///
/// - [`OceanPlugin::water_plugin_only()`] — thin mode: registers `WaterPlugin`
///   and inserts `WaterSettings` only when absent, but does **not** spawn a
///   plane.  Use this alongside `VoxelSimPlugin` which already spawns its own
///   water plane.
pub struct OceanPlugin {
    /// When `true` the plugin also spawns the wave-plane entity at `Startup`.
    /// Set `false` when `VoxelSimPlugin` (or another system) owns the spawn.
    pub spawn_plane: bool,
}

impl Default for OceanPlugin {
    fn default() -> Self {
        Self { spawn_plane: true }
    }
}

impl OceanPlugin {
    /// Thin constructor: register `WaterPlugin` infrastructure only.
    ///
    /// Use alongside `VoxelSimPlugin` to avoid a double water-plane.
    pub fn water_plugin_only() -> Self {
        Self { spawn_plane: false }
    }
}

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        // Honour a pre-inserted OceanQuality resource; fall back to High.
        let quality = app
            .world()
            .get_resource::<OceanQuality>()
            .copied()
            .unwrap_or_default();

        let water_quality = match quality {
            OceanQuality::High => WaterQuality::High,
            OceanQuality::Low => WaterQuality::Low,
        };

        // WaterPlugin provides the WGSL water shader + per-frame material
        // update system.  Register it once here so consumers only add
        // OceanPlugin rather than also importing bevy_water directly.
        app.add_plugins(WaterPlugin);

        // Insert WaterSettings only when absent: VoxelSimPlugin inserts its
        // own (with dims-relative height), so skip to avoid clobbering.
        if !app.world().contains_resource::<WaterSettings>() {
            app.insert_resource(build_water_settings(water_quality));
        }

        if self.spawn_plane {
            app.add_systems(Startup, spawn_ocean_plane);
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Build [`WaterSettings`] using the canonical [`WATER_LEVEL`] so the ocean
/// surface is always coherent with terrain generation regardless of voxel grid
/// dimensions.
///
/// Wave amplitude and colour values are hand-tuned for `HEIGHT_SCALE = 200 m`
/// and `WORLD_SIZE = 256 m`.
fn build_water_settings(water_quality: WaterQuality) -> WaterSettings {
    WaterSettings {
        // Exact match to the terrain constant so no seam or Z-fight at coasts.
        height: WATER_LEVEL,
        amplitude: 0.35 * (HEIGHT_SCALE / 200.0),
        clarity: 0.22,
        base_color: Color::srgba(0.07, 0.41, 0.67, 1.0),
        deep_color: Color::srgba(0.02, 0.18, 0.46, 1.0),
        shallow_color: Color::srgba(0.07, 0.68, 0.62, 1.0),
        edge_scale: 0.09,
        edge_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
        update_materials: true,
        // Let bevy_water auto-tile so we only need to spawn one canonical
        // plane entity rather than managing a tile grid here.
        spawn_tiles: None,
        water_quality,
        wave_direction: Vec2::new(1.0, 2.0),
        wave_direction_blend_duration: 2.0,
        ..Default::default()
    }
}

/// Startup system: spawn the tiled water plane centred on the world XZ extent
/// at exactly [`WATER_LEVEL`] Y.
///
/// The plane mesh is built from `bevy_water::water::WATER_SIZE` so the UV
/// coordinates match the pre-compiled water shader expectations.  A uniform XZ
/// scale expands it to cover the full [`WORLD_SIZE`] footprint.
fn spawn_ocean_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<StandardWaterMaterial>>,
    quality: Option<Res<OceanQuality>>,
) {
    let quality = quality.map(|q| *q).unwrap_or_default();
    let tile_size = bevy_water::water::WATER_SIZE as f32;

    // Subdivision density: High = 1 vert per 4 units, Low = 1 per 8 units.
    let subdivs = match quality {
        OceanQuality::High => (tile_size / 4.0) as u32,
        OceanQuality::Low => (tile_size / 8.0).max(1.0) as u32,
    };

    let mut plane_builder = PlaneMeshBuilder::from_length(tile_size);
    plane_builder = plane_builder.subdivisions(subdivs);
    let mesh = Mesh3d(meshes.add(plane_builder));

    // Scale the tile to cover the full WORLD_SIZE footprint.
    let world_scale = Vec3::new(WORLD_SIZE / tile_size, 1.0, WORLD_SIZE / tile_size);

    let material_extension = WaterMaterial {
        // coord_scale drives UV tiling of the wave normal-map; match to the
        // actual world footprint so wave density is spatially consistent.
        coord_scale: Vec2::splat(WORLD_SIZE),
        ..Default::default()
    };

    let material = MeshMaterial3d(water_materials.add(StandardWaterMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(0.07, 0.41, 0.67, 1.0),
            perceptual_roughness: 0.22,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: material_extension,
    }));

    commands.spawn((
        OceanSurface,
        mesh,
        material,
        WaterTile::default(),
        Transform {
            // Centre the plane over the terrain mesh (0..WORLD_SIZE on X and Z).
            translation: Vec3::new(WORLD_SIZE * 0.5, WATER_LEVEL, WORLD_SIZE * 0.5),
            scale: world_scale,
            ..default()
        },
        NotShadowCaster,
    ));
}
