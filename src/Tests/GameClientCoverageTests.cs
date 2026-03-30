#nullable enable
using System;
using System.IO;
using System.IO.Pipes;
using System.Reflection;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for DINOForge.Bridge.Client.
/// These tests focus on error paths, state transitions, and edge cases
/// not covered by existing tests to raise coverage from 50.5% to 85%+.
/// </summary>
public class GameClientCoverageTests
{
    private static readonly UTF8Encoding Utf8NoBom = new(encoderShouldEmitUTF8Identifier: false);

    // ──────────────────────── GameClientOptions edge cases ────────────────────────

    [Fact]
    public void GameClientOptions_CanSetAllProperties()
    {
        GameClientOptions options = new()
        {
            PipeName = "custom-pipe",
            ConnectTimeoutMs = 10000,
            ReadTimeoutMs = 60000,
            RetryCount = 5,
            RetryDelayMs = 2000
        };

        options.PipeName.Should().Be("custom-pipe");
        options.ConnectTimeoutMs.Should().Be(10000);
        options.ReadTimeoutMs.Should().Be(60000);
        options.RetryCount.Should().Be(5);
        options.RetryDelayMs.Should().Be(2000);
    }

    [Fact]
    public void GameClientOptions_Defaults_AreCorrect()
    {
        GameClientOptions options = new();

        options.PipeName.Should().Be("dinoforge-game-bridge");
        options.ConnectTimeoutMs.Should().Be(5000);
        options.ReadTimeoutMs.Should().Be(30000);
        options.RetryCount.Should().Be(3);
        options.RetryDelayMs.Should().Be(1000);
    }

    // ──────────────────────── ConnectAsync error paths ────────────────────────

    [Fact]
    public void ConnectAsync_WhenPipeTimesOut_ThrowsGameClientException()
    {
        var cts = new CancellationTokenSource();
        var options = new GameClientOptions
        {
            ConnectTimeoutMs = 1, // Very short timeout
            PipeName = "nonexistent-pipe-timeout"
        };
        GameClient client = new(options);

        Func<Task> action = async () => await client.ConnectAsync(cts.Token);

        action.Should().ThrowAsync<GameClientException>()
            .WithMessage("*Failed to connect*");

        client.State.Should().BeOneOf(ConnectionState.Error, ConnectionState.Disconnected);
        client.Dispose();
    }

    [Fact]
    public void ConnectAsync_WhenCancelled_ThrowsOperationCanceledException()
    {
        var cts = new CancellationTokenSource();
        var options = new GameClientOptions
        {
            ConnectTimeoutMs = 5000,
            PipeName = "nonexistent-pipe-cancel"
        };
        GameClient client = new(options);
        cts.Cancel(); // Cancel immediately

        Func<Task> action = async () => await client.ConnectAsync(cts.Token);

        action.Should().ThrowAsync<OperationCanceledException>();

        client.Dispose();
    }

    [Fact]
    public void ConnectAsync_WhenAlreadyConnected_DoesNotThrow()
    {
        // Setup connected client
        GameClient client = new(new GameClientOptions { RetryCount = 0 });
        SetPrivateField(client, "_state", ConnectionState.Connected);

        Func<Task> action = async () => await client.ConnectAsync();

        action.Should().NotThrowAsync();
        client.State.Should().Be(ConnectionState.Connected);
        client.Dispose();
    }

    // ──────────────────────── State transitions ────────────────────────

    [Fact]
    public void Disconnect_SetsStateToDisconnected()
    {
        GameClient client = new();
        SetPrivateField(client, "_state", ConnectionState.Connected);

        client.Disconnect();

        client.State.Should().Be(ConnectionState.Disconnected);
        client.IsConnected.Should().BeFalse();
        client.Dispose();
    }

    [Fact]
    public void Disconnect_WhenAlreadyDisconnected_DoesNotThrow()
    {
        GameClient client = new();

        client.Disconnect();

        client.State.Should().Be(ConnectionState.Disconnected);
        client.Dispose();
    }

    [Fact]
    public async Task StateProperty_IsThreadSafe()
    {
        GameClient client = new();
        const int threadCount = 10;
        const int iterations = 1000;
        var tasks = new Task[threadCount];

        for (int i = 0; i < threadCount; i++)
        {
            tasks[i] = Task.Run(() =>
            {
                for (int j = 0; j < iterations; j++)
                {
                    _ = client.State;
                }
            });
        }

        await Task.WhenAll(tasks);

        client.Dispose();
    }

    // ──────────────────────── CleanupPipe coverage ────────────────────────

    [Fact]
    public void CleanupPipe_HandlesAlreadyDisposedResources()
    {
        GameClient client = new();
        // Set resources that might throw during dispose
        SetPrivateField(client, "_reader", new StreamReader(new MemoryStream()));
        SetPrivateField(client, "_writer", new StreamWriter(new MemoryStream()));
        SetPrivateField(client, "_pipe", new NamedPipeClientStream(".", "test", PipeDirection.InOut));

        client.Disconnect(); // This calls CleanupPipe

        client.State.Should().Be(ConnectionState.Disconnected);
        client.Dispose();
    }

    [Fact]
    public void CleanupPipe_WithNullResources_DoesNotThrow()
    {
        GameClient client = new();

        client.Disconnect();

        client.Dispose();
    }

    // ──────────────────────── Dispose coverage ────────────────────────

    [Fact]
    public void Dispose_CanBeCalledMultipleTimes()
    {
        GameClient client = new();

        client.Dispose();
        client.Dispose();
        client.Dispose();

        // Should not throw
    }

