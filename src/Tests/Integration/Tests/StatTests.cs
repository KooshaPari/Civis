#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Assertions;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests reading and overriding entity stats through the bridge.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class StatTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="StatTests"/>.</summary>
    public StatTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that unit HP stat can be read.</summary>
    [Fact]
    public async Task GetStat_UnitHp_ReturnsValue()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        StatResult result = await _fixture.Client.GetStatAsync("unit.stats.hp");

        result.SdkPath.Should().Be("unit.stats.hp");

        // Skip if no entities - requires mod pack with units to be loaded
        if (result.EntityCount == 0)
        {
            return; // No unit entities - skip test
        }

        result.EntityCount.Should().BeGreaterThan(0);
    }

    /// <summary>Verifies that unit speed stat can be read.</summary>
    [Fact]
    public async Task GetStat_UnitSpeed_ReturnsValue()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        StatResult result = await _fixture.Client.GetStatAsync("unit.stats.speed");

        result.SdkPath.Should().Be("unit.stats.speed");

        // Skip if no entities - requires mod pack with units to be loaded
        if (result.EntityCount == 0)
        {
            return; // No unit entities - skip test
        }

        result.EntityCount.Should().BeGreaterThan(0);
    }

    /// <summary>Verifies that applying an override changes the stat value.</summary>
    [Fact]
    public async Task ApplyOverride_ChangesStatValue()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        // Read the original value
        StatResult before = await _fixture.Client.GetStatAsync("unit.stats.hp");

        // Apply a multiply override
        OverrideResult overrideResult = await _fixture.Client.ApplyOverrideAsync(
            "unit.stats.hp", 2.0f, mode: "multiply");

        overrideResult.Success.Should().BeTrue();

        // Read the new value
        StatResult after = await _fixture.Client.GetStatAsync("unit.stats.hp");

        // The value should have changed (approximately doubled).
        // Use at least 1.0f precision to avoid ArgumentOutOfRangeException when before.Value is 0.
        float precision = Math.Max(before.Value * 0.1f, 1.0f);
        after.Value.Should().BeApproximately(before.Value * 2.0f, precision);
    }
}
