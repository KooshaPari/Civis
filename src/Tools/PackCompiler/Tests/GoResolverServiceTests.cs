#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Xunit;
using FluentAssertions;
using DINOForge.Tools.PackCompiler.Services;
using DINOForge.SDK;

namespace DINOForge.Tools.PackCompiler.Tests
{
    /// <summary>
    /// Unit tests for GoResolverService with C# fallback validation.
    /// Tests both the service itself and the underlying C# resolution algorithm.
    /// </summary>
    [System.Diagnostics.CodeAnalysis.UnconditionalSuppressMessage("Trimming", "IL2026", Justification = "These tests intentionally invoke trim-sensitive resolver APIs in the non-trimmed test build.")]
    [Trait("Category", "Polyglot")]
    public class GoResolverServiceTests
    {
        private static PackManifest CreatePack(
            string id,
            string name,
            string version,
            List<string>? dependencies = null,
            int loadOrder = 0)
        {
            return new PackManifest
            {
                Id = id,
                Name = name,
                Version = version,
                FrameworkVersion = ">=0.1.0 <1.0.0",
                DependsOn = dependencies ?? new List<string>(),
                ConflictsWith = new List<string>(),
                LoadOrder = loadOrder,
                Type = "content"
            };
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_SinglePack_ReturnsItself()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            var pack = CreatePack("pack-a", "Pack A", "0.1.0");

            // Act
            var result = await resolver.ResolveDependenciesAsync(new List<PackManifest> { pack }, pack).ConfigureAwait(true);

            // Assert
            result.Should().ContainSingle().And.Contain("pack-a");
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_LinearChain_ReturnsInCorrectOrder()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            var packA = CreatePack("pack-a", "Pack A", "0.1.0");
            var packB = CreatePack("pack-b", "Pack B", "0.1.0", new List<string> { "pack-a" });
            var packC = CreatePack("pack-c", "Pack C", "0.1.0", new List<string> { "pack-b" });
            var available = new List<PackManifest> { packA, packB, packC };

            // Act
            var result = await resolver.ResolveDependenciesAsync(available, packC).ConfigureAwait(true);

            // Assert
            result.Should().HaveCount(3);
            result[0].Should().Be("pack-a");
            result[1].Should().Be("pack-b");
            result[2].Should().Be("pack-c");
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_DiamondDependency_ReturnsTopologicalOrder()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            //      C
            //     / \
            //    B   D
            //     \ /
            //      A
            var packA = CreatePack("pack-a", "Pack A", "0.1.0");
            var packB = CreatePack("pack-b", "Pack B", "0.1.0", new List<string> { "pack-a" });
            var packD = CreatePack("pack-d", "Pack D", "0.1.0", new List<string> { "pack-a" });
            var packC = CreatePack("pack-c", "Pack C", "0.1.0", new List<string> { "pack-b", "pack-d" });
            var available = new List<PackManifest> { packA, packB, packD, packC };

            // Act
            var result = await resolver.ResolveDependenciesAsync(available, packC).ConfigureAwait(true);

            // Assert
            result.Should().HaveCount(4);
            result.Should().Contain("pack-a"); // Must come first
            result.IndexOf("pack-a").Should().BeLessThan(result.IndexOf("pack-b"));
            result.IndexOf("pack-a").Should().BeLessThan(result.IndexOf("pack-d"));
            result.IndexOf("pack-b").Should().BeLessThan(result.IndexOf("pack-c"));
            result.IndexOf("pack-d").Should().BeLessThan(result.IndexOf("pack-c"));
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_MissingDependency_ThrowsArgumentException()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            var packA = CreatePack("pack-a", "Pack A", "0.1.0", new List<string> { "missing-pack" });
            var available = new List<PackManifest> { packA };

            // Act & Assert
            await Assert.ThrowsAsync<ArgumentException>(
                () => resolver.ResolveDependenciesAsync(available, packA)
            ).ConfigureAwait(true);
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_CircularDependency_ThrowsInvalidOperationException()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            var packA = CreatePack("pack-a", "Pack A", "0.1.0", new List<string> { "pack-c" });
            var packB = CreatePack("pack-b", "Pack B", "0.1.0", new List<string> { "pack-a" });
            var packC = CreatePack("pack-c", "Pack C", "0.1.0", new List<string> { "pack-b" });
            var available = new List<PackManifest> { packA, packB, packC };

            // Act & Assert
            await Assert.ThrowsAsync<InvalidOperationException>(
                () => resolver.ResolveDependenciesAsync(available, packA)
            ).ConfigureAwait(true);
        }

        [Fact]
        public async Task ResolveWithCSharpFallback_RespectLoadOrder_WhenNoDependencies()
        {
            // Arrange
            var resolver = new GoResolverService("/nonexistent/path/to/resolver");
            var packA = CreatePack("pack-a", "Pack A", "0.1.0", loadOrder: 10);
            var packB = CreatePack("pack-b", "Pack B", "0.1.0", loadOrder: 5);
            var packC = CreatePack("pack-c", "Pack C", "0.1.0", loadOrder: 0);
            var available = new List<PackManifest> { packA, packB, packC };

            // Act
            var result = await resolver.ResolveDependenciesAsync(available, packC).ConfigureAwait(true);

            // Assert
            // All three are independent, so load order should determine order
            result.Should().HaveCount(3);
            result.IndexOf("pack-c").Should().BeLessThan(result.IndexOf("pack-b"));
            result.IndexOf("pack-b").Should().BeLessThan(result.IndexOf("pack-a"));
        }
    }
}
