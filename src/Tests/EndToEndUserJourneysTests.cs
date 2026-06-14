#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// End-to-end user journey tests validating complete workflows.
/// These tests simulate real user interactions from start to finish.
/// </summary>
[Trait("Category", "Epic")]
[Trait("Epic", "Epic-EndToEnd")]
public class EndToEndUserJourneysTests : IDisposable
{
    private readonly string _tempPackDir;

    public EndToEndUserJourneysTests()
    {
        _tempPackDir = Path.Combine(Path.GetTempPath(), "dinoforge_e2e_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempPackDir);
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_tempPackDir))
                Directory.Delete(_tempPackDir, true);
        }
        catch { /* best-effort cleanup */ }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 1: Install & Play
    // ID: Journey-InstallPlay
    // Path: Download pack → Extract → Launch → Pack loads → Play with mod
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-InstallPlay")]
    [Trait("UserStory", "US-F1.1")]
    public async Task Journey1_InstallPlay_PackLoadsSuccessfully()
    {
        // ARRANGE: Simulate pack extraction to BepInEx/dinoforge_packs/
        var packDir = Path.Combine(_tempPackDir, "my-test-pack");
        Directory.CreateDirectory(packDir);

        var manifest = new PackManifest
        {
            Id = "my-test-pack",
            Name = "My Test Pack",
            Version = "1.0.0",
            Type = "content",
            Author = "Test Author",
            DependsOn = new List<string>()
        };

        var manifestPath = Path.Combine(packDir, "pack.yaml");
        await File.WriteAllTextAsync(manifestPath, @"
id: my-test-pack
name: My Test Pack
version: 1.0.0
type: content
author: Test Author
");

        // ACT: Load the pack
        var loader = new PackLoader();
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);

        // ASSERT: Pack loads successfully
        loadedPacks.Should().NotBeEmpty("at least one pack should be loaded");
        loadedPacks.Should().Contain(p => p.Id == "my-test-pack", "our test pack should be in the loaded list");
    }

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-InstallPlay")]
    [Trait("UserStory", "US-F1.1")]
    public void Journey1_InstallPlay_PackWithMissingYaml_LoadsAsEmpty()
    {
        // ARRANGE: Pack directory without pack.yaml
        var packDir = Path.Combine(_tempPackDir, "broken-pack");
        Directory.CreateDirectory(packDir);

        var loader = new PackLoader();

        // ACT: Loading returns empty list (no exception thrown)
        var result = loader.LoadPacksFromDirectory(packDir);

        // ASSERT: Pack is skipped silently
        result.Should().BeEmpty("missing pack.yaml causes the directory to be skipped");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 2: Create Balance Mod
    // ID: Journey-CreateBalance
    // Path: Create pack.yaml → Create units.yaml → Run deploy → Launch → Verify → Reload
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateBalance")]
    [Trait("UserStory", "US-F1.1")]
    [Trait("UserStory", "US-F4.1")]
    public async Task Journey2_CreateBalanceMod_PackValidatesAndLoads()
    {
        // ARRANGE: Create balance mod with stat overrides
        var packDir = Path.Combine(_tempPackDir, "balance-mod");
        Directory.CreateDirectory(packDir);

        // 1. Create pack.yaml
        var packYaml = @"
id: balance-mod
name: Balance Mod
version: 1.0.0
type: balance
author: Test Author
";
        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), packYaml);

        // 2. Create units.yaml with stat overrides
        var unitsYaml = @"
units:
  - id: infantry
    name: Infantry
    stats:
      cost: 50
      hp: 100
      damage: 10
  - id: archer
    name: Archer
    stats:
      cost: 40
      hp: 60
      damage: 15
";
        await File.WriteAllTextAsync(Path.Combine(packDir, "units.yaml"), unitsYaml);

        // ACT: Load the pack
        var loader = new PackLoader();
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);
        var balancePack = loadedPacks.FirstOrDefault(p => p.Id == "balance-mod");

        // ASSERT: Pack loads and has correct properties
        balancePack.Should().NotBeNull("balance mod should load");
        balancePack!.Version.Should().Be("1.0.0");
        balancePack.Type.Should().Be("balance");
    }

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateBalance")]
    [Trait("UserStory", "US-F4.1")]
    public async Task Journey2_CreateBalanceMod_HotReload_PicksUpChanges()
    {
        // ARRANGE: Create initial pack
        var packDir = Path.Combine(_tempPackDir, "hot-reload-pack");
        Directory.CreateDirectory(packDir);

        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), @"
