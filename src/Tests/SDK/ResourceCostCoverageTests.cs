using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Coverage tests for <see cref="ResourceCost"/> — validates the per-resource
    /// non-negativity rules in <see cref="ResourceCost.Validate"/>.
    /// </summary>
    public sealed class ResourceCostCoverageTests
    {
        [Fact]
        public void Validate_AllZero_IsValid()
        {
            ResourceCost cost = new();

            cost.Validate().IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_AllPositive_IsValid()
        {
            ResourceCost cost = new()
            {
                Food = 10, Wood = 20, Stone = 5, Iron = 2, Gold = 100, Population = 3
            };

            ValidationResult result = cost.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Theory]
        [InlineData("food")]
        [InlineData("wood")]
        [InlineData("stone")]
        [InlineData("iron")]
        [InlineData("gold")]
        [InlineData("population")]
        public void Validate_NegativeSingleField_FailsWithThatFieldError(string field)
        {
            ResourceCost cost = new();
            switch (field)
            {
                case "food": cost.Food = -1; break;
                case "wood": cost.Wood = -1; break;
                case "stone": cost.Stone = -1; break;
                case "iron": cost.Iron = -1; break;
                case "gold": cost.Gold = -1; break;
                case "population": cost.Population = -1; break;
            }

            ValidationResult result = cost.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Path == field);
        }

        [Fact]
        public void Validate_MultipleNegativeFields_ReportsAllErrors()
        {
            ResourceCost cost = new() { Food = -1, Gold = -5, Population = -2 };

            ValidationResult result = cost.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
        }
    }
}
