#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using Xunit;
using FluentAssertions;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;

namespace DINOForge.Tools.PackCompiler.Tests
{
    public class AssetValidationServiceTests
    {
        private readonly AssetValidationService _validationService = new();

        [Fact]
        public void ValidateConfiguration_WithEmptyPhases_ReturnsError()
        {
            // Arrange
            var config = new AssetPipelineConfig
            {
                Version = "0.7.0",
                PackId = "test-pack",
                TargetUnityVersion = "2021.3.45f2",
                AssetSettings = new AssetSettings { BasePath = "assets", OutputPath = "output" },
                Materials = new Dictionary<string, MaterialDefinition>(),
                Phases = new Dictionary<string, AssetPhase>(),
                Build = new BuildConfig { OutputDirectory = "out", AddressablesOutput = "addr" }
            };

            // Act
            var result = _validationService.ValidateConfiguration(config);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("No asset phases defined"));
        }

        [Fact]
        public void ValidateConfiguration_WithValidConfig_ReturnsSuccess()
        {
            // Arrange
            var config = new AssetPipelineConfig
            {
                Version = "0.7.0",
                PackId = "test-pack",
                TargetUnityVersion = "2021.3.45f2",
                AssetSettings = new AssetSettings { BasePath = "assets", OutputPath = "output" },
                Materials = new Dictionary<string, MaterialDefinition>
                {
                    { "default", new MaterialDefinition { Faction = "test", BaseColor = "#FFFFFF", EmissionColor = "#000000", EmissionIntensity = 1.0f } }
                },
                Phases = new Dictionary<string, AssetPhase>
                {
                    {
                        "v0.7.0", new AssetPhase
                        {
                            Description = "Initial assets",
                            Models = new List<AssetDefinition>
                            {
                                new AssetDefinition
                                {
                                    Id = "asset-001",
                                    File = "model.glb",
                                    Type = "infantry",
                                    Faction = "test",
                                    PolyCountTarget = 5000,
                                    Material = "default",
                                    AddressableKey = "asset/001",
                                    OutputPrefab = "Prefabs/asset001",
                                    LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
                                }
                            }
                        }
                    }
                },
                Build = new BuildConfig { OutputDirectory = "out", AddressablesOutput = "addr" }
            };

            // Act
            var result = _validationService.ValidateConfiguration(config);

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidateImportedAsset_WithEmptyMesh_ReturnsError()
        {
            // Arrange
            var asset = new ImportedAsset
            {
                AssetId = "test-asset",
                SourcePath = "/test/model.glb",
                Mesh = new MeshData
                {
                    Name = "test-mesh",
                    Vertices = Array.Empty<float>(),
                    Indices = Array.Empty<uint>()
                },
                Materials = new(),
                Metadata = new()
            };

            var definition = new AssetDefinition
            {
                Id = "test-asset",
                File = "model.glb",
                Type = "infantry",
                Faction = "test",
                PolyCountTarget = 5000,
                Material = "default",
                AddressableKey = "asset/test",
                OutputPrefab = "Prefabs/test",
                LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
            };

            // Act
            var result = _validationService.ValidateImportedAsset(asset, definition);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("empty"));
        }

