//! PBR biome material loader for the Civis Bevy reference client.
//!
//! Gated behind the `pbr-textures` cargo feature so the default CI build does
//! not require texture assets on disk. When the feature is enabled, this
//! module:
//!
//! 1. Defines a [`Biome`] enum keyed to the six Phase-1 ground materials
//!    (`grass_field`, `sand_beach`, `rock_cliff`, `snow_pure`,
//!    `forest_floor`, `dirt_ground`).
//! 2. Exposes a [`BiomeMaterials`] resource holding a
//!    `Handle<StandardMaterial>` per biome.
//! 3. Loads all maps (base_color + normal + ORM) through the asset server in a
//!    single `Startup` system ([`load_biome_materials`]).
//!
//! Texture conventions and source URLs live in
//! `docs/guides/asset-sources.md` and `assets/textures/README.md`. The full
//! integration roadmap lives in `docs/guides/pbr-materials-plan.md`.
//!
//! ## Usage sketch
//!
//! ```ignore
//! use civ_bevy_ref::materials::{BiomeMaterialsPlugin, Biome, BiomeMaterials};
//!
//! App::new()
//!     .add_plugins(BiomeMaterialsPlugin)
//!     .add_systems(Update, |materials: Res<BiomeMaterials>| {
//!         let _grass = materials.handle(Biome::GrassField).clone();
//!     });
//! ```

#![allow(dead_code)] // Biome consumer on terrain lands in a follow-up PR.

#[cfg(feature = "bevy")]
use bevy::prelude::*;

#[cfg(feature = "pbr-textures")]
use bevy::pbr::StandardMaterial;

/// Load CC0 ground textures with correct color spaces for Bevy PBR.
///
/// Drop full ORM packs from ambientCG / Poly Haven into `assets/textures/<slug>/`:
/// `metallic_roughness.jpg` (linear, G=roughness B=metallic per glTF), `occlusion.jpg`
/// (linear, R=AO), `height.jpg` (linear, parallax / depth). Wire via
/// [`BiomeAssetPaths::phase2_paths`] when present.
#[cfg(feature = "bevy")]
pub mod texture_load {
    use super::Biome;
    use bevy::asset::Handle;
    use bevy::image::ImageLoaderSettings;
    use bevy::prelude::*;

    /// sRGB albedo / base-color maps.
    #[must_use]
    pub fn load_albedo(asset_server: &AssetServer, path: &str) -> Handle<Image> {
        asset_server.load_with_settings(path, |s: &mut ImageLoaderSettings| {
            s.is_srgb = true;
        })
    }

    /// Linear data maps: normal, metallic-roughness, occlusion, height.
    #[must_use]
    pub fn load_linear_map(asset_server: &AssetServer, path: &str) -> Handle<Image> {
        asset_server.load_with_settings(path, |s: &mut ImageLoaderSettings| {
            s.is_srgb = false;
        })
    }

    /// Albedo + normal paths for a [`Biome`] slug under `assets/textures/`.
    #[must_use]
    pub fn biome_albedo_path(biome: Biome) -> String {
        format!("textures/{}/albedo.jpg", biome.slug())
    }

    #[must_use]
    pub fn biome_normal_path(biome: Biome) -> String {
        format!("textures/{}/normal.jpg", biome.slug())
    }
}

/// Number of biome materials managed by the Phase-1 loader.
pub const BIOME_COUNT: usize = 6;

/// Logical ground material classes selected by height-band on the terrain mesh.
///
/// Order is significant — it matches the height bands in
/// [`crate::terrain::color_for_height`] from low to high elevation so a single
/// `[Handle<StandardMaterial>; BIOME_COUNT]` array can be indexed by band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    /// Sandy coastal band just above the water level.
    SandBeach,
    /// Bare earth / packed dirt, low elevation transitions.
    DirtGround,
    /// Temperate grass plains.
    GrassField,
    /// Leaf litter / mossy forest floor.
    ForestFloor,
    /// Exposed cliff rock — tri-planar candidate in Phase 3.
    RockCliff,
    /// Clean alpine snow.
    SnowPure,
}

impl Biome {
    /// All biomes in canonical height-band order (lowest → highest elevation).
    pub const ALL: [Biome; BIOME_COUNT] = [
        Biome::SandBeach,
        Biome::DirtGround,
        Biome::GrassField,
        Biome::ForestFloor,
        Biome::RockCliff,
        Biome::SnowPure,
    ];

