#nullable enable
using System;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// REAL property-based tests using FsCheck generators and [Property] decorator.
    /// Validates version string round-tripping and pack manifest invariants
    /// across a large space of automatically generated test cases (hundreds per property).
    ///
    /// This is a POC demonstrating true property-based testing (vs the [Theory] parameterized tests elsewhere).
    /// FsCheck generates test cases automatically from generators and arbitrary instances.
    /// </summary>
    [Trait("Category", "Property")]
    public class RealFsCheckPropertyTests
    {
        /// <summary>
        /// Property: For any string that is non-null and reasonably-sized (< 100 chars),
        /// assignment to PackManifest.Version and retrieval preserves the string exactly.
        /// This validates that versions are stored as-is without mutation or validation.
        ///
        /// FsCheck will generate 100+ random strings automatically.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool VersionString_RoundTrips_Successfully(NonEmptyString version)
        {
            // Arrange: Use FsCheck's NonEmptyString to avoid trivial empty cases
            var manifest = new PackManifest
            {
                Id = "test-pack-" + Guid.NewGuid().ToString("N").Substring(0, 8),
                Name = "Test Pack",
                Version = version.Get
            };

            // Act & Assert: Verify round-trip
            var result = manifest.Version == version.Get;
            result.Should().BeTrue(because: "Version string should be preserved exactly on round-trip");
            return result;
        }

        /// <summary>
        /// Property: For any two pack manifests with different IDs but same version,
        /// the version field is independent and non-interfering across instances.
        /// This validates manifest field isolation (no static state corruption).
        ///
        /// Generates pairs of manifests and verifies they don't share state.
        /// </summary>
        [Property(MaxTest = 50)]
        public bool Two_Manifests_Have_Independent_Versions(NonEmptyString id1, NonEmptyString id2, NonEmptyString version)
        {
            // Arrange: Create two manifests with same version but different IDs
            var manifest1 = new PackManifest
            {
                Id = id1.Get,
                Name = "Pack 1",
                Version = version.Get
            };

            var manifest2 = new PackManifest
            {
                Id = id2.Get,
                Name = "Pack 2",
                Version = version.Get
            };

            // Act: Mutate manifest1's version
            manifest1.Version = "modified-1.0.0";

            // Assert: manifest2 should be unaffected (no shared state)
            var result = manifest2.Version == version.Get && manifest1.Version == "modified-1.0.0";
            result.Should().BeTrue(because: "Modifying one manifest's version should not affect others");
            return result;
        }

        /// <summary>
        /// Property: Framework version constraints (any string) are preserved as-is.
        /// This validates that FrameworkVersion field is not validated or mutated.
        ///
        /// FsCheck generates 50+ arbitrary strings.
        /// </summary>
        [Property(MaxTest = 50)]
        public bool FrameworkVersion_Preserved_As_String(NonEmptyString constraint)
        {
            // Arrange
            var manifest = new PackManifest
            {
                Id = "framework-test",
                Name = "Framework Test",
                FrameworkVersion = constraint.Get
            };

            // Act & Assert
            var result = manifest.FrameworkVersion == constraint.Get;
            result.Should().BeTrue(because: "Framework version constraint should be preserved as-is");
            return result;
        }
    }
}
