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

#![allow(dead_code)] // Phase 1 scaffold — consumers land in a follow-up PR.

#[cfg(feature = "bevy")]
use bevy::mesh::MeshVertexBufferLayoutRef;
#[cfg(feature = "pbr-textures")]
use bevy::pbr::StandardMaterial;
#[cfg(feature = "bevy")]
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
#[cfg(feature = "bevy")]
use bevy::prelude::*;
#[cfg(feature = "bevy")]
use bevy::reflect::TypePath;
#[cfg(feature = "bevy")]
use bevy::render::render_resource::{AsBindGroup, ShaderType, SpecializedMeshPipelineError};
#[cfg(feature = "bevy")]
use bevy::shader::{Shader, ShaderRef};
#[cfg(feature = "bevy")]
use std::collections::HashMap;
#[cfg(feature = "bevy")]
use std::sync::OnceLock;

#[cfg(feature = "bevy")]
pub mod texture_load {
    use super::Biome;
    use bevy::asset::Handle;
    use bevy::image::ImageLoaderSettings;
    use bevy::prelude::*;

    /// sRGB albedo / base-color maps.
    #[must_use]
    pub fn load_albedo(asset_server: &AssetServer, path: impl Into<String>) -> Handle<Image> {
        let path: String = path.into();
        asset_server.load_with_settings(path, |s: &mut ImageLoaderSettings| {
            s.is_srgb = true;
        })
    }

    /// Linear data maps: normal, metallic-roughness, occlusion, height.
    #[must_use]
    pub fn load_linear_map(asset_server: &AssetServer, path: impl Into<String>) -> Handle<Image> {
        let path: String = path.into();
        asset_server.load_with_settings(path, |s: &mut ImageLoaderSettings| {
            s.is_srgb = false;
        })
    }

    /// Albedo + normal paths for a [`Biome`] slug under `assets/textures/`.
    #[must_use]
    pub fn biome_albedo_path(biome: Biome) -> String {
        format!("textures/{}/albedo.jpg", biome.slug())
    }

    /// Albedo + normal paths for a [`Biome`] slug under `assets/textures/`.
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
            Biome::SnowPure => [0.941, 0.965, 1.000],  // #F0F6FF
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
        /// `assets/textures/<slug>/orm.ktx2` (Occlusion / Roughness / Metallic)
        pub orm: String,
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
                orm: format!("textures/{slug}/orm.ktx2"),
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
            let normal: Handle<Image> = asset_server.load(&paths.normal);

            let [r, g, b] = biome.tint_srgb();
            let (roughness, reflectance, metallic) = biome.surface_pbr();
            materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                base_color_texture: Some(albedo),
                normal_map_texture: Some(normal),
                perceptual_roughness: roughness,
                metallic,
                reflectance,
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
}

#[cfg(feature = "bevy")]
use civ_voxel::material::{
    ASH, BEDROCK, CLAY, DIRT, GRANITE, GRAVEL, MUD, PACKED_DIRT, PLANT, SALT, SAND, SNOW, STONE,
    WOOD,
};
#[cfg(feature = "bevy")]
use civ_voxel::MaterialId;

#[cfg(feature = "bevy")]
const TRI_SHADER: &str = "shaders/voxel_triplanar.wgsl";
#[cfg(feature = "bevy")]
static TRI_SHADER_HANDLE: OnceLock<Handle<Shader>> = OnceLock::new();
#[cfg(feature = "bevy")]
const TRI_SCALE: f32 = 0.22;

#[cfg(feature = "bevy")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainTextureLayer {
    Sand,
    Dirt,
    Grass,
    Forest,
    Rock,
    Snow,
}

#[cfg(feature = "bevy")]
impl TerrainTextureLayer {
    const ALL: [Self; 6] = [
        Self::Sand,
        Self::Dirt,
        Self::Grass,
        Self::Forest,
        Self::Rock,
        Self::Snow,
    ];

    #[must_use]
    pub const fn biome(self) -> Biome {
        match self {
            Self::Sand => Biome::SandBeach,
            Self::Dirt => Biome::DirtGround,
            Self::Grass => Biome::GrassField,
            Self::Forest => Biome::ForestFloor,
            Self::Rock => Biome::RockCliff,
            Self::Snow => Biome::SnowPure,
        }
    }
}

#[cfg(feature = "bevy")]
#[must_use]
pub fn terrain_layer_for_material(id: MaterialId) -> Option<TerrainTextureLayer> {
    match id {
        SAND | SALT => Some(TerrainTextureLayer::Sand),
        DIRT | MUD | CLAY | GRAVEL | PACKED_DIRT | ASH => Some(TerrainTextureLayer::Dirt),
        PLANT => Some(TerrainTextureLayer::Grass),
        WOOD => Some(TerrainTextureLayer::Forest),
        STONE | GRANITE | BEDROCK => Some(TerrainTextureLayer::Rock),
        SNOW => Some(TerrainTextureLayer::Snow),
        _ => None,
    }
}

