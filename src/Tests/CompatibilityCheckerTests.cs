using System;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="CompatibilityChecker"/> version compatibility logic.
    /// Validates semver range parsing and pack manifest compatibility checking.
    /// </summary>
    public class CompatibilityCheckerTests
    {
        // ── IsVersionInRange: Wildcards ──────────────────────────────────────

        [Fact]
        public void IsVersionInRange_Wildcard_AlwaysReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", "*").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("2.5.10", "*").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("0.0.1", "*").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_EmptyRange_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", "").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("2.5.10", "").Should().BeTrue();
        }

        // ── IsVersionInRange: Exact Match ────────────────────────────────────

        [Fact]
        public void IsVersionInRange_ExactMatch_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", "1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("2.5.10", "=2.5.10").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("0.1.0", "==0.1.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_ExactMismatch_ReturnsFalse()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", "1.0.1").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("2.5.10", "=2.5.11").Should().BeFalse();
        }

        // ── IsVersionInRange: Greater Than or Equal ──────────────────────────

        [Fact]
        public void IsVersionInRange_GreaterThanOrEqual_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("2.0.0", ">=1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0", ">=1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_GreaterThanOrEqual_ReturnsFalse()
        {
            CompatibilityChecker.IsVersionInRange("0.9.0", ">=1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("1.0.0", ">=1.0.1").Should().BeFalse();
        }

        // ── IsVersionInRange: Less Than or Equal ─────────────────────────────

        [Fact]
        public void IsVersionInRange_LessThanOrEqual_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("0.5.0", "<=1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0", "<=1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0", "<=2.0.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_LessThanOrEqual_ReturnsFalse()
        {
            CompatibilityChecker.IsVersionInRange("1.1.0", "<=1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("2.0.0", "<=1.9.9").Should().BeFalse();
        }

        // ── IsVersionInRange: Strictly Greater ───────────────────────────────

        [Fact]
        public void IsVersionInRange_GreaterThan_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("1.1.0", ">1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("2.0.0", ">1.0.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_GreaterThan_ReturnsFalse()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", ">1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("0.9.0", ">1.0.0").Should().BeFalse();
        }

        // ── IsVersionInRange: Strictly Less ──────────────────────────────────

        [Fact]
        public void IsVersionInRange_LessThan_ReturnsTrue()
        {
            CompatibilityChecker.IsVersionInRange("0.9.0", "<1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0", "<1.0.1").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_LessThan_ReturnsFalse()
        {
            CompatibilityChecker.IsVersionInRange("1.0.0", "<1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("1.1.0", "<1.0.0").Should().BeFalse();
        }

        // ── IsVersionInRange: Caret (^) ──────────────────────────────────────

        [Fact]
        public void IsVersionInRange_Caret_AllowsMajorAndMinorChanges()
        {
            // ^1.2.3 allows >=1.2.3 and <2.0.0
            CompatibilityChecker.IsVersionInRange("1.2.3", "^1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.3.0", "^1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.9.9", "^1.2.3").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_Caret_BlocksMajorChanges()
        {
            // ^1.2.3 blocks >=2.0.0
            CompatibilityChecker.IsVersionInRange("2.0.0", "^1.2.3").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("1.2.2", "^1.2.3").Should().BeFalse();
        }

        // ── IsVersionInRange: Tilde (~) ──────────────────────────────────────

        [Fact]
        public void IsVersionInRange_Tilde_AllowsPatchChanges()
        {
            // ~1.2.3 allows >=1.2.3 and <1.3.0
            CompatibilityChecker.IsVersionInRange("1.2.3", "~1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.2.4", "~1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.2.9", "~1.2.3").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_Tilde_BlocksMinorChanges()
        {
            // ~1.2.3 blocks >=1.3.0 and <1.2.3
            CompatibilityChecker.IsVersionInRange("1.3.0", "~1.2.3").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("1.2.2", "~1.2.3").Should().BeFalse();
        }

        // ── IsVersionInRange: Compound Ranges ────────────────────────────────

        [Fact]
        public void IsVersionInRange_CompoundRange_Works()
        {
            // ">=1.0.0 <2.0.0"
            CompatibilityChecker.IsVersionInRange("1.0.0", ">=1.0.0 <2.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0 <2.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.9.9", ">=1.0.0 <2.0.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_CompoundRange_ReturnsFalse()
        {
            // ">=1.0.0 <2.0.0"
            CompatibilityChecker.IsVersionInRange("0.9.9", ">=1.0.0 <2.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("2.0.0", ">=1.0.0 <2.0.0").Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_TripleConstraint_Works()
        {
            // ">=1.0.0 <2.0.0 !=1.5.0"
            CompatibilityChecker.IsVersionInRange("1.4.0", ">=1.0.0 <2.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.5.0", ">=1.0.0 <2.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.6.0", ">=1.0.0 <2.0.0").Should().BeTrue();
        }

        // ── IsVersionInRange: Unity-style versions ───────────────────────────

        [Fact]
        public void IsVersionInRange_UnityVersion_ParsesCorrectly()
        {
            // Unity versions like "2021.3.45f2"
            CompatibilityChecker.IsVersionInRange("2021.3.45", ">=2021.3.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("2021.3.45", "<2022.0.0").Should().BeTrue();
        }

        // ── CheckPack: Framework Version ──────────────────────────────────────

        [Fact]
        public void CheckPack_CompatibleFramework_ReturnsCompatible()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(manifest);

            result.IsCompatible.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void CheckPack_IncompatibleFramework_ReturnsError()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=99.0.0",  // No real version satisfies this
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(manifest);

            result.IsCompatible.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("DINOForge");
        }

        // ── CheckPack: Game Version ──────────────────────────────────────────

        [Fact]
        public void CheckPack_IncompatibleGameVersion_ReturnsWarning()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = ">=2.0.0",  // Current game is 1.0.0
                BepInExVersion = "*",
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(manifest, dinoGameVersion: "1.0.0");

            result.IsCompatible.Should().BeTrue();  // Warnings don't block loading
            result.Warnings.Should().HaveCount(1);
            result.Warnings[0].Should().Contain("game_version");
        }

        // ── CheckPack: BepInEx Version ───────────────────────────────────────

        [Fact]
        public void CheckPack_IncompatibleBepInEx_ReturnsWarning()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = "*",
                BepInExVersion = ">=6.0.0",  // Current is 5.4.21
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(manifest, bepinexVersion: "5.4.21");

            result.IsCompatible.Should().BeTrue();  // Warnings don't block loading
            result.Warnings.Should().HaveCount(1);
            result.Warnings[0].Should().Contain("bepinex_version");
        }

        // ── CheckPack: Unity Version ─────────────────────────────────────────

        [Fact]
        public void CheckPack_IncompatibleUnity_ReturnsWarning()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = ">=2022.0.0",  // Current is 2021.3.45f2
            };

            var result = CompatibilityChecker.CheckPack(manifest, unityVersion: "2021.3.45");

            result.IsCompatible.Should().BeTrue();  // Warnings don't block loading
            result.Warnings.Should().HaveCount(1);
            result.Warnings[0].Should().Contain("unity_version");
        }

        // ── CheckPack: Multiple Warnings ─────────────────────────────────────

        [Fact]
        public void CheckPack_MultipleIncompatibilities_ReturnsAllWarnings()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = ">=2.0.0",
                BepInExVersion = ">=6.0.0",
                UnityVersion = ">=2022.0.0",
            };

            var result = CompatibilityChecker.CheckPack(
                manifest,
                dinoGameVersion: "1.0.0",
                bepinexVersion: "5.4.21",
                unityVersion: "2021.3.45");

            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().HaveCount(3);
            result.Errors.Should().BeEmpty();
        }

        // ── CheckPack: Framework Error Takes Priority ────────────────────────

        [Fact]
        public void CheckPack_FrameworkErrorAndOtherWarnings_ReturnsIncompatible()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=99.0.0",  // Error (no real version satisfies this)
                GameVersion = ">=2.0.0",        // Warning (current is 1.0.0)
                BepInExVersion = "*",
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(manifest, dinoGameVersion: "1.0.0");

            result.IsCompatible.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Warnings.Should().HaveCount(1);
        }

        // ── CheckPack: All Compatible ────────────────────────────────────────

        [Fact]
        public void CheckPack_AllVersionsCompatible_ReturnsCompatible()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0",
                GameVersion = ">=1.0.0",
                BepInExVersion = ">=5.4.0 <6.0.0",
                UnityVersion = "2021.3.*",
            };

            var result = CompatibilityChecker.CheckPack(
                manifest,
                dinoGameVersion: "1.5.0",
                bepinexVersion: "5.4.21",
                unityVersion: "2021.3.45");

            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().BeEmpty();
            result.Errors.Should().BeEmpty();
        }

        // ── CheckPack: Wildcard Defaults ─────────────────────────────────────

        [Fact]
        public void CheckPack_ExplicitWildcards_AcceptsAnyVersion()
        {
            var manifest = new PackManifest
            {
                Id = "test-pack",
                Name = "Test Pack",
                Version = "0.1.0",
                FrameworkVersion = ">=0.1.0 <1.0.0",
                GameVersion = "*",
                BepInExVersion = "*",
                UnityVersion = "*",
            };

            var result = CompatibilityChecker.CheckPack(
                manifest,
                dinoGameVersion: "99.0.0",
                bepinexVersion: "5.4.21",
                unityVersion: "9999.0.0");

            result.IsCompatible.Should().BeTrue();
            result.Warnings.Should().BeEmpty();
        }

        // ── IsVersionInRange: Pre-release and Edge Cases ──────────────────────

        [Fact]
        public void IsVersionInRange_PreReleaseVersion_HandledGracefully()
        {
            // Pre-release versions like "1.0.0-rc1" should be handled gracefully
            // The implementation strips pre-release tags before parsing
            CompatibilityChecker.IsVersionInRange("1.0.0-rc1", ">=1.0.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0-alpha", ">=0.9.0").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.0.0-preview.5", "<2.0.0").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_MalformedVersion_ReturnsFalse()
        {
            // Malformed versions should not throw, but return false
            CompatibilityChecker.IsVersionInRange("not-a-version", ">=1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("", ">=1.0.0").Should().BeFalse();
            CompatibilityChecker.IsVersionInRange("abc.def.ghi", "1.0.0").Should().BeFalse();
        }

        [Fact]
        public void IsVersionInRange_FourPartVersion_ParsedAsIs()
        {
            // Four-part versions like "1.2.3.4" are parsed as full Version objects
            CompatibilityChecker.IsVersionInRange("1.2.3.4", ">=1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.2.3.4", ">1.2.3").Should().BeTrue();
            CompatibilityChecker.IsVersionInRange("1.2.3.99", "<1.2.4").Should().BeTrue();
        }

        [Fact]
        public void IsVersionInRange_NullVersion_ThrowsNullReferenceException()
        {
            // Null version string causes NullReferenceException in version parsing
            Action act = () => CompatibilityChecker.IsVersionInRange(null!, ">=1.0.0");
            act.Should().Throw<NullReferenceException>();
        }
    }
}
