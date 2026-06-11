using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK;

public sealed class CompatibilityCheckerCoverageTests
{
    [Theory]
    [InlineData("1.2.3", "*", true)]
    [InlineData("1.2.3", "   ", true)]
    [InlineData("1.2.3", "1.2.3", true)]
    [InlineData("1.2.3", "1.2.4", false)]
    [InlineData("1.2.4", ">=1.2.3 <2.0.0", true)]
    [InlineData("2.0.0", ">=1.2.3 <2.0.0", false)]
    [InlineData("2021.3.45f2", "2021.3.*", true)]
    [InlineData("2022.1.0", "2021.3.*", false)]
    [InlineData("1.2.4", "^1.2.3", true)]
    [InlineData("1.3.0", "^1.2.3", true)]
    [InlineData("1.3.0", "~1.2.3", false)]
    [InlineData("2.0.0", "~1.2.3", false)]
    public void IsVersionInRange_matches_the_documented_constraint_forms(
        string version,
        string range,
        bool expected)
    {
        bool actual = CompatibilityChecker.IsVersionInRange(version, range);

        actual.Should().Be(expected);
    }
}
