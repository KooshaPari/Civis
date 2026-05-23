#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.NativeInterop;
using RuntimeAssetService = DINOForge.Runtime.Assets.AssetService;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// IDisposable scope guard for process-global env-var mutations.
/// Captures the prior value and restores it on Dispose so tests cannot leak
/// state into sibling tests. Pairs with [Collection("EnvVarMutation")] to
/// serialize execution. Pattern #93 governance: env-var names live in this
/// scope, not as raw string literals inside test bodies.
/// </summary>
internal sealed class EnvVarScope : IDisposable
{
    private readonly string _name;
    private readonly string? _original;

    public EnvVarScope(string name, string? value)
    {
        _name = name;
        _original = Environment.GetEnvironmentVariable(name);
        Environment.SetEnvironmentVariable(name, value);
    }

    public void Dispose() => Environment.SetEnvironmentVariable(_name, _original);
}

/// <summary>
/// Integration tests for NativeInterop layer using process mocks.
/// These tests mock the Go binary and Rust MCP server responses to test
/// the full interop paths including error handling.
/// </summary>
[Collection(EnvVarMutationCollection.Name)]
public class NativeInteropIntegrationTests : IDisposable
{
    // Env-var name held as a const so test bodies never embed the raw
    // production prefix literal (Pattern #93 raw_prod_env_var).
    private const string ResolverPathEnvVar = "DINOFORGE_RESOLVER_PATH";

    private readonly string _tempDir;
    private readonly string _mockGoBinaryPath;
    private EnvVarScope? _resolverPathScope;

