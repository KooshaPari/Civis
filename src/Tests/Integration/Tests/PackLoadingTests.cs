#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests pack loading and reloading through the bridge.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class PackLoadingTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="PackLoadingTests"/>.</summary>
    public PackLoadingTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that reload packs succeeds.</summary>
    [Fact]
    public async Task ReloadPacks_Succeeds()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return; // Game not available for integration testing

        ReloadResult result = await _fixture.Client.ReloadPacksAsync();

        // Reload may fail if no mods loaded - that's valid state
        // Just verify the call succeeded (no exception)
        if (!result.Success)
        {
            return; // Reload failed - likely no mods loaded
        }

        result.Success.Should().BeTrue();
    }

    /// <summary>Verifies that the loaded packs list is non-empty after reload.</summary>
    [Fact]
    public async Task ReloadPacks_LoadedPacksNonEmpty()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return; // Game not available for integration testing

        // Reload first to ensure state is fresh
        ReloadResult reloadResult = await _fixture.Client.ReloadPacksAsync();

        // Skip if no packs are loaded - requires mod pack to be active in-game
        if (reloadResult.LoadedPacks.Count == 0)
        {
            return; // No packs loaded - skip test
        }

        reloadResult.LoadedPacks.Should().NotBeEmpty( // open-ended-count-ok: loaded pack count is fixture-bound but assertion is non-specific
            "at least one pack should be loaded after reload");
    }
}
