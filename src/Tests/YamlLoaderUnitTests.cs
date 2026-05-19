using System;
using System.IO;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;
using YamlDotNet.Core;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="YamlLoader"/> covering deserialization,
    /// serialization, naming convention, and edge cases.
    /// </summary>
    public class YamlLoaderUnitTests
    {
        // ─── Deserialize: valid YAML ──────────────────────────────────────────

        [Fact]
        public void Deserialize_ValidSimpleYaml_ReturnsDeserializedObject()
        {
            // Arrange
            var yaml = @"
name: Test Pack
version: 1.0.0
author: Test Author
type: content";

            // Act
            var result = YamlLoader.Deserialize<TestPackManifest>(yaml);

            // Assert
            result.Should().NotBeNull();
            result!.Name.Should().Be("Test Pack");
            result.Version.Should().Be("1.0.0");
            result.Author.Should().Be("Test Author");
            result.Type.Should().Be("content");
        }

        // ─── Deserialize: underscored naming convention ──────────────────────

        [Fact]
        public void Deserialize_YamlWithUnderscoresToCamelCase_ConvertsCorrectly()
        {
            // Arrange — YAML with underscored keys (YAML convention)
            var yaml = @"
name: Underscored Pack
framework_version: "">=0.1.0""
depends_on:
  - pack-a
  - pack-b";

            // Act
            var result = YamlLoader.Deserialize<TestPackManifestWithDependencies>(yaml);

            // Assert — UnderscoredNamingConvention should map framework_version → FrameworkVersion
            result.Should().NotBeNull();
            result!.Name.Should().Be("Underscored Pack");
            result.FrameworkVersion.Should().Be(">=0.1.0");
            result.DependsOn.Should().HaveCount(2);
            result.DependsOn[0].Should().Be("pack-a");
            result.DependsOn[1].Should().Be("pack-b");
        }

        // ─── Deserialize: empty string returns default ──────────────────────

        [Fact]
        public void Deserialize_EmptyString_ReturnsNull()
        {
            // Arrange
            string emptyYaml = "";

            // Act
            var result = YamlLoader.Deserialize<TestPackManifest>(emptyYaml);

            // Assert
            result.Should().BeNull();
        }

        // ─── Deserialize: whitespace-only string returns default ────────────

        [Fact]
        public void Deserialize_WhitespaceOnlyString_ReturnsNull()
        {
            // Arrange
            string whitespace = "   \n\t\n  ";

            // Act
            var result = YamlLoader.Deserialize<TestPackManifest>(whitespace);

            // Assert
            result.Should().BeNull();
        }

        // ─── Deserialize: malformed YAML throws YamlException ──────────────

        [Fact]
        public void Deserialize_MalformedYaml_ThrowsYamlException()
        {
            // Arrange — invalid YAML (bad indentation, unclosed quote)
            var malformedYaml = @"
name: Test
  broken: [unclosed
version: 1.0";

            // Act & Assert
            var action = () => YamlLoader.Deserialize<TestPackManifest>(malformedYaml);
            action.Should().Throw<YamlException>();
        }

        // ─── DeserializeFromFile: valid file ──────────────────────────────

        [Fact]
        public void DeserializeFromFile_ValidFile_ReturnsDeserializedObject()
        {
            // Arrange
            var tempFile = Path.Combine(Path.GetTempPath(), "test_pack_" + Guid.NewGuid() + ".yaml");
            try
            {
                string yaml = @"name: File Pack
version: 2.0.0
author: File Test
type: balance";
                File.WriteAllText(tempFile, yaml);

                // Act
                var result = YamlLoader.DeserializeFromFile<TestPackManifest>(tempFile);

                // Assert
                result.Should().NotBeNull();
                result!.Name.Should().Be("File Pack");
                result.Version.Should().Be("2.0.0");
                result.Type.Should().Be("balance");
            }
            finally
            {
                if (File.Exists(tempFile))
                    File.Delete(tempFile);
            }
        }

        // ─── DeserializeFromFile: missing file returns null ────────────────

        [Fact]
        public void DeserializeFromFile_FileDoesNotExist_ReturnsNull()
        {
            // Arrange
            var nonexistentPath = Path.Combine(Path.GetTempPath(), "nonexistent_" + Guid.NewGuid() + ".yaml");

            // Act
            var result = YamlLoader.DeserializeFromFile<TestPackManifest>(nonexistentPath);

            // Assert
            result.Should().BeNull();
        }

        // ─── Serialize: object to YAML string ────────────────────────────────

        [Fact]
        public void Serialize_ValidObject_ReturnsYamlWithUnderscoredKeys()
        {
            // Arrange
            var obj = new TestPackManifestWithDependencies
            {
                Name = "Serialized Pack",
                FrameworkVersion = ">=1.0.0",
                DependsOn = new[] { "dep-1", "dep-2" }
            };

            // Act
            var yaml = YamlLoader.Serialize(obj);

            // Assert — should produce YAML with underscored keys (via UnderscoredNamingConvention)
            yaml.Should().Contain("name: Serialized Pack");
            yaml.Should().Contain("framework_version");
            yaml.Should().Contain("depends_on");
            yaml.Should().Contain("dep-1");
        }

        // ─── Serialize: null object returns empty string ───────────────────

        [Fact]
        public void Serialize_NullObject_ReturnsEmptyString()
        {
            // Arrange
            TestPackManifest? nullObj = null;

            // Act
            var yaml = YamlLoader.Serialize(nullObj);

            // Assert
            yaml.Should().Be(string.Empty);
        }

        // ─── SerializeToFile: writes valid YAML ──────────────────────────────

        [Fact]
        public void SerializeToFile_ValidObject_WritesYamlToFile()
        {
            // Arrange
            var tempFile = Path.Combine(Path.GetTempPath(), "out_pack_" + Guid.NewGuid() + ".yaml");
            var obj = new TestPackManifest
            {
                Name = "Output Pack",
                Version = "0.5.0",
                Author = "Test",
                Type = "content"
            };

            try
            {
                // Act
                YamlLoader.SerializeToFile(tempFile, obj);

                // Assert
                File.Exists(tempFile).Should().BeTrue();
                var content = File.ReadAllText(tempFile);
                content.Should().Contain("Output Pack");
                content.Should().Contain("0.5.0");
            }
            finally
            {
                if (File.Exists(tempFile))
                    File.Delete(tempFile);
            }
        }

        // ─── SerializeToFile: null object does not write file ──────────────

        [Fact]
        public void SerializeToFile_NullObject_DoesNotCreateFile()
        {
            // Arrange
            var tempFile = Path.Combine(Path.GetTempPath(), "null_out_" + Guid.NewGuid() + ".yaml");
            TestPackManifest? nullObj = null;

            // Act
            YamlLoader.SerializeToFile(tempFile, nullObj);

            // Assert
            File.Exists(tempFile).Should().BeFalse();
        }

        // ─── Test Fixtures ────────────────────────────────────────────────────

        private class TestPackManifest
        {
            public string? Name { get; set; }
            public string? Version { get; set; }
            public string? Author { get; set; }
            public string? Type { get; set; }
        }

        private class TestPackManifestWithDependencies
        {
            public string? Name { get; set; }
            public string? FrameworkVersion { get; set; }
            public string[]? DependsOn { get; set; }
        }
    }
}
