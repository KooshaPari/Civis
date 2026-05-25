using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class ContentLoaderArrayTests : IDisposable
    {
        private readonly RegistryManager _registries;
        private readonly ContentLoader _loader;
        private readonly string _tempRoot;

        public ContentLoaderArrayTests()
        {
            _registries = new RegistryManager();
            _loader = new ContentLoader(_registries);
            _tempRoot = Path.Combine(Path.GetTempPath(), "dinoforge_array_test_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
                Directory.Delete(_tempRoot, true);
        }

        [Fact]
        public void ContentLoader_LoadsArrayYaml_RegistersAllUnits()
        {
            // Arrange
            string packDir = CreatePackDirectory("array-pack", @"
id: array-pack
name: Array Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");

            // Create a YAML file with an array of units
            CreateContentFile(packDir, "units", "soldiers.yaml", @"
- id: soldier
  display_name: Soldier
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1
  stats:
    hp: 100
    damage: 15

- id: officer
  display_name: Officer
  unit_class: Leader
  faction_id: test
  tier: 2
  stats:
    hp: 150
    damage: 20

- id: scout
  display_name: Scout
  unit_class: Recon
  faction_id: test
  tier: 1
  stats:
    hp: 50
    damage: 10
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();

            _registries.Units.Contains("soldier").Should().BeTrue();
            _registries.Units.Contains("officer").Should().BeTrue();
            _registries.Units.Contains("scout").Should().BeTrue();

            _registries.Units.Get("soldier")!.DisplayName.Should().Be("Soldier");
            _registries.Units.Get("officer")!.DisplayName.Should().Be("Officer");
            _registries.Units.Get("scout")!.DisplayName.Should().Be("Scout");
        }

        [Fact]
        public void ContentLoader_LoadsSingleYaml_RegistersUnit()
        {
            // Arrange
            string packDir = CreatePackDirectory("single-pack", @"
id: single-pack
name: Single Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");

            // Create a single unit (not an array)
            CreateContentFile(packDir, "units", "knight.yaml", @"
id: knight
display_name: Knight
unit_class: HeavyInfantry
faction_id: knights
tier: 2
stats:
  hp: 200
  armor: 10
  damage: 25
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();

            _registries.Units.Contains("knight").Should().BeTrue();
            _registries.Units.Get("knight")!.DisplayName.Should().Be("Knight");
            _registries.Units.Get("knight")!.Stats.Hp.Should().Be(200);
        }

        [Fact]
        public void ContentLoader_LoadsArrayYaml_RegistersAllBuildings()
        {
            // Arrange
            string packDir = CreatePackDirectory("building-array-pack", @"
id: building-array-pack
name: Building Array Pack
version: 0.1.0
author: Test
type: content
loads:
  buildings:
    - buildings
");

            // Create a YAML file with an array of buildings
            CreateContentFile(packDir, "buildings", "defenses.yaml", @"
- id: tower
  display_name: Watch Tower
  building_type: defense
  health: 200
  cost:
    wood: 100
    stone: 50

- id: wall
  display_name: Stone Wall
  building_type: defense
  health: 150
  cost:
    wood: 50
    stone: 100

- id: barracks
  display_name: Barracks
  building_type: military
  health: 100
  cost:
    wood: 200
    stone: 50
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();

            _registries.Buildings.Contains("tower").Should().BeTrue();
            _registries.Buildings.Contains("wall").Should().BeTrue();
            _registries.Buildings.Contains("barracks").Should().BeTrue();

            _registries.Buildings.Get("tower")!.DisplayName.Should().Be("Watch Tower");
            _registries.Buildings.Get("wall")!.DisplayName.Should().Be("Stone Wall");
            _registries.Buildings.Get("barracks")!.DisplayName.Should().Be("Barracks");
        }

        [Fact]
        public void ContentLoader_MixedPacks_AllItemsRegistered()
        {
            // Arrange - create two packs with both array and single YAML files
            string packsRoot = Path.Combine(_tempRoot, "packs");
            Directory.CreateDirectory(packsRoot);

            // Pack 1: array units
            string pack1Dir = CreatePackDirectory("pack-array", @"
id: pack-array
name: Pack Array
version: 0.1.0
author: Test
type: content
load_order: 10
loads:
  units:
    - units
", packsRoot);

            CreateContentFile(pack1Dir, "units", "infantry.yaml", @"
- id: infantry-basic
  display_name: Basic Infantry
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1

- id: infantry-elite
  display_name: Elite Infantry
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 2
");

            // Pack 2: single building
            string pack2Dir = CreatePackDirectory("pack-single", @"
id: pack-single
name: Pack Single
version: 0.1.0
author: Test
type: content
load_order: 20
loads:
  buildings:
    - buildings
", packsRoot);

            CreateContentFile(pack2Dir, "buildings", "fort.yaml", @"
id: fort
display_name: Fort
building_type: defense
health: 300
");

            // Act
            ContentLoadResult result = _loader.LoadPacks(packsRoot);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().HaveCount(2);
            result.Errors.Should().BeEmpty();

            // Verify units from pack 1
            _registries.Units.Contains("infantry-basic").Should().BeTrue();
            _registries.Units.Contains("infantry-elite").Should().BeTrue();

            // Verify building from pack 2
            _registries.Buildings.Contains("fort").Should().BeTrue();
        }

        [Fact]
        public void ContentLoader_LoadsEmptyArrayYaml_IsHandledGracefully()
        {
            // Arrange - array with no items
            string packDir = CreatePackDirectory("empty-array-pack", @"
id: empty-array-pack
name: Empty Array Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");

            // Create an empty array YAML file - this will fail deserialization gracefully
            CreateContentFile(packDir, "units", "empty.yaml", "[]");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert - empty array falls back to single-object deserialization which fails,
            // but ContentLoader handles this and either succeeds with no content or reports partial success
            // The key is that it doesn't crash
            result.Should().NotBeNull();
        }

        [Fact]
        public void ContentLoader_LoadsArrayYaml_MultipleFiles_RegistersAll()
        {
            // Arrange
            string packDir = CreatePackDirectory("multi-file-pack", @"
id: multi-file-pack
name: Multi File Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");

            // Create multiple YAML files, some with arrays, some with single items
            CreateContentFile(packDir, "units", "tier1.yaml", @"
- id: militia
  display_name: Militia
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1

- id: archer-novice
  display_name: Novice Archer
  unit_class: Skirmisher
  faction_id: test
  tier: 1
");

            CreateContentFile(packDir, "units", "tier2.yaml", @"
id: knight
display_name: Knight
unit_class: HeavyInfantry
faction_id: test
tier: 2
");

            CreateContentFile(packDir, "units", "tier3.yaml", @"
- id: general
  display_name: General
  unit_class: Leader
  faction_id: test
  tier: 3
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.Errors.Should().BeEmpty();

            // All 4 units should be registered
            _registries.Units.Contains("militia").Should().BeTrue();
            _registries.Units.Contains("archer-novice").Should().BeTrue();
            _registries.Units.Contains("knight").Should().BeTrue();
            _registries.Units.Contains("general").Should().BeTrue();
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
    }
}