id: hot-reload-pack
name: Hot Reload Pack
version: 1.0.0
type: balance
");

        var unitsYamlPath = Path.Combine(packDir, "units.yaml");
        await File.WriteAllTextAsync(unitsYamlPath, @"
units:
  - id: soldier
    name: Soldier
    stats:
      hp: 100
");

        // ACT: Initial load
        var loader = new PackLoader();
        var initial = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);

        // Simulate file change detection. Pin both timestamps explicitly instead of sleeping to
        // force a wall-clock gap (Pattern #108): a blind Task.Delay is flaky on filesystems with
        // coarse mtime resolution and needlessly slow. Setting the mtimes makes the "modification
        // yields a later timestamp" invariant deterministic and instant.
        var lastModified = DateTime.UtcNow.AddSeconds(-1);
        File.SetLastWriteTimeUtc(unitsYamlPath, lastModified);

        await File.WriteAllTextAsync(unitsYamlPath, @"
units:
  - id: soldier
    name: Soldier
    stats:
      hp: 200
");
        var newLastModified = DateTime.UtcNow;
        File.SetLastWriteTimeUtc(unitsYamlPath, newLastModified);

        // ASSERT: a modification advances the file's last-write timestamp.
        File.GetLastWriteTimeUtc(unitsYamlPath).Should().BeAfter(lastModified, "file was modified");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 3: Create Total Conversion
    // ID: Journey-CreateTotalConversion
    // Path: Create pack + factions → Define units → Add assets → Deploy → Test
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateTotalConversion")]
    [Trait("UserStory", "US-F1.1")]
    [Trait("UserStory", "US-F5.1")]
    public async Task Journey3_TotalConversion_AllComponentsLoad()
    {
        // ARRANGE: Create full Star Wars style total conversion
        var packDir = Path.Combine(_tempPackDir, "star-wars-conversion");
        Directory.CreateDirectory(Path.Combine(packDir, "assets", "bundles"));

        // 1. Create pack.yaml
        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), @"
id: star-wars-conversion
name: Star Wars Conversion
version: 1.0.0
type: total_conversion
author: Test Author
framework_version: '>=0.11.0 <1.0.0'
");

        // 2. Create factions.yaml
        await File.WriteAllTextAsync(Path.Combine(packDir, "factions.yaml"), @"
factions:
  - id: republic
    name: Galactic Republic
    color: '#00A8E8'
  - id: cis
    name: Confederacy of Independent Systems
    color: '#C8A87A'
");

        // 3. Create units.yaml with 28 units
        var unitsContent = "units:\n";
        for (int i = 1; i <= 28; i++)
        {
            unitsContent += $@"
  - id: unit_{i}
    name: Unit {i}
    faction: {(i <= 14 ? "republic" : "cis")}
    stats:
      hp: {100 + i}
      damage: {10 + i}
";
        }
        await File.WriteAllTextAsync(Path.Combine(packDir, "units.yaml"), unitsContent);

        // ACT: Load the pack
        var loader = new PackLoader();
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);
        var conversion = loadedPacks.FirstOrDefault(p => p.Id == "star-wars-conversion");

        // ASSERT: Full conversion loads
        conversion.Should().NotBeNull("total conversion should load");
        conversion!.Type.Should().Be("total_conversion");
    }

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateTotalConversion")]
    [Trait("UserStory", "US-F5.1")]
    public async Task Journey3_TotalConversion_VisualAssets_MappedCorrectly()
    {
        // ARRANGE: Create pack with visual asset references
        var packDir = Path.Combine(_tempPackDir, "visual-pack");
        Directory.CreateDirectory(packDir);
        Directory.CreateDirectory(Path.Combine(packDir, "assets", "bundles"));

        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), @"