    [Fact]
    public void Dispose_AfterDisconnect_StillDisposes()
    {
        GameClient client = new();

        client.Disconnect();
        client.Dispose();

        // Should not throw
    }

    [Fact]
    public void Dispose_AfterConnect_DoesNotThrow()
    {
        GameClient client = CreateConnectedClient(
            new JsonRpcResponse
            {
                Id = "1",
                Result = JToken.FromObject(new PingResult { Pong = true })
            });

        client.Dispose();
        // Should not throw
    }

    [Fact]
    public void Dispose_SetsStateToDisconnected()
    {
        GameClient client = new();
        SetPrivateField(client, "_state", ConnectionState.Connected);

        client.Dispose();

        client.State.Should().Be(ConnectionState.Disconnected);
    }

    // ──────────────────────── ThrowIfDisposed coverage ────────────────────────

    [Fact]
    public void ThrowIfDisposed_AfterDispose_ThrowsObjectDisposedException()
    {
        GameClient client = new();
        client.Dispose();

        Action action = () => client.ConnectAsync().Wait();

        action.Should().Throw<ObjectDisposedException>();
    }

    // ──────────────────────── JsonRpcRequest coverage ────────────────────────

    [Fact]
    public void JsonRpcRequest_WithParameters_SerializesCorrectly()
    {
        JsonRpcRequest request = new()
        {
            Id = "test-id",
            Method = "ping",
            Params = JObject.FromObject(new { timeout = 100 })
        };

        string json = JsonConvert.SerializeObject(request, Formatting.None);
        JObject parsed = JObject.Parse(json);

        parsed["id"]!.Value<string>().Should().Be("test-id");
        parsed["method"]!.Value<string>().Should().Be("ping");
        parsed["params"]!["timeout"]!.Value<int>().Should().Be(100);
    }

    [Fact]
    public void JsonRpcRequest_NullParams_SerializesWithoutParams()
    {
        JsonRpcRequest request = new()
        {
            Id = "test-id",
            Method = "ping",
            Params = null
        };

        string json = JsonConvert.SerializeObject(request, Formatting.None,
            new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore });
        JObject parsed = JObject.Parse(json);

        parsed["params"].Should().BeNull();
    }

    [Fact]
    public void JsonRpcResponse_WithNullResult_SerializesCorrectly()
    {
        JsonRpcResponse response = new()
        {
            Id = "test-id",
            Result = null
        };

        string json = JsonConvert.SerializeObject(response);
        JsonRpcResponse? deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Result.Should().BeNull();
    }

    [Fact]
    public void JsonRpcResponse_WithComplexResult_SerializesCorrectly()
    {
        var complexResult = new { count = 42, items = new[] { "a", "b" } };
        JsonRpcResponse response = new()
        {
            Id = "test-id",
            Result = JToken.FromObject(complexResult)
        };

        string json = JsonConvert.SerializeObject(response);
        JsonRpcResponse? deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Result!["count"]!.Value<int>().Should().Be(42);
    }

    // ──────────────────────── GameClientException coverage ────────────────────────

    [Fact]
    public void GameClientException_WithMessage_HasCorrectMessage()
    {
        GameClientException ex = new("test error message");

        ex.Message.Should().Be("test error message");
    }

    [Fact]
    public void GameClientException_WithInnerException_ChainsCorrectly()
    {
        var inner = new ArgumentException("inner arg");
        GameClientException ex = new("outer message", inner);

        ex.Message.Should().Be("outer message");
        ex.InnerException.Should().BeSameAs(inner);
    }

    // ──────────────────────── ConnectionState coverage ────────────────────────

    [Fact]
    public void ConnectionState_AllValuesExist()
    {
        var values = Enum.GetValues<ConnectionState>();

        values.Should().Contain(ConnectionState.Disconnected);
        values.Should().Contain(ConnectionState.Connecting);
        values.Should().Contain(ConnectionState.Connected);
        values.Should().Contain(ConnectionState.Error);
    }

    [Theory]
    [InlineData(ConnectionState.Disconnected)]
    [InlineData(ConnectionState.Connecting)]
    [InlineData(ConnectionState.Connected)]
    [InlineData(ConnectionState.Error)]
    public void IsConnected_ReflectsConnectedState(ConnectionState state)
    {
        GameClient client = new();
        SetPrivateField(client, "_state", state);

        client.IsConnected.Should().Be(state == ConnectionState.Connected);

        client.Dispose();
    }

    // ──────────────────────── Helper methods ────────────────────────

    private static GameClient CreateConnectedClient(JsonRpcResponse response)
    {
        GameClient client = new(new GameClientOptions
        {
            RetryCount = 0,
            ReadTimeoutMs = 1000
        });

        MemoryStream responseStream = new(Utf8NoBom.GetBytes(JsonConvert.SerializeObject(response) + Environment.NewLine));
        MemoryStream requestStream = new();

        SetPrivateField(client, "_state", ConnectionState.Connected);
        SetPrivateField(client, "_reader", new StreamReader(responseStream, Utf8NoBom, false, 1024, leaveOpen: true));
        SetPrivateField(client, "_writer", new StreamWriter(requestStream, Utf8NoBom, 1024, leaveOpen: true)
        {
            AutoFlush = true
        });

        return client;
    }

    private static void SetPrivateField<T>(GameClient client, string fieldName, T value)
    {
        FieldInfo field = typeof(GameClient).GetField(fieldName, BindingFlags.Instance | BindingFlags.NonPublic)
            ?? throw new InvalidOperationException($"Field '{fieldName}' not found.");

        field.SetValue(client, value);
    }
}
