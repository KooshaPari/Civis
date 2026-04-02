#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Assets;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Coverage gap tests for SDK components with lowest coverage.
/// Targets: FileDiscoveryService, AddressablesCatalog error paths, RegistryImportService edge cases.
/// </summary>
public class SDKCovGapTests : IDisposable
{
    private readonly string _tempDir;

    public SDKCovGapTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_sdk_gap_tests_" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
        try
        {
            if (Directory.Exists(_tempDir))
                Directory.Delete(_tempDir, true);
        }
        catch { /* best-effort cleanup */ }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // FileDiscoveryService Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class FileDiscoveryServiceTests
    {
        [Fact]
        public void GetFiles_WithNonExistentRoot_ReturnsEmptyList()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string nonexistentDir = Path.Combine(Path.GetTempPath(), "definitely_does_not_exist_12345");

            // Act
            var result = service.GetFiles(nonexistentDir, "*.yaml", SearchOption.AllDirectories);

            // Assert
            result.Should().BeEmpty("non-existent directory should return no files");
        }

        [Fact]
        public void GetFiles_WithEmptyDirectory_ReturnsEmptyList()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string emptyDir = Path.Combine(Path.GetTempPath(), "empty_dir_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(emptyDir);

            try
            {
                // Act
                var result = service.GetFiles(emptyDir, "*.yaml", SearchOption.AllDirectories);

                // Assert
                result.Should().BeEmpty("empty directory should return no files");
            }
            finally
            {
                Directory.Delete(emptyDir);
            }
        }

        [Fact]
        public void GetFiles_WithNoMatchingPattern_ReturnsEmptyList()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string testDir = Path.Combine(Path.GetTempPath(), "test_dir_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testDir);
            File.WriteAllText(Path.Combine(testDir, "readme.txt"), "Hello");

            try
            {
                // Act
                var result = service.GetFiles(testDir, "*.yaml", SearchOption.AllDirectories);

                // Assert
                result.Should().BeEmpty("no yaml files should return empty list");
            }
            finally
            {
                Directory.Delete(testDir, true);
            }
        }

        [Fact]
        public void GetFiles_WithTopDirectoryOnly_ReturnsMatchingFiles()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string testDir = Path.Combine(Path.GetTempPath(), "test_dir_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testDir);
            string yamlFile = Path.Combine(testDir, "pack.yaml");
            File.WriteAllText(yamlFile, "id: test");

            try
            {
                // Act
                var result = service.GetFiles(testDir, "*.yaml", SearchOption.TopDirectoryOnly);

                // Assert
                result.Should().HaveCount(1);
                result[0].Should().EndWith("pack.yaml");
            }
            finally
            {
                Directory.Delete(testDir, true);
            }
        }

        [Fact]
        public void GetFiles_WithRecursiveSearch_ReturnsNestedFiles()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string testDir = Path.Combine(Path.GetTempPath(), "test_dir_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testDir);
            Directory.CreateDirectory(Path.Combine(testDir, "subdir"));
            string yamlFile1 = Path.Combine(testDir, "root.yaml");
            string yamlFile2 = Path.Combine(testDir, "subdir", "nested.yaml");
            File.WriteAllText(yamlFile1, "id: root");
            File.WriteAllText(yamlFile2, "id: nested");

            try
            {
                // Act
                var result = service.GetFiles(testDir, "*.yaml", SearchOption.AllDirectories);

                // Assert
                result.Should().HaveCount(2);
            }
            finally
            {
                Directory.Delete(testDir, true);
            }
        }

        [Fact]
        public void FileDiscoveryService_WithNoDefaults_CanBeInstantiated()
        {
            // Arrange - no defaults mode
            var service = new FileDiscoveryService(useDefaults: false);

            // Assert
            service.Should().NotBeNull();
        }

        [Fact]
        public void FileDiscoveryService_WithCustomExclusions_CanBeInstantiated()
        {
            // Arrange - custom exclusions mode
            var service = new FileDiscoveryService(new[] { "custom_exclusion" });

            // Assert
            service.Should().NotBeNull();
            service.DefaultExclusions.Should().NotBeEmpty();
        }

        [Fact]
        public void GetDirectories_WithNonExistentRoot_ReturnsEmptyList()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string nonexistentDir = Path.Combine(Path.GetTempPath(), "definitely_does_not_exist_xyz789");

            // Act
            var result = service.GetDirectories(nonexistentDir, SearchOption.AllDirectories);

