using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class TotalConversionManifestCoverageTests
    {
        [Fact]
        public void Validate_DefaultManifest_ReturnsRequiredFieldErrors()
        {
            TotalConversionManifest manifest = new TotalConversionManifest();

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Be("Id is required.");
            result.Errors[0].Rule.Should().Be("validation");
            result.Errors[1].Path.Should().Be("name");
            result.Errors[1].Message.Should().Be("Name is required.");
            result.Errors[1].Rule.Should().Be("validation");
        }

        [Fact]
        public void Validate_WithCompleteManifest_ReturnsSuccess()
        {
            TotalConversionManifest manifest = CreateValidManifest();
            manifest.Factions.Add(new TcFactionEntry
            {
                Id = "faction.rebels",
                Replaces = "faction.vanilla",
                Name = "Rebels"
            });

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithInvalidFactionEntry_ReturnsIndexedFactionErrors()
        {
            TotalConversionManifest manifest = CreateValidManifest();
            manifest.Factions.Add(new TcFactionEntry());

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
            result.Errors[0].Path.Should().Be("factions[0].id");
            result.Errors[0].Message.Should().Be("Faction entry Id is required.");
            result.Errors[0].Rule.Should().Be("validation");
            result.Errors[1].Path.Should().Be("factions[0].replaces");
            result.Errors[1].Message.Should().Be("Faction entry Replaces is required.");
            result.Errors[1].Rule.Should().Be("validation");
            result.Errors[2].Path.Should().Be("factions[0].name");
            result.Errors[2].Message.Should().Be("Faction entry Name is required.");
            result.Errors[2].Rule.Should().Be("validation");
        }

        private static TotalConversionManifest CreateValidManifest()
        {
            return new TotalConversionManifest
            {
                Id = "tc.starwars",
                Name = "Star Wars Total Conversion",
                Version = "1.0.0",
                Type = "total_conversion",
                FrameworkVersion = ">=0.1.0 <1.0.0"
            };
        }
    }
}
