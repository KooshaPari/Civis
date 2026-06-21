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

#[cfg(feature = "pbr-textures")]
use bevy::pbr::StandardMaterial;
#[cfg(feature = "pbr-textures")]
use bevy::prelude::*;

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

    /// Asset paths for a single biome's PBR map set, relative to the `assets/`
    /// root. Phase 1 ships `albedo` + `normal`; `orm` is opt-in for Phase 2.
    #[derive(Debug, Clone)]
    pub struct BiomeAssetPaths {
        /// `assets/textures/<slug>/albedo.ktx2`
        pub albedo: String,
        /// `assets/textures/<slug>/normal.ktx2`
        pub normal: String,
        /// `assets/textures/<slug>/orm.ktx2` (G=roughness, B=metallic, R=occlusion).
        pub metallic_roughness: String,
        /// `assets/textures/<slug>/orm.ktx2` (R=occlusion). Shared with
        /// `metallic_roughness` when the source pack is already ORM.
        pub occlusion: String,
    }

    impl BiomeAssetPaths {
        #[must_use]
        pub fn for_biome(biome: Biome) -> Self {
            let slug = biome.slug();
            // Phase 1 ships .jpg downloads from Poly Haven / ambientCG (CC0).
            // Phase 2 will pack to .ktx2 for VRAM savings — keep the orm slot
            // pointing at the future packed file.
            Self {
                albedo: format!("textures/{slug}/albedo.jpg"),
                normal: format!("textures/{slug}/normal.jpg"),
                metallic_roughness: format!("textures/{slug}/orm.ktx2"),
                occlusion: format!("textures/{slug}/orm.ktx2"),
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
            Biome::ALL.iter().copied().zip(self.handles.iter())
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
    /// Phase 1: `base_color_texture` + `normal_map_texture` only.
    /// Phase 2: also wire `metallic_roughness_texture` + `occlusion_texture`
    /// (from the packed ORM image) — see `docs/guides/pbr-materials-plan.md`.
    pub fn load_biome_materials(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        // SAFETY: array-init via fold preserves ordering since Biome::ALL is
        // already in canonical index order.
        let handles: [Handle<StandardMaterial>; BIOME_COUNT] = std::array::from_fn(|i| {
            let biome = Biome::ALL[i];
            let paths = BiomeAssetPaths::for_biome(biome);
            let albedo: Handle<Image> = asset_server.load(&paths.albedo);
            let normal: Handle<Image> = texture_load::load_linear_map(&asset_server, &paths.normal);
            let metallic_roughness: Handle<Image> =
                texture_load::load_linear_map(&asset_server, &paths.metallic_roughness);
            let occlusion: Handle<Image> =
                texture_load::load_linear_map(&asset_server, &paths.occlusion);

            let [r, g, b] = biome.fallback_srgb();
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                base_color_texture: Some(albedo),
                normal_map_texture: Some(normal),
                perceptual_roughness: 0.95,
                metallic_roughness_texture: Some(metallic_roughness),
                occlusion_texture: Some(occlusion),
                metallic: 0.0,
                reflectance: 0.18,
                ..Default::default()
            })
        });

        commands.insert_resource(BiomeMaterials { handles });
    }
}

#[cfg(feature = "pbr-textures")]
pub use loader::{load_biome_materials, BiomeAssetPaths, BiomeMaterials, BiomeMaterialsPlugin};

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

    #[test]
    fn fr_civ_pbr_002_orm_channels_are_split_and_orm_source_fans_in() {
        let paths = BiomeAssetPaths::for_biome(Biome::RockCliff);
        assert_eq!(paths.metallic_roughness, "textures/rock_cliff/orm.ktx2");
        assert_eq!(paths.occlusion, "textures/rock_cliff/orm.ktx2");
        assert_eq!(paths.metallic_roughness, paths.occlusion);
    }

    #[test]
    fn fr_civ_pbr_006_texture_maps_are_color_space_partitioned() {
        assert_eq!(
            texture_load::biome_albedo_path(Biome::SandBeach),
            "textures/sand_beach/albedo.jpg"
        );
        assert_eq!(
            texture_load::biome_normal_path(Biome::SandBeach),
            "textures/sand_beach/normal.jpg"
        );
    }
}
