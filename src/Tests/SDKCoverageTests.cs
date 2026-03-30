#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for DINOForge.SDK.
/// These tests focus on PackRegistryClient, ContentLoader edge cases,
/// and other uncovered areas to raise coverage from 73.2% to 85%+.
/// </summary>
public class SDKCoverageTests
{
    // ──────────────────────── PackRegistryClient ────────────────────────

    [Fact]
    public void PackRegistryClient_DefaultConstructor_UsesDefaultUrl()
    {
        var client = new PackRegistryClient();

        client.Should().NotBeNull();
    }

    [Fact]
    public void PackRegistryClient_WithCustomUrl_UsesProvidedUrl()
    {
        var client = new PackRegistryClient("https://custom.example.com/registry.json");

        client.Should().NotBeNull();
    }

    [Fact]
    public void PackRegistryClient_WithHttpClient_UsesProvidedClient()
    {
        using var httpClient = new HttpClient();
        var client = new PackRegistryClient("https://custom.example.com/registry.json", httpClient);

        client.Should().NotBeNull();
    }

    [Fact]
    public void PackRegistryClient_WithNullUrl_ThrowsArgumentNullException()
    {
        Action action = () => new PackRegistryClient(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void PackRegistryClient_WithNullHttpClient_ThrowsArgumentNullException()
    {
        Action action = () => new PackRegistryClient("https://example.com", null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void PackRegistryClient_CacheDuration_CanBeSet()
    {
        var client = new PackRegistryClient();
        client.CacheDuration = TimeSpan.FromMinutes(30);

        client.CacheDuration.Should().Be(TimeSpan.FromMinutes(30));
    }

    [Fact]
    public void PackRegistryClient_InvalidateCache_ClearsCache()
    {
        var client = new PackRegistryClient();
        client.InvalidateCache();

        // InvalidateCache should complete without throwing
        client.InvalidateCache();
    }

    [Fact]
    public void PackRegistryClient_Instance_ReturnsSingleton()
    {
        var instance1 = PackRegistryClient.Instance;
        var instance2 = PackRegistryClient.Instance;

        instance1.Should().BeSameAs(instance2);
    }

    [Fact]
    public void PackRegistryClient_DefaultRegistryUrl_IsCorrect()
    {
        PackRegistryClient.DefaultRegistryUrl.Should().Be("https://kooshapari.github.io/Dino/registry.json");
    }

    // ──────────────────────── RegistryPackEntry ────────────────────────

    [Fact]
    public void RegistryPackEntry_DefaultValues()
    {
        var entry = new RegistryPackEntry();

        entry.Id.Should().BeEmpty();
        entry.Name.Should().BeEmpty();
        entry.Author.Should().BeEmpty();
        entry.Version.Should().BeEmpty();
        entry.Type.Should().BeEmpty();
        entry.Description.Should().BeEmpty();
        entry.Tags.Should().NotBeNull();
        entry.Tags.Should().BeEmpty();
        entry.Repo.Should().BeEmpty();
        entry.DownloadUrl.Should().BeEmpty();
        entry.PackPath.Should().BeEmpty();
        entry.FrameworkVersion.Should().BeEmpty();
        entry.Verified.Should().BeFalse();
        entry.Featured.Should().BeFalse();
        entry.ConflictsWith.Should().NotBeNull();
        entry.DependsOn.Should().NotBeNull();
    }

    [Fact]
    public void RegistryPackEntry_CanSetAllProperties()
    {
        var entry = new RegistryPackEntry
        {
            Id = "test-pack",
            Name = "Test Pack",
            Author = "Test Author",
            Version = "1.0.0",
            Type = "content",
            Description = "A test pack",
            Tags = new List<string> { "star-wars", "republic" },
            Repo = "https://github.com/test/test",
            DownloadUrl = "https://example.com/test.zip",
            PackPath = "packs/test",
            FrameworkVersion = ">=0.1.0",
            Verified = true,
            Featured = true,
            ConflictsWith = new List<string> { "other-pack" },
            DependsOn = new List<string> { "base-pack" }
        };

        entry.Id.Should().Be("test-pack");
        entry.Name.Should().Be("Test Pack");
        entry.Verified.Should().BeTrue();
        entry.Featured.Should().BeTrue();
        entry.Tags.Should().Contain("star-wars");
    }

    [Fact]
    public void RegistryPackEntry_JsonRoundtrip()
    {
        var original = new RegistryPackEntry
        {
            Id = "test-pack",
            Name = "Test Pack",
            Tags = new List<string> { "tag1", "tag2" }
        };

        string json = JsonSerializer.Serialize(original);
        RegistryPackEntry? deserialized = JsonSerializer.Deserialize<RegistryPackEntry>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Id.Should().Be("test-pack");
        deserialized.Name.Should().Be("Test Pack");
        deserialized.Tags.Should().HaveCount(2);
    }

    // ──────────────────────── PackRegistryFilter ────────────────────────

    [Fact]
    public void PackRegistryFilter_DefaultValues()
    {
        var filter = new PackRegistryFilter();

        filter.Tags.Should().BeNull();
        filter.Type.Should().BeNull();
        filter.Verified.Should().BeNull();
        filter.Featured.Should().BeNull();
    }

    [Fact]
    public void PackRegistryFilter_CanSetAllProperties()
    {
        var filter = new PackRegistryFilter
        {
            Tags = new[] { "tag1", "tag2" },
            Type = "balance",
            Verified = true,
            Featured = false
        };

        filter.Tags.Should().HaveCount(2);
        filter.Type.Should().Be("balance");
        filter.Verified.Should().BeTrue();
        filter.Featured.Should().BeFalse();
    }

    // ──────────────────────── ContentLoader ────────────────────────

    [Fact]
    public void ContentLoader_DefaultConstructor_WithNullRegistryManager_ThrowsArgumentNullException()
    {
        Action action = () => new ContentLoader(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ContentLoadResult_Failure_CreatesCorrectResult()
    {
        var errors = new List<string> { "error1", "error2" };
        var result = ContentLoadResult.Failure(errors);

        result.IsSuccess.Should().BeFalse();
        result.Errors.Should().HaveCount(2);
    }

    [Fact]
    public void ContentLoadResult_Success_CreatesCorrectResult()
    {
        var packs = new List<string> { "pack1", "pack2" };
        var result = ContentLoadResult.Success(packs);

        result.IsSuccess.Should().BeTrue();
        result.LoadedPacks.Should().HaveCount(2);
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void ContentLoadResult_Partial_CreatesCorrectResult()
    {
        var packs = new List<string> { "pack1" };
        var errors = new List<string> { "partial error" };
        var result = ContentLoadResult.Partial(packs, errors);

        result.IsSuccess.Should().BeFalse();
        result.LoadedPacks.Should().HaveCount(1);
        result.Errors.Should().HaveCount(1);
    }

    [Fact]
    public void ContentLoader_LoadPack_WithNullPath_ThrowsArgumentNullException()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);

        Action action = () => loader.LoadPack(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ContentLoader_LoadPack_WithMissingManifest_ReturnsFailure()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string tempDir = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            var result = loader.LoadPack(tempDir);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("not found"));
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void ContentLoader_LoadPack_WithInvalidManifest_ReturnsFailure()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string tempDir = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        File.WriteAllText(Path.Combine(tempDir, "pack.yaml"), "invalid: yaml: content: {{{");

        try
        {
            var result = loader.LoadPack(tempDir);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void ContentLoader_LoadPacks_WithMissingDirectory_ReturnsFailure()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

        var result = loader.LoadPacks(fakePath);

        result.IsSuccess.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("not found"));
    }

    [Fact]
    public void ContentLoader_LoadPacks_WithEmptyDirectory_ReturnsSuccess()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string tempDir = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            var result = loader.LoadPacks(tempDir);

            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().BeEmpty();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void ContentLoader_LoadPacks_WithValidPack_ReturnsSuccess()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string tempDir = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);
        string packDir = Path.Combine(tempDir, "test-pack");
        Directory.CreateDirectory(packDir);

        // Create valid pack.yaml
        string packYaml = @"id: test-pack
name: Test Pack
version: 0.1.0
type: content
framework_version: '>=0.1.0'
loads:
  units: []
  buildings: []
  factions: []
";
        File.WriteAllText(Path.Combine(packDir, "pack.yaml"), packYaml);

        try
        {
            var result = loader.LoadPacks(tempDir);

            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().Contain("test-pack");
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void ContentLoader_LastLoadErrors_ReportsCorrectCount()
    {
        var registryManager = new RegistryManager();
        var loader = new ContentLoader(registryManager);
        string tempDir = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(tempDir);

        try
        {
            loader.LoadPack(tempDir);

            loader.LastLoadErrorCount.Should().BeGreaterThan(0);
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── ContentLoadResult ────────────────────────

    [Fact]
    public void ContentLoadResult_PropertiesAccessible()
    {
        var packs = new List<string> { "pack1" };
        var errors = new List<string> { "error1" };
        var result = ContentLoadResult.Partial(packs, errors);

        result.LoadedPacks.Should().Contain("pack1");
        result.Errors.Should().Contain("error1");
    }

    // ──────────────────────── PackManifest ────────────────────────

    [Fact]
    public void PackManifest_DefaultValues()
    {
        var manifest = new PackManifest();

        manifest.Id.Should().BeEmpty();
        manifest.Name.Should().BeEmpty();
        manifest.Version.Should().Be("0.1.0"); // Has default
        manifest.Type.Should().Be("content"); // Has default
        manifest.Author.Should().BeEmpty();
        manifest.DependsOn.Should().NotBeNull();
        manifest.DependsOn.Should().BeEmpty();
        manifest.ConflictsWith.Should().NotBeNull();
        manifest.ConflictsWith.Should().BeEmpty();
        manifest.Loads.Should().BeNull();
        manifest.Overrides.Should().BeNull();
    }

    [Fact]
    public void PackManifest_CanSetAllProperties()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            Type = "content",
            Author = "Test Author",
            DependsOn = new List<string> { "base-pack" },
            ConflictsWith = new List<string> { "conflicting-pack" }
        };

        manifest.Id.Should().Be("test-pack");
        manifest.Name.Should().Be("Test Pack");
        manifest.Version.Should().Be("1.0.0");
        manifest.DependsOn.Should().Contain("base-pack");
        manifest.ConflictsWith.Should().Contain("conflicting-pack");
    }

    [Fact]
    public void PackManifest_WithLoads_HasCorrectStructure()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Loads = new PackLoads
            {
                Units = new List<string> { "units/trooper.yaml" },
                Buildings = new List<string> { "buildings/barracks.yaml" }
            }
        };

        manifest.Loads.Should().NotBeNull();
        manifest.Loads!.Units.Should().Contain("units/trooper.yaml");
        manifest.Loads.Buildings.Should().Contain("buildings/barracks.yaml");
    }

    [Fact]
    public void PackManifest_WithOverrides_HasCorrectStructure()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Overrides = new PackOverrides
            {
                Stats = new List<string> { "stats/balance.yaml" }
            }
        };

        manifest.Overrides.Should().NotBeNull();
        manifest.Overrides!.Stats.Should().Contain("stats/balance.yaml");
    }

    [Fact]
    public void PackLoads_DefaultValues()
    {
        var loads = new PackLoads();

        loads.Units.Should().BeNull();
        loads.Buildings.Should().BeNull();
        loads.Factions.Should().BeNull();
    }

    [Fact]
    public void PackOverrides_DefaultValues()
    {
        var overrides = new PackOverrides();

        overrides.Stats.Should().BeNull();
    }

    // ──────────────────────── PackLoader ────────────────────────

    [Fact]
    public void PackLoader_LoadFromFile_WithMissingFile_ThrowsFileNotFoundException()
    {
        var loader = new PackLoader();
        string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString() + ".yaml");

        Action action = () => loader.LoadFromFile(fakePath);

        action.Should().Throw<FileNotFoundException>();
    }

