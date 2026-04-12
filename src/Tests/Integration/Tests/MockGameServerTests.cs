#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for MockGameBridgeServer.
/// Verifies that the server correctly routes JSON-RPC calls to FakeGameBridge,
/// handles protocol errors, and supports concurrent client connections.
/// </summary>
[Trait("Category", "MockGameServer")]
[Trait("Category", "Integration")]
public class MockGameServerTests
{
    // ─────────────────────────────────────────────────────────────────────────────
    // Server lifecycle tests
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// GIVEN a MockGameBridgeServer
    /// WHEN the server starts
    /// THEN PipeName is exposed and accessible
    /// </summary>
    [Fact]
    public async Task Server_Starts_ExposesPipeName()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            // Act
            string pipeName = server.PipeName;

            // Assert
            pipeName.Should().NotBeNullOrWhiteSpace();
            pipeName.Should().Be("dinoforge-game-bridge");
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a MockGameBridgeServer
    /// WHEN the server stops
    /// THEN StopAsync completes without errors
    /// </summary>
    [Fact]
    public async Task Server_Stops_WithoutErrors()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        // Act
        Func<Task> stopAction = async () => await server.StopAsync();

        // Assert
        await stopAction.Should().NotThrowAsync();
        await server.DisposeAsync();
    }

    /// <summary>
    /// GIVEN a MockGameBridgeServer with custom pipe name
    /// WHEN initialized
    /// THEN PipeName reflects the custom name
    /// </summary>
    [Fact]
    public async Task Server_WithCustomPipeName_UsesThatName()
    {
        // Arrange
        string customName = "test-pipe-" + Guid.NewGuid().ToString("N")[..8];
        var server = new MockGameBridgeServer(customName);

        try
        {
            // Act
            await server.StartAsync();

            // Assert
            server.PipeName.Should().Be(customName);
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // GameClient connectivity tests
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// GIVEN a running MockGameBridgeServer
    /// WHEN a GameClient connects
    /// THEN the connection succeeds
    /// </summary>
    [Fact]
    public async Task GameClient_CanConnect_ToMockServer()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });

            // Act
            await client.ConnectAsync();

            // Assert
            client.IsConnected.Should().BeTrue();
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient connected to the mock server
    /// WHEN calling PingAsync
    /// THEN the server responds with Pong = true
    /// </summary>
    [Fact]
    public async Task GameClient_Ping_ReturnsValidResponse()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();

            // Act
            PingResult result = await client.PingAsync();

            // Assert
            result.Pong.Should().BeTrue();
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Bridge method tests
    // ─────────────────────────────────────────────────────────────────────────────


    /// <summary>
    /// GIVEN a GameClient
    /// WHEN ReloadPacksAsync is called
    /// THEN the server loads packs and returns success
    /// </summary>
    [Fact]
    public async Task GameClient_ReloadPacks_LoadsWarfareStarwars()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();

            // Act
            ReloadResult result = await client.ReloadPacksAsync();

            // Assert
            result.Success.Should().BeTrue();
            result.LoadedPacks.Should().Contain("warfare-starwars");
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient with packs loaded
    /// WHEN calling StatusAsync
    /// THEN entity count is greater than zero
    /// </summary>
    [Fact]
    public async Task GameClient_Status_ReportsEntitiesAfterPackLoad()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();

            // Act
            GameStatus status = await client.StatusAsync();

            // Assert
            status.WorldReady.Should().BeTrue();
            status.EntityCount.Should().BeGreaterThan(0);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient with packs loaded
    /// WHEN ApplyOverrideAsync is called
    /// THEN the override succeeds with ModifiedCount > 0
    /// </summary>
    [Fact]
    public async Task GameClient_ApplyOverride_ModifiesEntities()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();

            // Act
            OverrideResult result = await client.ApplyOverrideAsync("unit.stats.hp", 200f, "override", "rep_clone_trooper");

            // Assert
            result.Success.Should().BeTrue();
            result.ModifiedCount.Should().BeGreaterThan(0);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient
    /// WHEN GetStatAsync is called with "unit.stats.hp"
    /// THEN the default value (100.0) is returned
    /// </summary>
    [Fact]
    public async Task GameClient_GetStat_ReturnsDefaultValue()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();

            // Act
            StatResult result = await client.GetStatAsync("unit.stats.hp");

            // Assert
            result.Value.Should().Be(100f);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient with override applied
    /// WHEN GetStatAsync is called
    /// THEN the overridden value is returned
    /// </summary>
    [Fact]
    public async Task GameClient_GetStat_ReturnsOverriddenValue()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();
            await client.ApplyOverrideAsync("unit.stats.hp", 999f, "override", "rep_clone_trooper");

            // Act
            StatResult result = await client.GetStatAsync("unit.stats.hp");

            // Assert
            result.Value.Should().BeApproximately(999f, 0.01f);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient
    /// WHEN GetCatalogAsync is called after pack load
    /// THEN a catalog with units is returned
    /// </summary>
    [Fact]
    public async Task GameClient_GetCatalog_ReturnsCatalogAfterPackLoad()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();

            // Act
            CatalogSnapshot catalog = await client.GetCatalogAsync();

            // Assert
            catalog.Units.Should().NotBeEmpty();
            int totalUnits = catalog.Units.Sum(u => u.EntityCount);
            totalUnits.Should().Be(28, "warfare-starwars has 28 units (14 Republic + 14 CIS)");
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Concurrent connection tests
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// GIVEN a MockGameBridgeServer
    /// WHEN multiple GameClients connect concurrently and call methods
    /// THEN all requests complete successfully
    /// </summary>
    [Fact]
    public async Task GameClient_MultipleConcurrentConnections_AllSucceed()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var tasks = new List<Task>();

            for (int i = 0; i < 5; i++)
            {
                tasks.Add(Task.Run(async () =>
                {
                    var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
                    await client.ConnectAsync();

                    try
                    {
                        PingResult ping = await client.PingAsync();
                        ping.Pong.Should().BeTrue();

                        GameStatus status = await client.StatusAsync();
                        status.Running.Should().BeTrue();
                    }
                    finally
                    {
                        client.Disconnect();
                    }
                }));
            }

            // Act & Assert
            await Task.WhenAll(tasks);
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Protocol and message tracking tests
    // ─────────────────────────────────────────────────────────────────────────────


    /// <summary>
    /// GIVEN a GameClient calling a non-existent method
    /// WHEN the method is invoked
    /// THEN the server returns a method not found error
    /// </summary>
    [Fact]
    public async Task GameClient_InvalidMethod_ReturnsError()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();

            // Act & Assert
            await Assert.ThrowsAsync<GameClientException>(async () =>
                await client.InvokeBridgeMethodAsync("nonexistentMethod", new { }));
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Override mode tests
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// GIVEN a GameClient
    /// WHEN stat overrides are applied with "add" mode
    /// THEN values accumulate correctly
    /// </summary>
    [Fact]
    public async Task GameClient_Override_AddMode_AccumulatesValues()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();

            // Act
            await client.ApplyOverrideAsync("unit.stats.hp", 50f, "add", null);
            StatResult result = await client.GetStatAsync("unit.stats.hp");

            // Assert (100 + 50 = 150)
            result.Value.Should().Be(150f);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a GameClient
    /// WHEN stat overrides are applied with "multiply" mode
    /// THEN values multiply correctly
    /// </summary>
    [Fact]
    public async Task GameClient_Override_MultiplyMode_MultipliesValues()
    {
        // Arrange
        var server = new MockGameBridgeServer();
        await server.StartAsync();

        try
        {
            var client = new GameClient(new GameClientOptions { PipeName = server.PipeName });
            await client.ConnectAsync();
            await client.ReloadPacksAsync();

            // Act
            await client.ApplyOverrideAsync("unit.stats.hp", 2.0f, "multiply", null);
            StatResult result = await client.GetStatAsync("unit.stats.hp");

            // Assert (100 * 2.0 = 200)
            result.Value.Should().Be(200f);
            client.Disconnect();
        }
        finally
        {
            await server.DisposeAsync();
        }
    }
}
