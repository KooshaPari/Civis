#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Edge case tests for SDK compatibility services (CompatibilityChecker, ContentDiscoveryService).
/// These tests target uncovered branches in compatibility and content discovery logic.
/// </summary>
public class SdkCompatibilityEdgeCaseTests
{
    // ──────────────────────── CompatibilityChecker Edge Cases ────────────────────────

    [Fact]
    public void CompatibilityChecker_FrameworkVersion_ReturnsValidVersion()
    {
        var version = CompatibilityChecker.FrameworkVersion;

        version.Should().NotBeNull();
        version.Major.Should().BeGreaterThanOrEqualTo(0);
        version.Minor.Should().BeGreaterThanOrEqualTo(0);
    }

    [Fact]
    public void CompatibilityChecker_CheckPackWithAllWildcards_IsAlwaysCompatible()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            FrameworkVersion = "*",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        var result = CompatibilityChecker.CheckPack(manifest);

        result.IsCompatible.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void CompatibilityChecker_CheckPackWithMatchingFrameworkVersion_IsCompatible()
    {
        var currentVersion = CompatibilityChecker.FrameworkVersion.ToString();
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            FrameworkVersion = currentVersion,
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        var result = CompatibilityChecker.CheckPack(manifest, "*", "*", "*");

        result.IsCompatible.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityChecker_CheckPackWithMismatchedFrameworkVersion_IsIncompatible()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            FrameworkVersion = "99.0.0",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        var result = CompatibilityChecker.CheckPack(manifest);

        result.IsCompatible.Should().BeFalse();
        result.Errors.Should().NotBeEmpty();
    }

    [Fact]
    public void CompatibilityChecker_CheckPackWithMismatchedGameVersion_HasWarningNotError()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test Pack",
            Version = "1.0.0",
            FrameworkVersion = "*",
            GameVersion = "2.0.0",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        var result = CompatibilityChecker.CheckPack(manifest, "1.0.0");

