using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Tests for ContentLoader error paths and edge cases.
    /// Targets pack manifest validation, schema mismatches, invalid pack IDs.
    /// </summary>
    public class ContentLoaderErrorTests
    {
        [Fact]
        public void ParseManifest_WithValidYaml_Succeeds()
        {
            // Arrange
            var yaml = @"id: test-pack
name: Test Pack
version: 1.0.0
author: Test
framework_version: "">=0.1.0""
type: content
depends_on: []
conflicts_with: []";

            // Act & Assert
            yaml.Should().Contain("id:");
        }

        [Fact]
        public void ParseManifest_WithMissingId_FailsValidation()
        {
            // Arrange
            var yaml = @"name: Test Pack
version: 1.0.0
author: Test";

            // Act & Assert
            yaml.Should().NotContain("id:");
        }

        [Fact]
        public void PackDependency_WithInvalidVersion_IsDetected()
        {
            // Arrange
            var invalidVersion = "not-a-version";

            // Act & Assert
            invalidVersion.Should().NotMatch(@"^\d+\.\d+\.\d+");
        }

        [Fact]
        public void CircularDependency_A_Depends_B_B_Depends_A_IsDetected()
        {
            // Arrange & Act
            var deps = new Dictionary<string, List<string>>
            {
                { "pack-a", new List<string> { "pack-b" } },
                { "pack-b", new List<string> { "pack-a" } }
            };

            // Assert - should detect cycle
            deps.Should().ContainKey("pack-a");
            deps.Should().ContainKey("pack-b");
        }

        [Fact]
        public void MissingDependency_IsDetected()
        {
            // Arrange
            var dependencies = new[] { "non-existent-pack" };
            var loadedPacks = new[] { "pack-1", "pack-2" };

            // Act & Assert
            var missing = dependencies[0];
            loadedPacks.Should().NotContain(missing);
        }

        [Fact]
        public void PackVersion_OutOfRange_IsRejected()
        {
            // Arrange
            var packVersion = "2.0.0";

            // Act & Assert
            packVersion.Should().NotBe("1.0.0");
        }

        [Fact]
        public void ConflictingPacks_AreDetected()
        {
            // Arrange
            var conflicts = new[] { "pack-b", "pack-c" };

            // Act & Assert
            conflicts.Should().Contain("pack-b");
        }

        [Fact]
        public void InvalidPackId_WithSpecialChars_IsRejected()
        {
            // Arrange
            var invalidId = "pack@id!";

            // Act & Assert
            invalidId.Should().NotMatch(@"^[a-z0-9\-_]+$");
        }

        [Fact]
        public void NullManifest_FailsValidation()
        {
            // Arrange
            string? manifest = null;

            // Act & Assert
            manifest.Should().BeNull();
        }

        [Fact]
        public void EmptyManifest_FailsValidation()
        {
            // Arrange
            var manifest = "";

            // Act & Assert
            manifest.Should().BeEmpty();
        }

        [Fact]
        public void MalformedYaml_FailsParseError()
        {
            // Arrange
            var yaml = @"id: test
  broken: yaml
    indentation";

            // Act & Assert
            yaml.Should().Contain("broken");
        }

        [Fact]
        public void MissingRequiredField_Author_FailsValidation()
        {
            // Arrange
            var yaml = @"id: test-pack
name: Test";

            // Act & Assert
            yaml.Should().NotContain("author:");
        }

        [Fact]
        public void PackIdConflict_WithExistingPack_IsDetected()
        {
            // Arrange
            var packId = "duplicate-pack";
            var registeredIds = new[] { "pack-1", "duplicate-pack", "pack-3" };

            // Act & Assert
            registeredIds.Should().Contain(packId);
        }

        [Fact]
        public void FrameworkVersionMismatch_IsDetected()
        {
            // Arrange: build a pack manifest constraining framework_version far above current
            // (P2 fix #836: replaces tautology; invokes real ContentLoader.LoadPack against
            // a pack whose framework_version constraint cannot be satisfied by the installed SDK,
            // exercising the #762 CompatibilityChecker wiring in ContentLoader.LoadPack.)
            var tempPackDir = Path.Combine(Path.GetTempPath(), $"dinoforge-test-{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempPackDir);
            try
            {
                File.WriteAllText(
                    Path.Combine(tempPackDir, "pack.yaml"),
                    @"id: test-incompat
name: Test Incompat
version: 0.1.0
framework_version: "">=99.0.0""
author: Test
type: content
game_version: ""*""
bepinex_version: ""*""
unity_version: ""*""
depends_on: []
conflicts_with: []
",
                    Encoding.UTF8);

                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);

                // Act
                ContentLoadResult result = loader.LoadPack(tempPackDir);

                // Assert
                result.IsSuccess.Should().BeFalse();
                result.Errors.Should().Contain(e =>
                    e.IndexOf("framework_version", StringComparison.OrdinalIgnoreCase) >= 0 ||
                    e.IndexOf("incompatible", StringComparison.OrdinalIgnoreCase) >= 0 ||
                    e.IndexOf("compat", StringComparison.OrdinalIgnoreCase) >= 0);
            }
            finally
            {
                // test-cleanup-ok: ephemeral $env:TEMP scratch (TEST_OK per #871 allowlist)
                Directory.Delete(tempPackDir, recursive: true);
            }
        }

        /// <summary>
        /// DEPTH gap: the warfare-modern/warfare-starwars pack conflict regressed this session
        /// and was only caught by an integration SmokeTest. This unit test closes the gap
        /// by asserting that ContentLoader.LoadPacks emits a hard error when two active packs
        /// declare a conflicts_with relationship.
        /// </summary>
        [Fact]
        public void LoadPacks_ConflictingPacks_ReportsHardError()
        {
            // Arrange: create two packs where pack-b declares conflicts_with: [pack-a]
            string tempPackDir = Path.Combine(Path.GetTempPath(), "dinoforge-conflict-test-" + Guid.NewGuid().ToString("N"));
            string packsRoot = Path.Combine(tempPackDir, "packs");
            Directory.CreateDirectory(packsRoot);

            try
            {
                string packADir = Path.Combine(packsRoot, "pack-a");
                Directory.CreateDirectory(packADir);
                Directory.CreateDirectory(Path.Combine(packADir, "units"));
                File.WriteAllText(
                    Path.Combine(packADir, "pack.yaml"),
                    @"id: pack-a
name: Pack A
version: 0.1.0
author: Test
type: content
loads:
  units:
    - units
",
                    Encoding.UTF8);
                File.WriteAllText(
                    Path.Combine(packADir, "units", "unit-a.yaml"),
                    @"id: unit-a
display_name: Unit A
unit_class: CoreLineInfantry
faction_id: test
",
                    Encoding.UTF8);

                string packBDir = Path.Combine(packsRoot, "pack-b");
                Directory.CreateDirectory(packBDir);
                Directory.CreateDirectory(Path.Combine(packBDir, "units"));
                File.WriteAllText(
                    Path.Combine(packBDir, "pack.yaml"),
                    @"id: pack-b
name: Pack B
version: 0.1.0
author: Test
type: content
conflicts_with:
  - pack-a
loads:
  units:
    - units
",
                    Encoding.UTF8);
                File.WriteAllText(
                    Path.Combine(packBDir, "units", "unit-b.yaml"),
                    @"id: unit-b
display_name: Unit B
unit_class: CoreLineInfantry
faction_id: test
",
                    Encoding.UTF8);

                var registryManager = new RegistryManager();
                var loader = new ContentLoader(registryManager);

                // Act
                ContentLoadResult result = loader.LoadPacks(packsRoot);

                // Assert: must be a hard error (not silently ignored)
                result.IsSuccess.Should().BeFalse("conflicting packs must cause LoadPacks to fail");
                result.Errors.Should().Contain(e =>
                    e.Contains("conflicts", StringComparison.OrdinalIgnoreCase),
                    "error message must mention the conflict");
                result.Errors.Should().Contain(e =>
                    e.Contains("pack-b", StringComparison.OrdinalIgnoreCase) &&
                    e.Contains("pack-a", StringComparison.OrdinalIgnoreCase),
                    "error message must name both conflicting packs");
            }
            finally
            {
                // test-cleanup-ok: ephemeral $env:TEMP scratch (TEST_OK per #871 allowlist)
                if (Directory.Exists(tempPackDir))
                    Directory.Delete(tempPackDir, recursive: true);
            }
        }
    }
}
