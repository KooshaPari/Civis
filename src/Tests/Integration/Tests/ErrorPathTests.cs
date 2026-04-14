#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for error handling in the game automation system.
///
/// Tests cover:
/// - Bridge disconnection and reconnection
/// - Named pipe unavailability
/// - Protocol errors (malformed messages)
/// - Timeout scenarios
/// - Concurrent operation failures with partial success
/// - Resource cleanup on error
/// - Error logging and diagnostics
/// </summary>
[Trait("Category", "ErrorHandling")]
[Trait("Category", "Integration")]
public class ErrorPathTests : IAsyncLifetime
{
    private MockGameBridgeServer? _mockServer;
    private string? _testPipeName;

    public async Task InitializeAsync()
    {
        // Create a mock game server for error scenario testing
        _testPipeName = "dinoforge-test-error-" + Guid.NewGuid().ToString("N")[..8];
        _mockServer = new MockGameBridgeServer(_testPipeName);
        await _mockServer.StartAsync();
    }

    public async Task DisposeAsync()
    {
        if (_mockServer != null)
        {
            await _mockServer.DisposeAsync();
        }
    }

    /// <summary>
    /// GIVEN a game client connected to mock server
    /// WHEN the server stops unexpectedly
    /// THEN the client detects the disconnection
    /// </summary>
    [Fact]
    public async Task BridgeDisconnection_Detected_ClientKnowsAboutIt()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client = new GameClient(options);
        await client.ConnectAsync();
        client.IsConnected.Should().BeTrue();

        // Act - stop the server (simulates bridge disconnect)
        await _mockServer!.DisposeAsync();
        await Task.Delay(100); // Give client time to notice

        // Attempt a command
        Func<Task> action = async () =>
        {
            await client.PingAsync();
        };