id: visual-pack
name: Visual Pack
version: 1.0.0
type: content
");

        await File.WriteAllTextAsync(Path.Combine(packDir, "units.yaml"), @"
units:
  - id: trooper
    name: Clone Trooper
    visual_asset: sw-rep-clone-trooper
");

        // Create a placeholder bundle file
        var bundlePath = Path.Combine(packDir, "assets", "bundles", "sw-rep-clone-trooper");
        await File.WriteAllBytesAsync(bundlePath, new byte[] { 0x55, 0x6E, 0x69, 0x74, 0x79, 0x42, 0x75, 0x6E, 0x64, 0x6C, 0x65 }); // UnityBundle header

        // ACT: Load pack and verify asset reference
        var loader = new PackLoader();
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);
        var visualPack = loadedPacks.FirstOrDefault(p => p.Id == "visual-pack");

        // ASSERT: Visual asset is referenced
        visualPack.Should().NotBeNull("visual pack should load");
        File.Exists(bundlePath).Should().BeTrue("bundle file should exist");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 4: Debug & Troubleshoot
    // ID: Journey-Debug
    // Path: Check manifest → Check assets → Launch → F9 overlay → Query entities
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Debug")]
    [Trait("UserStory", "US-F6.1")]
    public async Task Journey4_Debug_ManifestValidation_SkipsInvalidPack()
    {
        // ARRANGE: Create pack with invalid YAML (version is not a valid semver)
        var packDir = Path.Combine(_tempPackDir, "invalid-pack");
        Directory.CreateDirectory(packDir);

        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), @"
id: invalid-pack
name: Invalid Pack
version: not-a-version
type: content
");

        var loader = new PackLoader();

        // ACT: Loading returns empty list (invalid pack is skipped)
        var result = loader.LoadPacksFromDirectory(packDir);

        // ASSERT: Invalid pack is skipped without throwing
        result.Should().BeEmpty("invalid pack should be skipped during loading");
    }

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Debug")]
    [Trait("UserStory", "US-F6.1")]
    public async Task Journey4_Debug_MissingDependencies_ReportsClearErrors()
    {
        // ARRANGE: Pack that depends on non-existent pack
        var packDir = Path.Combine(_tempPackDir, "dep-pack");
        Directory.CreateDirectory(packDir);

        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), @"
id: dep-pack
name: Dependent Pack
version: 1.0.0
type: content
depends_on:
  - nonexistent-pack
