#nullable enable
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="ResourceDefinition.Validate"/> — required Id/Name and the
/// non-negative StorageCapacity / ProductionRate rules. First Economy-domain coverage.
/// </summary>
public class ResourceDefinitionCoverageTests
{
    private static ResourceDefinition Valid() => new()
    {
        Id = "res:gold",
        Name = "Gold",
        ProductionRate = 5f,
        StorageCapacity = 1000f
    };

    [Fact]
    public void Validate_PopulatedResource_IsValid()
    {
        Valid().Validate().IsValid.Should().BeTrue();
    }

    [Fact]
    public void Validate_DefaultCtor_SetsSaneDefaults()
    {
        ResourceDefinition d = new();

        d.ProductionRate.Should().Be(1.0f);
        d.StorageCapacity.Should().Be(1000.0f);
        d.IsTradeableDefault.Should().BeTrue();
        // Id/Name empty by default → not yet valid until set.
        d.Validate().IsValid.Should().BeFalse();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Validate_MissingId_Fails(string id)
    {
        ResourceDefinition d = Valid();
        d.Id = id;

        ValidationResult result = d.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "id");
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Validate_MissingName_Fails(string name)
    {
        ResourceDefinition d = Valid();
        d.Name = name;

        ValidationResult result = d.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "name");
    }

    [Fact]
    public void Validate_NegativeStorageCapacity_Fails()
    {
        ResourceDefinition d = Valid();
        d.StorageCapacity = -1f;

        ValidationResult result = d.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "storage_capacity");
    }

    [Fact]
    public void Validate_NegativeProductionRate_Fails()
    {
        ResourceDefinition d = Valid();
        d.ProductionRate = -0.5f;

        ValidationResult result = d.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "production_rate");
    }

    [Fact]
    public void Validate_MultipleViolations_ReportsAll()
    {
        ResourceDefinition d = new() { Id = "", Name = "", StorageCapacity = -1f, ProductionRate = -1f };

        ValidationResult result = d.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().HaveCount(4);
    }
}