    /// Asset directory slug under `assets/textures/`.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Biome::SandBeach => "sand_beach",
            Biome::DirtGround => "dirt_ground",
            Biome::GrassField => "grass_field",
            Biome::ForestFloor => "forest_floor",
            Biome::RockCliff => "rock_cliff",
            Biome::SnowPure => "snow_pure",
        }
    }

    /// Stable index in `0..BIOME_COUNT`, matches [`Biome::ALL`] ordering.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Biome::SandBeach => 0,
            Biome::DirtGround => 1,
            Biome::GrassField => 2,
            Biome::ForestFloor => 3,
            Biome::RockCliff => 4,
            Biome::SnowPure => 5,
        }
    }

    /// Fallback flat sRGB colour used when textures fail to load or when the
    /// `pbr-textures` feature is off. Matches the current
    /// [`crate::terrain::color_for_height`] palette so visuals do not regress.
    #[must_use]
    pub const fn fallback_srgb(self) -> [f32; 3] {
        match self {
            Biome::SandBeach => [0.86, 0.78, 0.52],
            Biome::DirtGround => [0.52, 0.40, 0.28],
            Biome::GrassField => [0.28, 0.58, 0.24],
            Biome::ForestFloor => [0.12, 0.34, 0.12],
            Biome::RockCliff => [0.50, 0.50, 0.52],
            Biome::SnowPure => [0.97, 0.97, 0.97],
        }
    }

    /// Per-biome surface PBR `(perceptual_roughness, reflectance, metallic)`
    /// from `docs/design/lighting-biomes-art.md` §4.2. Replaces the old uniform
    /// `0.95 / 0.18` block (the flat-RGB bug). Keeps a ≥0.40 roughness spread
    /// across the set so shiny families (Snow `0.45`) read against the matte
    /// ones (Forest `0.95`). Reflectance is the wet/dry knob; all ground is
    /// dielectric (metallic `0.0`).
    #[must_use]
    pub const fn surface_pbr(self) -> (f32, f32, f32) {
        match self {
            Biome::SandBeach => (0.65, 0.42, 0.0), // wet shore
            Biome::DirtGround => (0.90, 0.18, 0.0),
            Biome::GrassField => (0.88, 0.25, 0.0),
            Biome::ForestFloor => (0.95, 0.18, 0.0), // flattest / most matte
            Biome::RockCliff => (0.78, 0.35, 0.0),
            Biome::SnowPure => (0.45, 0.55, 0.0), // shiniest / brightest
        }
    }

    /// Per-biome `base_color` tint (sRGB) from §4.2, multiplied over the albedo
    /// texture. Near-white-warm so the texture dominates; tint nudges mood.
    #[must_use]
    pub const fn tint_srgb(self) -> [f32; 3] {
        match self {
            Biome::SandBeach => [0.788, 0.722, 0.478], // #C9B87A wet shore
            Biome::DirtGround => [0.478, 0.369, 0.220], // #7A5E38 packed
            Biome::GrassField => [0.290, 0.604, 0.239], // #4A9A3D
            Biome::ForestFloor => [0.122, 0.341, 0.122], // #1F571F canopy floor
            Biome::RockCliff => [0.424, 0.439, 0.455], // #6C7074
            Biome::SnowPure => [0.941, 0.965, 1.000], // #F0F6FF
        }
    }

    /// Pick a biome from a normalised terrain height (`0.0..=1.0`) using the
    /// same band thresholds as [`crate::terrain::color_for_height`]. Returns
    /// `SandBeach` for underwater bands so callers can detect water separately
    /// (the water surface is rendered as its own pass).
    #[must_use]
    pub fn from_height_norm(t: f32) -> Self {
        if t < 0.24 {
            Biome::SandBeach
        } else if t < 0.36 {
            Biome::DirtGround
        } else if t < 0.48 {
            Biome::GrassField
        } else if t < 0.68 {
            Biome::ForestFloor
        } else if t < 0.85 {
            Biome::RockCliff
        } else {
            Biome::SnowPure
        }
    }
}

#[cfg(feature = "pbr-textures")]
mod loader {
    use super::*;

    use super::texture_load;

    /// Asset paths for a single biome's PBR map set, relative to the `assets/`
    /// root. Phase 1 ships `albedo` + `normal`; Phase 2 adds split ORM maps.
    #[derive(Debug, Clone)]
    pub struct BiomeAssetPaths {
        pub albedo: String,
        pub normal: String,
        /// Packed ORM (legacy slot); prefer split maps below.
        pub orm: String,
        /// glTF-style metallic-roughness (G=roughness, B=metallic), linear.
        pub metallic_roughness: String,
        /// Separate occlusion (R=AO), linear — not the same binding as MR.
        pub occlusion: String,
        /// Height / parallax depth, linear.
        pub height: String,
    }

    impl BiomeAssetPaths {
        #[must_use]
        pub fn for_biome(biome: Biome) -> Self {
            let slug = biome.slug();
            Self {
                albedo: texture_load::biome_albedo_path(biome),
                normal: texture_load::biome_normal_path(biome),
                orm: format!("textures/{slug}/orm.ktx2"),
                metallic_roughness: format!("textures/{slug}/metallic_roughness.jpg"),
                occlusion: format!("textures/{slug}/occlusion.jpg"),
                height: format!("textures/{slug}/height.jpg"),
            }
        }
    }

    /// PBR material handles for every [`Biome`], indexed by [`Biome::index`].
    #[derive(Resource, Debug, Clone)]
    pub struct BiomeMaterials {
        handles: [Handle<StandardMaterial>; BIOME_COUNT],
    }

