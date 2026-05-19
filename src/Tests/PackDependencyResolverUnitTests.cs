using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class PackDependencyResolverUnitTests
    {
        private readonly PackDependencyResolver _resolver = new PackDependencyResolver();

        [Fact]
        public void ResolveDependencies_EmptyList_ReturnsSuccess()
        {
            // Arrange
            var available = new List<PackManifest>();
            var target = new PackManifest { Id = "pack-a", DependsOn = new List<string>() };

            // Act
            var result = _resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().HaveCount(1);
            result.LoadOrder[0].Id.Should().Be("pack-a");
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ResolveDependencies_LinearChain_ResolveInOrder()
        {
            // Arrange
            var packA = new PackManifest { Id = "pack-a", DependsOn = new List<string>() };
            var packB = new PackManifest { Id = "pack-b", DependsOn = new List<string> { "pack-a" } };
            var packC = new PackManifest { Id = "pack-c", DependsOn = new List<string> { "pack-b" } };

            var available = new List<PackManifest> { packA, packB, packC };

            // Act
            var result = _resolver.ResolveDependencies(available, packC);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().HaveCount(3);
            result.LoadOrder[0].Id.Should().Be("pack-a");
            result.LoadOrder[1].Id.Should().Be("pack-b");
            result.LoadOrder[2].Id.Should().Be("pack-c");
        }

        [Fact]
        public void ResolveDependencies_MissingDependency_ReturnsFailure()
        {
            // Arrange
            var available = new List<PackManifest>();
            var target = new PackManifest
            {
                Id = "pack-a",
                DependsOn = new List<string> { "pack-missing" }
            };

            // Act
            var result = _resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("pack-missing");
            result.LoadOrder.Should().BeEmpty();
        }

        [Fact]
        public void ResolveDependencies_TransitiveDependencies_ResolveCorrectly()
        {
            // Arrange
            var packA = new PackManifest { Id = "pack-a", DependsOn = new List<string>() };
            var packB = new PackManifest { Id = "pack-b", DependsOn = new List<string> { "pack-a" } };
            var packC = new PackManifest
            {
                Id = "pack-c",
                DependsOn = new List<string> { "pack-b" }
            };
            var packD = new PackManifest
            {
                Id = "pack-d",
                DependsOn = new List<string> { "pack-b", "pack-c" }
            };

            var available = new List<PackManifest> { packA, packB, packC, packD };

            // Act
            var result = _resolver.ResolveDependencies(available, packD);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().HaveCount(4);
            // Verify ordering: A before B, B before C, B and C before D
            var order = result.LoadOrder.Select(p => p.Id).ToList();
            order.IndexOf("pack-a").Should().BeLessThan(order.IndexOf("pack-b"));
            order.IndexOf("pack-b").Should().BeLessThan(order.IndexOf("pack-c"));
            order.IndexOf("pack-b").Should().BeLessThan(order.IndexOf("pack-d"));
            order.IndexOf("pack-c").Should().BeLessThan(order.IndexOf("pack-d"));
        }

        [Fact]
        public void ComputeLoadOrder_CircularDependency_ReturnsFailure()
        {
            // Arrange
            var packA = new PackManifest { Id = "pack-a", DependsOn = new List<string> { "pack-b" } };
            var packB = new PackManifest { Id = "pack-b", DependsOn = new List<string> { "pack-a" } };

            var packs = new List<PackManifest> { packA, packB };

            // Act
            var result = _resolver.ComputeLoadOrder(packs);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Should().Contain("Circular dependency");
            result.LoadOrder.Should().BeEmpty();
        }

        [Fact]
        public void DetectConflicts_NoConflicts_ReturnsEmpty()
        {
            // Arrange
            var packA = new PackManifest { Id = "pack-a", ConflictsWith = new List<string>() };
            var packB = new PackManifest { Id = "pack-b", ConflictsWith = new List<string>() };

            var active = new List<PackManifest> { packA, packB };

            // Act
            var conflicts = _resolver.DetectConflicts(active);

            // Assert
            conflicts.Should().BeEmpty();
        }

        [Fact]
        public void DetectConflicts_WithConflicts_ReturnsErrors()
        {
            // Arrange
            var packA = new PackManifest
            {
                Id = "pack-a",
                ConflictsWith = new List<string> { "pack-b" }
            };
            var packB = new PackManifest
            {
                Id = "pack-b",
                ConflictsWith = new List<string>()
            };

            var active = new List<PackManifest> { packA, packB };

            // Act
            var conflicts = _resolver.DetectConflicts(active);

            // Assert
            conflicts.Should().HaveCount(1);
            conflicts[0].Should().Contain("pack-a");
            conflicts[0].Should().Contain("pack-b");
        }

        [Fact]
        public void CheckFrameworkCompatibility_MatchingVersion_ReturnsTrue()
        {
            // Arrange
            var pack = new PackManifest
            {
                Id = "test-pack",
                FrameworkVersion = ">=0.20.0"
            };

            // Act
            var isCompatible = _resolver.CheckFrameworkCompatibility(pack, "0.20.0");

            // Assert
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void CheckFrameworkCompatibility_NoVersion_ReturnsTrue()
        {
            // Arrange
            var pack = new PackManifest
            {
                Id = "test-pack",
                FrameworkVersion = ""
            };

            // Act
            var isCompatible = _resolver.CheckFrameworkCompatibility(pack, "0.20.0");

            // Assert
            isCompatible.Should().BeTrue();
        }

        [Fact]
        public void CheckFrameworkCompatibility_NonMatchingVersion_ReturnsFalse()
        {
            // Arrange
            var pack = new PackManifest
            {
                Id = "test-pack",
                FrameworkVersion = ">=0.20.0"
            };

            // Act
            var isCompatible = _resolver.CheckFrameworkCompatibility(pack, "0.15.0");

            // Assert
            isCompatible.Should().BeFalse();
        }

        [Fact]
        public void ComputeLoadOrder_WithLoadOrderTiebreaker_SortsByLoadOrder()
        {
            // Arrange - Two packs with no dependencies, different LoadOrder values
            var packA = new PackManifest
            {
                Id = "pack-a",
                DependsOn = new List<string>(),
                LoadOrder = 200
            };
            var packB = new PackManifest
            {
                Id = "pack-b",
                DependsOn = new List<string>(),
                LoadOrder = 100
            };

            var packs = new List<PackManifest> { packA, packB };

            // Act
            var result = _resolver.ComputeLoadOrder(packs);

            // Assert
            result.IsSuccess.Should().BeTrue();
            // Lower LoadOrder (100) should come before higher (200)
            result.LoadOrder[0].Id.Should().Be("pack-b");
            result.LoadOrder[1].Id.Should().Be("pack-a");
        }

        [Fact]
        public void ResolveDependencies_MultipleMissingDependencies_ReturnsAllErrors()
        {
            // Arrange
            var available = new List<PackManifest>();
            var target = new PackManifest
            {
                Id = "pack-a",
                DependsOn = new List<string> { "pack-missing-1", "pack-missing-2" }
            };

            // Act
            var result = _resolver.ResolveDependencies(available, target);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
        }
    }
}
