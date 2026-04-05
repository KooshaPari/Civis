#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests the SDK-to-ECS component mapping table.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class ComponentMapTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="ComponentMapTests"/>.</summary>
    public ComponentMapTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that the component map has entries.</summary>
    [Fact]
    public async Task ComponentMap_HasEntries()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return; // Game not available for integration testing

        ComponentMapResult result = await _fixture.Client.GetComponentMapAsync();

        result.Mappings.Should().NotBeEmpty("the component map should have registered mappings");
    }

    /// <summary>Verifies that filtering by SDK path works.</summary>
    [Fact]
    public async Task ComponentMap_CanFilterBySdkPath()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return; // Game not available for integration testing

        ComponentMapResult result = await _fixture.Client.GetComponentMapAsync(sdkPath: "unit");

        result.Mappings.Should().NotBeEmpty("filtering by 'unit' should return unit-related mappings");
        result.Mappings.Should().OnlyContain(
            m => m.SdkPath.StartsWith("unit", StringComparison.OrdinalIgnoreCase),
            "all mappings should match the 'unit' path prefix");
    }
}