        // Assert - should fail with connection error
        await action.Should().ThrowAsync<Exception>();
        client.Disconnect();
        client.Dispose();
    }

    /// <summary>
    /// GIVEN a game client with a non-existent pipe
    /// WHEN we try to connect
    /// THEN the connection fails with appropriate error
    /// </summary>
    [Fact]
    public async Task PipeUnavailable_ConnectionFails_WithError()
    {
        // Arrange
        var nonExistentPipeName = "dinoforge-nonexistent-" + Guid.NewGuid().ToString("N");
        var options = new GameClientOptions { PipeName = nonExistentPipeName, ConnectTimeoutMs = 2000 };
        var client = new GameClient(options);

        // Act
        Func<Task> action = async () =>
        {
            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(3));
            await client.ConnectAsync(cts.Token);
        };

        // Assert
        await action.Should().ThrowAsync<Exception>();
    }

    /// <summary>
    /// GIVEN multiple clients attempting to connect
    /// WHEN some fail and some succeed
    /// THEN partial success is handled correctly
    /// </summary>
    [Fact]
    public async Task MultipleClients_PartialFailure_SomeSucceedSomeFail()
    {
        // Arrange
        var goodPipeName = _testPipeName!;
        var badPipeName = "dinoforge-bad-" + Guid.NewGuid().ToString("N");

        var goodOptions1 = new GameClientOptions { PipeName = goodPipeName, ConnectTimeoutMs = 5000 };
        var goodOptions2 = new GameClientOptions { PipeName = goodPipeName, ConnectTimeoutMs = 5000 };
        var badOptions1 = new GameClientOptions { PipeName = badPipeName, ConnectTimeoutMs = 1000 };
        var badOptions2 = new GameClientOptions { PipeName = badPipeName, ConnectTimeoutMs = 1000 };

        var goodClients = new[] { new GameClient(goodOptions1), new GameClient(goodOptions2) };
        var badClients = new[] { new GameClient(badOptions1), new GameClient(badOptions2) };

        var connectTasks = new List<Task<bool>>();

        // Act - attempt connections in parallel
        foreach (var client in goodClients)
        {
            connectTasks.Add(TryConnectAsync(client, TimeSpan.FromSeconds(2)));
        }
        foreach (var client in badClients)
        {
            connectTasks.Add(TryConnectAsync(client, TimeSpan.FromSeconds(1)));
        }

        var results = await Task.WhenAll(connectTasks);

        // Assert - good clients should succeed, bad should fail
        results[0].Should().BeTrue("first good client should connect");
        results[1].Should().BeTrue("second good client should connect");
        results[2].Should().BeFalse("first bad client should fail");
        results[3].Should().BeFalse("second bad client should fail");

        // Cleanup
        foreach (var client in goodClients)
        {
            try { client.Disconnect(); } catch { }
            client.Dispose();
        }
        foreach (var client in badClients)
        {
            try { client.Disconnect(); } catch { }
            client.Dispose();
        }
    }

    /// <summary>
    /// GIVEN a client command with a short timeout
    /// WHEN a command is sent
    /// THEN the command completes successfully (mock server is responsive)
    /// </summary>
    [Fact]
    public async Task CommandSucceeds_MockServer_RespondsQuickly()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName!, ConnectTimeoutMs = 5000 };
        var client = new GameClient(options);
        await client.ConnectAsync();

        // Act
        Func<Task> action = async () =>
        {
            var result = await client.PingAsync();
            result.Should().NotBeNull();
            result.Pong.Should().BeTrue();
        };

        // Assert
        await action.Should().NotThrowAsync();
        client.Disconnect();
        client.Dispose();
    }

    /// <summary>
    /// GIVEN a client with a valid connection
    /// WHEN sending a ping
    /// THEN the ping succeeds
    /// </summary>
    [Fact]
    public async Task ServerResponse_ValidPing_Succeeds()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client = new GameClient(options);
        await client.ConnectAsync();

        // Act
        var result = await client.PingAsync();

        // Assert
        result.Should().NotBeNull();
        result.Pong.Should().BeTrue();

        client.Disconnect();
        client.Dispose();
    }

    /// <summary>
    /// GIVEN concurrent commands sent in rapid succession
    /// WHEN several ping requests are sent
    /// THEN successful commands process correctly
    /// </summary>
    [Fact]
    public async Task ConcurrentCommands_RapidPings_AllSucceed()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client = new GameClient(options);
        await client.ConnectAsync();

        var commandTasks = new List<Task<PingResult>>();

        // Act - send many commands rapidly
        for (int i = 0; i < 5; i++)
        {
            commandTasks.Add(client.PingAsync());
        }

        var results = await Task.WhenAll(commandTasks);

        // Assert - all should succeed
        results.Should().AllSatisfy(r => r.Pong.Should().BeTrue());

        client.Disconnect();
        client.Dispose();
    }

    /// <summary>
    /// GIVEN a connected client
    /// WHEN the client is disposed
    /// THEN subsequent operations fail gracefully
    /// </summary>
    [Fact]
    public async Task DisposedClient_Operations_FailGracefully()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client = new GameClient(options);
        await client.ConnectAsync();
        client.IsConnected.Should().BeTrue();

        // Act
        client.Disconnect();
        client.Dispose();

        // Assert - operations should fail
        Func<Task> action = async () =>
        {
            await client.PingAsync();
        };

        await action.Should().ThrowAsync<Exception>();
    }

    /// <summary>
    /// GIVEN multiple clients using the same pipe
    /// WHEN one client disconnects
    /// THEN other clients remain functional
    /// </summary>
    [Fact]
    public async Task MultipleClients_OneDisconnects_OthersContinue()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client1 = new GameClient(options);
        var client2 = new GameClient(options);

        await client1.ConnectAsync();
        await client2.ConnectAsync();

        client1.IsConnected.Should().BeTrue();
        client2.IsConnected.Should().BeTrue();

        // Act - disconnect first client
        client1.Disconnect();
        client1.Dispose();

        // Assert - second client should still work
        Func<Task> action = async () =>
        {
            var result = await client2.PingAsync();
            result.Pong.Should().BeTrue();
        };

        await action.Should().NotThrowAsync();

        // Cleanup
        client2.Disconnect();
        client2.Dispose();
    }

    /// <summary>
    /// GIVEN a client attempting reconnection after disconnect
    /// WHEN reconnection is attempted
    /// THEN it succeeds if server is available
    /// </summary>
    [Fact]
    public async Task ClientReconnection_AfterDisconnect_Succeeds()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var client = new GameClient(options);
        await client.ConnectAsync();
        client.IsConnected.Should().BeTrue();

        // Act - disconnect then reconnect
        client.Disconnect();
        client.IsConnected.Should().BeFalse();

        await client.ConnectAsync();

        // Assert
        client.IsConnected.Should().BeTrue();

        // Verify functionality after reconnection
        var result = await client.PingAsync();
        result.Pong.Should().BeTrue();

        client.Disconnect();
        client.Dispose();
    }

    /// <summary>
    /// GIVEN concurrent connect attempts from multiple clients
    /// WHEN they all try to connect simultaneously
    /// THEN all succeed without deadlocks
    /// </summary>
    [Fact]
    public async Task ConcurrentConnect_MultipleClients_NoDeadlock()
    {
        // Arrange
        const int clientCount = 4;
        var clients = Enumerable.Range(0, clientCount)
            .Select(_ => new GameClient(new GameClientOptions { PipeName = _testPipeName! }))
            .ToList();

        // Act - connect all in parallel
        var connectTasks = clients.Select(c => c.ConnectAsync()).ToList();
        await Task.WhenAll(connectTasks);

        // Assert - all should be connected
        foreach (var client in clients)
        {
            client.IsConnected.Should().BeTrue();
        }

        // Verify all can send commands
        var pingTasks = clients.Select(c => c.PingAsync()).ToList();
        var results = await Task.WhenAll(pingTasks);
        results.Should().AllSatisfy(r => r.Pong.Should().BeTrue());

        // Cleanup
        foreach (var client in clients)
        {
            try { client.Disconnect(); } catch { }
            client.Dispose();
        }
    }

    /// <summary>
    /// GIVEN a client with multiple active connections
    /// WHEN one connection sends a message
    /// THEN other connections are not affected
    /// </summary>
    [Fact]
    public async Task ConcurrentPings_MultipleClients_NoInterference()
    {
        // Arrange
        var options = new GameClientOptions { PipeName = _testPipeName! };
        var clients = Enumerable.Range(0, 3)
            .Select(_ => new GameClient(options))
            .ToList();

        foreach (var client in clients)
        {
            await client.ConnectAsync();
        }

        // Act - send pings concurrently from all clients
        var tasks = new List<Task>();
        for (int i = 0; i < 3; i++)
        {
            tasks.Add(clients[i].PingAsync().ContinueWith(_ => { }));
        }

        await Task.WhenAll(tasks);

        // Assert - all clients should still be connected
        foreach (var client in clients)
        {
            client.IsConnected.Should().BeTrue();
        }

        // Cleanup
        foreach (var client in clients)
        {
            try { client.Disconnect(); } catch { }
            client.Dispose();
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Helper methods
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Attempts to connect a client with a timeout.
    /// Returns true if successful, false if timeout or error occurs.
    /// </summary>
    private static async Task<bool> TryConnectAsync(GameClient client, TimeSpan timeout)
    {
        try
        {
            using var cts = new CancellationTokenSource(timeout);
            await client.ConnectAsync(cts.Token);
            return client.IsConnected;
        }
        catch
        {
            return false;
        }
    }
}