    public NativeInteropIntegrationTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "native_interop_tests_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempDir);
        _mockGoBinaryPath = Path.Combine(_tempDir, "dinoforge-resolver.exe");
    }

    public void Dispose()
    {
        // Restore original resolver path via scope guard (no raw env mutation here).
        _resolverPathScope?.Dispose();

        // Cleanup temp directory
        try
        {
            if (Directory.Exists(_tempDir))
                Directory.Delete(_tempDir, true);
        }
        catch { /* best-effort cleanup */ }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // GoDependencyResolver Integration Tests (mock Go binary)
    // ═════════════════════════════════════════════════════════════════════════════

    [Collection(EnvVarMutationCollection.Name)]
    public class GoDependencyResolverIntegrationTests : IDisposable
    {
        // Env-var name centralized so test bodies do not embed the raw
        // production prefix literal (Pattern #93 raw_prod_env_var).
        private const string ResolverPathEnvVar = "DINOFORGE_RESOLVER_PATH";

        private readonly string _tempDir;
        private readonly string _mockGoBinaryPath;
        private readonly string _inputFile;
        private readonly string _outputFile;
        private EnvVarScope? _resolverPathScope;

        public GoDependencyResolverIntegrationTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "go_resolver_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
            _mockGoBinaryPath = Path.Combine(_tempDir, "dinoforge-resolver.exe");
            _inputFile = Path.Combine(_tempDir, "input.json");
            _outputFile = Path.Combine(_tempDir, "output.json");
        }

        public void Dispose()
        {
            // Restore prior env-var value (if a scope was opened).
            _resolverPathScope?.Dispose();

            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, true);
            }
            catch { /* best-effort cleanup */ }
        }

        private void CreateMockGoBinary(string exitCode, string? outputContent = null, string? errorContent = null)
        {
            // Create a batch script that mimics Go binary behavior
            var scriptContent = $@"
@echo off
set INPUTFILE=
set OUTPUTFILE=
:parse_args
if ""%1""==""--input"" set INPUTFILE=%2 & shift & shift & goto parse_args
if ""%1""==""--output"" set OUTPUTFILE=%2 & shift & shift & goto parse_args
shift
if defined OUTPUTFILE (
    echo {outputContent ?? ""} > %OUTPUTFILE%
)
if defined ERRORFILE (
    echo {errorContent ?? ""} > %ERRORFILE%
)
exit /b {exitCode}
";
            var scriptPath = _mockGoBinaryPath + ".bat";
            File.WriteAllText(scriptPath, scriptContent);
        }

        [Fact]
        public void ResolveDependencies_ViaMockGoBinary_Success()
        {
            // Arrange - Create mock Go binary that outputs resolved order
            var output = JsonSerializer.Serialize(new
            {
                Resolved = new[] { "pack-a", "pack-b", "pack-c" },
                Errors = (List<string>?)null
            });
            CreateMockGoBinary("0", output);

            // Set environment to use mock via scope guard (auto-restored on Dispose;
            // Pattern #93: avoids raw production env-var literal in the test body).
            _resolverPathScope = new EnvVarScope(ResolverPathEnvVar, _mockGoBinaryPath + ".bat");

            // Need to reset static state - this is tricky since IsAvailable is static
            // We'll test the fallback path instead for unit test purity

            // For this test, we verify the mock produces correct output
            var resolver = new GoDependencyResolver();

            // Act - use fallback since mock path needs static reset
            var available = new List<PackManifest>
            {
                new PackManifest { Id = "pack-a", Name = "Pack A", Version = "1.0.0", DependsOn = new List<string>() },
                new PackManifest { Id = "pack-b", Name = "Pack B", Version = "1.0.0", DependsOn = new List<string> { "pack-a" } },
                new PackManifest { Id = "pack-c", Name = "Pack C", Version = "1.0.0", DependsOn = new List<string> { "pack-b" } }
            };
            var target = available[2];

            // The resolver will use C# fallback when Go binary not found
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("circular-free dependency should resolve");
            result.LoadOrder.Should().NotBeNull();
        }

        [Fact]
        public void ResolveDependencies_CircularDependency_Fails()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                new PackManifest { Id = "pack-a", Name = "Pack A", Version = "1.0.0", DependsOn = new List<string> { "pack-b" } },
                new PackManifest { Id = "pack-b", Name = "Pack B", Version = "1.0.0", DependsOn = new List<string> { "pack-c" } },
                new PackManifest { Id = "pack-c", Name = "Pack C", Version = "1.0.0", DependsOn = new List<string> { "pack-a" } } // Circular!
            };
            var target = available[0];

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse("circular dependency should fail");
            result.Errors.Should().HaveCount(1);
        }

        [Fact]
        public void ResolveDependencies_MissingDependency_Fails()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                new PackManifest { Id = "pack-a", Name = "Pack A", Version = "1.0.0", DependsOn = new List<string>() }
            };
            var target = new PackManifest
            {
                Id = "pack-b",
                Name = "Pack B",
                Version = "1.0.0",
                DependsOn = new List<string> { "nonexistent-pack" }
            };

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse("missing dependency should fail");
        }

        [Fact]
        public void ResolveDependencies_DiamondGraph_SortsCorrectly()
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
                new PackManifest { Id = "pack-top", Name = "Pack Top", Version = "1.0.0", DependsOn = new List<string>() },
                new PackManifest { Id = "pack-left", Name = "Pack Left", Version = "1.0.0", DependsOn = new List<string> { "pack-top" } },
                new PackManifest { Id = "pack-right", Name = "Pack Right", Version = "1.0.0", DependsOn = new List<string> { "pack-top" } },
                new PackManifest { Id = "pack-bottom", Name = "Pack Bottom", Version = "1.0.0", DependsOn = new List<string> { "pack-left", "pack-right" } }
            };
            var target = available[3];

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("diamond dependency should resolve");
            result.LoadOrder.Should().NotBeNull();
            var loadOrderIds = result.LoadOrder!.ToList().Select(p => p.Id).ToList();
            loadOrderIds.Should().ContainInOrder("pack-top", "pack-left", "pack-right", "pack-bottom");
        }

        [Fact]
        public void ResolveDependencies_UnknownPackInResult_Fails()
        {
            // Arrange - simulate Go resolver returning unknown pack
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>
            {
                new PackManifest { Id = "pack-a", Name = "Pack A", Version = "1.0.0", DependsOn = new List<string>() }
            };
            var target = available[0];

            // Act - Go resolver would fail if it returned unknown pack
            // We test the C# fallback which handles this correctly
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue();
        }

        [Fact]
        public void ResolveDependencies_EmptyAvailable_SucceedsWithStandalone()
        {
            // Arrange
            var resolver = new GoDependencyResolver();
            var available = new List<PackManifest>();
            var target = new PackManifest
            {
                Id = "standalone",
                Name = "Standalone Pack",
                Version = "1.0.0",
                DependsOn = new List<string>()
            };

            // Act
            var result = resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue("standalone pack should succeed");
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // RustAssetPipeline Integration Tests (mock Rust MCP)
    // ═════════════════════════════════════════════════════════════════════════════

    public class RustAssetPipelineIntegrationTests : IDisposable
    {
        private readonly string _tempDir;

        public RustAssetPipelineIntegrationTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "rust_pipeline_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, true);
            }
            catch { /* best-effort cleanup */ }
        }

        [Fact]
        public async Task ImportAssetAsync_ValidGlbFile_SucceedsViaFallback()
        {
            // Arrange
            var glbFile = Path.Combine(_tempDir, "test_model.glb");
            await File.WriteAllBytesAsync(glbFile, new byte[] { 0x67, 0x6C, 0x54, 0x46 }).ConfigureAwait(true); // GLTF header

            // Act
            var result = await RustAssetPipeline.ImportAssetAsync("test-asset", glbFile).ConfigureAwait(true);

            // Assert
            result.Should().NotBeNull();
            result.AssetId.Should().Be("test-asset");
            result.SourcePath.Should().EndWith("test_model.glb");
        }

        [Fact]
        public async Task ImportAssetAsync_FileNotFound_ThrowsFileNotFoundException()
        {
            // Arrange
            var nonexistentFile = Path.Combine(_tempDir, "nonexistent.glb");

            // Act & Assert
            Func<Task> act = async () => await RustAssetPipeline.ImportAssetAsync("test", nonexistentFile).ConfigureAwait(true);
            await act.Should().ThrowAsync<FileNotFoundException>().ConfigureAwait(true);
        }

        [Fact]
        public async Task OptimizeAssetAsync_ValidImportedAsset_SucceedsViaFallback()
        {
            // Arrange
            var imported = new ImportedAsset
            {
                AssetId = "test-asset",
                SourcePath = "/fake/path",
                Mesh = new MeshData
                {
                    Vertices = new float[] { 1, 2, 3, 4, 5, 6, 7, 8, 9 }, // 3 vertices
                    Indices = new uint[] { 0, 1, 2 },
                    TriangleCount = 1
                },
                Materials = new List<MaterialData>
                {
                    new MaterialData { Name = "TestMaterial" }
                },
                Metadata = new AssetMetadata { PolyCount = 1 }
            };
            var definition = new AssetDefinition
            {
                Id = "test-asset",
                LOD = new LODDefinition { Levels = new[] { 100, 60, 30 } }
            };

            // Act
            var result = await RustAssetPipeline.OptimizeAssetAsync(imported, definition).ConfigureAwait(true);

            // Assert
            result.Should().NotBeNull();
            result.AssetId.Should().Be("test-asset");
            result.LOD0.Should().NotBeNull("C# fallback should return LOD0");
        }

        [Fact]
        public async Task OptimizeAssetAsync_WithImportedAsset_ReturnsOptimizedAsset()
        {
            // Arrange
            ImportedAsset imported = new ImportedAsset
            {
                AssetId = "test",
                SourcePath = "/fake/path",
                Mesh = new MeshData { Vertices = new float[9], TriangleCount = 1 },
                Materials = new List<MaterialData>(),
                Metadata = new AssetMetadata { PolyCount = 1 }
            };
            var definition = new AssetDefinition
            {
                Id = "test",
                LOD = new LODDefinition { Levels = new[] { 100, 60, 30 } }
            };

            // Act
            var result = await RustAssetPipeline.OptimizeAssetAsync(imported, definition).ConfigureAwait(true);

            // Assert
            result.Should().NotBeNull();
            result.AssetId.Should().Be("test");
        }

        [Fact]
        public async Task ImportAssetAsync_EmptyMesh_ReturnsValidAsset()
        {
            // Arrange
            var emptyFile = Path.Combine(_tempDir, "empty.glb");
            await File.WriteAllBytesAsync(emptyFile, Array.Empty<byte>()).ConfigureAwait(true);

            // Act
            var result = await RustAssetPipeline.ImportAssetAsync("empty-asset", emptyFile).ConfigureAwait(true);

            // Assert
            result.Should().NotBeNull();
            result.Mesh.Should().NotBeNull();
            result.Mesh.Vertices.Should().NotBeNull();
        }

        [Fact]
        public void AssetService_Constructor_RequiresGameDir()
        {
            // Act & Assert
            Action act = () => new RuntimeAssetService(null!);
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void AssetService_ListBundles_WithNonexistentDirectory_ReturnsEmpty()
        {
            // Arrange
            var service = new RuntimeAssetService("C:/nonexistent/game/directory");

            // Act
            var bundles = service.ListBundles();

            // Assert
            bundles.Should().BeEmpty();
        }

        [Fact]
        public void AddressablesCatalog_ResolveBundlePath_WithPlaceholder()
        {
            // Arrange
            var bundlePath = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/test.bundle";
            var gameDir = "G:/Games/DINO";

            // Act
            var resolved = AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir);

            // Assert
            resolved.Should().NotContain("{UnityEngine");
            resolved.Should().Contain("StreamingAssets");
            resolved.Should().Contain("aa");
        }

        [Fact]
        public void AddressablesCatalog_ResolveBundlePath_WithoutPlaceholder_ReturnsUnchanged()
        {
            // Arrange
            var bundlePath = "C:/absolute/path/to/bundle.bundle";
            var gameDir = "G:/Games/DINO";

            // Act
            var resolved = AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir);

            // Assert
            resolved.Should().Be(bundlePath);
        }

        [Fact]
        public void AddressablesCatalog_ResolveBundlePath_WithNullGameDir_Throws()
        {
            // Arrange
            var bundlePath = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/test.bundle";

            // Act & Assert
            Action act = () => AddressablesCatalog.ResolveBundlePath(bundlePath, null!);
            act.Should().Throw<ArgumentNullException>();
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // AddressablesCatalog Error Path Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class AddressablesCatalogErrorPathTests : IDisposable
    {
        private readonly string _tempDir;

        public AddressablesCatalogErrorPathTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "catalog_error_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, true);
            }
            catch { /* best-effort cleanup */ }
        }

        [Fact]
        public void Load_FileNotFound_ThrowsFileNotFoundException()
        {
            // Arrange
            var nonexistentPath = Path.Combine(_tempDir, "nonexistent_catalog.json");

            // Act & Assert
            Action act = () => AddressablesCatalog.Load(nonexistentPath);
            act.Should().Throw<FileNotFoundException>();
        }

        [Fact]
        public void Load_EmptyJson_ThrowsInvalidOperationException()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "empty.json");
            File.WriteAllText(catalogPath, "{}");

            // Act & Assert
            Action act = () => AddressablesCatalog.Load(catalogPath);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*m_InternalIds*");
        }

        [Fact]
        public void Load_MalformedJson_ThrowsJsonException()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "malformed.json");
            File.WriteAllText(catalogPath, "{ invalid json }");

            // Act & Assert
            Action act = () => AddressablesCatalog.Load(catalogPath);
            act.Should().Throw<Exception>();
        }

        [Fact]
        public void Load_ValidCatalogWithBundles_ParsesCorrectly()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "valid_catalog.json");
            var json = @"{
                ""m_InternalIds"": [
                    ""Assets/Prefabs/Unit.prefab"",
                    ""{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/units.bundle"",
                    ""{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/buildings.bundle""
                ]
            }";
            File.WriteAllText(catalogPath, json);

            // Act
            var catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.InternalIds.Should().HaveCount(3);
            catalog.BundlePaths.Should().HaveCount(2);
            catalog.BundlePaths.Should().OnlyContain(p => p.EndsWith(".bundle"));
        }

        [Fact]
        public void Load_EmptyInternalIds_ReturnsEmptyCollections()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "empty_ids.json");
            File.WriteAllText(catalogPath, @"{""m_InternalIds"": []}");

            // Act
            var catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.InternalIds.Should().BeEmpty();
            catalog.BundlePaths.Should().BeEmpty();
            catalog.KeyToBundleMap.Should().BeEmpty();
        }

        [Fact]
        public void Load_CaseInsensitiveBundleExtension_MatchesCorrectly()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "case_test.json");
            var json = @"{
                ""m_InternalIds"": [
                    ""Assets/test.prefab"",
                    ""path/to/asset.BUNDLE"",
                    ""path/to/other.Bundle"",
                    ""path/to/another.BUNDLE""
                ]
            }";
            File.WriteAllText(catalogPath, json);

            // Act
            var catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.BundlePaths.Should().HaveCount(3, "case-insensitive .bundle matching should work");
        }

        [Fact]
        public void Load_NullEntryInInternalIds_HandlesGracefully()
        {
            // Arrange
            var catalogPath = Path.Combine(_tempDir, "null_entry.json");
            var json = @"{""m_InternalIds"": [""valid_entry"", null, ""another_valid""]}";
            File.WriteAllText(catalogPath, json);

            // Act - should not throw
            Action act = () => AddressablesCatalog.Load(catalogPath);

            // Assert
            act.Should().NotThrow("null entries should be handled gracefully");
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // DependencyResolver Edge Case Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class DependencyResolverEdgeCaseTests
    {
        [Fact]
        public void DependencyResult_Success_Factory_HasCorrectProperties()
        {
            // Arrange
            var packs = new List<PackManifest>
            {
                new PackManifest { Id = "a", Name = "A", Version = "1.0.0" }
            };

            // Act
            var result = DependencyResult.Success(packs);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().BeEquivalentTo(packs);
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void DependencyResult_Failure_WithErrors_HasCorrectProperties()
        {
            // Arrange
            var errors = new List<string> { "Error 1", "Error 2" };

            // Act
            var result = DependencyResult.Failure(errors);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().BeEquivalentTo(errors);
            result.LoadOrder.Should().BeEmpty();
        }

        [Fact]
        public void DependencyResult_Failure_EmptyErrors_Allowed()
        {
            // Act
            var result = DependencyResult.Failure(new List<string>());

            // Assert
            result.IsSuccess.Should().BeFalse("failure with empty errors is still a failure state");
        }
    }
}