#[cfg(feature = "bevy")]
#[derive(Clone, Copy, Default, ShaderType, Debug)]
pub struct TriplanarParams {
    pub scale: f32,
    pub normal_strength: f32,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    /// `1.0` = flat vertex-color only (LOD / fallback); `0.0` = full triplanar.
    pub vertex_color_blend: f32,
    pub sun_strength: f32,
    pub ambient: f32,
    pub light_dir: Vec3,
}

#[cfg(feature = "bevy")]
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelTriplanarMaterial {
    #[uniform(0)]
    pub params: TriplanarParams,
    #[texture(1)]
    #[sampler(2)]
    pub albedo: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub normal_map: Handle<Image>,
    pub alpha_mode: AlphaMode,
}

#[cfg(feature = "bevy")]
impl Material for VoxelTriplanarMaterial {
    fn fragment_shader() -> ShaderRef {
        TRI_SHADER_HANDLE
            .get()
            .cloned()
            .map(ShaderRef::Handle)
            .unwrap_or_else(|| TRI_SHADER.into())
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[cfg(feature = "bevy")]
#[derive(Resource)]
pub struct VoxelPbrBank {
    layers: [LayerTextures; 6],
    cache: HashMap<MaterialId, Handle<VoxelTriplanarMaterial>>,
}

#[cfg(feature = "bevy")]
#[derive(Clone)]
struct LayerTextures {
    albedo: Handle<Image>,
    normal: Handle<Image>,
}

#[cfg(feature = "bevy")]
impl VoxelPbrBank {
    /// Drop cached handles when the voxel world is regenerated.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Cached triplanar material for a terrain voxel, if mapped.
    pub fn material_for(
        &mut self,
        id: MaterialId,
        assets: &mut bevy::asset::Assets<VoxelTriplanarMaterial>,
    ) -> Option<Handle<VoxelTriplanarMaterial>> {
        if let Some(h) = self.cache.get(&id) {
            return Some(h.clone());
        }
        let layer = terrain_layer_for_material(id)?;
        let handle = assets.add(self.build_material(layer));
        self.cache.insert(id, handle.clone());
        Some(handle)
    }

    fn build_material(&self, layer: TerrainTextureLayer) -> VoxelTriplanarMaterial {
        let idx = layer_index(layer);
        let biome = layer.biome();
        let (roughness, reflectance, metallic) = biome.surface_pbr();
        VoxelTriplanarMaterial {
            params: TriplanarParams {
                scale: TRI_SCALE,
                normal_strength: 0.85,
                perceptual_roughness: roughness,
                metallic,
                reflectance,
                vertex_color_blend: 0.0,
                sun_strength: 0.72,
                ambient: 0.28,
                light_dir: Vec3::new(0.35, 0.85, 0.38).normalize(),
            },
            albedo: self.layers[idx].albedo.clone(),
            normal_map: self.layers[idx].normal.clone(),
            alpha_mode: AlphaMode::Opaque,
        }
    }
}

#[cfg(feature = "bevy")]
fn layer_index(layer: TerrainTextureLayer) -> usize {
    TerrainTextureLayer::ALL
        .iter()
        .position(|&l| l == layer)
        .unwrap_or(0)
}

#[cfg(feature = "bevy")]
pub struct VoxelTriplanarPlugin;

#[cfg(feature = "bevy")]
impl Plugin for VoxelTriplanarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelTriplanarMaterial>::default())
            .add_systems(Startup, load_voxel_pbr_bank);
    }
}

#[cfg(feature = "bevy")]
fn load_voxel_pbr_bank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let shader = shaders.add(Shader::from_wgsl(
        include_str!("../assets/shaders/voxel_triplanar.wgsl"),
        TRI_SHADER,
    ));
    let _ = TRI_SHADER_HANDLE.set(shader);

    let layers = std::array::from_fn(|i| {
        let biome = TerrainTextureLayer::ALL[i].biome();
        LayerTextures {
            albedo: texture_load::load_albedo(
                &asset_server,
                &texture_load::biome_albedo_path(biome),
            ),
            normal: texture_load::load_linear_map(
                &asset_server,
                &texture_load::biome_normal_path(biome),
            ),
        }
    });
    commands.insert_resource(VoxelPbrBank {
        layers,
        cache: HashMap::new(),
    });
}
