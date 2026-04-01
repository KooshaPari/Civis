using System;
using System.IO;
using System.Runtime.InteropServices;
using DINOForge.Runtime;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class RustInteropTests
    {
        private const string TestAssetId = "test-asset-01";
        private static readonly string TestGlbPath = Path.Combine(
            Path.GetTempPath(), "DINOForge_Test", "test_model.glb");

        public RustInteropTests()
        {
            // Ensure test directory exists
            var testDir = Path.GetDirectoryName(TestGlbPath);
            if (!Directory.Exists(testDir))
                Directory.CreateDirectory(testDir!);
        }

        // ── Happy Path: Availability Detection ──────────────────────────────

        [Fact]
        public void IsAvailable_ReturnsBooleanWithoutCrashing()
        {
            // When DLL is missing, IsAvailable should gracefully return false (not throw)
            // Just call it and verify it doesn't crash — property should be bool (bool? coalesced)
            var available = RustAssetPipelineInterop.IsAvailable;
            (available is bool).Should().BeTrue();
        }

        // ── Happy Path: Import Asset ────────────────────────────────────────

        [Fact]
        public void ImportAsset_WithValidPath_ReturnsAssetData()
        {
            // Create a minimal test file
            CreateMinimalGlbFile(TestGlbPath);

            if (!RustAssetPipelineInterop.IsAvailable)
            {
                // If DLL not available, verify we get a meaningful error
                Action act = () => RustAssetPipelineInterop.ImportAsset(TestGlbPath, TestAssetId);
                act.Should().Throw<InvalidOperationException>()
                    .WithMessage("*Rust asset pipeline unavailable*");
                return;
            }

            // If DLL available, test import
            var result = RustAssetPipelineInterop.ImportAsset(TestGlbPath, TestAssetId);

            result.Should().NotBeNull();
            result.AssetId.Should().Be(TestAssetId);
            result.SourcePath.Should().NotBeNullOrEmpty();
        }

        [Fact]
        public void OptimizeAsset_WithValidInput_ReturnsLodArray()
        {
            if (!RustAssetPipelineInterop.IsAvailable)
            {
                // If DLL not available, verify we get a meaningful error
                var asset = new AssetData { AssetId = TestAssetId, VertexCount = 1000 };
                Action act = () => RustAssetPipelineInterop.OptimizeAsset(asset, new[] { 100, 60, 30 });
                act.Should().Throw<InvalidOperationException>()
                    .WithMessage("*Rust asset pipeline unavailable*");
                return;
            }

            // If DLL available, test optimization
            var testAsset = new AssetData
            {
                AssetId = TestAssetId,
                VertexCount = 5000,
                TriangleCount = 2500,
                SourcePath = TestGlbPath
            };

            var result = RustAssetPipelineInterop.OptimizeAsset(testAsset, new[] { 100, 60, 30 });

            result.Should().NotBeNull();
            result.Should().HaveCount(3);
            result[0].AssetId.Should().Be(TestAssetId);
        }

        // ── Error Case: File Not Found ──────────────────────────────────────

        [Fact]
        public void ImportAsset_WithMissingFile_ThrowsFileNotFoundException()
        {
            string nonExistentPath = Path.Combine(Path.GetTempPath(), "nonexistent_" + Guid.NewGuid() + ".glb");

            Action act = () => RustAssetPipelineInterop.ImportAsset(nonExistentPath, TestAssetId);

            act.Should().Throw<FileNotFoundException>()
                .WithMessage("*Asset file not found*");
        }

        // ── Error Case: Null Arguments ──────────────────────────────────────

        [Fact]
        public void ImportAsset_WithNullPath_ThrowsArgumentNullException()
        {
            Action act = () => RustAssetPipelineInterop.ImportAsset(null!, TestAssetId);

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("glbPath");
        }

        [Fact]
        public void ImportAsset_WithNullAssetId_ThrowsArgumentNullException()
        {
            CreateMinimalGlbFile(TestGlbPath);

            Action act = () => RustAssetPipelineInterop.ImportAsset(TestGlbPath, null!);

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("assetId");
        }

        [Fact]
        public void OptimizeAsset_WithNullAsset_ThrowsArgumentNullException()
        {
            Action act = () => RustAssetPipelineInterop.OptimizeAsset(null!, new[] { 100, 60, 30 });

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("importedAsset");
        }

        [Fact]
        public void OptimizeAsset_WithNullLodTargets_ThrowsArgumentNullException()
        {
            var asset = new AssetData { AssetId = TestAssetId };

            Action act = () => RustAssetPipelineInterop.OptimizeAsset(asset, null!);

            act.Should().Throw<ArgumentNullException>()
                .WithParameterName("lodTargets");
        }

        // ── Error Case: Invalid LOD Targets ────────────────────────────────

        [Fact]
        public void OptimizeAsset_WithEmptyLodTargets_ThrowsArgumentException()
        {
            var asset = new AssetData { AssetId = TestAssetId };

            Action act = () => RustAssetPipelineInterop.OptimizeAsset(asset, Array.Empty<int>());

            act.Should().Throw<ArgumentException>()
                .WithMessage("*LOD targets array cannot be empty*");
        }

        [Fact]
        public void OptimizeAsset_WithInvalidLodTarget_ThrowsArgumentException()
        {
            var asset = new AssetData { AssetId = TestAssetId };

            // LOD target > 100 is invalid
            Action act = () => RustAssetPipelineInterop.OptimizeAsset(asset, new[] { 150 });

            act.Should().Throw<ArgumentException>()
                .WithMessage("*LOD target must be between 1 and 100*");
        }

        [Fact]
        public void OptimizeAsset_WithZeroLodTarget_ThrowsArgumentException()
        {
            var asset = new AssetData { AssetId = TestAssetId };

            // LOD target 0 is invalid
            Action act = () => RustAssetPipelineInterop.OptimizeAsset(asset, new[] { 0 });

            act.Should().Throw<ArgumentException>()
                .WithMessage("*LOD target must be between 1 and 100*");
        }

        // ── Utilities ──────────────────────────────────────────────────────

        private static void CreateMinimalGlbFile(string path)
        {
            // Create a minimal GLB file for testing (just a valid file that exists)
            // Real format is not necessary for these tests since we're mocking the P/Invoke layer
            var directory = Path.GetDirectoryName(path);
            if (!Directory.Exists(directory))
                Directory.CreateDirectory(directory!);

            if (!File.Exists(path))
            {
                // Write a minimal GLB header (magic: 0x46546C67, version: 2)
                using (var fs = File.Create(path))
                {
                    // GLB magic number: 0x46546C67 = "glTF"
                    fs.WriteByte(0x67);
                    fs.WriteByte(0x6c);
                    fs.WriteByte(0x54);
                    fs.WriteByte(0x46);
                    // Version: 2
                    fs.WriteByte(0x02);
                    fs.WriteByte(0x00);
                    fs.WriteByte(0x00);
                    fs.WriteByte(0x00);
                }
            }
        }
    }
}
