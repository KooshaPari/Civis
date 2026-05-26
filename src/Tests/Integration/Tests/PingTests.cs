#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests that the game bridge responds to ping requests.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class PingTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="PingTests"/>.</summary>
    public PingTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that ping returns pong.</summary>
    [Fact]
    public async Task Ping_ReturnsPong()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        PingResult result = await _fixture.Client.PingAsync();

        result.Pong.Should().BeTrue();
    }

    /// <summary>Verifies that version string is present.</summary>
    [Fact]
    public async Task Ping_VersionIsNotEmpty()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        PingResult result = await _fixture.Client.PingAsync();

        result.Version.Should().NotBeNullOrEmpty();
    }
}
