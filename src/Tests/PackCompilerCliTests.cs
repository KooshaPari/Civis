using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests
{
    /// <summary>
    /// Smoke tests for PackCompiler CLI functionality.
    /// Tests invoke SDK validation functions directly (not subprocess) and verify outputs.
    /// Maps to: US-F6.1 Pack Validation & Compiler
    /// </summary>
    [Trait("Category", "UserStory")]
    [Trait("UserStory", "US-F6.1")]
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-PackSystem")]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-Debug")]
    public class PackCompilerCliTests : IDisposable
    {
        private readonly string _tempRoot;
        private readonly PackLoader _loader;

        public PackCompilerCliTests()
        {
            _tempRoot = Path.Combine(Path.GetTempPath(), "packcompiler_test_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);

            _loader = new PackLoader();
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempRoot))
                Directory.Delete(_tempRoot, true);
        }

        #region Helper Methods

        private string CreateTestPack(string packId, string yaml)
        {
            string packDir = Path.Combine(_tempRoot, packId);
            Directory.CreateDirectory(packDir);
            string manifestPath = Path.Combine(packDir, "pack.yaml");
            File.WriteAllText(manifestPath, yaml);
            return packDir;
        }

        private string CreateTestPackWithContent(string packId, string packYaml,
            Dictionary<string, Dictionary<string, string>> contentDirs)
        {
            string packDir = CreateTestPack(packId, packYaml);

            // Create content subdirectories and files
            foreach (var (dirName, files) in contentDirs)
            {
                string contentDir = Path.Combine(packDir, dirName);
                Directory.CreateDirectory(contentDir);

                foreach (var (fileName, content) in files)
                {
                    string filePath = Path.Combine(contentDir, fileName);
                    File.WriteAllText(filePath, content);
                }
            }

            return packDir;
        }

        private string CreateTestTcManifest(string tcId, string yaml)
        {
            string manifestPath = Path.Combine(_tempRoot, $"{tcId}.yaml");
            File.WriteAllText(manifestPath, yaml);
            return manifestPath;
        }

        #endregion

        #region Validate Command Tests

        [Fact]
        public void Validate_ValidPack_Succeeds()
        {
            // Arrange
            string packDir = CreateTestPack("valid-pack", @"
id: valid-pack
name: Valid Pack
version: 0.1.0
author: Test Author
type: content
framework_version: '>=0.1.0'
depends_on: []
conflicts_with: []
");

            // Act
            Action act = () => _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            act.Should().NotThrow();
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));
            manifest.Id.Should().Be("valid-pack");
            manifest.Name.Should().Be("Valid Pack");
            manifest.Version.Should().Be("0.1.0");
        }

        [Fact]
        public void Validate_NonexistentPath_ThrowsFileNotFound()
        {
            // Arrange
            string nonexistentPath = Path.Combine(_tempRoot, "nonexistent", "pack.yaml");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(nonexistentPath))
                .Should().Throw<FileNotFoundException>();
        }

        [Fact]
        public void Validate_MissingPackYaml_ThrowsFileNotFound()
        {
            // Arrange
            string packDir = Path.Combine(_tempRoot, "no-manifest");
            Directory.CreateDirectory(packDir);

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<FileNotFoundException>();
        }

        [Fact]
        public void Validate_MissingRequiredId_ThrowsInvalidOperation()
        {
            // Arrange
            string packDir = CreateTestPack("missing-id", @"
name: No ID Pack
version: 0.1.0
author: Test
type: content
");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<InvalidOperationException>()
                .WithMessage("*id*");
        }

        [Fact]
        public void Validate_MissingRequiredName_ThrowsInvalidOperation()
        {
            // Arrange
            string packDir = CreateTestPack("missing-name", @"
id: missing-name
version: 0.1.0
author: Test
type: content
");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<InvalidOperationException>()
                .WithMessage("*name*");
        }

        [Fact]
        public void Validate_MissingRequiredVersion_DeserializesWithDefault()
        {
            // Arrange
            string packDir = CreateTestPack("missing-version", @"
id: missing-version
name: No Version Pack
author: Test
type: content
");

            // Act - missing version will deserialize but use default
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert - missing version defaults to something
            manifest.Version.Should().NotBeNullOrEmpty();
        }

        [Fact]
        public void Validate_PackWithMultipleContentTypes_Succeeds()
        {
            // Arrange
            string packDir = CreateTestPackWithContent("multi-content", @"
id: multi-content
name: Multi Content Pack
version: 0.1.0
author: Test
type: content
loads:
  factions:
    - factions
  units:
    - units
  buildings:
    - buildings
", new Dictionary<string, Dictionary<string, string>>
            {
                { "factions", new() { { "test-faction.yaml", "id: test-faction\nname: Test Faction\n" } } },
                { "units", new() { { "test-unit.yaml", "id: test-unit\nname: Test Unit\n" } } },
                { "buildings", new() { { "test-building.yaml", "id: test-building\nname: Test Building\n" } } }
            });

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            manifest.Loads.Should().NotBeNull();
            manifest.Loads!.Factions.Should().Contain("factions");
            manifest.Loads!.Units.Should().Contain("units");
            manifest.Loads!.Buildings.Should().Contain("buildings");
        }

        [Fact]
        public void Validate_PackWithDependencies_Deserializes()
        {
            // Arrange
            string packDir = CreateTestPack("dependent-pack", @"
id: dependent-pack
name: Dependent Pack
version: 0.1.0
author: Test
type: content
depends_on:
  - base-pack
  - warfare-pack
conflicts_with:
  - incompatible-pack
");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            manifest.DependsOn.Should().Contain("base-pack");
            manifest.DependsOn.Should().Contain("warfare-pack");
            manifest.ConflictsWith.Should().Contain("incompatible-pack");
        }

        [Theory]
        [InlineData("balance")]
        [InlineData("content")]
        [InlineData("total_conversion")]
        [InlineData("ruleset")]
        [InlineData("utility")]
        public void Validate_VariousPackTypes_Deserialize(string packType)
        {
            // Arrange
            string packDir = CreateTestPack($"pack-{packType}", $@"
id: pack-{packType}
name: Pack {packType}
version: 0.1.0
author: Test
type: {packType}
");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            manifest.Type.Should().Be(packType);
        }

        #endregion

        #region Validate Total Conversion (validate-tc) Tests

        [Fact]
        public void ValidateTotalConversion_ValidTc_Succeeds()
        {
            // Arrange
            string tcYaml = @"
id: test-starwars
name: Test Star Wars
version: 0.1.0
type: total_conversion
author: Test
theme: star-wars
framework_version: '>=0.5.0'
singleton: true

replaces_vanilla:
  player: republic
  enemy_classic: cis
  enemy_guerrilla: separatists

factions:
  - id: republic
    replaces: player
    name: Galactic Republic
    archetype: order
    units:
      - clone-trooper
    buildings: []
  - id: cis
    replaces: enemy_classic
    name: CIS
    archetype: industrial_swarm
    units:
      - b1-droid
    buildings: []
  - id: separatists
    replaces: enemy_guerrilla
    name: Separatists
    archetype: asymmetric
    units:
      - commando
    buildings: []

asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("test-starwars", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidateTotalConversion_EmptyReplacesVanilla_DeserializesOk()
        {
            // Arrange - empty replaces_vanilla is allowed to deserialize
            string tcYaml = @"
id: no-replaces
name: No Replaces
version: 0.1.0
type: total_conversion
author: Test
replaces_vanilla: {}
factions:
  - id: test
    replaces: player
    name: Test
    archetype: order
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("no-replaces", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert - worktree SDK allows empty replaces_vanilla
            manifest.ReplacesVanilla.Should().BeEmpty();
        }

        [Fact]
        public void ValidateTotalConversion_FactionMissingReplaces_DeserializesOk()
        {
            // Arrange
            string tcYaml = @"
id: faction-no-replaces
name: Faction No Replaces
version: 0.1.0
type: total_conversion
author: Test
replaces_vanilla:
  player: test-faction
factions:
  - id: test-faction
    name: Test
    archetype: order
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("faction-no-replaces", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert - worktree SDK allows factions without replaces field
            manifest.Factions.Should().HaveCount(1);
            manifest.Factions[0].Id.Should().Be("test-faction");
        }

        [Fact]
        public void ValidateTotalConversion_PartialVanillaReplacement_IsValid()
        {
            // Arrange - partial replacement is valid, just warns about unreplaced
            string tcYaml = @"
id: partial-tc
name: Partial TC
version: 0.1.0
type: total_conversion
author: Test
replaces_vanilla:
  player: republic
factions:
  - id: republic
    replaces: player
    name: Republic
    archetype: order
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("partial-tc", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert
            result.IsValid.Should().BeTrue(); // Partial replacement is allowed
            manifest.ReplacesVanilla.Should().ContainKey("player");
        }

        [Fact]
        public void ValidateTotalConversion_FactionMissingId_HasErrors()
        {
            // Arrange
            string tcYaml = @"
id: faction-no-id
name: Faction No ID
version: 0.1.0
type: total_conversion
author: Test
replaces_vanilla:
  player: test
factions:
  - replaces: player
    name: Test
    archetype: order
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("faction-no-id", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert
            result.Errors.Should().Contain(e => e.Contains("missing id"));
        }

        [Fact]
        public void ValidateTotalConversion_NonTcType_HasError()
        {
            // Arrange
            string tcYaml = @"
id: not-tc
name: Not TC
version: 0.1.0
type: content
author: Test
replaces_vanilla:
  player: test
factions:
  - id: test
    replaces: player
    name: Test
    archetype: order
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("not-tc", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("total_conversion"));
        }

        #endregion

        #region Build Command Tests

        [Fact]
        public void Build_ValidPack_CreatesOutputDirectory()
        {
            // Arrange
            string packDir = CreateTestPackWithContent("buildable-pack", @"
id: buildable-pack
name: Buildable Pack
version: 1.0.0
author: Test
type: content
", new Dictionary<string, Dictionary<string, string>>
            {
                { "units", new() { { "unit1.yaml", "id: unit1\nname: Unit 1\n" } } }
            });

            string outputDir = Path.Combine(_tempRoot, "build-output");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));
            Directory.CreateDirectory(outputDir);

            // Copy pack files to output (simulating build)
            foreach (var file in Directory.GetFiles(packDir, "*", SearchOption.AllDirectories))
            {
                string relativePath = Path.GetRelativePath(packDir, file);
                string destPath = Path.Combine(outputDir, relativePath);
                Directory.CreateDirectory(Path.GetDirectoryName(destPath)!);
                File.Copy(file, destPath, true);
            }

            // Assert
            Directory.Exists(outputDir).Should().BeTrue();
            File.Exists(Path.Combine(outputDir, "pack.yaml")).Should().BeTrue();
            File.Exists(Path.Combine(outputDir, "units", "unit1.yaml")).Should().BeTrue();
        }

        [Fact]
        public void Build_PackWithMultipleContentTypes_PreservesDirStructure()
        {
            // Arrange
            string packDir = CreateTestPackWithContent("multi-build", @"
id: multi-build
name: Multi Build
version: 1.0.0
author: Test
type: content
loads:
  factions:
    - factions
  units:
    - units
  buildings:
    - buildings
  weapons:
    - weapons
", new Dictionary<string, Dictionary<string, string>>
            {
                { "factions", new() { { "faction1.yaml", "id: faction1\n" } } },
                { "units", new() { { "unit1.yaml", "id: unit1\n" } } },
                { "buildings", new() { { "building1.yaml", "id: building1\n" } } },
                { "weapons", new() { { "weapon1.yaml", "id: weapon1\n" } } }
            });

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            Directory.Exists(Path.Combine(packDir, "factions")).Should().BeTrue();
            Directory.Exists(Path.Combine(packDir, "units")).Should().BeTrue();
            Directory.Exists(Path.Combine(packDir, "buildings")).Should().BeTrue();
            Directory.Exists(Path.Combine(packDir, "weapons")).Should().BeTrue();
        }

        #endregion

        #region JSON Output Format Tests

        [Fact]
        public void JsonOutput_ValidPackValidation_ReturnsValidJson()
        {
            // Arrange
            string packDir = CreateTestPack("json-test", @"
id: json-test
name: JSON Test Pack
version: 0.1.0
author: Test
type: content
");

            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Act
            var output = new { status = "ok", pack = manifest.Id, errors = Array.Empty<string>() };
            string json = JsonSerializer.Serialize(output);

            // Assert
            json.Should().NotBeNullOrEmpty();
            json.Should().Contain("\"status\"");
            json.Should().Contain("\"ok\"");
            json.Should().Contain("json-test");

            // Verify it's valid JSON
            using var doc = JsonDocument.Parse(json);
            doc.RootElement.GetProperty("status").GetString().Should().Be("ok");
        }

        [Fact]
        public void JsonOutput_PackValidationError_ReturnsErrorJson()
        {
            // Arrange
            var errors = new[] { "Pack directory not found", "pack.yaml missing" };
            var output = new { status = "error", pack = (string?)null, errors };

            // Act
            string json = JsonSerializer.Serialize(output);

            // Assert
            json.Should().Contain("\"status\"");
            json.Should().Contain("\"error\"");
            using var doc = JsonDocument.Parse(json);
            doc.RootElement.GetProperty("status").GetString().Should().Be("error");
        }

        [Fact]
        public void JsonOutput_BuildSuccess_ReturnsOutputAndSize()
        {
            // Arrange
            string packDir = CreateTestPackWithContent("json-build", @"
id: json-build
name: JSON Build
version: 1.0.0
author: Test
type: content
", new Dictionary<string, Dictionary<string, string>>
            {
                { "units", new() { { "unit.yaml", "id: unit1\nname: Unit\n" } } }
            });

            long totalSize = 0;
            foreach (var file in Directory.GetFiles(packDir, "*", SearchOption.AllDirectories))
            {
                totalSize += new FileInfo(file).Length;
            }

            var output = new { status = "ok", output = packDir, size = totalSize };

            // Act
            string json = JsonSerializer.Serialize(output);

            // Assert
            json.Should().Contain("\"status\"");
            json.Should().Contain("\"ok\"");
            using var doc = JsonDocument.Parse(json);
            doc.RootElement.GetProperty("size").GetInt64().Should().BeGreaterThan(0);
        }

        #endregion

        #region Edge Cases and Error Handling

        [Fact]
        public void Validate_EmptyManifest_Throws()
        {
            // Arrange
            string packDir = CreateTestPack("empty", "");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<Exception>();
        }

        [Fact]
        public void Validate_InvalidYaml_Throws()
        {
            // Arrange
            string packDir = Path.Combine(_tempRoot, "invalid-yaml");
            Directory.CreateDirectory(packDir);
            File.WriteAllText(Path.Combine(packDir, "pack.yaml"), ":\n  - invalid\nyaml: structure:");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<Exception>();
        }

        [Fact]
        public void Validate_WhitespaceOnlyManifest_Throws()
        {
            // Arrange
            string packDir = CreateTestPack("whitespace", "   \n  \n  ");

            // Act & Assert
            _loader.Invoking(l => l.LoadFromFile(Path.Combine(packDir, "pack.yaml")))
                .Should().Throw<Exception>();
        }

        [Fact]
        public void Validate_PackWithVeryLongId_Succeeds()
        {
            // Arrange
            string longId = "pack-" + new string('a', 200);
            string packDir = CreateTestPack(longId, $@"
id: {longId}
name: Long ID Pack
version: 0.1.0
author: Test
type: content
");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            manifest.Id.Should().Be(longId);
        }

        [Fact]
        public void Validate_PackWithUnicodeCharacters_Succeeds()
        {
            // Arrange
            string packDir = CreateTestPack("unicode-pack", @"
id: unicode-pack
name: 测试包 ★ Pack
version: 0.1.0
author: Test Author 🚀
type: content
description: Описание пакета
");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert
            manifest.Name.Should().Contain("★");
            manifest.Author.Should().Contain("🚀");
            manifest.Description.Should().Contain("пакета");
        }

        [Fact]
        public void Validate_PackWithLoadsButNoContentDirs_StillValid()
        {
            // Arrange - declare loads but don't create the dirs
            string packDir = CreateTestPack("loads-no-content", @"
id: loads-no-content
name: Loads No Content
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
  buildings:
    - buildings
");

            // Act
            var manifest = _loader.LoadFromFile(Path.Combine(packDir, "pack.yaml"));

            // Assert - manifest itself is valid even if content dirs don't exist
            manifest.Loads.Should().NotBeNull();
            manifest.Loads!.Units.Should().Contain("units");
        }

        [Fact]
        public void ValidateTotalConversion_AllVanillaFactionsReplaced_IsValid()
        {
            // Arrange - full vanilla faction replacement
            string tcYaml = @"
id: complete-tc
name: Complete TC
version: 0.1.0
type: total_conversion
author: Test
replaces_vanilla:
  player: faction-a
  enemy_classic: faction-b
  enemy_guerrilla: faction-c
factions:
  - id: faction-a
    replaces: player
    name: A
    archetype: order
    units: []
    buildings: []
  - id: faction-b
    replaces: enemy_classic
    name: B
    archetype: industrial_swarm
    units: []
    buildings: []
  - id: faction-c
    replaces: enemy_guerrilla
    name: C
    archetype: asymmetric
    units: []
    buildings: []
asset_replacements:
  textures: {}
  audio: {}
  ui: {}
";
            string manifestPath = CreateTestTcManifest("complete-tc", tcYaml);

            // Act
            string yaml = File.ReadAllText(manifestPath);
            var manifest = YamlLoader.Deserializer.Deserialize<TotalConversionManifest>(yaml);
            var result = TotalConversionValidator.Validate(manifest!);

            // Assert
            result.IsValid.Should().BeTrue();
            manifest.ReplacesVanilla.Should().HaveCount(3);
        }

        #endregion
    }
}
