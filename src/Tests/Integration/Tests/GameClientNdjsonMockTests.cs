#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Live game bridge and <see cref="MockGameBridgeServer"/> speak NDJSON (line-delimited JSON).
/// <see cref="GameClientOptions.UseMessageFraming"/> defaults to <c>true</c> (length-prefix);
/// tools such as GameControlCli set <c>false</c> to match the server.
/// </summary>
[Trait("Category", "Integration")]
[Trait("Category", "MockGameServer")]
public class GameClientNdjsonMockTests
{
    /// <summary>
    /// GIVEN a MockGameBridgeServer (NDJSON line protocol)
    /// WHEN a GameClient uses the same options as GameControlCli (UseMessageFraming=false)
    /// THEN StatusAsync succeeds without protocol/bridge errors
    /// </summary>
    [Fact]
    public async Task StatusAsync_GivenNdjsonClientOptions_ConnectsToMockBridge()
    {
        var server = new MockGameBridgeServer();
        await server.StartAsync().ConfigureAwait(true);

        try
        {
            var options = new GameClientOptions
            {
                PipeName = server.PipeName,
                UseMessageFraming = false,
                ReadTimeoutMs = 5000,
                ConnectTimeoutMs = 5000,
            };

            using var client = new GameClient(options);
            await client.ConnectAsync().ConfigureAwait(true);

            GameStatus status = await client.StatusAsync().ConfigureAwait(true);

            status.Running.Should().BeTrue();
            status.WorldReady.Should().BeFalse("mock starts unloaded until ReloadPacks");
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync().ConfigureAwait(true);
        }
    }
}
