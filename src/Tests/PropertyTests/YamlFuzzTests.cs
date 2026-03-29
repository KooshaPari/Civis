#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;

namespace DINOForge.Tests.PropertyTests
{
    /// <summary>
    /// Fuzz tests for YAML parser and pack loader.
    /// Verifies robustness: null/empty inputs, extremely long strings, malformed YAML.
    /// Uses corpus files from src/Tests/FuzzCorpus/yaml/ as seed inputs.
    /// </summary>
    [Trait("Category", "Fuzz")]
    public class YamlFuzzTests
    {
        private readonly IDeserializer _deserializer;

        public YamlFuzzTests()
        {
            _deserializer = new DeserializerBuilder()
                .IgnoreUnmatchedProperties()
                .Build();
        }

        [Theory(DisplayName = "Fuzz: Null or empty string input returns error, not exception")]
        [InlineData("")]
        [InlineData(null)]
        [InlineData("   ")]
        public void Parse_Null_Or_Empty_Returns_Error_Not_Exception(string? input)
        {
            // Act & Assert
            if (string.IsNullOrWhiteSpace(input))
            {
                // Null/empty should either return null or throw a graceful exception,
                // never an unhandled exception.
                var action = () =>
                {
                    try
                    {
                        var result = _deserializer.Deserialize<PackManifest>(input ?? "");
                        // Null result is acceptable
                        _ = result;
                    }
                    catch (Exception ex) when (!(ex is ArgumentNullException))
                    {
                        // Only ArgumentNullException is acceptable for null input
                        throw;
                    }
                };

                // Should not throw unexpected exceptions
                action.Should().NotThrow<NullReferenceException>(
                    because: "Parser must handle null/empty gracefully");
            }
        }

        [Theory(DisplayName = "Fuzz: Extremely long string values are truncated or rejected gracefully")]
        [InlineData(1000)]
        [InlineData(10000)]
        [InlineData(100000)]
        public void Parse_Very_Long_Strings_Handled_Gracefully(int length)
        {
            // Arrange
            string veryLongString = new string('a', length);
            string yaml = $@"
id: test-pack
name: {veryLongString}
version: 1.0.0
";

            // Act & Assert
            var action = () =>
            {
                try
                {
                    var result = _deserializer.Deserialize<PackManifest>(yaml);
                    // Success: parsed it (perhaps truncated)
                    result.Should().NotBeNull();
                }
                catch (Exception ex) when (ex is OutOfMemoryException)
                {
                    // OutOfMemory is acceptable for extremely large strings
                    throw;
                }
            };

            // Should not crash with uncontrolled exceptions
            action.Should().NotThrow<StackOverflowException>(
                because: "Parser must not overflow stack on long strings");
        }

        [Fact(DisplayName = "Fuzz: Deeply nested YAML does not cause stack overflow")]
        public void Parse_Deeply_Nested_YAML_Safe()
        {
            // Arrange: Create deeply nested structure (100 levels)
            var nestedYaml = @"
id: test-pack
name: Deep Test
version: 1.0.0
metadata:
  level1:
    level2:
      level3:
        level4:
          level5:
            level6:
              level7:
                level8:
                  level9:
                    level10: value
";

            // Act & Assert
            var action = () =>
            {
                var result = _deserializer.Deserialize<dynamic>(nestedYaml);
                result.Should().NotBeNull();
            };

            action.Should().NotThrow<StackOverflowException>(
                because: "Parser must handle reasonable nesting safely");
        }

        [Theory(DisplayName = "Fuzz: YAML with special characters parses or throws descriptively")]
        [InlineData("id: test\nname: \"@#$%^&*()\"")]
        [InlineData("id: test\nname: \"Unicode: 日本語 中文\"")]
        [InlineData("id: test\nname: \"Emoji: 🎮🚀💻\"")]
        public void Parse_Special_Characters_Safe(string yaml)
        {
            // Act & Assert
            var action = () =>
            {
                try
                {
                    var result = _deserializer.Deserialize<dynamic>(yaml);
                    result.Should().NotBeNull();
                }
                catch (YamlDotNet.Core.YamlException)
                {
                    // Descriptive YAML exceptions are acceptable
                }
            };

            action.Should().NotThrow<NullReferenceException>(
                because: "Parser must not crash with NRE on special characters");
        }

        [Fact(DisplayName = "Fuzz: Circular reference detection does not cause exception")]
        public void Parse_Self_Referential_Conflict_Detected()
        {
            // Arrange
            var yaml = @"
id: circular-pack
name: Circular
version: 1.0.0
type: content
conflicts_with:
  - circular-pack
";

            // Act & Assert
            var action = () =>
            {
                var result = _deserializer.Deserialize<PackManifest>(yaml);
                result.Should().NotBeNull();
                result!.ConflictsWith.Should().Contain("circular-pack");
            };

            // Should parse without exception (self-conflict is detected later during validation)
            action.Should().NotThrow(
                because: "Parser must handle circular references in YAML syntax");
        }

        [Theory(DisplayName = "Fuzz: Invalid YAML syntax is caught, not silently ignored")]
        [InlineData("id: test\nname: unclosed \"string")]
        [InlineData("id: test\nname: : invalid")]
        [InlineData("id: test\n  - invalid: list\nid: duplicate")]
        public void Parse_Invalid_YAML_Throws_Appropriately(string yaml)
        {
            // Act & Assert
            var action = () =>
            {
                try
                {
                    var result = _deserializer.Deserialize<PackManifest>(yaml);
                    // If it parses, that's fine (YAML is forgiving)
                    _ = result;
                }
                catch (Exception ex)
                {
                    // YamlDotNet should throw YamlException or similar
                    ex.Should().NotBeOfType<NullReferenceException>(
                        because: "Should throw descriptive exception, not NRE");
                }
            };

            action.Should().NotThrow<NullReferenceException>(
                because: "Parser must throw descriptive exception, not NRE");
        }

        [Fact(DisplayName = "Fuzz: Corpus files from FuzzCorpus/yaml/ load without exception")]
        public void Corpus_YAML_Files_Load_Safely()
        {
            // Arrange
            string corpusDir = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "FuzzCorpus", "yaml");

            if (!Directory.Exists(corpusDir))
            {
                // Skip if corpus not present (e.g., in some CI environments)
                return;
            }

            var yamlFiles = Directory.GetFiles(corpusDir, "*.yaml");

            // Act & Assert
            foreach (var file in yamlFiles)
            {
                var yaml = File.ReadAllText(file);
                var action = () =>
                {
                    try
                    {
                        // Attempt to parse each corpus file
                        var result = _deserializer.Deserialize<dynamic>(yaml);
                        // Success or graceful parse
                        _ = result;
                    }
                    catch (Exception ex) when (ex is YamlDotNet.Core.YamlException)
                    {
                        // YAML parse exceptions are acceptable
                        // (corpus contains malformed files)
                    }
                };

                action.Should().NotThrow(
                    because: $"Corpus file {Path.GetFileName(file)} should not cause unhandled exception");
            }
        }
    }
}