        result.IsCompatible.Should().BeTrue();
        result.Warnings.Should().NotBeEmpty();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithWildcard_AlwaysTrue()
    {
        var result = CompatibilityChecker.IsVersionInRange("anything", "*");

        result.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithEmptyRange_AlwaysTrue()
    {
        var result = CompatibilityChecker.IsVersionInRange("1.0.0", "");

        result.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithWhitespaceRange_AlwaysTrue()
    {
        var result = CompatibilityChecker.IsVersionInRange("1.0.0", "   ");

        result.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithExactVersion_MatchesCorrectly()
    {
        var result = CompatibilityChecker.IsVersionInRange("1.0.0", "1.0.0");

        result.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithExactVersionMismatch_ReturnsFalse()
    {
        var result = CompatibilityChecker.IsVersionInRange("1.0.0", "2.0.0");

        result.Should().BeFalse();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithGreaterThanOrEqual_WorksCorrectly()
    {
        var result1 = CompatibilityChecker.IsVersionInRange("2.0.0", ">=1.0.0");
        var result2 = CompatibilityChecker.IsVersionInRange("1.0.0", ">=1.0.0");
        var result3 = CompatibilityChecker.IsVersionInRange("0.9.0", ">=1.0.0");

        result1.Should().BeTrue();
        result2.Should().BeTrue();
        result3.Should().BeFalse();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithLessThanOrEqual_WorksCorrectly()
    {
        var result1 = CompatibilityChecker.IsVersionInRange("1.0.0", "<=2.0.0");
        var result2 = CompatibilityChecker.IsVersionInRange("2.0.0", "<=2.0.0");
        var result3 = CompatibilityChecker.IsVersionInRange("3.0.0", "<=2.0.0");

        result1.Should().BeTrue();
        result2.Should().BeTrue();
        result3.Should().BeFalse();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithMultipleConstraints_SatisfiesAll()
    {
        var result1 = CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0 <=2.0.0");
        var result2 = CompatibilityChecker.IsVersionInRange("0.5.0", ">=1.0.0 <=2.0.0");
        var result3 = CompatibilityChecker.IsVersionInRange("2.5.0", ">=1.0.0 <=2.0.0");

        result1.Should().BeTrue();
        result2.Should().BeFalse();
        result3.Should().BeFalse();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithGreaterThan_WorksCorrectly()
    {
        var result1 = CompatibilityChecker.IsVersionInRange("1.0.1", ">1.0.0");
        var result2 = CompatibilityChecker.IsVersionInRange("1.0.0", ">1.0.0");

        result1.Should().BeTrue();
        result2.Should().BeFalse();
    }

    [Fact]
    public void CompatibilityChecker_IsVersionInRange_WithLessThan_WorksCorrectly()
    {
        var result1 = CompatibilityChecker.IsVersionInRange("0.9.0", "<1.0.0");
        var result2 = CompatibilityChecker.IsVersionInRange("1.0.0", "<1.0.0");

        result1.Should().BeTrue();
        result2.Should().BeFalse();
    }

    // ──────────────────────── CompatibilityResult Edge Cases ────────────────────────

    [Fact]
    public void CompatibilityResult_NewInstance_InitializesCollections()
    {
        var result = new CompatibilityResult();

        result.Errors.Should().NotBeNull();
        result.Warnings.Should().NotBeNull();
    }

    [Fact]
    public void CompatibilityResult_CanAddErrors()
    {
        var result = new CompatibilityResult();
        result.Errors.Add("Test error");

        result.Errors.Should().ContainSingle();
    }

    [Fact]
    public void CompatibilityResult_CanAddWarnings()
    {
        var result = new CompatibilityResult();
        result.Warnings.Add("Test warning");

        result.Warnings.Should().ContainSingle();
    }

    [Fact]
    public void CompatibilityResult_CanSetCompatibilityStatus()
    {
        var result = new CompatibilityResult();
        result.IsCompatible = true;

        result.IsCompatible.Should().BeTrue();
    }

    [Fact]
    public void CompatibilityResult_CanStoreBothErrorsAndWarnings()
    {
        var result = new CompatibilityResult();
        result.Errors.Add("Test error");
        result.Warnings.Add("Test warning");

        result.Errors.Should().ContainSingle();
        result.Warnings.Should().ContainSingle();
    }

    // ──────────────────────── PackManifest Compatibility Edge Cases ────────────────────────

    [Fact]
    public void PackManifest_WithMinimalFields_CreatesValidManifest()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test",
            Version = "1.0.0",
            FrameworkVersion = "*",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        manifest.Should().NotBeNull();
        manifest.Id.Should().Be("test-pack");
    }

    [Fact]
    public void PackManifest_WithEmptyId_StoresEmpty()
    {
        var manifest = new PackManifest
        {
            Id = "",
            Name = "Test",
            Version = "1.0.0",
            FrameworkVersion = "*",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        manifest.Id.Should().BeEmpty();
    }

    [Fact]
    public void PackManifest_WithSpecialCharactersInId_StoresAsIs()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack_v1.0",
            Name = "Test",
            Version = "1.0.0",
            FrameworkVersion = "*",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        manifest.Id.Should().Be("test-pack_v1.0");
    }

    [Fact]
    public void PackManifest_WithNullVersion_StoresNull()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test",
            Version = null,
            FrameworkVersion = "*",
            GameVersion = "*",
            BepInExVersion = "*",
            UnityVersion = "*"
        };

        manifest.Version.Should().BeNull();
    }

    [Fact]
    public void PackManifest_WithComplexVersionConstraints_StoresCorrectly()
    {
        var manifest = new PackManifest
        {
            Id = "test-pack",
            Name = "Test",
            Version = "1.0.0",
            FrameworkVersion = ">=0.5.0 <1.0.0",
            GameVersion = ">=1.0.0 <=2.0.0",
            BepInExVersion = "*",
            UnityVersion = "2021.3.45f2"
        };

        manifest.FrameworkVersion.Should().Contain(">=");
        manifest.GameVersion.Should().Contain("<=");
    }
}
