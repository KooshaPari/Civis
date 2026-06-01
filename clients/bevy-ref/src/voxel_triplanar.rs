//! Triplanar PBR materials for solid voxel terrain (no UVs).
//!
//! Vendored minimal WGSL (`assets/shaders/voxel_triplanar.wgsl`) — one material
//! instance per terrain texture layer, keyed by [`crate::materials::Biome`] slug.
//! Liquids / emissive voxels keep [`StandardMaterial`] in `voxel_sim`.

use std::collections::HashMap;

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType, SpecializedMeshPipelineError};
use bevy::shader::ShaderRef;

use civ_voxel::material::{
    ASH, BEDROCK, CLAY, DIRT, GRANITE, GRAVEL, MUD, PACKED_DIRT, PLANT, SALT, SAND, SNOW, STONE,
    WOOD,
};
use civ_voxel::MaterialId;

use crate::materials::{texture_load, Biome};

const TRI_SHADER: &str = "shaders/voxel_triplanar.wgsl";
const TRI_SCALE: f32 = 0.22;

/// Terrain texture layer — matches [`Biome`] / `assets/textures/*` slugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerrainTextureLayer {
    Sand,
    Dirt,
    Grass,
    Forest,
    Rock,
    Snow,
}

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

/// Map solid terrain [`MaterialId`] values to a CC0 ground texture layer.
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

/// GPU uniform for [`VoxelTriplanarMaterial`].
#[derive(Clone, Copy, Default, ShaderType)]
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

/// Triplanar terrain material — albedo + normal, world-space projection.
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

impl Material for VoxelTriplanarMaterial {
    fn fragment_shader() -> ShaderRef {
        TRI_SHADER.into()
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
        if let Some(primitive) = &mut descriptor.primitive {
            primitive.cull_mode = None;
        }
        Ok(())
    }
}

/// Loaded CC0 maps + cached [`VoxelTriplanarMaterial`] handles per [`MaterialId`].
#[derive(Resource)]
pub struct VoxelPbrBank {
    layers: [LayerTextures; 6],
    cache: HashMap<MaterialId, Handle<VoxelTriplanarMaterial>>,
}

#[derive(Clone)]
struct LayerTextures {
    albedo: Handle<Image>,
    normal: Handle<Image>,
}

impl VoxelPbrBank {
    /// Drop cached handles when the voxel world is regenerated.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Cached triplanar material for a terrain voxel, if mapped.
    pub fn material_for(
        &mut self,
        id: MaterialId,
        assets: &mut Assets<VoxelTriplanarMaterial>,
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

fn layer_index(layer: TerrainTextureLayer) -> usize {
    TerrainTextureLayer::ALL
        .iter()
        .position(|&l| l == layer)
        .unwrap_or(0)
}

/// Registers triplanar materials and preloads six CC0 ground texture pairs.
pub struct VoxelTriplanarPlugin;

impl Plugin for VoxelTriplanarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelTriplanarMaterial>::default())
            .add_systems(Startup, load_voxel_pbr_bank);
    }
}

fn load_voxel_pbr_bank(mut commands: Commands, asset_server: Res<AssetServer>) {
    let layers = std::array::from_fn(|i| {
        let biome = TerrainTextureLayer::ALL[i].biome();
        LayerTextures {
            albedo: texture_load::load_albedo(&asset_server, &texture_load::biome_albedo_path(biome)),
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
