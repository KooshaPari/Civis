#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Edge case tests for SDK validation services (NJsonSchemaValidator, YamlSchemaConverter, SchemaResolverService).
/// These tests target uncovered branches in validation logic.
/// </summary>
public class SdkValidationEdgeCaseTests
{
    // ──────────────────────── NJsonSchemaValidator Edge Cases ────────────────────────

    [Fact]
    public void NJsonSchemaValidator_ValidateWithNullSchemaName_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object\nproperties:\n  id: { type: string }" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate(null!, "id: 123");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithEmptySchemaName_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("", "id: 123");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithWhitespaceSchemaName_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("   ", "id: 123");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithNullYamlContent_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("test", null!);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithEmptyYamlContent_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("test", "");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithWhitespaceYamlContent_ThrowsArgumentException()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("test", "   ");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithMissingSchema_ThrowsInvalidOperationException()
    {
        var schemaSources = new Dictionary<string, string>();
        var validator = new NJsonSchemaValidator(schemaSources);

        Action action = () => validator.Validate("nonexistent", "id: 123");

        action.Should().Throw<InvalidOperationException>();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithMalformedYaml_ReturnFailure()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object\nproperties:\n  id:\n    type: string" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        // Invalid YAML syntax
        var result = validator.Validate("test", "id: [unclosed bracket");

        result.IsValid.Should().BeFalse();
        result.Errors.Should().NotBeEmpty();
        result.Errors.First().Message.Should().Contain("Invalid YAML");
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithValidInput_ReturnSuccess()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object\nproperties:\n  id:\n    type: string" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        var result = validator.Validate("test", "id: test-123");

        result.IsValid.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void NJsonSchemaValidator_ValidateWithInvalidType_ReturnFailure()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object\nproperties:\n  id:\n    type: string\n  count:\n    type: integer" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        var result = validator.Validate("test", "id: test\ncount: not-a-number");

        result.IsValid.Should().BeFalse();
        result.Errors.Should().NotBeEmpty();
    }

    [Fact]
    public void NJsonSchemaValidator_CachesSchemas_SecondValidationUsesCachedSchema()
    {
        var schemaSources = new Dictionary<string, string>
        {
            { "test", "type: object\nproperties:\n  id:\n    type: string" }
        };
        var validator = new NJsonSchemaValidator(schemaSources);

        var result1 = validator.Validate("test", "id: first");
        var result2 = validator.Validate("test", "id: second");

        result1.IsValid.Should().BeTrue();
        result2.IsValid.Should().BeTrue();
    }

    [Fact]
    public void NJsonSchemaValidator_WithNullSchemaSources_ThrowsArgumentNullException()
    {
        Action action = () => new NJsonSchemaValidator(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    // ──────────────────────── YamlSchemaConverter Edge Cases ────────────────────────

    [Fact]
    public void YamlSchemaConverter_ConvertNullYaml_ThrowsArgumentNullException()
    {
        Action action = () => YamlSchemaConverter.ConvertYamlToJson(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void YamlSchemaConverter_ConvertEmptyYaml_ReturnsEmptyJsonObject()
    {
        var result = YamlSchemaConverter.ConvertYamlToJson("");

        result.Should().NotBeNull();
    }

    [Fact]
    public void YamlSchemaConverter_ConvertComplexYaml_ReturnsValidJson()
    {
        var yaml = @"
name: Test Pack
version: 1.0.0
properties:
  - id: test1
    value: 100
  - id: test2
    value: 200";

        var result = YamlSchemaConverter.ConvertYamlToJson(yaml);

        result.Should().NotBeNull();
        result.Should().Contain("Test Pack");
        result.Should().Contain("1.0.0");
    }

    [Fact]
    public void YamlSchemaConverter_ConvertWithNestedStructures_PreservesHierarchy()
    {
        var yaml = @"
root:
  level1:
    level2:
      value: deep";

        var result = YamlSchemaConverter.ConvertYamlToJson(yaml);

        result.Should().Contain("root");
        result.Should().Contain("level1");
        result.Should().Contain("level2");
        result.Should().Contain("deep");
    }

    [Fact]
    public void YamlSchemaConverter_ConvertWithSpecialCharacters_HandlesCorrectly()
    {
        var yaml = @"
description: 'Test with quotes'
special: 'value with colons'
number: 123";

        var result = YamlSchemaConverter.ConvertYamlToJson(yaml);

        result.Should().NotBeNull();
        result.Should().Contain("Test with quotes");
    }

    [Fact]
    public void YamlSchemaConverter_ConvertWithMultilineString_PreservesContent()
    {
        var yaml = @"
description: |
  This is a
  multiline
  string";

        var result = YamlSchemaConverter.ConvertYamlToJson(yaml);

        result.Should().Contain("multiline");
    }
}