        [Fact]
        public void ValidateImportedAsset_WithRealisticMesh_ReturnsSuccess()
        {
            // Arrange
            var vertices = new float[3000];  // 1000 vertices at ~1000 polycount
            for (int i = 0; i < vertices.Length; i++) vertices[i] = 0f;

            var indices = new uint[3000];  // 1000 triangles
            for (int i = 0; i < indices.Length; i++) indices[i] = (uint)(i % 1000);

            var asset = new ImportedAsset
            {
                AssetId = "test-asset",
                SourcePath = "/test/model.glb",
                Mesh = new MeshData
                {
                    Name = "test-mesh",
                    Vertices = vertices,
                    Indices = indices,
                    Normals = vertices,  // Same size as vertices
                    Bounds = (new[] { 0f, 0f, 0f }, new[] { 10f, 10f, 10f })
                },
                Materials = new(),
                Metadata = new()
            };

            var definition = new AssetDefinition
            {
                Id = "test-asset",
                File = "model.glb",
                Type = "infantry",
                Faction = "test",
                PolyCountTarget = 1000,  // Matches actual polycount
                Material = "default",
                AddressableKey = "asset/test",
                OutputPrefab = "Prefabs/test",
                LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
            };

            // Act
            var result = _validationService.ValidateImportedAsset(asset, definition);

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }
    }

    public class AssetOptimizationServiceTests
    {
        private readonly AssetOptimizationService _optimizationService = new();

        [Fact]
        public async Task OptimizeAsync_WithValidAsset_GeneratesLODVariants()
        {
            // Arrange
            var vertices = new float[3000];  // 1000 vertices
            for (int i = 0; i < vertices.Length; i++) vertices[i] = 0f;

            var indices = new uint[3000];  // 1000 triangles
            for (int i = 0; i < indices.Length; i++) indices[i] = (uint)(i % 1000);

            var asset = new ImportedAsset
            {
                AssetId = "test-asset",
                SourcePath = "/test/model.glb",
                Mesh = new MeshData
                {
                    Name = "test-mesh",
                    Vertices = vertices,
                    Indices = indices,
                    Bounds = (new[] { 0f, 0f, 0f }, new[] { 10f, 10f, 10f })
                },
                Materials = new(),
                Metadata = new()
            };

            var definition = new AssetDefinition
            {
                Id = "test-asset",
                File = "model.glb",
                Type = "infantry",
                Faction = "test",
                PolyCountTarget = 1000,
                Material = "default",
                AddressableKey = "asset/test",
                OutputPrefab = "Prefabs/test",
                LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
            };

            // Act
            var result = await _optimizationService.OptimizeAsync(asset, definition);

            // Assert
            result.Should().NotBeNull();
            result.AssetId.Should().Be("test-asset");
            result.LOD0.Should().NotBeNull();
            result.LOD1.Should().NotBeNull();
            result.LOD2.Should().NotBeNull();
            result.LOD0.TriangleCount.Should().Be(1000);
            result.LOD1.TriangleCount.Should().BeLessThan(result.LOD0.TriangleCount);
            result.LOD2.TriangleCount.Should().BeLessThan(result.LOD1.TriangleCount);
            result.Metadata.OptimizationMethod.Should().Contain("Decimation");
        }
    }

    public class PrefabGenerationServiceTests
    {
        private readonly PrefabGenerationService _prefabService = new();

        [Fact]
        public async Task GeneratePrefabAsync_CreatesYamlFile()
        {
            // Arrange
            var tempDir = Path.Combine(Path.GetTempPath(), "dinoforge-test-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);

            var asset = new OptimizedAsset
            {
                AssetId = "test-asset",
                LOD0 = new MeshData { Name = "LOD0", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                LOD1 = new MeshData { Name = "LOD1", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                LOD2 = new MeshData { Name = "LOD2", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                Materials = new(),
                Metadata = new(),
                OptimizedAt = DateTime.UtcNow
            };

            var definition = new AssetDefinition
            {
                Id = "test-asset",
                File = "model.glb",
                Type = "infantry",
                Faction = "test",
                PolyCountTarget = 5000,
                Scale = 1.0f,
                Material = "default",
                AddressableKey = "asset/test",
                OutputPrefab = "Prefabs/test",
                LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
            };

            string outputPath = Path.Combine(tempDir, "test-asset.prefab");

            try
            {
                // Act
                await _prefabService.GeneratePrefabAsync(asset, definition, outputPath);

                // Assert
                File.Exists(outputPath).Should().BeTrue();
                var content = File.ReadAllText(outputPath, Encoding.UTF8);
                content.Should().Contain("%YAML");
                content.Should().Contain("test-asset");
            }
            finally
            {
                // Cleanup
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }
    }

    public class AddressablesServiceTests
    {
        private readonly AddressablesService _addressablesService = new();

        [Fact]
        public async Task GenerateCatalogAsync_CreatesValidCatalog()
        {
            // Arrange
            var tempDir = Path.Combine(Path.GetTempPath(), "dinoforge-addr-" + Guid.NewGuid());
            Directory.CreateDirectory(tempDir);

            var asset = new OptimizedAsset
            {
                AssetId = "test-asset",
                LOD0 = new MeshData { Name = "LOD0", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                LOD1 = new MeshData { Name = "LOD1", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                LOD2 = new MeshData { Name = "LOD2", Vertices = new[] { 0f, 0f, 0f }, Indices = new uint[] { 0 } },
                Materials = new(),
                Metadata = new(),
                OptimizedAt = DateTime.UtcNow
            };

            var definition = new AssetDefinition
            {
                Id = "test-asset",
                File = "model.glb",
                Type = "infantry",
                Faction = "test",
                PolyCountTarget = 5000,
                Scale = 1.0f,
                Material = "default",
                AddressableKey = "asset/test",
                OutputPrefab = "Prefabs/test",
                LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } }
            };

            var assets = new List<(OptimizedAsset, AssetDefinition)> { (asset, definition) };
            string catalogPath = Path.Combine(tempDir, "catalog.txt");

            try
            {
                // Act
                await _addressablesService.GenerateCatalogAsync(assets, catalogPath);

                // Assert
                File.Exists(catalogPath).Should().BeTrue();
                var content = File.ReadAllText(catalogPath, Encoding.UTF8);
                content.Should().Contain("asset/test");
                content.Should().Contain("infantry");
            }
            finally
            {
                if (Directory.Exists(tempDir))
                    Directory.Delete(tempDir, true);
            }
        }
    }
}
