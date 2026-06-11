using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Dependencies;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class DependencyResultCoverageTests
    {
        [Fact]
        public void Success_ShouldPreserveLoadOrder_AndCreateEmptyErrors()
        {
            IReadOnlyList<PackManifest> loadOrder = new List<PackManifest>();

            DependencyResult result = DependencyResult.Success(loadOrder);

            result.IsSuccess.Should().BeTrue();
            result.LoadOrder.Should().BeSameAs(loadOrder);
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Failure_ShouldPreserveErrors_AndCreateEmptyLoadOrder()
        {
            IReadOnlyList<string> errors = new List<string>
            {
                "Missing dependency: base-pack"
            };

            DependencyResult result = DependencyResult.Failure(errors);

            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().BeSameAs(errors);
            result.LoadOrder.Should().BeEmpty();
        }
    }
}
