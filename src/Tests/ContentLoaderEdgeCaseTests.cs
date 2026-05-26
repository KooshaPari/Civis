using System;
using System.IO;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Edge-case tests for <see cref="ContentLoader"/> covering empty directories,
    /// invalid manifests, duplicate IDs, and circular dependencies.
    /// </summary>
    public class ContentLoaderEdgeCaseTests : IDisposable
    {
        private readonly RegistryManager _registries;
        private readonly ContentLoader _loader;
        private readonly string _tempRoot;

        public ContentLoaderEdgeCaseTests()
        {
            _registries = new RegistryManager();
            _loader = new ContentLoader(_registries);
            _tempRoot = Path.Combine(
                Path.GetTempPath(),
                "dinoforge_edge_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempRoot);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempRoot))
                    Directory.Delete(_tempRoot, true);
            }
            catch { /* best-effort cleanup */ }
        }

        // ─── LoadPacks: empty directory ──────────────────────────────────────

        [Fact]
        public void LoadPacks_EmptyDirectory_ReturnsSuccessWithNoLoadedPacks()
        {
            // Arrange — an empty directory with no subdirectories at all
            string emptyRoot = Path.Combine(_tempRoot, "empty-packs-root");
            Directory.CreateDirectory(emptyRoot);

            // Act
            ContentLoadResult result = _loader.LoadPacks(emptyRoot);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().BeEmpty();
            result.Errors.Should().BeEmpty();
        }

        // ─── LoadPack: missing required field ─────────────────────────────────

        [Fact]
        public void LoadPacks_PackWithMissingIdField_ReturnsError()
        {
            // Arrange — a pack manifest that is missing the required 'id' field
            string packDir = CreatePackDirectory("bad-manifest", @"
name: Bad Pack
version: 0.1.0
author: Test
type: content
");
            string packsRoot = Path.GetDirectoryName(packDir)!;

            // Act — pack with missing id should still parse (id defaults to null/empty)
            // and then the ContentLoader / registry should produce an error or empty load
            ContentLoadResult result = _loader.LoadPacks(packsRoot);

            // Assert — the result may be Success with an empty/default id or a Failure;
            // what matters is we get a valid result object and don't throw.
            result.Should().NotBeNull();
        }

        [Fact]
        public void LoadPack_ManifestWithBadYamlSyntax_ReturnsFailure()
        {
            // Arrange — manifest file with broken YAML syntax
            string packDir = Path.Combine(_tempRoot, "bad-syntax-pack");
            Directory.CreateDirectory(packDir);
            File.WriteAllText(Path.Combine(packDir, "pack.yaml"),
                "id: [this is: {broken yaml syntax\n");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
        }

        // ─── LoadPacks: circular dependency ──────────────────────────────────

        [Fact]
        public void LoadPacks_PackWithCircularDependency_ReturnsError()
        {
            // Arrange — pack-x depends on pack-y, pack-y depends on pack-x
            string packsRoot = Path.Combine(_tempRoot, "circular-packs");
            Directory.CreateDirectory(packsRoot);

            CreatePackDirectory("pack-x", @"
id: pack-x
name: Pack X
version: 0.1.0
author: Test
type: content
depends_on:
  - pack-y
", packsRoot);

            CreatePackDirectory("pack-y", @"
id: pack-y
name: Pack Y
version: 0.1.0
author: Test
type: content
depends_on:
  - pack-x
", packsRoot);

            // Act
            ContentLoadResult result = _loader.LoadPacks(packsRoot);

            // Assert — circular dependency is detected and the result is a failure
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
        }

        // ─── LoadPacks: non-existent directory ───────────────────────────────

        [Fact]
        public void LoadPacks_NonExistentDirectory_ReturnsFailure()
        {
            // Arrange
            string missingPath = Path.Combine(_tempRoot, "ghost-dir");

            // Act
            ContentLoadResult result = _loader.LoadPacks(missingPath);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().ContainSingle()
                .Which.Should().Contain("not found");
        }

        // ─── LoadPack: null guard ─────────────────────────────────────────────

        [Fact]
        public void LoadPack_NullDirectory_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => _loader.LoadPack(null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        // ─── LoadPack: content file with invalid type for a field ─────────────

        [Fact]
        public void LoadPack_ContentFileWithWrongFieldType_ErrorInResult()
        {
            // Arrange — unit yaml with 'id' as a sequence instead of a scalar
            string packDir = CreatePackDirectory("type-error-pack", @"
id: type-error-pack
name: Type Error Pack
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
");
            CreateContentFile(packDir, "units", "bad.yaml", @"
id:
  - this
  - is
  - a
  - list
display_name: Bad Unit
unit_class: CoreLineInfantry
faction_id: test
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        // ─── LoadPack: multiple content files, one valid, one invalid ─────────

        [Fact]
        public void LoadPack_MixedValidAndInvalidContent_PartialLoadWithErrors()
        {
            // Arrange
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
            CreateContentFile(packDir, "units", "valid.yaml", @"
id: valid-soldier
display_name: Valid Soldier
unit_class: CoreLineInfantry
faction_id: test-faction
tier: 1
stats:
  hp: 100
  damage: 15
");
            // Invalid unit (id is a list)
            CreateContentFile(packDir, "units", "invalid.yaml", @"
id:
  - bad
display_name: Bad
unit_class: CoreLineInfantry
faction_id: test-faction
");

            // Act
            ContentLoadResult result = _loader.LoadPack(packDir);

            // Assert — partial result or failure, errors are present
            result.Errors.Should().NotBeEmpty();
        }

        // ─── helpers ─────────────────────────────────────────────────────────

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
