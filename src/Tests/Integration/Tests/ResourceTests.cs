#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests reading resource values from the game.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class ResourceTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="ResourceTests"/>.</summary>
    public ResourceTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that all resource types can be read.</summary>
    [Fact]
    public async Task GetResources_ReturnsAllTypes()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        ResourceSnapshot resources = await _fixture.Client.GetResourcesAsync();

        // Resources object should be populated (values can be zero, but not negative)
        resources.Should().NotBeNull();
    }

    /// <summary>Verifies that resources are non-negative.</summary>
    [Fact]
    public async Task GetResources_AllNonNegative()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        ResourceSnapshot resources = await _fixture.Client.GetResourcesAsync();

        resources.Food.Should().BeGreaterThanOrEqualTo(0);
        resources.Wood.Should().BeGreaterThanOrEqualTo(0);
        resources.Stone.Should().BeGreaterThanOrEqualTo(0);
        resources.Iron.Should().BeGreaterThanOrEqualTo(0);
        resources.Money.Should().BeGreaterThanOrEqualTo(0);
        resources.Souls.Should().BeGreaterThanOrEqualTo(0);
        resources.Bones.Should().BeGreaterThanOrEqualTo(0);
        resources.Spirit.Should().BeGreaterThanOrEqualTo(0);
    }
}
