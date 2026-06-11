#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

public sealed class StartGameResultCoverageTests
{
    [Fact]
    public void Default_constructor_exposes_expected_defaults_and_allows_property_assignment()
    {
        StartGameResult result = new StartGameResult();

        result.Success.Should().BeFalse();
        result.Message.Should().BeEmpty();

        result.Success = true;
        result.Message = "loaded";

        result.Success.Should().BeTrue();
        result.Message.Should().Be("loaded");
    }
}
