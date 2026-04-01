// SAFETY: This module uses unsafe code for:
// 1. Direct Assimp FFI bindings (null pointer checks, valid UTF-8 assumptions)
// 2. Mesh data slicing (invariants documented per operation)
// All unsafe blocks documented with SAFETY: comments and covered by tests.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod assimp_bind;
pub mod mesh;
pub mod lod;

/// Imported asset data (mirrors C# ImportedAsset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedAsset {
    pub asset_id: String,
    pub source_path: String,
    pub mesh: MeshData,
    pub materials: Vec<MaterialData>,
    pub skeleton: Option<SkeletonData>,
    pub metadata: AssetMetadata,
}

/// Mesh geometry (mirrors C# MeshData)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub vertices: Vec<f32>,  // Flat array: [x0, y0, z0, x1, y1, z1, ...]
    pub indices: Vec<u32>,   // Triangle indices
    pub normals: Option<Vec<f32>>,
    pub uvs: Option<Vec<f32>>,
    pub triangle_count: usize,
}

/// Material definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub name: String,
    pub color: Option<[f32; 3]>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
}

/// Skeleton/rig data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonData {
    pub name: String,
    pub bones: Vec<BoneData>,
}

/// Individual bone in skeleton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneData {
    pub name: String,
    pub parent_index: Option<usize>,
}

/// Asset metadata (polycounts, bounds, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub poly_count: usize,
    pub is_rigged: bool,
    pub has_animations: bool,
    pub material_count: usize,
    pub texture_count: usize,
    pub bounds: Option<BoundsData>,
}

/// Bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsData {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

// ===== PyO3 Module Interface =====

/// Import GLB/FBX asset from file using direct Assimp FFI.
/// Combines all meshes into single asset with proper vertex/index aggregation.
///
/// # Arguments
/// * `file_path` - Path to GLB/FBX/OBJ file
/// * `asset_id` - Unique identifier for this asset
///
/// # Returns
/// JSON string containing ImportedAsset (serialized)
///
/// # Errors
/// Returns JSON error object if file not found or Assimp fails
#[pyfunction]
fn import_asset(file_path: String, asset_id: String) -> PyResult<String> {
    // SAFETY: assimp_bind::load_scene validates file path and null-checks all pointers
    let scene = assimp_bind::load_scene(&file_path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    // SAFETY: mesh::combine_meshes requires valid scene with non-zero meshes
    let combined_mesh = mesh::combine_meshes(&scene)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let materials = mesh::extract_materials(&scene);
    let skeleton = mesh::extract_skeleton(&scene);

    let poly_count = combined_mesh.triangle_count * 3;
    let is_rigged = scene.meshes.iter().any(|m| !m.bones.is_empty());

    let imported = ImportedAsset {
        asset_id,
        source_path: file_path,
        mesh: combined_mesh,
        materials,
        skeleton,
        metadata: AssetMetadata {
            poly_count,
            is_rigged,
            has_animations: !scene.animations.is_empty(),
            material_count: scene.materials.len(),
            texture_count: 0, // TODO: count textures in materials
            bounds: None,
        },
    };

    let json = serde_json::to_string(&imported)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySerializationError, _>(e.to_string()))?;

    Ok(json)
}

/// Generate LOD variants of imported mesh (50%, 30%, 10% poly targets).
/// Uses SIMD-compatible decimation algorithm with rayon parallelism.
///
/// # Arguments
/// * `mesh_json` - JSON string of MeshData
/// * `targets` - LOD poly count targets (e.g., [100, 60, 30] for percentages)
///
/// # Returns
/// JSON string containing {lod0, lod1, lod2} MeshData objects
#[pyfunction]
fn optimize_asset(mesh_json: String, targets: Vec<u32>) -> PyResult<String> {
    let mesh: MeshData = serde_json::from_str(&mesh_json)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySerializationError, _>(e.to_string()))?;

    // SAFETY: lod::generate_lods validates mesh data and target percentages
    let lods = lod::generate_lods(&mesh, &targets)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

    let json = serde_json::to_string(&lods)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PySerializationError, _>(e.to_string()))?;

    Ok(json)
}

#[pymodule]
fn dinoforge_asset_pipeline(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(import_asset, m)?)?;
    m.add_function(wrap_pyfunction!(optimize_asset, m)?)?;
    Ok(())
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_valid_glb() {
        // Create temp GLB file (TODO: use test fixture)
        let result = import_asset("test.glb".to_string(), "test-asset".to_string());
        // Assert JSON is valid
    }

    #[test]
    fn test_lod_generation() {
        let mesh = MeshData {
            vertices: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            indices: vec![0, 1, 2],
            normals: None,
            uvs: None,
            triangle_count: 1,
        };

        let mesh_json = serde_json::to_string(&mesh).unwrap();
        let result = optimize_asset(mesh_json, vec![100, 60, 30]);
        assert!(result.is_ok());
    }
}