            // Assert
            result.Should().BeEmpty("non-existent directory should return no directories");
        }

        [Fact]
        public void GetDirectories_WithDirectoryStructure_ReturnsSubdirectories()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string testDir = Path.Combine(Path.GetTempPath(), "test_dir_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(testDir);
            Directory.CreateDirectory(Path.Combine(testDir, "subdir1"));
            Directory.CreateDirectory(Path.Combine(testDir, "subdir2"));

            try
            {
                // Act
                var result = service.GetDirectories(testDir, SearchOption.TopDirectoryOnly);

                // Assert
                result.Should().HaveCount(2);
            }
            finally
            {
                Directory.Delete(testDir, true);
            }
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // AddressablesCatalog Error Path Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class AddressablesCatalogErrorPathTests
    {
        private readonly string _tempDir;

        public AddressablesCatalogErrorPathTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_catalog_err_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        [Fact]
        public void Load_WithNullInternalIdEntry_HandlesGracefully()
        {
            // Arrange — JSON with null entry in m_InternalIds
            string catalogPath = Path.Combine(_tempDir, "null_entry.json");
            string json = @"{""m_InternalIds"":[""valid_entry"", null, ""another_valid""]}";
            File.WriteAllText(catalogPath, json);

            // Act - should not throw
            Action act = () => AddressablesCatalog.Load(catalogPath);

            // Assert
            act.Should().NotThrow("null entries should be handled gracefully");
        }

        [Fact]
        public void Load_WithMalformedJson_ThrowsJsonException()
        {
            // Arrange
            string catalogPath = Path.Combine(_tempDir, "malformed.json");
            File.WriteAllText(catalogPath, "{ invalid json }");

            // Act & Assert
            Action act = () => AddressablesCatalog.Load(catalogPath);
            act.Should().Throw<Exception>("malformed JSON should throw an exception");
        }

        [Fact]
        public void Load_WithCaseInsensitiveBundleExtension_MatchesBundles()
        {
            // Arrange — .BUNDLE and .Bundle should also be detected
            string catalogPath = Path.Combine(_tempDir, "case_insensitive.json");
            string json = @"{""m_InternalIds"":[""Assets/test.prefab"", ""path/to/asset.BUNDLE"", ""path/to/other.Bundle""]}";
            File.WriteAllText(catalogPath, json);

            // Act
            var catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.BundlePaths.Should().HaveCount(2, "case-insensitive .bundle matching should work");
        }

        [Fact]
        public void ResolveBundlePath_WithNullGameDir_ThrowsArgumentNullException()
        {
            // Arrange
            string bundlePath = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/test.bundle";
            string? gameDir = null;

            // Act & Assert - null gameDir should throw ArgumentNullException
            Action act = () => AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir!);
            act.Should().Throw<ArgumentNullException>("null game directory should throw ArgumentNullException");
        }

        [Fact]
        public void ResolveBundlePath_WithEmptyGameDir_HandlesGracefully()
        {
            // Arrange
            string bundlePath = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/test.bundle";
            string gameDir = "";

            // Act
            string resolved = AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir);

            // Assert - should not crash with empty string
            resolved.Should().NotBeNull();
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // RegistryImportService Error Path Tests
    // ═════════════════════════════════════════════════════════════════════════════

    public class RegistryImportServiceTests
    {
        private readonly string _tempDir;

        public RegistryImportServiceTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_reg_import_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        [Fact]
        public void RegistryManager_Units_Property_ReturnsUnitRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var unitRegistry = manager.Units;

            // Assert
            unitRegistry.Should().NotBeNull("RegistryManager should always return a unit registry");
        }

        [Fact]
        public void RegistryManager_Buildings_Property_ReturnsBuildingRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var buildingRegistry = manager.Buildings;

            // Assert
            buildingRegistry.Should().NotBeNull("RegistryManager should always return a building registry");
        }

        [Fact]
        public void RegistryManager_Factions_Property_ReturnsFactionRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var factionRegistry = manager.Factions;

            // Assert
            factionRegistry.Should().NotBeNull("RegistryManager should always return a faction registry");
        }

        [Fact]
        public void RegistryManager_Doctrines_Property_ReturnsDoctrineRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var doctrineRegistry = manager.Doctrines;

            // Assert
            doctrineRegistry.Should().NotBeNull("RegistryManager should always return a doctrine registry");
        }

        [Fact]
        public void RegistryManager_Waves_Property_ReturnsWaveRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var waveRegistry = manager.Waves;

            // Assert
            waveRegistry.Should().NotBeNull("RegistryManager should always return a wave registry");
        }

        [Fact]
        public void RegistryManager_Weapons_Property_ReturnsWeaponRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var weaponRegistry = manager.Weapons;

            // Assert
            weaponRegistry.Should().NotBeNull("RegistryManager should always return a weapon registry");
        }

        [Fact]
        public void RegistryManager_Projectiles_Property_ReturnsProjectileRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var projectileRegistry = manager.Projectiles;

            // Assert
            projectileRegistry.Should().NotBeNull("RegistryManager should always return a projectile registry");
        }

        [Fact]
        public void RegistryManager_Skills_Property_ReturnsSkillRegistry()
        {
            // Arrange
            var manager = new RegistryManager();

            // Act
            var skillRegistry = manager.Skills;

            // Assert
            skillRegistry.Should().NotBeNull("RegistryManager should always return a skill registry");
        }
    }

}
