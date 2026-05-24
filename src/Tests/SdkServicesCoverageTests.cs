#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Additional unit tests for <see cref="YamlLoader"/>, <see cref="FileDiscoveryService"/>,
/// and SDK validation helpers — targets coverage gaps not exercised by sibling test classes.
/// </summary>
public class SdkServicesCoverageTests : IDisposable
{
    private readonly string _tempDir;

    public SdkServicesCoverageTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "sdk_services_cov_" + Guid.NewGuid().ToString("N"));
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

    /// <summary>Null YAML input must short-circuit before YamlDotNet parsing.</summary>
    [Fact]
    public void YamlLoader_Deserialize_NullString_ReturnsDefault()
    {
        var result = YamlLoader.Deserialize<SimpleManifest>(null!);

        result.Should().BeNull();
    }

    /// <summary>IgnoreUnmatchedProperties allows forward-compatible pack YAML.</summary>
    [Fact]
    public void YamlLoader_Deserialize_ExtraYamlKeys_AreIgnored()
    {
        const string yaml = @"
name: Forward Compatible
future_field: will ship later
version: 1.0.0";

        var result = YamlLoader.Deserialize<SimpleManifest>(yaml);

        result.Should().NotBeNull();
        result!.Name.Should().Be("Forward Compatible");
        result.Version.Should().Be("1.0.0");
    }

    /// <summary>Shared deserializer instance must match static Deserialize behavior.</summary>
    [Fact]
    public void YamlLoader_DeserializerProperty_MatchesStaticDeserialize()
    {
        const string yaml = "name: Shared\nversion: 2.0.0";

        var viaStatic = YamlLoader.Deserialize<SimpleManifest>(yaml);
        var viaProperty = YamlLoader.Deserializer.Deserialize<SimpleManifest>(yaml);

        viaProperty.Should().BeEquivalentTo(viaStatic);
    }

    /// <summary>Empty on-disk YAML files should not throw during file load.</summary>
    [Fact]
    public void YamlLoader_DeserializeFromFile_EmptyFile_ReturnsNull()
    {
        string path = Path.Combine(_tempDir, "empty.yaml");
        File.WriteAllText(path, string.Empty);

        var result = YamlLoader.DeserializeFromFile<SimpleManifest>(path);

        result.Should().BeNull();
    }

    /// <summary>Without default exclusions, underscore dirs are discoverable (author tooling).</summary>
    [Fact]
    public void FileDiscoveryService_UseDefaultsFalse_IncludesUnderscoreDirectories()
    {
        Directory.CreateDirectory(Path.Combine(_tempDir, "_wip"));
        Directory.CreateDirectory(Path.Combine(_tempDir, "live"));
        var service = new FileDiscoveryService(useDefaults: false);

        string[] dirs = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

        dirs.Should().HaveCount(2);
        dirs.Select(Path.GetFileName).Should().Contain("_wip").And.Contain("live");
    }

    /// <summary>Recursive file search must skip default excluded subtrees (e.g. generated).</summary>
    [Fact]
    public void FileDiscoveryService_GetFilesRecursive_SkipsGeneratedDirectory()
    {
        string generatedDir = Path.Combine(_tempDir, "generated");
        Directory.CreateDirectory(generatedDir);
        File.WriteAllText(Path.Combine(_tempDir, "root.yaml"), "id: root");
        File.WriteAllText(Path.Combine(generatedDir, "hidden.yaml"), "id: hidden");
        var service = new FileDiscoveryService();

        string[] files = service.GetFiles(_tempDir, "*.yaml", SearchOption.AllDirectories);

        files.Should().HaveCount(1);
        files[0].Should().EndWith("root.yaml");
    }

    /// <summary>Weapon/doctrine content types must resolve for schema validation in RegistryImportService.</summary>
    [Fact]
    public void SchemaResolverService_TryResolveSchemaName_ResolvesPipelineContentTypes()
    {
        var resolver = new SchemaResolverService();
        var expected = new Dictionary<string, string>
        {
            ["weapons"] = "weapon",
            ["projectiles"] = "projectile",
            ["doctrines"] = "doctrine",
            ["faction_patches"] = "faction-patch",
        };

        foreach (var pair in expected)
        {
            resolver.TryResolveSchemaName(pair.Key, out string schemaName).Should().BeTrue(
                because: $"content type '{pair.Key}' is registered");
            schemaName.Should().Be(pair.Value);
        }
    }

    /// <summary>Required-field violations must surface structured validation errors.</summary>
    [Fact]
    public void NJsonSchemaValidator_Validate_MissingRequiredField_ReturnsFailure()
    {
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

        ValidationResult result = validator.Validate("unit", "display_name: Trooper\n");

        result.IsValid.Should().BeFalse();
        result.Errors.Should().NotBeEmpty();
    }

    private sealed class SimpleManifest
    {
        public string? Name { get; set; }
        public string? Version { get; set; }
    }
}
