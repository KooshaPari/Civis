#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;

namespace DINOForge.Tools.PackCompiler.Models
{
    /// <summary>
    /// In-memory representation of an imported 3D model (GLB/FBX).
    /// Intermediate format between raw GLB/FBX and optimized LOD variants.
    /// </summary>
    public class ImportedAsset : IValidatable
    {
        /// <summary>Asset identifier (e.g., clone_trooper_phase2)</summary>
        public required string AssetId { get; init; }

        /// <summary>Source GLB/FBX file path</summary>
        public required string SourcePath { get; init; }

        /// <summary>Mesh data (vertices, indices, normals, tangents, UVs)</summary>
        public required MeshData Mesh { get; init; }

        /// <summary>Material data (shader refs, colors, properties)</summary>
        public List<MaterialData> Materials { get; init; } = new();

        /// <summary>Skeleton data (bones, weights) if rigged</summary>
        public SkeletonData? Skeleton { get; init; }

        /// <summary>Metadata extracted from import</summary>
        public AssetMetadata Metadata { get; init; } = new();

        /// <summary>Timestamp of import</summary>
        public DateTime ImportedAt { get; init; } = DateTime.UtcNow;

        /// <summary>
        /// Validates that the imported asset has required fields and valid structure.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            // AssetId must not be empty
            if (string.IsNullOrWhiteSpace(AssetId))
                errors.Add(new ValidationError("asset_id", "AssetId is required and cannot be empty.", "validation"));

            // Mesh must exist
            if (Mesh == null)
                errors.Add(new ValidationError("mesh", "Mesh is required.", "validation"));
            else
            {
                // Validate mesh vertices are multiple of 3 (Vector3 per vertex)
                if (Mesh.Vertices == null || Mesh.Vertices.Length == 0)
                    errors.Add(new ValidationError("mesh.vertices", "Vertices array cannot be empty.", "validation"));
                else if (Mesh.Vertices.Length % 3 != 0)
                    errors.Add(new ValidationError("mesh.vertices", "Vertices array length must be a multiple of 3 (Vector3).", "validation"));

                // Validate mesh indices are multiple of 3 (triangle indices)
                if (Mesh.Indices == null || Mesh.Indices.Length == 0)
                    errors.Add(new ValidationError("mesh.indices", "Indices array cannot be empty.", "validation"));
                else if (Mesh.Indices.Length % 3 != 0)
                    errors.Add(new ValidationError("mesh.indices", "Indices array length must be a multiple of 3 (triangles).", "validation"));
            }

            return errors.Count == 0 ? ValidationResult.Success() : ValidationResult.Failure((IReadOnlyList<ValidationError>)errors);
        }
    }

    /// <summary>Mesh geometry data</summary>
    public class MeshData
    {
        /// <summary>Mesh name</summary>
        public required string Name { get; init; }

        /// <summary>Vertex positions (Vector3[])</summary>
        public required float[] Vertices { get; init; }

        /// <summary>Mesh indices (uint[])</summary>
        public required uint[] Indices { get; init; }

        /// <summary>Vertex normals</summary>
        public float[]? Normals { get; init; }

        /// <summary>Vertex tangents</summary>
        public float[]? Tangents { get; init; }

        /// <summary>UV coordinates (channel 0)</summary>
        public float[]? UVs { get; init; }

        /// <summary>Vertex colors if present</summary>
        public float[]? Colors { get; init; }

        /// <summary>Bone weights for rigged meshes</summary>
        public BoneWeight[]? BoneWeights { get; init; }

        /// <summary>Total vertex count</summary>
        public int VertexCount => Vertices.Length / 3;

        /// <summary>Total triangle count</summary>
        public int TriangleCount => Indices.Length / 3;

        /// <summary>Calculated bounds (min, max)</summary>
        public (float[] Min, float[] Max)? Bounds { get; set; }
    }

    /// <summary>Bone weight data for skinning</summary>
    public class BoneWeight
    {
        /// <summary>Bone indices (up to 4)</summary>
        public required int[] Indices { get; init; }

        /// <summary>Blend weights (normalized to sum to 1.0)</summary>
        public required float[] Weights { get; init; }
    }

    /// <summary>Material properties</summary>
    public class MaterialData
    {
        /// <summary>Material name</summary>
        public required string Name { get; init; }

        /// <summary>Shader name/path</summary>
        public string? ShaderName { get; init; }

        /// <summary>True when the importer could only preserve placeholder material metadata.</summary>
        public bool IsFallbackMaterial { get; init; }

        /// <summary>Albedo/diffuse color</summary>
        public float[]? BaseColor { get; init; }  // RGBA

        /// <summary>Normal map texture path (if embedded)</summary>
        public string? NormalMap { get; init; }

        /// <summary>Material-level import warnings that should be surfaced by validation.</summary>
        public List<string> MaterialWarnings { get; init; } = new();

        /// <summary>Roughness value</summary>
        public float Roughness { get; set; } = 0.5f;

        /// <summary>Metallic value</summary>
        public float Metallic { get; set; } = 0f;

        /// <summary>Custom properties dict for extensibility</summary>
        public Dictionary<string, object> Properties { get; init; } = new();
    }

    /// <summary>Skeleton/armature data for rigged models</summary>
    public class SkeletonData
    {
        /// <summary>Root bone name</summary>
        public required string RootBone { get; init; }

        /// <summary>All bones in hierarchy</summary>
        public required List<BoneInfo> Bones { get; init; }

        /// <summary>Bone count</summary>
        public int BoneCount => Bones.Count;
    }

    /// <summary>Individual bone information</summary>
    public class BoneInfo
    {
        /// <summary>Bone name</summary>
        public required string Name { get; init; }

        /// <summary>Parent bone name (null if root)</summary>
        public string? ParentName { get; init; }

        /// <summary>Local position relative to parent</summary>
        public required float[] LocalPosition { get; init; }  // [x, y, z]

        /// <summary>Local rotation (quaternion)</summary>
        public required float[] LocalRotation { get; init; }  // [x, y, z, w]

        /// <summary>Local scale</summary>
        public required float[] LocalScale { get; init; }  // [x, y, z]
    }

    /// <summary>Metadata extracted from imported model</summary>
    public class AssetMetadata
    {
        /// <summary>Actual polycount (vertex count * 3)</summary>
        public int PolyCount { get; set; }

        /// <summary>Model bounds (for scaling/positioning)</summary>
        public (float[] Min, float[] Max)? Bounds { get; set; }

        /// <summary>Is model rigged/skeletal?</summary>
        public bool IsRigged { get; set; }

        /// <summary>Has animations?</summary>
        public bool HasAnimations { get; set; }

        /// <summary>Material count</summary>
        public int MaterialCount { get; set; }

        /// <summary>Embedded texture count</summary>
        public int TextureCount { get; set; }

        /// <summary>Import notes/warnings</summary>
        public List<string> Notes { get; init; } = new();
    }
}