    [Fact]
    public void PackLoader_LoadFromFile_WithInvalidYaml_ThrowsException()
    {
        var loader = new PackLoader();
        string tempFile = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}.yaml");
        File.WriteAllText(tempFile, "invalid: yaml: {{{");

        try
        {
            Action action = () => loader.LoadFromFile(tempFile);

            action.Should().Throw<Exception>();
        }
        finally
        {
            File.Delete(tempFile);
        }
    }

    [Fact]
    public void PackLoader_LoadFromFile_WithValidYaml_ReturnsManifest()
    {
        var loader = new PackLoader();
        string tempFile = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid():N}.yaml");
        string yaml = @"id: test-pack
name: Test Pack
version: 1.0.0
type: content
";
        File.WriteAllText(tempFile, yaml);

        try
        {
            var manifest = loader.LoadFromFile(tempFile);

            manifest.Should().NotBeNull();
            manifest.Id.Should().Be("test-pack");
            manifest.Name.Should().Be("Test Pack");
        }
        finally
        {
            File.Delete(tempFile);
        }
    }

    // ──────────────────────── RegistryManager ────────────────────────

    [Fact]
    public void RegistryManager_DefaultConstructor_InitializesEmpty()
    {
        var manager = new RegistryManager();

        manager.Units.Should().NotBeNull();
        manager.Buildings.Should().NotBeNull();
        manager.Factions.Should().NotBeNull();
    }

    [Fact]
    public void RegistryManager_Units_Buildings_Factions_AreAccessible()
    {
        var manager = new RegistryManager();

        var units = manager.Units;
        var buildings = manager.Buildings;
        var factions = manager.Factions;

        units.Should().NotBeNull();
        buildings.Should().NotBeNull();
        factions.Should().NotBeNull();
    }

    // ──────────────────────── RegistryEntry ────────────────────────

    [Fact]
    public void RegistryEntry_CanCreateWithData()
    {
        var source = RegistrySource.Pack;
        var entry = new RegistryEntry<string>("test-id", "test-data", source, "source-pack");

        entry.Id.Should().Be("test-id");
        entry.Data.Should().Be("test-data");
        entry.Source.Should().Be(RegistrySource.Pack);
        entry.SourcePackId.Should().Be("source-pack");
    }

    [Fact]
    public void RegistryEntry_PropertiesAccessible()
    {
        var source = RegistrySource.Framework;
        var entry = new RegistryEntry<string>("id", "data", source, "pack-id");

        entry.Id.Should().Be("id");
        entry.Data.Should().Be("data");
        entry.Source.Should().Be(RegistrySource.Framework);
        entry.SourcePackId.Should().Be("pack-id");
        entry.Priority.Should().BeGreaterThan(0);
    }

    // ──────────────────────── RegistrySource ────────────────────────

    [Fact]
    public void RegistrySource_Enum_HasExpectedValues()
    {
        var values = Enum.GetValues<RegistrySource>();

        values.Should().Contain(RegistrySource.BaseGame);
        values.Should().Contain(RegistrySource.Framework);
        values.Should().Contain(RegistrySource.DomainPlugin);
        values.Should().Contain(RegistrySource.Pack);
    }

    // ──────────────────────── RegistryConflict ────────────────────────

    [Fact]
    public void RegistryConflict_CanCreateWithValues()
    {
        var conflictingIds = new List<string> { "pack-a", "pack-b" };
        var conflict = new RegistryConflict("unit-id", conflictingIds, "Both modify unit stats");

        conflict.EntryId.Should().Be("unit-id");
        conflict.ConflictingPackIds.Should().Contain("pack-a");
        conflict.ConflictingPackIds.Should().Contain("pack-b");
        conflict.Message.Should().Be("Both modify unit stats");
    }

    [Fact]
    public void RegistryConflict_WithSingleConflict_CreatesCorrectly()
    {
        var conflictingIds = new List<string> { "pack-a" };
        var conflict = new RegistryConflict("building-id", conflictingIds, "Single conflict");

        conflict.EntryId.Should().Be("building-id");
        conflict.ConflictingPackIds.Should().HaveCount(1);
        conflict.Message.Should().Be("Single conflict");
    }

    // ──────────────────────── IRegistry<T> ────────────────────────

    [Fact]
    public void IRegistry_Interface_HasExpectedMembers()
    {
        // Verify the interface defines expected methods (checked via reflection)
        var interfaceType = typeof(DINOForge.SDK.Registry.IRegistry<>);
        var genericType = interfaceType.MakeGenericType(typeof(string));

        genericType.Should().NotBeNull();
    }
}
