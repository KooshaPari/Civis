using System;
using System.IO;
using System.Text;
using DINOForge.SDK.Assets;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="AddressablesCatalog"/> covering path resolution, loading,
    /// and parsing behaviour.
    /// </summary>
    public class AddressablesCatalogTests : IDisposable
    {
        private readonly string _tempDir;

        public AddressablesCatalogTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "dinoforge_catalog_tests_" + Guid.NewGuid().ToString("N"));
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

        [Fact]
        public void Load_NonExistentPath_ThrowsFileNotFoundException()
        {
            // Arrange
            string nonExistentPath = Path.Combine(_tempDir, "does_not_exist.json");

            // Act
            Action act = () => AddressablesCatalog.Load(nonExistentPath);

            // Assert
            act.Should().Throw<FileNotFoundException>();
        }

        [Fact]
        public void Load_EmptyJson_ThrowsInvalidOperationException()
        {
            // Arrange — valid JSON but missing m_InternalIds
            string catalogPath = Path.Combine(_tempDir, "empty_catalog.json");
            File.WriteAllText(catalogPath, "{}");

            // Act
            Action act = () => AddressablesCatalog.Load(catalogPath);

            // Assert
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*m_InternalIds*");
        }

        [Fact]
        public void Load_ValidCatalogWithBundlePaths_ParsesBundlePaths()
        {
            // Arrange — minimal valid catalog JSON with two bundle paths and one asset key
            string catalogPath = WriteCatalogJson(new[]
            {
                "Assets/Prefabs/Units/Swordsman.prefab",
                "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/units_bundle.bundle",
                "{UnityEngine.AddressableAssets.Addressables.RuntimePath}/StandaloneWindows64/buildings_bundle.bundle"
            });

            // Act
            AddressablesCatalog catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.InternalIds.Should().HaveCount(3);
            catalog.BundlePaths.Should().HaveCount(2);
            catalog.BundlePaths.Should().Contain(p => p.EndsWith("units_bundle.bundle"));
            catalog.BundlePaths.Should().Contain(p => p.EndsWith("buildings_bundle.bundle"));
        }

        [Fact]
        public void Load_CatalogWithOnlyNonBundleIds_HasEmptyBundlePaths()
        {
            // Arrange — IDs that are all asset paths (no .bundle extension)
            string catalogPath = WriteCatalogJson(new[]
            {
                "Assets/Prefabs/Units/Swordsman.prefab",
                "Assets/Textures/Diffuse.png",
                "Assets/Audio/Battle.wav"
            });

            // Act
            AddressablesCatalog catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.BundlePaths.Should().BeEmpty();
            catalog.InternalIds.Should().HaveCount(3);
        }

        [Fact]
        public void Load_CatalogWithEmptyInternalIds_ReturnsEmptyCollections()
        {
            // Arrange — catalog with an empty m_InternalIds array
            string catalogPath = Path.Combine(_tempDir, "empty_ids_catalog.json");
            File.WriteAllText(catalogPath, @"{""m_InternalIds"": []}");

            // Act
            AddressablesCatalog catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.InternalIds.Should().BeEmpty();
            catalog.BundlePaths.Should().BeEmpty();
            catalog.KeyToBundleMap.Should().BeEmpty();
        }

        [Fact]
        public void ResolveBundlePath_WithPlaceholder_ReplacesWithStreamingAssetsPath()
        {
            // Arrange
            const string runtimePath = "{UnityEngine.AddressableAssets.Addressables.RuntimePath}";
            string bundlePath = $"{runtimePath}/StandaloneWindows64/units.bundle";
            string gameDir = @"C:\Games\DINO";

            // Act
            string resolved = AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir);

            // Assert
            resolved.Should().Contain("StreamingAssets");
            resolved.Should().Contain("aa");
            resolved.Should().Contain("units.bundle");
            resolved.Should().NotContain(runtimePath);
        }

        [Fact]
        public void ResolveBundlePath_WithoutPlaceholder_ReturnsPathUnchanged()
        {
            // Arrange
            string bundlePath = "/absolute/path/to/units.bundle";
            string gameDir = @"C:\Games\DINO";

            // Act
            string resolved = AddressablesCatalog.ResolveBundlePath(bundlePath, gameDir);

            // Assert
            resolved.Should().Be(bundlePath);
        }

        [Fact]
        public void Load_CatalogWithMixedIds_CorrectlySeparatesBundlesFromAssets()
        {
            // Arrange
            string catalogPath = WriteCatalogJson(new[]
            {
                "Assets/Prefabs/Unit.prefab",
                "my_custom_assets.bundle",
                "Assets/Textures/Tex.png",
                "another.bundle"
            });

            // Act
            AddressablesCatalog catalog = AddressablesCatalog.Load(catalogPath);

            // Assert
            catalog.BundlePaths.Should().HaveCount(2);
            catalog.BundlePaths.Should().Contain("my_custom_assets.bundle");
            catalog.BundlePaths.Should().Contain("another.bundle");
        }

        // ─── helpers ───────────────────────────────────────────────────────────

        /// <summary>
        /// Writes a minimal Addressables-style catalog JSON with the given internal IDs
        /// and returns the full path to the written file.
        /// </summary>
        private string WriteCatalogJson(string[] internalIds)
        {
            StringBuilder sb = new StringBuilder();
            sb.Append(@"{""m_InternalIds"":[");
            for (int i = 0; i < internalIds.Length; i++)
            {
                sb.Append('"');
                sb.Append(internalIds[i].Replace("\\", "\\\\"));
                sb.Append('"');
                if (i < internalIds.Length - 1)
                    sb.Append(',');
            }
            sb.Append("]}");

            string path = Path.Combine(_tempDir, $"catalog_{Guid.NewGuid():N}.json");
            File.WriteAllText(path, sb.ToString());
            return path;
        }
    }
}
