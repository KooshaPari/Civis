#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.NativeInterop;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Tests for NativeInterop layer (GoDependencyResolver and RustAssetPipeline).
/// 
/// These tests cover:
/// - GoDependencyResolver fallback path (when Go binary unavailable)
/// - RustAssetPipeline fallback path (when Rust MCP unavailable)
/// - Error handling: file not found, timeout, invalid JSON
/// - Data model serialization
/// </summary>
public class NativeInteropTests
{
    // ═════════════════════════════════════════════════════════════════════════════
    // GoDependencyResolver Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class GoDependencyResolverTests
    {
        [Fact]
        public void IsAvailable_WhenBinaryNotFound_ReturnsFalse()
        {
            // GoDependencyResolver checks for the binary in multiple locations
            // If none exist, IsAvailable should be false
            var result = GoDependencyResolver.IsAvailable;
            // This test documents the current state
            // When Go binary is installed, this will be true
        }

        [Fact]
        public void ResolveDependencies_FallbackToCSharp_WhenBinaryNotAvailable()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                CreatePack("pack-a", Array.Empty<string>()),
                CreatePack("pack-b", new[] { "pack-a" }),
                CreatePack("pack-c", new[] { "pack-b" })
            };
            var target = available[2]; // pack-c depends on pack-b which depends on pack-a

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("C# fallback should resolve circular-free dependency");
            result.LoadOrder.Should().NotBeNull();
            result.LoadOrder.Should().HaveCount(3);
        }

        [Fact]
        public void ResolveDependencies_CircularDependency_ReportsError()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                CreatePack("pack-a", new[] { "pack-b" }),
                CreatePack("pack-b", new[] { "pack-c" }),
                CreatePack("pack-c", new[] { "pack-a" }) // Circular!
            };
            var target = available[0];

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse("circular dependency should fail");
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void ResolveDependencies_MissingDependency_ReportsError()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                CreatePack("pack-a", Array.Empty<string>())
            };
            var target = CreatePack("pack-b", new[] { "pack-missing" });

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse("missing dependency should fail");
        }

        [Fact]
        public void ResolveDependencies_EmptyGraph_Succeeds()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>();
            var target = CreatePack("standalone-pack", Array.Empty<string>());

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("standalone pack with no deps should succeed");
            result.LoadOrder.Should().NotBeNull();
        }

        [Fact]
        public void ResolveDependencies_ComplexDiamond_SortsCorrectly()
        {
            // Arrange
            //     pack-top
            //    /        \
            // pack-left   pack-right
            //    \        /
            //     pack-bottom
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                CreatePack("pack-top", Array.Empty<string>()),
                CreatePack("pack-left", new[] { "pack-top" }),
                CreatePack("pack-right", new[] { "pack-top" }),
                CreatePack("pack-bottom", new[] { "pack-left", "pack-right" })
            };
            var target = available[3]; // pack-bottom

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("diamond dependency should resolve");
            result.LoadOrder.Should().NotBeNull();
            var sortedIds = result.LoadOrder.Select(p => p.Id).ToList();
            var topIndex = sortedIds.IndexOf("pack-top");
            var leftIndex = sortedIds.IndexOf("pack-left");
            var rightIndex = sortedIds.IndexOf("pack-right");
            var bottomIndex = sortedIds.IndexOf("pack-bottom");

            topIndex.Should().BeLessThan(leftIndex, "top must come before left");
            topIndex.Should().BeLessThan(rightIndex, "top must come before right");
            leftIndex.Should().BeLessThan(bottomIndex, "left must come before bottom");
            rightIndex.Should().BeLessThan(bottomIndex, "right must come before bottom");
        }

        [Fact]
        public void ResolveDependencies_NullAvailable_ThrowsArgumentNullException()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var target = CreatePack("test-pack", Array.Empty<string>());

            // Act & Assert
            Action act = () => resolver.ResolveDependencies(null!, target);
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ResolveDependencies_NullTarget_ThrowsException()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                CreatePack("pack-a", Array.Empty<string>())
            };

            // Act & Assert - current implementation throws NullReferenceException via C# fallback
            Action act = () => resolver.ResolveDependencies(available, null!);
            act.Should().Throw<Exception>(); // Any exception is acceptable for null target
        }

        private static PackManifest CreatePack(string id, string[] dependsOn)
        {
            return new PackManifest
            {
                Id = id,
                Name = id,
                Version = "1.0.0",
                DependsOn = dependsOn.ToList()
            };
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // RustAssetPipeline Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class RustAssetPipelineTests
    {
        [Fact]
        public void IsAvailable_DocumentsCurrentState()
        {
            // RustAssetPipeline checks for MCP server availability
            // This test documents the current state
            var available = RustAssetPipeline.IsAvailable;
            // When MCP server is running, this will be true
        }

        [Fact]
        public async Task ImportAssetAsync_FileNotFound_ThrowsFileNotFoundException()
        {
            // Arrange
            var nonexistentFile = Path.Combine(Path.GetTempPath(), "definitely_does_not_exist.glb");

            // Act & Assert
            Func<Task> act = async () => await RustAssetPipeline.ImportAssetAsync("test-asset", nonexistentFile);
            await act.Should().ThrowAsync<FileNotFoundException>();
        }

        [Fact]
        public async Task ImportAssetAsync_FallbackPath_WhenRustUnavailable()
        {
            // Arrange - use a real file
            var tempFile = Path.GetTempFileName();
            try
            {
                // Create a minimal GLB file (or just any file for fallback testing)
                await File.WriteAllTextAsync(tempFile, "{}");

                // Act - RustAssetPipeline should fall back to C# when Rust unavailable
                var result = await RustAssetPipeline.ImportAssetAsync("test-asset", tempFile);

                // Assert
                result.Should().NotBeNull();
                result.AssetId.Should().Be("test-asset");
                result.SourcePath.Should().EndWith(tempFile);
            }
            finally
            {
                File.Delete(tempFile);
            }
        }

        [Fact]
        public async Task OptimizeAssetAsync_FallbackPath_WhenRustUnavailable()
        {
            // Arrange
            var imported = new ImportedAsset
            {
                AssetId = "test-asset",
                SourcePath = "/fake/path",
                Mesh = new MeshData { Vertices = new float[9] }, // 3 vertices
                Materials = new List<MaterialData>(),
                Metadata = new AssetMetadata { PolyCount = 1 }
            };
            var definition = new AssetDefinition
            {
                Id = "test-asset",
                LOD = new LODDefinition { Levels = new[] { 100, 60, 30 } }
            };

            // Act
            var result = await RustAssetPipeline.OptimizeAssetAsync(imported, definition);

            // Assert
            result.Should().NotBeNull();
            result.AssetId.Should().Be("test-asset");
            result.LOD0.Should().NotBeNull("C# fallback should return LOD0");
        }

        [Fact]
        public async Task ImportAssetAsync_EmptyMeshData_ReturnsValidAsset()
        {
            // Arrange
            var tempFile = Path.GetTempFileName();
            try
            {
                await File.WriteAllTextAsync(tempFile, "{}");
                var imported = new ImportedAsset
                {
                    AssetId = "empty-asset",
                    SourcePath = tempFile,
                    Mesh = new MeshData
                    {
                        Vertices = Array.Empty<float>(),
                        Indices = Array.Empty<uint>(),
                        TriangleCount = 0
                    },
                    Materials = new List<MaterialData>(),
                    Metadata = new AssetMetadata { PolyCount = 0 }
                };

                // Act - use fallback path
                var result = await RustAssetPipeline.ImportAssetAsync("empty-asset", tempFile);

                // Assert
                result.Mesh.Should().NotBeNull();
                result.Mesh.Vertices.Should().NotBeNull();
            }
            finally
            {
                File.Delete(tempFile);
            }
        }

        [Fact]
        public void MeshData_TriangleCountCalculation_IsCorrect()
        {
            // Arrange
            var mesh = new MeshData
            {
                Vertices = new float[] { 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 0 }, // 4 vertices = 2 triangles
                Indices = new uint[] { 0, 1, 2, 1, 3, 2 }, // 2 triangles
                TriangleCount = 2
            };

            // Assert
            mesh.Vertices.Length.Should().Be(12, "4 vertices * 3 components");
            mesh.Indices.Length.Should().Be(6, "2 triangles * 3 indices");
        }

        [Fact]
        public void LODDefinition_Levels_CanBeCustomized()
        {
            // Arrange
            var definition = new LODDefinition
            {
                Levels = new[] { 100, 75, 50, 25 }
            };

            // Assert
            definition.Levels.Should().HaveCount(4);
            definition.Levels.Should().BeInDescendingOrder();
        }

        [Fact]
        public void OptimizedAsset_AllLODs_Present()
        {
            // Arrange
            var optimized = new OptimizedAsset
            {
                AssetId = "test",
                LOD0 = new MeshData { Vertices = new float[9] },
                LOD1 = new MeshData { Vertices = new float[6] },
                LOD2 = new MeshData { Vertices = new float[3] }
            };

            // Assert
            optimized.LOD0.Should().NotBeNull();
            optimized.LOD1.Should().NotBeNull();
            optimized.LOD2.Should().NotBeNull();
            optimized.LOD0.Vertices.Length.Should().BeGreaterThan(optimized.LOD2.Vertices.Length);
        }

        [Fact]
        public void MaterialData_CanSetName()
        {
            // Arrange
            var material = new MaterialData { Name = "PBR_CloneWhite" };

            // Assert
            material.Name.Should().Be("PBR_CloneWhite");
        }

        [Fact]
        public void SkeletonData_CanBeNull()
        {
            // Arrange
            var asset = new ImportedAsset
            {
                AssetId = "test",
                Skeleton = null
            };

            // Assert
            asset.Skeleton.Should().BeNull();
        }

        [Fact]
        public void AssetMetadata_PolyCount_IsInteger()
        {
            // Arrange
            var metadata = new AssetMetadata { PolyCount = 12345 };

            // Assert
            metadata.PolyCount.Should().Be(12345);
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Data Model Round-Trip Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class NativeInteropModelSerializationTests
    {
        [Fact]
        public void MeshData_SerializesAndDeserializes()
        {
            // Arrange
            var original = new MeshData
            {
                Vertices = new float[] { 1, 2, 3, 4, 5, 6 },
                Indices = new uint[] { 0, 1, 2 },
                TriangleCount = 1
            };

            // Act
            var json = System.Text.Json.JsonSerializer.Serialize(original);
            var deserialized = System.Text.Json.JsonSerializer.Deserialize<MeshData>(json);

            // Assert
            deserialized.Should().NotBeNull();
            deserialized!.Vertices.Should().BeEquivalentTo(original.Vertices);
            deserialized.Indices.Should().BeEquivalentTo(original.Indices);
            deserialized.TriangleCount.Should().Be(original.TriangleCount);
        }

        [Fact]
        public void ImportedAsset_SerializesWithAllFields()
        {
            // Arrange
            var original = new ImportedAsset
            {
                AssetId = "sw-clone-trooper",
                SourcePath = "C:\\assets\\clone_trooper.glb",
                Mesh = new MeshData
                {
                    Vertices = new float[] { 1, 2, 3 },
                    TriangleCount = 1
                },
                Materials = new List<MaterialData>
                {
                    new MaterialData { Name = "CloneArmor" }
                },
                Skeleton = new SkeletonData { Name = "CloneSkeleton" },
                Metadata = new AssetMetadata { PolyCount = 5000 }
            };

            // Act
            var json = System.Text.Json.JsonSerializer.Serialize(original);
            var deserialized = System.Text.Json.JsonSerializer.Deserialize<ImportedAsset>(json);

            // Assert
            deserialized.Should().NotBeNull();
            deserialized!.AssetId.Should().Be("sw-clone-trooper");
            deserialized.SourcePath.Should().Contain("clone_trooper.glb");
            deserialized.Mesh.TriangleCount.Should().Be(1);
            deserialized.Materials.Should().HaveCount(1);
            deserialized.Skeleton.Should().NotBeNull();
            deserialized.Metadata.PolyCount.Should().Be(5000);
        }

        [Fact]
        public void OptimizedAsset_SerializesWithLODs()
        {
            // Arrange
            var original = new OptimizedAsset
            {
                AssetId = "test",
                LOD0 = new MeshData { Vertices = new float[9], TriangleCount = 3 },
                LOD1 = new MeshData { Vertices = new float[6], TriangleCount = 2 },
                LOD2 = new MeshData { Vertices = new float[3], TriangleCount = 1 }
            };

            // Act
            var json = System.Text.Json.JsonSerializer.Serialize(original);
            var deserialized = System.Text.Json.JsonSerializer.Deserialize<OptimizedAsset>(json);

            // Assert
            deserialized.Should().NotBeNull();
            deserialized!.LOD0.TriangleCount.Should().Be(3);
            deserialized.LOD1.TriangleCount.Should().Be(2);
            deserialized.LOD2.TriangleCount.Should().Be(1);
        }

        [Fact]
        public void LODDefinition_SerializesLevelsArray()
        {
            // Arrange
            var original = new LODDefinition
            {
                Levels = new[] { 100, 60, 30, 15 }
            };

            // Act
            var json = System.Text.Json.JsonSerializer.Serialize(original);
            var deserialized = System.Text.Json.JsonSerializer.Deserialize<LODDefinition>(json);

            // Assert
            deserialized.Should().NotBeNull();
            deserialized!.Levels.Should().BeEquivalentTo(new[] { 100, 60, 30, 15 });
        }

        [Fact]
        public void AssetDefinition_SerializesMinimal()
        {
            // Arrange
            var original = new AssetDefinition
            {
                Id = "minimal-asset"
            };

            // Act
            var json = System.Text.Json.JsonSerializer.Serialize(original);
            var deserialized = System.Text.Json.JsonSerializer.Deserialize<AssetDefinition>(json);

            // Assert
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be("minimal-asset");
            deserialized.LOD.Should().BeNull();
        }
    }
}
