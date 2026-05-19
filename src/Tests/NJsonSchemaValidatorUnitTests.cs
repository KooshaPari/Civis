using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for NJsonSchemaValidator covering schema loading, validation success/failure, and error messages.
    /// </summary>
    public class NJsonSchemaValidatorUnitTests
    {
        private static readonly Dictionary<string, string> MinimalSchemaSources = new()
        {
            ["pack-manifest"] = @"
type: object
properties:
  id:
    type: string
  name:
    type: string
  version:
    type: string
required:
  - id
  - name
"
        };

        private static readonly Dictionary<string, string> SchemaWithRef = new()
        {
            ["simple-schema"] = @"
type: object
properties:
  items:
    type: array
    items:
      $ref: '#/definitions/item'
definitions:
  item:
    type: object
    properties:
      id:
        type: string
    required:
      - id
",
            ["pack-manifest"] = @"
type: object
properties:
  id:
    type: string
required:
  - id
"
        };

        [Fact]
        public void Constructor_WithNullSchemaSources_ThrowsArgumentNullException()
        {
            // Act & Assert
            var act = () => new NJsonSchemaValidator(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("schemaSources");
        }

        [Fact]
        public void Validate_WithNullSchemaName_ThrowsArgumentException()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var validYaml = "id: test-pack\nname: Test Pack";

            // Act & Assert
            var act = () => validator.Validate(null!, validYaml);
            act.Should().Throw<ArgumentException>().WithParameterName("schemaName");
        }

        [Fact]
        public void Validate_WithEmptySchemaName_ThrowsArgumentException()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var validYaml = "id: test-pack\nname: Test Pack";

            // Act & Assert
            var act = () => validator.Validate("", validYaml);
            act.Should().Throw<ArgumentException>().WithParameterName("schemaName");
        }

        [Fact]
        public void Validate_WithNullYamlContent_ThrowsArgumentException()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);

            // Act & Assert
            var act = () => validator.Validate("pack-manifest", null!);
            act.Should().Throw<ArgumentException>().WithParameterName("yamlContent");
        }

        [Fact]
        public void Validate_WithEmptyYamlContent_ThrowsArgumentException()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);

            // Act & Assert
            var act = () => validator.Validate("pack-manifest", "");
            act.Should().Throw<ArgumentException>().WithParameterName("yamlContent");
        }

        [Fact]
        public void Validate_WithSchemaNotFound_ThrowsInvalidOperationException()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var validYaml = "id: test-pack\nname: Test Pack";

            // Act & Assert
            var act = () => validator.Validate("nonexistent-schema", validYaml);
            act.Should().Throw<InvalidOperationException>()
                .WithMessage("*Schema 'nonexistent-schema' not found*");
        }

        [Fact]
        public void Validate_WithValidYaml_ReturnsSuccess()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var validYaml = "id: test-pack\nname: Test Pack";

            // Act
            var result = validator.Validate("pack-manifest", validYaml);

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithInvalidYaml_ReturnsMalformedError()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var invalidYaml = "id: test-pack\n  name: [unclosed"; // Invalid YAML syntax

            // Act
            var result = validator.Validate("pack-manifest", invalidYaml);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
            result.Errors[0].Message.Should().Contain("Invalid YAML");
            result.Errors[0].Rule.Should().Be("yaml-parse-error");
        }

        [Fact]
        public void Validate_WithMissingRequiredField_ReturnsError()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var incompleteYaml = "id: test-pack\n"; // Missing required 'name'

            // Act
            var result = validator.Validate("pack-manifest", incompleteYaml);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void Validate_WithWrongTypeField_ReturnsError()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var wrongTypeYaml = "id: 123\nname: Test Pack\n"; // 'id' should be string, not number

            // Act
            var result = validator.Validate("pack-manifest", wrongTypeYaml);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void Validate_WithSchemaRef_ValidatesNestedTypes()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(SchemaWithRef);
            var validYaml = @"
items:
  - id: item-1
  - id: item-2
";

            // Act
            var result = validator.Validate("simple-schema", validYaml);

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithSchemaRef_RejectsInvalidNestedItems()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(SchemaWithRef);
            var invalidYaml = @"
items:
  - id: item-1
  - { }
"; // Second item missing required 'id'

            // Act
            var result = validator.Validate("simple-schema", invalidYaml);

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void Validate_CachesLoadedSchemas()
        {
            // Arrange
            var validator = new NJsonSchemaValidator(MinimalSchemaSources);
            var validYaml = "id: test-pack\nname: Test Pack";

            // Act
            var result1 = validator.Validate("pack-manifest", validYaml);
            var result2 = validator.Validate("pack-manifest", validYaml);

            // Assert (both should succeed, second uses cached schema)
            result1.IsValid.Should().BeTrue();
            result2.IsValid.Should().BeTrue();
        }
    }
}
