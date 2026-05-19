using System;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class ContentLoaderIntegrationTests : IDisposable
    {
        private readonly RegistryManager _registries;
        private readonly ContentLoader _loader;
        private readonly string _tempRoot;

        public ContentLoaderIntegrationTests()
        {
            _registries = new RegistryManager();
            _loader = new ContentLoader(_registries);
            _tempRoot = Path.Combine(Path.GetTempPath(), "dinoforge_integ_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
                Directory.Delete(_tempRoot, true);
        }

        [Fact]
        public void LoadPack_ExampleBalancePack_SucceedsFromDisk()
        {
            // Find the example-balance pack relative to the repo root
            string packsDir = FindPacksDirectory();
            string examplePackDir = Path.Combine(packsDir, "example-balance");

            if (!Directory.Exists(examplePackDir))
            {
                // Skip if pack doesn't exist in this environment
                return;
            }

            ContentLoadResult result = _loader.LoadPack(examplePackDir);

            result.IsSuccess.Should().BeTrue(
                because: "the example-balance pack should load without errors. Errors: {0}",
                string.Join("; ", result.Errors));
            result.LoadedPacks.Should().Contain("example-balance");
        }

        [Fact]
        public void LoadPack_ExampleBalancePack_PopulatesRegistries()
        {
            string packsDir = FindPacksDirectory();
            string examplePackDir = Path.Combine(packsDir, "example-balance");

            if (!Directory.Exists(examplePackDir))
                return;

            _loader.LoadPack(examplePackDir);

            // The example-balance pack should register units, buildings, and factions
            _registries.Units.Contains("militia").Should().BeTrue("example-balance pack includes militia unit");
            _registries.Buildings.Contains("barracks").Should().BeTrue("example-balance pack includes barracks building");
            _registries.Factions.Contains("defenders").Should().BeTrue("example-balance pack includes defenders faction");
        }

        [Fact]
        public void LoadPack_MixedValidInvalidContent_ReportsErrors()
        {
            string packDir = CreatePackDirectory("mixed-pack", @"
id: mixed-pack
name: Mixed Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");
            // Valid unit
            CreateContentFile(packDir, "units", "valid-unit.yaml", @"
id: valid-unit
display_name: Valid Unit
unit_class: CoreLineInfantry
faction_id: test
tier: 1
stats:
  hp: 100
  damage: 10
");
            // Invalid unit - array where string expected
            CreateContentFile(packDir, "units", "invalid-unit.yaml", @"
id:
  - broken
  - array
display_name: Invalid
unit_class: CoreLineInfantry
faction_id: test
");

            ContentLoadResult result = _loader.LoadPack(packDir);

            // Should report errors from the invalid file
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void LoadPacks_MultiplePacksWithDependencies_LoadsInOrder()
        {
            string packsRoot = Path.Combine(_tempRoot, "packs");
            Directory.CreateDirectory(packsRoot);

            // Core pack (no dependencies)
            string coreDir = CreatePackDirectory("core", @"
id: core
name: Core Pack
version: 0.1.0
author: Test
type: content
load_order: 10
loads:
  units:
    - units
", packsRoot);
            CreateContentFile(coreDir, "units", "base-soldier.yaml", @"
id: base-soldier
display_name: Base Soldier
unit_class: CoreLineInfantry
faction_id: core
tier: 1
stats:
  hp: 80
");

            // Addon pack (depends on core)
            string addonDir = CreatePackDirectory("addon", @"
id: addon
name: Addon Pack
version: 0.1.0
author: Test
type: content
depends_on:
  - core
load_order: 20
loads:
  units:
    - units
", packsRoot);
            CreateContentFile(addonDir, "units", "elite-soldier.yaml", @"
id: elite-soldier
display_name: Elite Soldier
unit_class: EliteLineInfantry
faction_id: core
tier: 3
stats:
  hp: 200
");

            ContentLoadResult result = _loader.LoadPacks(packsRoot);

            result.IsSuccess.Should().BeTrue(
                because: "both packs should load. Errors: {0}",
                string.Join("; ", result.Errors));
            result.LoadedPacks.Should().HaveCount(2);
            result.LoadedPacks[0].Should().Be("core");
            result.LoadedPacks[1].Should().Be("addon");
            _registries.Units.Contains("base-soldier").Should().BeTrue();
            _registries.Units.Contains("elite-soldier").Should().BeTrue();
        }

        [Fact]
        public void LoadPack_AllContentTypes_RegisteredCorrectly()
        {
            string packDir = CreatePackDirectory("full-pack", @"
id: full-pack
name: Full Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
  buildings:
    - buildings
  factions:
    - factions
  weapons:
    - weapons
  doctrines:
    - doctrines
");
            CreateContentFile(packDir, "units", "trooper.yaml", @"
id: trooper
display_name: Trooper
unit_class: CoreLineInfantry
faction_id: test
");
            CreateContentFile(packDir, "buildings", "fort.yaml", @"
id: fort
display_name: Fort
building_type: defense
health: 300
");
            CreateContentFile(packDir, "factions", "alliance.yaml", @"
faction:
  id: alliance
  display_name: Alliance
  theme: fantasy
  archetype: order
");
            CreateContentFile(packDir, "weapons", "rifle.yaml", @"
id: rifle
display_name: Rifle
weapon_class: rifle
base_damage: 15
range: 20
");
            CreateContentFile(packDir, "doctrines", "defense.yaml", @"
id: defense-doctrine
display_name: Defensive Doctrine
modifiers:
  armor: 1.2
  speed: 0.9
");

            ContentLoadResult result = _loader.LoadPack(packDir);

            result.IsSuccess.Should().BeTrue(
                because: "all content types should load. Errors: {0}",
                string.Join("; ", result.Errors));
            _registries.Units.Contains("trooper").Should().BeTrue();
            _registries.Buildings.Contains("fort").Should().BeTrue();
            _registries.Factions.Contains("alliance").Should().BeTrue();
            _registries.Weapons.Contains("rifle").Should().BeTrue();
            _registries.Doctrines.Contains("defense-doctrine").Should().BeTrue();
        }

        [Fact]
        public void LoadPack_EmptyLoadsSections_Succeeds()
        {
            string packDir = CreatePackDirectory("empty-loads", @"
id: empty-loads
name: Empty Loads
version: 0.1.0
author: Test
type: content
loads:
  units: []
  buildings: []
");

            ContentLoadResult result = _loader.LoadPack(packDir);

            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().Contain("empty-loads");
        }

        [Fact]
        public void LoadPacks_NonexistentDirectory_Fails()
        {
            ContentLoadResult result = _loader.LoadPacks(Path.Combine(_tempRoot, "nonexistent"));

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void LoadPack_ExampleBalancePack_LoadsStatOverrides()
        {
            string packsDir = FindPacksDirectory();
            string examplePackDir = Path.Combine(packsDir, "example-balance");

            if (!Directory.Exists(examplePackDir))
                return;

            ContentLoadResult result = _loader.LoadPack(examplePackDir);

            result.LoadedPacks.Should().Contain("example-balance");
            _loader.LoadedOverrides.Should().NotBeEmpty(
                because: "the example-balance pack contains stats/melee-buff.yaml");

            StatOverrideDefinition overrideDef = _loader.LoadedOverrides[0];
            overrideDef.Overrides.Should().HaveCount(2);
            overrideDef.Overrides[0].Target.Should().Be("unit.stats.hp");
            overrideDef.Overrides[0].Value.Should().Be(1.5f);
            overrideDef.Overrides[0].Mode.Should().Be(StatOverrideMode.Multiply);
            overrideDef.Overrides[0].Filter.Should().Be("Components.MeleeUnit");
            overrideDef.Overrides[1].Target.Should().Be("unit.stats.speed");
        }

        [Fact]
        public void LoadPack_ReturnsFailureForMissingDir()
        {
            string missingDir = Path.Combine(_tempRoot, "no-such-pack");

            ContentLoadResult result = _loader.LoadPack(missingDir);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("not found");
        }

        [Fact]
        public void LoadPacks_ResolveDependencies_LoadsInCorrectOrder()
        {
            string packsRoot = Path.Combine(_tempRoot, "dep-packs");
            Directory.CreateDirectory(packsRoot);

            // Base pack
            string baseDir = CreatePackDirectory("base-mod", @"
id: base-mod
name: Base Mod
version: 1.0.0
author: Test
type: content
load_order: 10
", packsRoot);
            CreateContentFile(baseDir, "stats", "base-stats.yaml", @"
overrides:
  - target: unit.stats.hp
    value: 100
    mode: override
");

            // Dependent pack
            string depDir = CreatePackDirectory("dep-mod", @"
id: dep-mod
name: Dependent Mod
version: 1.0.0
author: Test
type: balance
depends_on:
  - base-mod
load_order: 20
", packsRoot);
            CreateContentFile(depDir, "stats", "dep-stats.yaml", @"
overrides:
  - target: unit.stats.hp
    value: 1.5
    mode: multiply
    filter: Components.MeleeUnit
");

            ContentLoadResult result = _loader.LoadPacks(packsRoot);

            result.IsSuccess.Should().BeTrue(
                because: "both packs should load. Errors: {0}",
                string.Join("; ", result.Errors));
            result.LoadedPacks.Should().HaveCount(2);
            result.LoadedPacks[0].Should().Be("base-mod");
            result.LoadedPacks[1].Should().Be("dep-mod");

            _loader.LoadedOverrides.Should().HaveCount(2,
                because: "each pack contributes one stat override file");
        }

        private string CreatePackDirectory(string name, string manifestYaml, string? parentDir = null)
        {
            string dir = Path.Combine(parentDir ?? _tempRoot, name);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, "pack.yaml"), manifestYaml);
            return dir;
        }

        private void CreateContentFile(string packDir, string subDir, string fileName, string yamlContent)
        {
            string dir = Path.Combine(packDir, subDir);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, fileName), yamlContent);
        }

        private static string FindPacksDirectory()
        {
            string? dir = Directory.GetCurrentDirectory();
            while (dir != null)
            {
                string candidate = Path.Combine(dir, "packs");
                if (Directory.Exists(candidate))
                    return candidate;
                dir = Directory.GetParent(dir)?.FullName;
            }

            string assemblyDir = Path.GetDirectoryName(typeof(ContentLoaderIntegrationTests).Assembly.Location) ?? "";
            dir = assemblyDir;
            while (dir != null)
            {
                string candidate = Path.Combine(dir, "packs");
                if (Directory.Exists(candidate))
                    return candidate;
                dir = Directory.GetParent(dir)?.FullName;
            }

            return Path.Combine(Directory.GetCurrentDirectory(), "packs");
        }
    }
}
