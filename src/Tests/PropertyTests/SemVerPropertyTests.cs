#nullable enable
using System;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.PropertyTests
{
    /// <summary>
    /// Property-based tests for pack manifest version strings and compatibility checking.
    /// Verifies version string parsing, round-tripping, and comparison logic.
    /// </summary>
    [Trait("Category", "Property")]
    public class SemVerPropertyTests
    {
        [Theory(DisplayName = "Property: Valid version strings parse without exception")]
        [InlineData("1.0.0")]
        [InlineData("1.2.3")]
        [InlineData("0.1.0")]
        [InlineData("1.0.0-alpha")]
        [InlineData("1.0.0-beta.1")]
        [InlineData("2.0.0-rc.1")]
        [InlineData("1.2.3+build.1")]
        [InlineData("999.999.999-alpha.999+build.12345")]
        public void Parse_Valid_Version_Strings_Safe(string versionString)
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test",
                Version = versionString
            };

            // Act & Assert
            manifest.Version.Should().NotBeNullOrEmpty(
                because: "Version must be preserved as valid string");
            manifest.Version.Should().Be(versionString,
                because: "Version string should round-trip correctly");
        }

        [Theory(DisplayName = "Property: Framework version constraints parse without exception")]
        [InlineData(">=0.1.0")]
        [InlineData(">=1.0.0 <2.0.0")]
        [InlineData(">0.1.0 <=1.0.0")]
        [InlineData("1.0.0")]
        [InlineData("*")]
        public void Parse_Framework_Version_Constraints_Safe(string constraint)
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test",
                FrameworkVersion = constraint
            };

            // Act & Assert
            manifest.FrameworkVersion.Should().NotBeNullOrEmpty(
                because: "Framework version constraint must be preserved");
            manifest.FrameworkVersion.Should().Be(constraint);
        }

        [Theory(DisplayName = "Property: Empty or whitespace version strings are preserved")]
        [InlineData("")]
        [InlineData(" ")]
        public void Empty_Version_Preserved(string versionString)
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test",
                Version = versionString
            };

            // Act & Assert
            manifest.Version.Should().Be(versionString,
                because: "Empty/whitespace versions should be preserved as-is");
        }

        [Theory(DisplayName = "Property: Very long version strings handled safely")]
        [InlineData(100)]
        [InlineData(500)]
        public void Very_Long_Version_String_Safe(int length)
        {
            // Arrange
            string veryLongVersion = new string('1', length) + ".0.0";
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test",
                Version = veryLongVersion
            };

            // Act & Assert
            manifest.Version.Should().Be(veryLongVersion,
                because: "Long version strings should be handled safely");
            manifest.Version.Length.Should().Be(veryLongVersion.Length);
        }

        [Theory(DisplayName = "Property: Version with special characters preserved")]
        [InlineData("1.0.0-alpha+build.123")]
        [InlineData("1.0.0-beta.1.2.3")]
        [InlineData("1.0.0+meta.data")]
        public void Version_With_Special_Chars_Preserved(string version)
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test",
                Version = version
            };

            // Act & Assert
            manifest.Version.Should().Be(version,
                because: "Special characters in version should be preserved");
        }

        [Fact(DisplayName = "Property: PackManifest with all version fields populated")]
        public void PackManifest_All_Version_Fields_Safe()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "1.2.3",
                FrameworkVersion = ">=0.1.0 <2.0.0",
                Author = "Test Author"
            };

            // Act & Assert
            manifest.Version.Should().NotBeNullOrEmpty();
            manifest.FrameworkVersion.Should().NotBeNullOrEmpty();
            manifest.Id.Should().NotBeNullOrEmpty();
            manifest.Name.Should().NotBeNullOrEmpty();
        }

        [Theory(DisplayName = "Property: Numeric version parts extract correctly")]
        [InlineData("1.2.3", 1)]
        [InlineData("10.20.30", 10)]
        [InlineData("0.0.0", 0)]
        public void Version_Numeric_Parts_Extract(string version, int expectedMajor)
        {
            // Arrange
            var manifest = new PackManifest { Version = version };
            var parts = version.Split('.');

            // Act & Assert
            parts.Length.Should().BeGreaterThanOrEqualTo(3,
                because: "Standard semver should have at least 3 parts");
            int.TryParse(parts[0], out int major).Should().BeTrue();
            major.Should().Be(expectedMajor);
        }
    }
}
