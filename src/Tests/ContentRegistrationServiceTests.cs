using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Negative tests for task #210 Phase 2 — JsonGuard / IValidatable wiring at
    /// <see cref="RegistryImportService"/> deserialize sites (list path + single-item
    /// path). Mirrors the Phase 1 pattern landed at <c>PackLoaderTests</c>
    /// (PackLoader_RejectsManifest*).
    ///
    /// Adoption surface:
    /// - <c>UnitDefinition.Validate()</c> (blank id / display_name / hp&lt;=0 / accuracy out of [0,1]).
    /// - <c>BuildingDefinition.Validate()</c> (blank id / display_name / health&lt;0).
    ///
    /// Both flow through <c>JsonGuard.ValidateOrThrow</c> in
    /// <c>RegistryImportService.RegisterItems&lt;T&gt;</c>.
    /// </summary>
    public class ContentRegistrationServiceTests : IDisposable
    {
        private readonly RegistryManager _registries;
        private readonly ContentLoader _loader;
        private readonly string _tempRoot;

        public ContentRegistrationServiceTests()
        {
            _registries = new RegistryManager();
            _loader = new ContentLoader(_registries);
            _tempRoot = Path.Combine(
                Path.GetTempPath(),
                "dinoforge_content_reg_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
            {
                try
                {
                    Directory.Delete(_tempRoot, true);
                }
                catch
                {
                    // Best-effort temp cleanup — harness may keep handles in CI.
                }
            }
        }

        // ── List path: blank id is rejected ──────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void RegisterFromYaml_RejectsItemsWithBlankId()
        {
            // Arrange — array YAML where one unit has a blank id.
            string packDir = CreatePackDirectory("blank-id-pack", @"
id: blank-id-pack
name: Blank Id Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");
            CreateContentFile(packDir, "units", "units.yaml", @"
- id: ''
  display_name: Phantom Soldier
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1
  stats:
    hp: 100
    damage: 15
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert — JsonGuard surfaces the violation as InvalidDataException;
            // ContentLoader.LoadAndRegisterContent catches it and adds to
            // result.Errors so the pack-load-summary remains intact.
            result.IsSuccess.Should().BeFalse(
                "JsonGuard should reject the blank-id unit before registration");
            result.Errors.Should().Contain(e => e.Contains("id"));
            _registries.Units.Contains("").Should().BeFalse();
        }

        // ── List path: missing required field (hp<=0) is rejected ────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void RegisterFromYaml_RejectsItemsWithMissingRequiredField()
        {
            // Arrange — unit has explicit hp: 0 (sentinel for missing/uninitialised
            // health). Validate() requires hp > 0.
            string packDir = CreatePackDirectory("zero-hp-pack", @"
id: zero-hp-pack
name: Zero HP Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");
            CreateContentFile(packDir, "units", "units.yaml", @"
- id: ghost
  display_name: Ghost
  unit_class: CoreLineInfantry
  faction_id: test
  tier: 1
  stats:
    hp: 0
    damage: 15
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeFalse(
                "JsonGuard should reject the hp<=0 unit before registration");
            result.Errors.Should().Contain(e => e.Contains("hp"));
            _registries.Units.Contains("ghost").Should().BeFalse();
        }

        // ── Single-item path: blank id is rejected ───────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void RegisterFromYaml_SingleItem_RejectsBlankId()
        {
            // Arrange — single (non-list) unit YAML with blank id.
            string packDir = CreatePackDirectory("single-blank-id-pack", @"
id: single-blank-id-pack
name: Single Blank Id Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");
            CreateContentFile(packDir, "units", "knight.yaml", @"
id: ''
display_name: Anonymous Knight
unit_class: HeavyInfantry
faction_id: test
tier: 2
stats:
  hp: 200
  damage: 25
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("id"));
        }

        // ── Building path: negative health is rejected ───────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void RegisterFromYaml_RejectsBuildingsWithNegativeHealth()
        {
            // Arrange — building YAML with negative health (impossible value).
            string packDir = CreatePackDirectory("bad-building-pack", @"
id: bad-building-pack
name: Bad Building Pack
version: 0.1.0
author: Test
type: content
loads:
  buildings:
    - buildings
");
            CreateContentFile(packDir, "buildings", "wall.yaml", @"
- id: broken-wall
  display_name: Broken Wall
  building_type: defense
  health: -5
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("health"));
            _registries.Buildings.Contains("broken-wall").Should().BeFalse();
        }

        // ── Direct unit-level Validate() — pins the IValidatable contract ─────

        [Fact]
        [Trait("Category", "Validation")]
        public void UnitDefinition_Validate_RejectsBlankId()
        {
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "Test",
                Stats = new UnitStats { Hp = 100 }
            };

            var result = unit.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UnitDefinition_Validate_RejectsAccuracyOutOfRange()
        {
            var unit = new UnitDefinition
            {
                Id = "test",
                DisplayName = "Test",
                FactionId = "test-faction",
                Stats = new UnitStats { Hp = 100, Accuracy = 1.5f }
            };

            var result = unit.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "stats.accuracy");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void BuildingDefinition_Validate_RejectsBlankId()
        {
            var building = new BuildingDefinition
            {
                Id = "",
                DisplayName = "Test",
                Health = 100
            };

            var result = building.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void BuildingDefinition_Validate_RejectsNegativeHealth()
        {
            var building = new BuildingDefinition
            {
                Id = "test",
                DisplayName = "Test",
                Health = -10
            };

            var result = building.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "health");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void UnitDefinition_Validate_AcceptsValidUnit()
        {
            var unit = new UnitDefinition
            {
                Id = "valid-soldier",
                DisplayName = "Valid Soldier",
                UnitClass = "CoreLineInfantry",
                FactionId = "test",
                Stats = new UnitStats
                {
                    Hp = 100,
                    Damage = 15,
                    Accuracy = 0.8f,
                    FireRate = 1.2f
                }
            };

            var result = unit.Validate();

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void BuildingDefinition_Validate_AcceptsValidBuilding()
        {
            var building = new BuildingDefinition
            {
                Id = "valid-tower",
                DisplayName = "Valid Tower",
                Health = 200
            };

            var result = building.Validate();

            result.IsValid.Should().BeTrue();
        }

        // ── Phase 2b: remaining 6 ContentRegistrationService types (#240) ────

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionDefinition_Validate_RejectsBlankId()
        {
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo
                {
                    Id = "",
                    DisplayName = "Republic"
                }
            };

            var result = faction.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "faction.id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionDefinition_Validate_RejectsMalformedHexColor()
        {
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "republic", DisplayName = "Republic" },
                Visuals = new FactionVisuals { PrimaryColor = "not-a-hex" }
            };

            var result = faction.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "visuals.primary_color");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionDefinition_Validate_AcceptsValidFaction()
        {
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "republic", DisplayName = "Republic" },
                Visuals = new FactionVisuals { PrimaryColor = "#F5F5F5", AccentColor = "#1A3A6B" }
            };

            var result = faction.Validate();

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void WeaponDefinition_Validate_RejectsBlankId()
        {
            var weapon = new WeaponDefinition
            {
                Id = "",
                DisplayName = "Blaster",
                BaseDamage = 10f,
                Range = 5f
            };

            var result = weapon.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void WeaponDefinition_Validate_RejectsNegativeDamage()
        {
            var weapon = new WeaponDefinition
            {
                Id = "blaster",
                DisplayName = "Blaster",
                BaseDamage = -1f
            };

            var result = weapon.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "base_damage");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ProjectileDefinition_Validate_RejectsBlankId()
        {
            var projectile = new ProjectileDefinition
            {
                Id = "",
                DisplayName = "Bolt",
                Speed = 50f
            };

            var result = projectile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ProjectileDefinition_Validate_RejectsNegativeSpeed()
        {
            var projectile = new ProjectileDefinition
            {
                Id = "bolt",
                DisplayName = "Bolt",
                Speed = -10f
            };

            var result = projectile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "speed");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void DoctrineDefinition_Validate_RejectsBlankId()
        {
            var doctrine = new DoctrineDefinition
            {
                Id = "",
                DisplayName = "Elite Discipline"
            };

            var result = doctrine.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void DoctrineDefinition_Validate_RejectsNegativeModifierValue()
        {
            var doctrine = new DoctrineDefinition
            {
                Id = "test",
                DisplayName = "Test Doctrine",
                Modifiers = new Dictionary<string, float>
                {
                    { "attack_speed", -0.5f }
                }
            };

            var result = doctrine.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "modifiers.attack_speed");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void StatOverrideDefinition_Validate_RejectsEmptyOverrides()
        {
            var statOverride = new StatOverrideDefinition
            {
                Overrides = new List<StatOverrideEntry>()
            };

            var result = statOverride.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "overrides");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void StatOverrideDefinition_Validate_RejectsBlankTargetAndUnknownMode()
        {
            var statOverride = new StatOverrideDefinition
            {
                Overrides = new List<StatOverrideEntry>
                {
                    new StatOverrideEntry { Target = "", Value = 1f, Mode = (StatOverrideMode)999 }
                }
            };

            var result = statOverride.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "overrides[0].target");
            result.Errors.Should().Contain(e => e.Path == "overrides[0].mode");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void StatOverrideDefinition_Validate_AcceptsKnownMode()
        {
            var statOverride = new StatOverrideDefinition
            {
                Overrides = new List<StatOverrideEntry>
                {
                    new StatOverrideEntry { Target = "unit.stats.hp", Value = 200f, Mode = StatOverrideMode.Multiply }
                }
            };

            var result = statOverride.Validate();

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionPatchDefinition_Validate_RejectsBlankTargetFaction()
        {
            var patch = new FactionPatchDefinition
            {
                TargetFaction = "",
                Add = new FactionPatchAdditions { Units = new List<string> { "soldier" } }
            };

            var result = patch.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "target_faction");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionPatchDefinition_Validate_RejectsEmptyAdditions()
        {
            var patch = new FactionPatchDefinition
            {
                TargetFaction = "player"
            };

            var result = patch.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "add");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void FactionPatchDefinition_Validate_AcceptsValidPatch()
        {
            var patch = new FactionPatchDefinition
            {
                TargetFaction = "player",
                Add = new FactionPatchAdditions
                {
                    Units = new List<string> { "republic-clone-trooper" }
                }
            };

            var result = patch.Validate();

            result.IsValid.Should().BeTrue();
        }

        // ── helpers ──────────────────────────────────────────────────────────

        private string CreatePackDirectory(string name, string manifestYaml)
        {
            string dir = Path.Combine(_tempRoot, name);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, "pack.yaml"), manifestYaml);
            return dir;
        }

        private void CreateContentFile(
            string packDir,
            string subDir,
            string fileName,
            string yamlContent)
        {
            string dir = Path.Combine(packDir, subDir);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, fileName), yamlContent);
        }
    }
}
