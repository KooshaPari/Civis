using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="CompatibilityChecker"/> version compatibility logic.
    /// </summary>
    public class CompatibilityCheckerUnitTests
    {
        [Fact]
        public void IsVersionInRange_WithWildcard_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", "*");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_WithEmptyRange_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2.0.0", "");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_ExactMatch_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", "1.5.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_ExactMismatch_ReturnsFalse()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", "1.5.1");

            // Assert
            result.Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_GreaterThanOrEqual_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2.0.0", ">=1.5.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_GreaterThanOrEqualBoundary_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.5.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_LessThan_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.0.0", "<2.0.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_LessThanFails_ReturnsFalse()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2.5.0", "<2.0.0");

            // Assert
            result.Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_MultipleConstraints_AllMustPass()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0 <2.0.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_MultipleConstraintsFails_ReturnsFalse()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2.5.0", ">=1.0.0 <2.0.0");

            // Assert
            result.Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_WildcardVersion_MatchesPrefix()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2021.3.45f2", "2021.3.*");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_WildcardVersionMismatch_ReturnsFalse()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2021.4.1", "2021.3.*");

            // Assert
            result.Should().BeFalse();
        }

        [Fact]
        public void CheckPack_FrameworkVersionMismatch_ReturnsError()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Version = "1.0.0",
                FrameworkVersion = "99.0.0",
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = "*"
            };

            // Act
            var result = CompatibilityChecker.CheckPack(manifest);

            // Assert
            result.IsCompatible.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("DINOForge");
        }

        [Fact]
        public void CheckPack_GameVersionMismatch_ReturnsWarning()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Version = "1.0.0",
                FrameworkVersion = "*",
                GameVersion = "0.5.0",
                BepInExVersion = "*",
                UnityVersion = "*"
            };

            // Act
            var result = CompatibilityChecker.CheckPack(manifest, "1.0.0");

            // Assert
            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().HaveCount(1);
        }

        [Fact]
        public void CheckPack_AllWildcards_ReturnsCompatible()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Version = "1.0.0",
                FrameworkVersion = "*",
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = "*"
            };

            // Act
            var result = CompatibilityChecker.CheckPack(manifest);

            // Assert
            result.IsCompatible.Should().BeTrue();
            result.Errors.Should().BeEmpty();
            result.Warnings.Should().BeEmpty();
        }

        [Fact]
        public void CheckPack_MultipleVersionMismatches_ReturnsAllWarnings()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Version = "1.0.0",
                FrameworkVersion = "*",
                GameVersion = "2.0.0",
                BepInExVersion = "5.0.0",
                UnityVersion = "2020.3.0"
            };

            // Act
            var result = CompatibilityChecker.CheckPack(manifest, "1.0.0", "5.4.0", "2021.3.0");

            // Assert
            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().HaveCount(3);
        }

        [Fact]
        public void IsVersionInRange_StrictlyGreater_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2.0.0", ">1.5.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_StrictlyGreaterBoundaryFails_ReturnsFalse()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", ">1.5.0");

            // Assert
            result.Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_LessThanOrEqual_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", "<=1.5.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_UnityVersionFormat_HandlesAlphaNumericSuffix()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2021.3.45f2", ">=2021.3.0");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_PrereleaseSyntax_StripsSemverTag()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("1.5.0", "1.5.0-alpha.1");

            // Assert
            result.Should().BeTrue();
        }

        [Fact]
        public void CheckPack_BepInExVersionWarning_ReturnsWarning()
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Version = "1.0.0",
                FrameworkVersion = "*",
                GameVersion = "*",
                BepInExVersion = "5.0.0",
                UnityVersion = "*"
            };

            // Act
            var result = CompatibilityChecker.CheckPack(manifest, "*", "5.4.0");

            // Assert
            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().HaveCount(1);
            result.Warnings[0].Should().Contain("BepInEx");
        }

        [Fact]
        public void IsVersionInRange_CaseInsensitive_ReturnsTrue()
        {
            // Act
            var result = CompatibilityChecker.IsVersionInRange("2021.3.45F2", "2021.3.*");

            // Assert
            result.Should().BeTrue();
        }
    }
}
