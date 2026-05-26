using System.Collections.Generic;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for YamlSchemaConverter covering YAML-to-JSON conversion and type coercion.
    /// </summary>
    public class YamlSchemaConverterUnitTests
    {
        [Fact]
        public void ConvertYamlToJson_WithSimpleObject_PreservesStructure()
        {
            // Arrange
            var yaml = @"
id: test-pack
name: Test Pack
version: 1.0.0
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["id"]!.Value<string>().Should().Be("test-pack");
            parsed["name"]!.Value<string>().Should().Be("Test Pack");
            parsed["version"]!.Value<string>().Should().Be("1.0.0");
        }

        [Fact]
        public void ConvertYamlToJson_WithIntegerValue_CoercesToLong()
        {
            // Arrange
            var yaml = @"
count: 42
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["count"]!.Type.Should().Be(JTokenType.Integer);
            parsed["count"]!.Value<long>().Should().Be(42);
        }

        [Fact]
        public void ConvertYamlToJson_WithFloatValue_CoercesToDouble()
        {
            // Arrange
            var yaml = @"
rate: 3.14
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["rate"]!.Type.Should().Be(JTokenType.Float);
            parsed["rate"]!.Value<double>().Should().BeApproximately(3.14, 0.01);
        }

        [Fact]
        public void ConvertYamlToJson_WithBooleanTrue_CoercesToBoolean()
        {
            // Arrange
            var yaml = @"
enabled: true
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["enabled"]!.Type.Should().Be(JTokenType.Boolean);
            parsed["enabled"]!.Value<bool>().Should().BeTrue();
        }

        [Fact]
        public void ConvertYamlToJson_WithBooleanFalse_CoercesToBoolean()
        {
            // Arrange
            var yaml = @"
disabled: false
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["disabled"]!.Type.Should().Be(JTokenType.Boolean);
            parsed["disabled"]!.Value<bool>().Should().BeFalse();
        }

        [Fact]
        public void ConvertYamlToJson_WithNullValue_CoercesToNull()
        {
            // Arrange
            var yaml = @"
value: null
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["value"]!.Type.Should().Be(JTokenType.Null);
        }

        [Fact]
        public void ConvertYamlToJson_WithTildeNull_CoercesToNull()
        {
            // Arrange
            var yaml = @"
value: ~
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["value"]!.Type.Should().Be(JTokenType.Null);
        }

        [Fact]
        public void ConvertYamlToJson_WithNestedObject_PreservesHierarchy()
        {
            // Arrange
            var yaml = @"
metadata:
  id: pack-id
  version: 1.0.0
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed["metadata"]!["id"]!.Value<string>().Should().Be("pack-id");
            parsed["metadata"]!["version"]!.Value<string>().Should().Be("1.0.0");
        }

        [Fact]
        public void ConvertYamlToJson_WithArray_PreservesItemsAndTypes()
        {
            // Arrange
            var yaml = @"
items:
  - value: first
    count: 1
  - value: second
    count: 2
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            var items = parsed["items"]!;
            items.Should().HaveCount(2);
            items[0]!["value"]!.Value<string>().Should().Be("first");
            items[0]!["count"]!.Value<long>().Should().Be(1);
            items[1]!["value"]!.Value<string>().Should().Be("second");
            items[1]!["count"]!.Value<long>().Should().Be(2);
        }

        [Fact]
        public void ConvertYamlToJson_WithSnakeCaseKeys_PreservesKeyNames()
        {
            // Arrange
            var yaml = @"
framework_version: '>=1.0.0'
depends_on:
  - other-pack
conflicts_with: []
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            parsed.Should().ContainKey("framework_version");
            parsed.Should().ContainKey("depends_on");
            parsed.Should().ContainKey("conflicts_with");
            parsed["framework_version"]!.Value<string>().Should().Be(">=1.0.0");
        }

        [Fact]
        public void ConvertYamlToJson_WithEmptyArray_ProducesEmptyJsonArray()
        {
            // Arrange
            var yaml = @"
items: []
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            var items = parsed["items"]!;
            items.Type.Should().Be(JTokenType.Array);
            items.Should().BeEmpty();
        }

        [Fact]
        public void ConvertYamlToJson_WithComplexMixedTypes_CoercesCorrectly()
        {
            // Arrange
            var yaml = @"
pack:
  id: modern-warfare
  version: 2.0
  enabled: true
  tags:
    - modern
    - realistic
  stats:
    health: 100
    damage: 50.5
    bonus: null
";

            // Act
            var json = YamlSchemaConverter.ConvertYamlToJson(yaml);
            var parsed = JObject.Parse(json);

            // Assert
            var pack = parsed["pack"]!;
            pack["id"]!.Value<string>().Should().Be("modern-warfare");
            pack["version"]!.Value<double>().Should().Be(2.0);
            pack["enabled"]!.Value<bool>().Should().BeTrue();
            pack["tags"]!.Should().HaveCount(2);
            pack["stats"]!["health"]!.Value<long>().Should().Be(100);
            pack["stats"]!["damage"]!.Value<double>().Should().BeApproximately(50.5, 0.01);
            pack["stats"]!["bonus"]!.Type.Should().Be(JTokenType.Null);
        }

    }
}
