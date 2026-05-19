using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.Assets;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="AssetService"/> class.
    /// Tests constructor validation, path handling, and pure-function behavior.
    /// Integration tests (bundle loading, asset extraction) are excluded due to native AssetsTools.NET dependency.
    /// </summary>
    public sealed class AssetServiceUnitTests : IDisposable
    {
        private readonly string _testGameDir;

        public AssetServiceUnitTests()
        {
            _testGameDir = Path.Combine(Path.GetTempPath(), $"AssetServiceTest_{Guid.NewGuid()}");
            Directory.CreateDirectory(_testGameDir);
        }

        [Fact]
        public void Constructor_WithValidGameDir_InitializesService()
        {
            // Arrange, Act & Assert
            using var service = new AssetService(_testGameDir);
            service.Should().NotBeNull("Service should be created successfully");
        }

        [Fact]
        public void Constructor_WithNullGameDir_ThrowsArgumentNullException()
        {
            // Arrange, Act & Assert
            var act = () => new AssetService(null!);
            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("gameDir");
        }

        [Fact]
        public void ExpectedUnityVersion_IsCorrect()
        {
            // Arrange, Act & Assert
            AssetService.ExpectedUnityVersion.Should().Be("2021.3");
        }

        [Fact]
        public void ListBundles_WithNonexistentDirectory_ReturnsEmpty()
        {
            // Arrange
            var nonexistentDir = Path.Combine(_testGameDir, "nonexistent");
            using var service = new AssetService(nonexistentDir);

            // Act
            var bundles = service.ListBundles();

            // Assert
            bundles.Should().BeEmpty("Nonexistent streaming assets directory should return empty list");
        }

        [Fact]
        public void ListBundles_WithEmptyDirectory_ReturnsEmpty()
        {
            // Arrange
            var bundlesDir = Path.Combine(_testGameDir, "Diplomacy is Not an Option_Data", "StreamingAssets", "aa", "StandaloneWindows64");
            Directory.CreateDirectory(bundlesDir);

            using var service = new AssetService(_testGameDir);

            // Act
            var bundles = service.ListBundles();

            // Assert
            bundles.Should().BeEmpty("Empty bundles directory should return empty list");
        }

        [Fact]
        public void ListAssets_WithNonexistentFile_ThrowsFileNotFoundException()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var nonexistentPath = Path.Combine(_testGameDir, "nonexistent.bundle");

            // Act & Assert
            var act = () => service.ListAssets(nonexistentPath);
            act.Should().Throw<FileNotFoundException>()
                .Where(ex => ex.Message.Contains("not found"));
        }

        [Fact]
        public void ReadCatalog_WithNonexistentFile_ReturnsEmptyDictionary()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);

            // Act
            var catalog = service.ReadCatalog();

            // Assert
            catalog.Should().BeEmpty("Nonexistent catalog should return empty dictionary");
        }

        [Fact]
        public void ExtractAsset_WithNonexistentBundle_ReturnsNull()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var nonexistentPath = Path.Combine(_testGameDir, "nonexistent.bundle");

            // Act
            var result = service.ExtractAsset(nonexistentPath, "asset-name");

            // Assert
            result.Should().BeNull("Nonexistent bundle should return null");
        }

        [Fact]
        public void ValidateModBundle_WithNonexistentFile_ReturnsFailure()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var nonexistentPath = Path.Combine(_testGameDir, "nonexistent.bundle");

            // Act
            var result = service.ValidateModBundle(nonexistentPath);

            // Assert
            result.IsValid.Should().BeFalse("Validation should fail for nonexistent file");
            result.Errors.Should().NotBeEmpty();
            result.Errors[0].Should().Contain("not found");
        }

        [Fact]
        public void ValidateModBundle_Result_HasExpectedProperties()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var nonexistentPath = Path.Combine(_testGameDir, "nonexistent.bundle");

            // Act
            var result = service.ValidateModBundle(nonexistentPath);

            // Assert
            result.Should().NotBeNull();
            result.UnityVersion.Should().NotBeNullOrEmpty();
            result.Errors.Should().BeAssignableTo<IReadOnlyList<string>>();
            result.Assets.Should().BeAssignableTo<IReadOnlyList<AssetInfo>>();
        }

        [Fact]
        public void ReplaceAsset_WithNonexistentSourceBundle_ReturnsFalse()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var nonexistentPath = Path.Combine(_testGameDir, "nonexistent.bundle");
            var outputPath = Path.Combine(_testGameDir, "output.bundle");

            // Act
            var result = service.ReplaceAsset(nonexistentPath, "asset-name", new byte[] { 1, 2, 3 }, outputPath);

            // Assert
            result.Should().BeFalse("ReplaceAsset should fail for nonexistent source bundle");
        }

        [Fact]
        public void ReplaceAsset_WithEmptyNewData_ReturnsFalse()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var bundlePath = Path.Combine(_testGameDir, "test.bundle");
            var outputPath = Path.Combine(_testGameDir, "output.bundle");

            // Act
            var result = service.ReplaceAsset(bundlePath, "asset-name", Array.Empty<byte>(), outputPath);

            // Assert
            result.Should().BeFalse("ReplaceAsset should fail with empty data");
        }

        [Fact]
        public void ReplaceAsset_WithNullNewData_ReturnsFalse()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);
            var bundlePath = Path.Combine(_testGameDir, "test.bundle");
            var outputPath = Path.Combine(_testGameDir, "output.bundle");

            // Act
            var result = service.ReplaceAsset(bundlePath, "asset-name", null!, outputPath);

            // Assert
            result.Should().BeFalse("ReplaceAsset should fail with null data");
        }

        [Fact]
        public void FindBundlesWithType_WithNonexistentDirectory_ReturnsEmpty()
        {
            // Arrange
            var nonexistentDir = Path.Combine(_testGameDir, "nonexistent");
            using var service = new AssetService(nonexistentDir);

            // Act
            var bundles = service.FindBundlesWithType("Texture2D");

            // Assert
            bundles.Should().BeEmpty("Search in nonexistent directory should return empty");
        }

        [Fact]
        public void Dispose_CanBeCalledMultipleTimes()
        {
            // Arrange
            using var service = new AssetService(_testGameDir);

            // Act & Assert - should not throw
            service.Dispose();
            service.Dispose(); // Second dispose should be safe
        }

        [Fact]
        public void Constructor_WithDifferentGameDirs_CreatesDistinctServices()
        {
            // Arrange
            var dir2 = Path.Combine(Path.GetTempPath(), $"AssetServiceTest_{Guid.NewGuid()}");
            Directory.CreateDirectory(dir2);

            try
            {
                using var service1 = new AssetService(_testGameDir);
                using var service2 = new AssetService(dir2);

                // Act & Assert
                service1.Should().NotBeSameAs(service2);
            }
            finally
            {
                if (Directory.Exists(dir2))
                {
                    Directory.Delete(dir2, true);
                }
            }
        }

        public void Dispose()
        {
            if (Directory.Exists(_testGameDir))
            {
                try
                {
                    Directory.Delete(_testGameDir, true);
                }
                catch
                {
                    // Ignore cleanup errors in tests
                }
            }
        }
    }
}
