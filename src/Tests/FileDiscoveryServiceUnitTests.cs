using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="FileDiscoveryService"/> covering file/directory discovery,
    /// exclusion patterns, and edge cases.
    /// </summary>
    public class FileDiscoveryServiceUnitTests : IDisposable
    {
        private readonly string _tempDir;

        public FileDiscoveryServiceUnitTests()
        {
            _tempDir = Path.Combine(Path.GetTempPath(), "file_discovery_tests_" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_tempDir);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_tempDir))
                    Directory.Delete(_tempDir, recursive: true);
            }
            catch { /* best-effort cleanup */ }
        }

        [Fact]
        public void GetFiles_WithEmptyDirectory_ReturnsEmptyArray()
        {
            // Arrange
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, "*.txt");

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void GetFiles_WithMatchingFiles_ReturnsAllMatches()
        {
            // Arrange
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "file2.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "file3.json"), "content");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, "*.txt", SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(2);
            result.Should().AllSatisfy(p => Path.GetExtension(p).Should().Be(".txt"));
        }

        [Fact]
        public void GetFiles_WithMultiplePatterns_ReturnsCombinedResults()
        {
            // Arrange
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "file2.json"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "file3.yaml"), "content");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, new[] { "*.txt", "*.json" }, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(2);
        }

        [Fact]
        public void GetFiles_WithNonExistentDirectory_ReturnsEmptyArray()
        {
            // Arrange
            string nonExistentPath = Path.Combine(_tempDir, "does_not_exist");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(nonExistentPath, "*.txt");

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void GetFiles_WithNullPattern_ThrowsArgumentNullException()
        {
            // Arrange
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            var service = new FileDiscoveryService();

            // Act & Assert
            Assert.Throws<ArgumentNullException>(() => service.GetFiles(_tempDir, (string)null));
        }

        [Fact]
        public void GetFiles_WithEmptyPatternArray_ReturnsEmptyArray()
        {
            // Arrange
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, Array.Empty<string>());

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void GetFiles_RecursiveSearch_IncludesFilesInSubdirectories()
        {
            // Arrange
            string subDir = Path.Combine(_tempDir, "subdir");
            Directory.CreateDirectory(subDir);
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            File.WriteAllText(Path.Combine(subDir, "file2.txt"), "content");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, "*.txt", SearchOption.AllDirectories);

            // Assert
            result.Should().HaveCount(2);
        }

        [Fact]
        public void GetFiles_WithExcludedDirectory_SkipsFilesInExcludedDir()
        {
            // Arrange
            string binDir = Path.Combine(_tempDir, "bin");
            Directory.CreateDirectory(binDir);
            File.WriteAllText(Path.Combine(_tempDir, "file1.txt"), "content");
            File.WriteAllText(Path.Combine(binDir, "file2.txt"), "content");
            var service = new FileDiscoveryService(); // uses default exclusions (includes "bin")

            // Act
            string[] result = service.GetFiles(_tempDir, "*.txt", SearchOption.AllDirectories);

            // Assert
            result.Should().HaveCount(1);
            result.First().Should().EndWith("file1.txt");
        }

        [Fact]
        public void GetDirectories_WithEmptyDirectory_ReturnsEmptyArray()
        {
            // Arrange
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void GetDirectories_WithSubdirectories_ReturnsAllNonExcluded()
        {
            // Arrange
            Directory.CreateDirectory(Path.Combine(_tempDir, "dir1"));
            Directory.CreateDirectory(Path.Combine(_tempDir, "dir2"));
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin")); // excluded by default
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(2);
            result.Should().AllSatisfy(d => Path.GetFileName(d).Should().NotBe("bin"));
        }

        [Fact]
        public void GetDirectories_RecursiveSearch_IncludesNestedDirectories()
        {
            // Arrange
            string dir1 = Path.Combine(_tempDir, "dir1");
            string dir1Sub = Path.Combine(dir1, "subdir");
            Directory.CreateDirectory(dir1Sub);
            Directory.CreateDirectory(Path.Combine(_tempDir, "dir2"));
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetDirectories(_tempDir, SearchOption.AllDirectories);

            // Assert
            result.Should().HaveCount(3);
        }

        [Fact]
        public void GetDirectories_WithNonExistentPath_ReturnsEmptyArray()
        {
            // Arrange
            string nonExistentPath = Path.Combine(_tempDir, "does_not_exist");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetDirectories(nonExistentPath);

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void AddExclusion_AddsPatternToExclusions()
        {
            // Arrange
            var service = new FileDiscoveryService();
            string customDir = Path.Combine(_tempDir, "custom_exclude");
            Directory.CreateDirectory(customDir);
            Directory.CreateDirectory(Path.Combine(_tempDir, "normal"));
            File.WriteAllText(Path.Combine(customDir, "file.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "normal", "file.txt"), "content");

            // Act
            service.AddExclusion("custom_exclude");
            string[] result = service.GetDirectories(_tempDir, SearchOption.AllDirectories);

            // Assert
            result.Should().HaveCount(1);
            result.First().Should().EndWith("normal");
        }

        [Fact]
        public void RemoveExclusion_RemovesPatternFromExclusions()
        {
            // Arrange
            var service = new FileDiscoveryService();
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin"));
            Directory.CreateDirectory(Path.Combine(_tempDir, "obj"));

            // Act
            service.RemoveExclusion("bin");
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(1);
            result.First().Should().EndWith("bin");
        }

        [Fact]
        public void ClearExclusions_RemovesAllExclusions()
        {
            // Arrange
            var service = new FileDiscoveryService();
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin"));
            Directory.CreateDirectory(Path.Combine(_tempDir, "obj"));
            Directory.CreateDirectory(Path.Combine(_tempDir, "node_modules"));

            // Act
            service.ClearExclusions();
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(3);
        }

        [Fact]
        public void ResetToDefaults_RestoresDefaultExclusions()
        {
            // Arrange
            var service = new FileDiscoveryService();
            service.ClearExclusions();
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin"));

            // Act
            service.ResetToDefaults();
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().BeEmpty(); // bin is excluded again
        }

        [Fact]
        public void DiscoverPackDirectories_WithPackManifests_ReturnsPackDirs()
        {
            // Arrange
            string pack1 = Path.Combine(_tempDir, "pack1");
            string pack2 = Path.Combine(_tempDir, "pack2");
            string notPack = Path.Combine(_tempDir, "notpack");
            Directory.CreateDirectory(pack1);
            Directory.CreateDirectory(pack2);
            Directory.CreateDirectory(notPack);
            File.WriteAllText(Path.Combine(pack1, "pack.yaml"), "id: pack1");
            File.WriteAllText(Path.Combine(pack2, "pack.yaml"), "id: pack2");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.DiscoverPackDirectories(_tempDir);

            // Assert
            result.Should().HaveCount(2);
        }

        [Fact]
        public void DiscoverPackDirectories_WithNoPacks_ReturnsEmptyArray()
        {
            // Arrange
            Directory.CreateDirectory(Path.Combine(_tempDir, "notpack"));
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.DiscoverPackDirectories(_tempDir);

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void FileDiscoveryService_WithCustomExclusions_UsesOnlyCustom()
        {
            // Arrange
            var customExclusions = new[] { "custom_exclude" };
            var service = new FileDiscoveryService(customExclusions);
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin")); // default, but not in custom
            Directory.CreateDirectory(Path.Combine(_tempDir, "custom_exclude"));

            // Act
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().HaveCount(1);
            result.First().Should().EndWith("bin");
        }

        [Fact]
        public void DefaultExclusions_Property_ReturnsCopyOfDefaults()
        {
            // Arrange
            var service = new FileDiscoveryService();

            // Act
            var defaults = service.DefaultExclusions;

            // Assert
            defaults.Should().NotBeEmpty();
            defaults.Should().Contain("bin");
            defaults.Should().Contain("obj");
        }

        [Fact]
        public void GetFiles_WithNullDirectory_ReturnsEmptyArray()
        {
            // Arrange
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(null, "*.txt");

            // Assert
            result.Should().BeEmpty();
        }

        [Fact]
        public void GetFiles_SortedOutput()
        {
            // Arrange
            File.WriteAllText(Path.Combine(_tempDir, "zebra.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "apple.txt"), "content");
            File.WriteAllText(Path.Combine(_tempDir, "banana.txt"), "content");
            var service = new FileDiscoveryService();

            // Act
            string[] result = service.GetFiles(_tempDir, "*.txt");

            // Assert
            var fileNames = result.Select(Path.GetFileName).ToArray();
            fileNames.Should().Equal("apple.txt", "banana.txt", "zebra.txt");
        }

        [Fact]
        public void AddExclusion_WithWhitespaceOnly_IgnoresIt()
        {
            // Arrange
            var service = new FileDiscoveryService();
            service.AddExclusion("   ");
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin"));

            // Act
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().BeEmpty(); // bin is still excluded (default)
        }

        [Fact]
        public void RemoveExclusion_WithNullPattern_DoesNothing()
        {
            // Arrange
            var service = new FileDiscoveryService();
            Directory.CreateDirectory(Path.Combine(_tempDir, "bin"));

            // Act
            service.RemoveExclusion(null);
            string[] result = service.GetDirectories(_tempDir, SearchOption.TopDirectoryOnly);

            // Assert
            result.Should().BeEmpty(); // bin still excluded
        }
    }
}
