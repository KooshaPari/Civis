#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.Tools.PackCompiler.Services;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;
using PackValidationResult = DINOForge.Tools.PackCompiler.Services.ValidationResult;
using SdkValidationResult = DINOForge.SDK.Validation.ValidationResult;

namespace DINOForge.Tests;

/// <summary>
/// Coverage gap tests for SDK schema resolution, RegistryImportService validation paths,
/// and PackCompiler asset/definition services (source-linked from net11.0 PackCompiler).
/// </summary>
public class SdkPackCompilerValidationGapTests : IDisposable
{
    private readonly string _tempDir;

    public SdkPackCompilerValidationGapTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_gap_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempDir))
        {
            try
            {
                Directory.Delete(_tempDir, true);
            }
            catch
            {
                // Best-effort cleanup.
            }
        }
    }

    // ── SchemaResolverService ────────────────────────────────────────────────

    /// <summary>Unknown content types must not resolve to a schema (pack loader skips validation).</summary>
    [Theory]
    [InlineData("unknown_type")]
    [InlineData("")]
    [InlineData("scenarios")]
    public void SchemaResolverService_TryResolveSchemaName_UnknownType_ReturnsFalse(string contentType)
    {
        var resolver = new SchemaResolverService();

        bool resolved = resolver.TryResolveSchemaName(contentType, out string schemaName);

        resolved.Should().BeFalse();
        schemaName.Should().BeEmpty();
    }

    /// <summary>Content type keys are case-insensitive per SchemaNames dictionary.</summary>
    [Theory]
    [InlineData("UNITS", "unit")]
    [InlineData("Buildings", "building")]
    [InlineData("FACTION_PATCHES", "faction-patch")]
    public void SchemaResolverService_TryResolveSchemaName_CaseInsensitive_ResolvesSchema(
        string contentType,
        string expectedSchema)
    {
        var resolver = new SchemaResolverService();

        resolver.TryResolveSchemaName(contentType, out string schemaName).Should().BeTrue();
        schemaName.Should().Be(expectedSchema);
    }

    // ── RegistryImportService ────────────────────────────────────────────────

    /// <summary>Missing YAML files surface as read errors instead of crashing the loader.</summary>
    [Fact]
    public void RegistryImportService_LoadAndRegisterContent_MissingFile_AddsReadError()
    {
        var registries = new RegistryManager();
        var errors = new List<string>();
        var service = CreateRegistryImportService(registries, errors: errors);
        var manifest = new PackManifest { Id = "test-pack", LoadOrder = 100 };
        string missingPath = Path.Combine(_tempDir, "does-not-exist.yaml");

        service.LoadAndRegisterContent(missingPath, "units", manifest, errors);

        errors.Should().ContainSingle(e => e.Contains("Failed to read") && e.Contains(missingPath));
        registries.Units.Contains("any").Should().BeFalse();
    }

    /// <summary>Schema validation failures must block registration and accumulate per-field errors.</summary>
    [Fact]
    public void RegistryImportService_LoadAndRegisterContent_SchemaValidationFailure_AddsValidationErrors()
    {
        var registries = new RegistryManager();
        var errors = new List<string>();
        var schemaSources = new Dictionary<string, string>
        {
            ["unit"] = @"
type: object
properties:
  id:
    type: string
required:
  - id
"
        };
        var validator = new NJsonSchemaValidator(schemaSources);
        var service = CreateRegistryImportService(registries, validator, errors);
        var manifest = new PackManifest { Id = "schema-pack", LoadOrder = 100 };
        string yamlPath = Path.Combine(_tempDir, "unit.yaml");
        File.WriteAllText(yamlPath, @"
- display_name: Trooper
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1
");

        service.LoadAndRegisterContent(yamlPath, "units", manifest, errors);

        errors.Should().NotBeEmpty();
        errors.Should().Contain(e => e.Contains("Validation error"));
        registries.Units.Contains("trooper").Should().BeFalse();
    }

    /// <summary>Validator infrastructure failures must not be treated as valid content (#764).</summary>
    [Fact]
    public void RegistryImportService_LoadAndRegisterContent_SchemaValidatorThrows_AddsSchemaFailureError()
    {
        var registries = new RegistryManager();
        var errors = new List<string>();
        var logs = new List<string>();
        var service = CreateRegistryImportService(
            registries,
            new ThrowingSchemaValidator(),
            errors,
            logs);
        var manifest = new PackManifest { Id = "throw-pack", LoadOrder = 100 };
        string yamlPath = Path.Combine(_tempDir, "unit.yaml");
        File.WriteAllText(yamlPath, "- id: trooper\ndisplay_name: Trooper\n");

        service.LoadAndRegisterContent(yamlPath, "units", manifest, errors);

        errors.Should().ContainSingle(e => e.Contains("Schema validation failed"));
        logs.Should().Contain(l => l.Contains("WARNING"));
        registries.Units.Contains("trooper").Should().BeFalse();
    }

    /// <summary>Valid unit YAML flows through schema + JsonGuard and registers in the unit registry.</summary>
    [Fact]
    public void RegistryImportService_LoadAndRegisterContent_ValidUnit_RegistersInRegistry()
    {
        var registries = new RegistryManager();
        var errors = new List<string>();
        var service = CreateRegistryImportService(registries, errors: errors);
        var manifest = new PackManifest { Id = "good-pack", LoadOrder = 200 };
        string yamlPath = Path.Combine(_tempDir, "trooper.yaml");
        File.WriteAllText(yamlPath, @"
- id: republic-trooper
  display_name: Republic Trooper
  unit_class: CoreLineInfantry
  faction_id: republic
  tier: 1
  stats:
    hp: 100
    damage: 10
");

        service.LoadAndRegisterContent(yamlPath, "units", manifest, errors);

        errors.Should().BeEmpty();
        registries.Units.Contains("republic-trooper").Should().BeTrue();
        registries.Units.Get("republic-trooper")!.DisplayName.Should().Be("Republic Trooper");
    }

    // ── AssetValidationService ─────────────────────────────────────────────────

    /// <summary>Imported assets without mesh data fail fast before polycount checks.</summary>
    [Fact]
    public void AssetValidationService_ValidateImportedAsset_NullMesh_ReturnsInvalid()
    {
        var service = new AssetValidationService();
        var asset = new ImportedAsset
        {
            AssetId = "test-asset",
            SourcePath = "model.glb",
            Mesh = null!
        };
        var definition = CreateAssetDefinition("test-asset", polyTarget: 5000);

        PackValidationResult result = service.ValidateImportedAsset(asset, definition);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("no mesh"));
    }

    /// <summary>Polycount below the pipeline minimum is rejected for bundle quality gates.</summary>
    [Fact]
    public void AssetValidationService_ValidateImportedAsset_PolycountTooLow_ReturnsError()
    {
        var service = new AssetValidationService();
        var asset = new ImportedAsset
        {
            AssetId = "low-poly",
            SourcePath = "model.glb",
            Mesh = CreateMesh(triangleCount: 10)
        };
        var definition = CreateAssetDefinition("low-poly", polyTarget: 5000);

        PackValidationResult result = service.ValidateImportedAsset(asset, definition);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("Polycount too low"));
    }

    /// <summary>LOD chains must decrease in complexity from LOD0 to LOD2.</summary>
    [Fact]
    public void AssetValidationService_ValidateOptimizedAsset_InvalidLodProgression_ReturnsError()
    {
        var service = new AssetValidationService();
        var asset = new OptimizedAsset
        {
            AssetId = "bad-lods",
            LOD0 = CreateMesh(200),
            LOD1 = CreateMesh(250),
            LOD2 = CreateMesh(100),
            Metadata = new OptimizationMetadata(),
            ScreenSizes = new LODScreenSize { LOD0Min = 50, LOD0Max = 100, LOD1Min = 50, LOD1Max = 20, LOD2Min = 20 }
        };
        var definition = CreateAssetDefinition("bad-lods", polyTarget: 5000);

        PackValidationResult result = service.ValidateOptimizedAsset(asset, definition);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("LOD1") && e.Contains("smaller than LOD0"));
    }

    // ── DefinitionUpdateService ──────────────────────────────────────────────

    /// <summary>Pre-flight validation catches missing definition files before pipeline writes.</summary>
    [Fact]
    public void DefinitionUpdateService_ValidateDefinitionUpdates_MissingDefinitionFile_ReturnsError()
    {
        var service = new DefinitionUpdateService();
        var asset = new OptimizedAsset
        {
            AssetId = "trooper",
            LOD0 = CreateMesh(100),
            LOD1 = CreateMesh(60),
            LOD2 = CreateMesh(30)
        };
        var config = CreateAssetDefinition(
            "trooper",
            polyTarget: 5000,
            updateFile: "units/missing.yaml",
            updateId: "trooper",
            updateField: "visual_asset",
            outputPrefab: "prefabs/trooper");

        (bool isValid, List<string> errors, _) = service.ValidateDefinitionUpdates(
            new List<(OptimizedAsset, AssetDefinition)> { (asset, config) },
            _tempDir);

        isValid.Should().BeFalse();
        errors.Should().Contain(e => e.Contains("Definition file not found"));
    }

    /// <summary>Definition updater rewrites an existing visual_asset field in pack YAML.</summary>
    [Fact]
    public async Task DefinitionUpdateService_UpdateDefinitionsAsync_UpdatesExistingVisualAssetField()
    {
        var service = new DefinitionUpdateService();
        string unitsDir = Path.Combine(_tempDir, "units");
        Directory.CreateDirectory(unitsDir);
        string defFile = Path.Combine(unitsDir, "trooper.yaml");
        File.WriteAllText(defFile, @"
units:
  - id: trooper
    display_name: Trooper
    visual_asset: old/path
");

        var asset = new OptimizedAsset
        {
            AssetId = "trooper",
            LOD0 = CreateMesh(100),
            LOD1 = CreateMesh(60),
            LOD2 = CreateMesh(30)
        };
        var config = CreateAssetDefinition(
            "trooper",
            polyTarget: 5000,
            updateFile: "units/trooper.yaml",
            updateId: "trooper",
            updateField: "visual_asset",
            outputPrefab: "prefabs/trooper");

        await service.UpdateDefinitionsAsync(
            new List<(OptimizedAsset, AssetDefinition)> { (asset, config) },
            _tempDir);

        string updated = File.ReadAllText(defFile);
        updated.Should().Contain("visual_asset: prefabs/trooper");
        updated.Should().NotContain("old/path");
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    private static RegistryImportService CreateRegistryImportService(
        RegistryManager registries,
        ISchemaValidator? schemaValidator = null,
        List<string>? errors = null,
        List<string>? logs = null)
    {
        var overrides = new List<StatOverrideDefinition>();
        List<string> logSink = logs ?? new List<string>();
        return new RegistryImportService(
            registries,
            schemaValidator,
            new SchemaResolverService(),
            new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build(),
            overrides,
            msg => logSink.Add(msg));
    }

    private static AssetDefinition CreateAssetDefinition(
        string id,
        int polyTarget,
        string? updateFile = null,
        string? updateId = null,
        string? updateField = null,
        string? outputPrefab = null)
    {
        DefinitionUpdateConfig? updateConfig = updateFile != null
            ? new DefinitionUpdateConfig
            {
                Enabled = true,
                File = updateFile,
                Id = updateId,
                Field = updateField
            }
            : null;

        return new AssetDefinition
        {
            Id = id,
            File = "models/test.glb",
            Type = "infantry",
            Faction = "republic",
            PolyCountTarget = polyTarget,
            Scale = 1.0f,
            LOD = new LODDefinition { Levels = new List<int> { 100, 60, 30 } },
            Material = "republic",
            AddressableKey = $"assets/{id}",
            OutputPrefab = outputPrefab ?? $"prefabs/{id}",
            UpdateDefinition = updateConfig
        };
    }

    private static MeshData CreateMesh(int triangleCount)
    {
        int indexCount = triangleCount * 3;
        var indices = new uint[indexCount];
        for (uint i = 0; i < indexCount; i++)
        {
            indices[i] = i % (uint)Math.Max(1, triangleCount);
        }

        int vertexCount = triangleCount * 3;
        var vertices = new float[vertexCount * 3];
        return new MeshData
        {
            Name = "test-mesh",
            Vertices = vertices,
            Indices = indices,
            Normals = new float[vertexCount * 3],
            UVs = new float[vertexCount * 2],
            Bounds = (new float[] { 0, 0, 0 }, new float[] { 1, 1, 1 })
        };
    }

    private sealed class ThrowingSchemaValidator : ISchemaValidator
    {
        public SdkValidationResult Validate(string schemaName, string yamlContent)
            => throw new InvalidOperationException("Simulated schema infrastructure failure");
    }
}
