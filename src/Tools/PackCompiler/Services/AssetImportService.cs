#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Assimp;
using DINOForge.Tools.PackCompiler.Models;
using Quaternion = Assimp.Quaternion;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Service for importing 3D models (GLB/FBX/OBJ) using AssimpNet.
    /// Parses geometry, materials, and skeleton data into ImportedAsset format.
    /// </summary>
    public partial class AssetImportService
    {
        private readonly AssimpContext _assimpContext = new();

        /// <summary>
        /// Import a model from disk (GLB, FBX, OBJ, etc.)
        /// </summary>
        public async Task<ImportedAsset> ImportAsync(string assetId, string filePath)
        {
            if (!File.Exists(filePath))
            {
                throw new FileNotFoundException($"Model file not found: {filePath}");
            }

            return await Task.Run(() =>
            {
                var scene = _assimpContext.ImportFile(filePath, PostProcessSteps.Triangulate | PostProcessSteps.GenerateNormals);

                if (scene == null || scene.MeshCount == 0)
                {
                    throw new InvalidOperationException($"Failed to import model or no meshes found: {filePath}");
                }

                // Combine all meshes into single asset with proper vertex/index aggregation
                var combinedMeshData = CombineMultipleMeshes(scene.Meshes);
                var materials = ExtractMaterials(scene);

                // Extract skeleton from first rigged mesh if available
                SkeletonData? skeleton = null;
                foreach (var mesh in scene.Meshes)
                {
                    if (mesh.HasBones)
                    {
                        skeleton = ExtractSkeleton(mesh);
                        break;
                    }
                }

                // Calculate total polygon count across all meshes
                int totalPolyCount = scene.Meshes.Sum(m => m.VertexCount);
                bool isRigged = scene.Meshes.Any(m => m.HasBones);

                return new ImportedAsset
                {
                    AssetId = assetId,
                    SourcePath = filePath,
                    Mesh = combinedMeshData,
                    Materials = materials,
                    Skeleton = skeleton,
                    Metadata = new AssetMetadata
                    {
                        PolyCount = totalPolyCount,
                        IsRigged = isRigged,
                        HasAnimations = scene.AnimationCount > 0,
                        MaterialCount = scene.MaterialCount,
                        TextureCount = ExtractTextureCount(scene),
                        Bounds = CalculateBounds(combinedMeshData),
                        Notes = new List<string>
                        {
                            scene.MeshCount > 1 ? $"Combined {scene.MeshCount} meshes into single asset" : "Single mesh model"
                        }
                    }
                };
            });
        }

        /// <summary>
        /// Combines multiple meshes into a single MeshData by aggregating vertices and indices.
        /// Handles vertex offset remapping for indices and combines all mesh attributes.
        /// </summary>
        private MeshData CombineMultipleMeshes(IList<Mesh> meshes)
        {
            if (meshes.Count == 0)
            {
                throw new InvalidOperationException("Cannot combine zero meshes");
            }

            // If only one mesh, extract it directly
            if (meshes.Count == 1)
            {
                return ExtractMeshData(meshes[0]);
            }

            // Calculate total vertex and face counts
            int totalVertexCount = meshes.Sum(m => m.VertexCount);
            int totalFaceCount = meshes.Sum(m => m.FaceCount);

            // Combined arrays
            var combinedVertices = new float[totalVertexCount * 3];
            var combinedIndices = new uint[totalFaceCount * 3];
            var combinedNormals = meshes.All(m => m.HasNormals) ? new float[totalVertexCount * 3] : null;
            var combinedUVs = meshes.All(m => m.HasTextureCoords(0)) ? new float[totalVertexCount * 2] : null;
            var combinedBoneWeights = meshes.Any(m => m.HasBones) ? new BoneWeight[totalVertexCount] : null;

            // Initialize bone weight indices
            for (int i = 0; i < totalVertexCount; i++)
            {
                if (combinedBoneWeights != null)
                {
                    combinedBoneWeights[i] = new BoneWeight
                    {
                        Indices = new int[4],
                        Weights = new float[4]
                    };
                }
            }

            int vertexOffset = 0;
            int faceOffset = 0;

            // Merge each mesh's data
            foreach (var mesh in meshes)
            {
                // Copy vertices
                for (int i = 0; i < mesh.VertexCount; i++)
                {
                    var v = mesh.Vertices[i];
                    combinedVertices[(vertexOffset + i) * 3] = v.X;
                    combinedVertices[(vertexOffset + i) * 3 + 1] = v.Y;
                    combinedVertices[(vertexOffset + i) * 3 + 2] = v.Z;
                }

                // Copy indices with vertex offset
                for (int i = 0; i < mesh.FaceCount; i++)
                {
                    var face = mesh.Faces[i];
                    if (face.IndexCount == 3)
                    {
                        combinedIndices[(faceOffset + i) * 3] = (uint)(face.Indices[0] + vertexOffset);
                        combinedIndices[(faceOffset + i) * 3 + 1] = (uint)(face.Indices[1] + vertexOffset);
                        combinedIndices[(faceOffset + i) * 3 + 2] = (uint)(face.Indices[2] + vertexOffset);
                    }
                }

                // Copy normals
                if (combinedNormals != null && mesh.HasNormals)
                {
                    for (int i = 0; i < mesh.VertexCount; i++)
                    {
                        var n = mesh.Normals[i];
                        combinedNormals[(vertexOffset + i) * 3] = n.X;
                        combinedNormals[(vertexOffset + i) * 3 + 1] = n.Y;
                        combinedNormals[(vertexOffset + i) * 3 + 2] = n.Z;
                    }
                }

                // Copy UVs
                if (combinedUVs != null && mesh.HasTextureCoords(0))
                {
                    for (int i = 0; i < mesh.VertexCount; i++)
                    {
                        var uv = mesh.TextureCoordinateChannels[0][i];
                        combinedUVs[(vertexOffset + i) * 2] = uv.X;
                        combinedUVs[(vertexOffset + i) * 2 + 1] = uv.Y;
                    }
                }

                // Copy bone weights
                if (combinedBoneWeights != null && mesh.HasBones)
                {
                    var meshBoneWeights = ExtractBoneWeights(mesh);
                    for (int i = 0; i < mesh.VertexCount; i++)
                    {
                        if (vertexOffset + i < combinedBoneWeights.Length)
                        {
                            combinedBoneWeights[vertexOffset + i] = meshBoneWeights[i];
                        }
                    }
                }

                vertexOffset += mesh.VertexCount;
                faceOffset += mesh.FaceCount;
            }

            var meshName = $"{meshes[0].Name}" + (meshes.Count > 1 ? $"_combined_{meshes.Count}" : "");

            return new MeshData
            {
                Name = meshName,
                Vertices = combinedVertices,
                Indices = combinedIndices,
                Normals = combinedNormals,
                UVs = combinedUVs,
                BoneWeights = combinedBoneWeights,
                Bounds = CalculateBounds(combinedVertices)
            };
        }

        /// <summary>Extract mesh geometry (vertices, indices, normals, UVs)</summary>
        private MeshData ExtractMeshData(Mesh mesh)
        {
            var vertices = new float[mesh.VertexCount * 3];
            for (int i = 0; i < mesh.VertexCount; i++)
            {
                var v = mesh.Vertices[i];
                vertices[i * 3] = v.X;
                vertices[i * 3 + 1] = v.Y;
                vertices[i * 3 + 2] = v.Z;
            }

            var indices = new uint[mesh.FaceCount * 3];
            for (int i = 0; i < mesh.FaceCount; i++)
            {
                var face = mesh.Faces[i];
                if (face.IndexCount == 3)
                {
                    indices[i * 3] = (uint)face.Indices[0];
                    indices[i * 3 + 1] = (uint)face.Indices[1];
                    indices[i * 3 + 2] = (uint)face.Indices[2];
                }
            }

            // Extract normals
            float[]? normals = null;
            if (mesh.HasNormals)
            {
                normals = new float[mesh.VertexCount * 3];
                for (int i = 0; i < mesh.VertexCount; i++)
                {
                    var n = mesh.Normals[i];
                    normals[i * 3] = n.X;
                    normals[i * 3 + 1] = n.Y;
                    normals[i * 3 + 2] = n.Z;
                }
            }

            // Extract UVs (channel 0)
            float[]? uvs = null;
            if (mesh.HasTextureCoords(0))
            {
                uvs = new float[mesh.VertexCount * 2];
                for (int i = 0; i < mesh.VertexCount; i++)
                {
                    var uv = mesh.TextureCoordinateChannels[0][i];
                    uvs[i * 2] = uv.X;
                    uvs[i * 2 + 1] = uv.Y;
                }
            }

            // Extract bone weights if rigged
            BoneWeight[]? boneWeights = null;
            if (mesh.HasBones)
            {
                boneWeights = ExtractBoneWeights(mesh);
            }

            return new MeshData
            {
                Name = mesh.Name,
                Vertices = vertices,
                Indices = indices,
                Normals = normals,
                UVs = uvs,
                BoneWeights = boneWeights,
                Bounds = CalculateBounds(vertices)
            };
        }

        /// <summary>Extract material definitions</summary>
        private List<MaterialData> ExtractMaterials(Scene scene)
        {
            var materials = new List<MaterialData>();
            foreach (var material in scene.Materials)
            {
                var baseColor = material.ColorDiffuse;
                materials.Add(new MaterialData
                {
                    Name = material.Name,
                    ShaderName = "Standard",
                    BaseColor = new[] { baseColor.R, baseColor.G, baseColor.B, baseColor.A },
                    Roughness = 0.5f,
                    Metallic = 0f
                });
            }
            return materials;
        }

        /// <summary>Extract skeleton/bone data</summary>
        private SkeletonData? ExtractSkeleton(Mesh mesh)
        {
            if (!mesh.HasBones || mesh.BoneCount == 0)
                return null;

            var bones = new List<BoneInfo>();
            foreach (var bone in mesh.Bones)
            {
                var matrix = bone.OffsetMatrix;
                var pos = new Vector3D(matrix.A4, matrix.B4, matrix.C4);
                var rot = ExtractQuaternion(matrix);

                bones.Add(new BoneInfo
                {
                    Name = bone.Name,
                    LocalPosition = new[] { pos.X, pos.Y, pos.Z },
                    LocalRotation = new[] { rot.X, rot.Y, rot.Z, rot.W },
                    LocalScale = new[] { 1f, 1f, 1f }
                });
            }

            return new SkeletonData
            {
                RootBone = mesh.Bones.FirstOrDefault()?.Name ?? "Armature",
                Bones = bones
            };
        }

        /// <summary>Extract bone weights from mesh</summary>
        private BoneWeight[] ExtractBoneWeights(Mesh mesh)
        {
            var weights = new BoneWeight[mesh.VertexCount];
            for (int i = 0; i < mesh.VertexCount; i++)
            {
                weights[i] = new BoneWeight
                {
                    Indices = new int[4],
                    Weights = new float[4]
                };
            }

            for (int b = 0; b < mesh.BoneCount && b < 4; b++)
            {
                var bone = mesh.Bones[b];
                foreach (var vertWeight in bone.VertexWeights)
                {
                    if (vertWeight.VertexID < weights.Length)
                    {
                        weights[vertWeight.VertexID].Indices[b] = b;
                        weights[vertWeight.VertexID].Weights[b] = vertWeight.Weight;
                    }
                }
            }

            return weights;
        }

        /// <summary>Count embedded textures in scene</summary>
        private int ExtractTextureCount(Scene scene)
        {
            return scene.TextureCount;
        }

        /// <summary>Calculate mesh bounds (min/max)</summary>
        private (float[] Min, float[] Max) CalculateBounds(MeshData meshData)
        {
            return CalculateBounds(meshData.Vertices);
        }

        /// <summary>Calculate bounds from vertex array</summary>
        private (float[] Min, float[] Max) CalculateBounds(float[] vertices)
        {
            if (vertices.Length < 3)
                return (new[] { 0f, 0f, 0f }, new[] { 0f, 0f, 0f });

            float minX = vertices[0], minY = vertices[1], minZ = vertices[2];
            float maxX = minX, maxY = minY, maxZ = minZ;

            for (int i = 0; i < vertices.Length; i += 3)
            {
                minX = Math.Min(minX, vertices[i]);
                minY = Math.Min(minY, vertices[i + 1]);
                minZ = Math.Min(minZ, vertices[i + 2]);

                maxX = Math.Max(maxX, vertices[i]);
                maxY = Math.Max(maxY, vertices[i + 1]);
                maxZ = Math.Max(maxZ, vertices[i + 2]);
            }

            return (new[] { minX, minY, minZ }, new[] { maxX, maxY, maxZ });
        }

        /// <summary>Extract quaternion from transformation matrix</summary>
        private Quaternion ExtractQuaternion(Matrix4x4 matrix)
        {
            // Extract quaternion from rotation matrix using standard algorithm
            // Based on Shepperd's method
            float trace = matrix.A1 + matrix.B2 + matrix.C3;

            if (trace > 0)
            {
                float s = 0.5f / (float)Math.Sqrt(trace + 1.0f);
                return new Quaternion(
                    (matrix.B3 - matrix.C2) * s,
                    (matrix.C1 - matrix.A3) * s,
                    (matrix.A2 - matrix.B1) * s,
                    0.25f / s
                );
            }
            else if (matrix.A1 > matrix.B2 && matrix.A1 > matrix.C3)
            {
                float s = 2.0f * (float)Math.Sqrt(1.0f + matrix.A1 - matrix.B2 - matrix.C3);
                return new Quaternion(
                    0.25f * s,
                    (matrix.B1 + matrix.A2) / s,
                    (matrix.C1 + matrix.A3) / s,
                    (matrix.B3 - matrix.C2) / s
                );
            }
            else if (matrix.B2 > matrix.C3)
            {
                float s = 2.0f * (float)Math.Sqrt(1.0f + matrix.B2 - matrix.A1 - matrix.C3);
                return new Quaternion(
                    (matrix.B1 + matrix.A2) / s,
                    0.25f * s,
                    (matrix.C2 + matrix.B3) / s,
                    (matrix.C1 - matrix.A3) / s
                );
            }
            else
            {
                float s = 2.0f * (float)Math.Sqrt(1.0f + matrix.C3 - matrix.A1 - matrix.B2);
                return new Quaternion(
                    (matrix.C1 + matrix.A3) / s,
                    (matrix.C2 + matrix.B3) / s,
                    0.25f * s,
                    (matrix.A2 - matrix.B1) / s
                );
            }
        }
    }
}