    impl BiomeMaterials {
        /// Look up the `StandardMaterial` for a biome.
        #[must_use]
        pub fn handle(&self, biome: Biome) -> &Handle<StandardMaterial> {
            &self.handles[biome.index()]
        }

        /// Iterate over `(biome, handle)` pairs in canonical order.
        pub fn iter(&self) -> impl Iterator<Item = (Biome, &Handle<StandardMaterial>)> {
            Biome::ALL
                .iter()
                .copied()
                .zip(self.handles.iter())
        }
    }

    /// Plugin that loads all six biome materials at startup.
    pub struct BiomeMaterialsPlugin;

    impl Plugin for BiomeMaterialsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, load_biome_materials);
        }
    }

    /// Startup system: build a `StandardMaterial` per biome and register it
    /// under the [`BiomeMaterials`] resource.
    ///
    /// Phase 1: `base_color_texture` + `normal_map_texture` + per-biome PBR constants.
    /// Phase 2: drop `metallic_roughness.jpg`, `occlusion.jpg`, `height.jpg` per slug
    /// (ambientCG / Poly Haven CC0) — helpers wire them when files exist.
    pub fn load_biome_materials(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let handles: [Handle<StandardMaterial>; BIOME_COUNT] =
            std::array::from_fn(|i| materials.add(build_biome_material(&asset_server, Biome::ALL[i])));

        commands.insert_resource(BiomeMaterials { handles });
    }

    fn build_biome_material(asset_server: &AssetServer, biome: Biome) -> StandardMaterial {
        let paths = BiomeAssetPaths::for_biome(biome);
        let albedo = texture_load::load_albedo(asset_server, &paths.albedo);
        let normal = texture_load::load_linear_map(asset_server, &paths.normal);
        let [r, g, b] = biome.tint_srgb();
        let (roughness, reflectance, metallic) = biome.surface_pbr();
        let mut mat = StandardMaterial {
            base_color: Color::srgb(r, g, b),
            base_color_texture: Some(albedo),
            normal_map_texture: Some(normal),
            perceptual_roughness: roughness,
            metallic,
            reflectance,
            ..Default::default()
        };
        wire_phase2_maps_if_present(asset_server, &paths, &mut mat);
        mat
    }

    /// Phase 2: when CC0 ORM packs land, wire split MR / AO / height (all linear).
    fn wire_phase2_maps_if_present(
        asset_server: &AssetServer,
        paths: &BiomeAssetPaths,
        mat: &mut StandardMaterial,
    ) {
        if map_exists_on_disk(&paths.metallic_roughness) {
            mat.metallic_roughness_texture =
                Some(texture_load::load_linear_map(asset_server, &paths.metallic_roughness));
        }
        if map_exists_on_disk(&paths.occlusion) {
            mat.occlusion_texture =
                Some(texture_load::load_linear_map(asset_server, &paths.occlusion));
        }
        if map_exists_on_disk(&paths.height) {
            mat.depth_map = Some(texture_load::load_linear_map(asset_server, &paths.height));
            mat.parallax_mapping_method = bevy::pbr::ParallaxMappingMethod::Occlusion;
            mat.parallax_depth_scale = 0.02;
        }
    }

    fn map_exists_on_disk(asset_relative: &str) -> bool {
        ["assets", "clients/bevy-ref/assets"]
            .into_iter()
            .any(|root| std::path::Path::new(root).join(asset_relative).is_file())
    }
}

#[cfg(feature = "pbr-textures")]
pub use loader::{BiomeAssetPaths, BiomeMaterials, BiomeMaterialsPlugin, load_biome_materials};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_all_matches_count() {
        assert_eq!(Biome::ALL.len(), BIOME_COUNT);
    }

    #[test]
    fn biome_index_matches_all_ordering() {
        for (i, biome) in Biome::ALL.iter().copied().enumerate() {
            assert_eq!(biome.index(), i, "{biome:?} index drift");
        }
    }

    #[test]
    fn biome_slug_is_stable_directory_name() {
        assert_eq!(Biome::GrassField.slug(), "grass_field");
        assert_eq!(Biome::SnowPure.slug(), "snow_pure");
    }

    #[test]
    fn from_height_norm_walks_bands_low_to_high() {
        assert_eq!(Biome::from_height_norm(0.20), Biome::SandBeach);
        assert_eq!(Biome::from_height_norm(0.30), Biome::DirtGround);
        assert_eq!(Biome::from_height_norm(0.42), Biome::GrassField);
        assert_eq!(Biome::from_height_norm(0.55), Biome::ForestFloor);
        assert_eq!(Biome::from_height_norm(0.75), Biome::RockCliff);
        assert_eq!(Biome::from_height_norm(0.95), Biome::SnowPure);
    }

    #[test]
    fn fallback_colors_are_in_unit_cube() {
        for biome in Biome::ALL {
            for c in biome.fallback_srgb() {
                assert!((0.0..=1.0).contains(&c), "{biome:?} fallback out of gamut");
            }
        }
    }
}