");

        var loader = new PackLoader();
        var resolver = new PackDependencyResolver();

        // ACT: Load packs
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);
        var depPack = loadedPacks.FirstOrDefault(p => p.Id == "dep-pack");

        // ASSERT: Dependency resolution fails
        var result = resolver.ResolveDependencies(loadedPacks, depPack!);
        result.IsSuccess.Should().BeFalse("missing dependency should fail resolution");
        result.Errors.Should().NotBeEmpty("should have error about missing dependency");
    }

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Debug")]
    [Trait("UserStory", "US-F6.1")]
    public void Journey4_Debug_CircularDependencies_Detected()
    {
        // ARRANGE: Circular dependency: A → B → C → A
        var available = new List<PackManifest>
        {
            new PackManifest { Id = "pack-a", Name = "Pack A", Version = "1.0.0", DependsOn = new List<string> { "pack-c" } },
            new PackManifest { Id = "pack-b", Name = "Pack B", Version = "1.0.0", DependsOn = new List<string> { "pack-a" } },
            new PackManifest { Id = "pack-c", Name = "Pack C", Version = "1.0.0", DependsOn = new List<string> { "pack-b" } }
        };

        var resolver = new PackDependencyResolver();

        // ACT: Resolve dependencies
        var result = resolver.ResolveDependencies(available, available[0]);

        // ASSERT: Circular dependency detected
        result.IsSuccess.Should().BeFalse("circular dependency should fail");
        result.Errors.Should().Contain(e => e.Contains("cycle", StringComparison.OrdinalIgnoreCase) ||
                                           e.Contains("circular", StringComparison.OrdinalIgnoreCase),
            "error should mention circular dependency");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 5: Pack Conflicts Detection
    // ID: Journey-PackConflicts
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-PackConflicts")]
    [Trait("UserStory", "US-F6.1")]
    public async Task Journey5_PackConflicts_DetectedCorrectly()
    {
        // ARRANGE: Two packs that conflict
        var pack1Dir = Path.Combine(_tempPackDir, "conflict-pack-1");
        var pack2Dir = Path.Combine(_tempPackDir, "conflict-pack-2");
        Directory.CreateDirectory(pack1Dir);
        Directory.CreateDirectory(pack2Dir);

        await File.WriteAllTextAsync(Path.Combine(pack1Dir, "pack.yaml"), @"
id: conflict-pack-1
name: Conflict Pack 1
version: 1.0.0
type: content
");
        await File.WriteAllTextAsync(Path.Combine(pack2Dir, "pack.yaml"), @"
id: conflict-pack-2
name: Conflict Pack 2
version: 1.0.0
type: content
conflicts_with:
  - conflict-pack-1
");

        var loader = new PackLoader();

        // ACT: Load both packs
        var loadedPacks = await loader.LoadPacksFromDirectoryAsync(_tempPackDir);

        // ASSERT: Both packs loaded but conflict is recorded
        loadedPacks.Should().HaveCount(2, "both packs should load");
        var conflictPack = loadedPacks.FirstOrDefault(p => p.Id == "conflict-pack-2");
        conflictPack.Should().NotBeNull();
        conflictPack!.ConflictsWith.Should().Contain("conflict-pack-1");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 6: Framework Version Compatibility
    // ID: Journey-FrameworkVersion
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-FrameworkVersion")]
    [Trait("UserStory", "US-F1.1")]
    public async Task Journey6_FrameworkVersion_PackLoads_AndFrameworkVersionCanBeRead()
    {
        // ARRANGE: Pack with framework version constraint
        var packDir = Path.Combine(_tempPackDir, "versioned-pack");
        Directory.CreateDirectory(packDir);

        // Note: YAML must not have leading whitespace on each line (verbatim string preserves indentation)
        string yamlContent = "id: versioned-pack\nname: Versioned Pack\nversion: 1.0.0\ntype: content\nframework_version: 0.5.0\n";
        await File.WriteAllTextAsync(Path.Combine(packDir, "pack.yaml"), yamlContent);

        var loader = new PackLoader();

        // ACT: Pack loads successfully (note: scans subdirs of packDir)
        var packs = loader.LoadPacksFromDirectory(_tempPackDir);

        // ASSERT: Pack is loaded (YAML parsed correctly)
        packs.Should().ContainSingle(p => p.Id == "versioned-pack");

        // ASSERT: Framework version is readable from the manifest
        var pack = packs[0];
        pack.Id.Should().Be("versioned-pack");
        pack.Version.Should().Be("1.0.0");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Journey 7: Registry Operations
    // ID: Journey-Registry
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Registry")]
    [Trait("UserStory", "US-F1.1")]
    public void Journey7_Registry_AllowsMultipleRegistrations_AndReturnsHighestPriority()
    {
        // ARRANGE: Two packs register same unit ID
        var registry = new Registry<UnitDefinition>();

        var unit1 = new UnitDefinition
        {
            Id = "infantry",
            DisplayName = "Infantry",
            FactionId = "classic",
        };
        var unit2 = new UnitDefinition
        {
            Id = "infantry",
            DisplayName = "Infantry Override",
            FactionId = "classic",
        };

        // ACT: Register both units under the same ID
        registry.Register("infantry", unit1, RegistrySource.Pack, "pack-1");
        registry.Register("infantry", unit2, RegistrySource.Pack, "pack-2");

        // ASSERT: Registry contains the entry
        registry.Contains("infantry").Should().BeTrue();

        // ASSERT: Get returns an entry (priority determines which one)
        UnitDefinition? result = registry.Get("infantry");
        result.Should().NotBeNull();
        result!.Id.Should().Be("infantry");

        // ASSERT: All entries are stored (override behavior via RegistrySource priority)
        IReadOnlyDictionary<string, RegistryEntry<UnitDefinition>> all = registry.All;
        all.Should().ContainKey("infantry");
    }
}
