#nullable enable
using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Tests for GameClient timeout configurability and message framing.
/// </summary>
public class GameClientFramingTests
{
    [Fact]
    public void GameClientOptions_HasDefaultTimeouts()
    {
        // Arrange & Act
        var options = new GameClientOptions();

        // Assert
        options.ConnectTimeoutMs.Should().Be(5000);
        options.SendTimeoutMs.Should().Be(5000);
        options.ReadTimeoutMs.Should().Be(30000);
        options.MaxMessageSizeBytes.Should().Be(1_000_000);
        options.UseMessageFraming.Should().BeTrue();
    }

    [Fact]
    public void GameClientOptions_TimeoutValuesAreConfigurable()
    {
        // Arrange & Act
        var options = new GameClientOptions
        {
            ConnectTimeoutMs = 2000,
            SendTimeoutMs = 3000,
            ReadTimeoutMs = 10000,
            MaxMessageSizeBytes = 500_000,
            UseMessageFraming = false
        };

        // Assert
        options.ConnectTimeoutMs.Should().Be(2000);
        options.SendTimeoutMs.Should().Be(3000);
        options.ReadTimeoutMs.Should().Be(10000);
        options.MaxMessageSizeBytes.Should().Be(500_000);
        options.UseMessageFraming.Should().BeFalse();
    }

    [Fact]
    public async Task ConnectAsync_WithoutTimeout_UsesOptionsDefault()
    {
        // Arrange — unique pipe name so we never connect to a live dinoforge-game-bridge
        // (handshake + bounded retries can hang 20+ min if the default pipe exists).
        var options = new GameClientOptions
        {
            PipeName = "dinoforge-framing-timeout-" + Guid.NewGuid().ToString("N"),
            ConnectTimeoutMs = 3000,
            PerformConnectHandshake = false,
            RetryCount = 0,
        };
        var client = new GameClient(options);

        // Act & Assert — pipe doesn't exist; options ConnectTimeoutMs (3s) must drive the failure.
        GameClientException ex = await Assert.ThrowsAsync<GameClientException>(
            async () => await client.ConnectAsync(CancellationToken.None).ConfigureAwait(true));

        ex.InnerException.Should().BeAssignableTo<TimeoutException>();
        ex.InnerException!.Message.Should().Contain("3");
        client.Dispose();
    }

    [Fact]
    public async Task ConnectAsync_WithCustomTimeout_UsesProvidedValue()
    {
        // Arrange — unique pipe name; custom connectTimeout must win over options default.
        var options = new GameClientOptions
        {
            PipeName = "dinoforge-framing-custom-timeout-" + Guid.NewGuid().ToString("N"),
            ConnectTimeoutMs = 10000,
            PerformConnectHandshake = false,
            RetryCount = 0,
        };
        var client = new GameClient(options);
        var customTimeout = TimeSpan.FromMilliseconds(1000);

        // Act & Assert — custom connectTimeout (1s) must win over options default (10s).
        GameClientException ex = await Assert.ThrowsAsync<GameClientException>(
            async () => await client.ConnectAsync(customTimeout, CancellationToken.None).ConfigureAwait(true));

        ex.InnerException.Should().BeAssignableTo<TimeoutException>();
        ex.InnerException!.Message.Should().Contain("1");
        client.Dispose();
    }

    [Fact]
    public void GameClient_IsDisposable()
    {
        // Arrange
        var client = new GameClient();

        // Act
        var disposable = (IDisposable)client;

        // Assert
        disposable.Should().NotBeNull();
        client.Dispose(); // Should not throw
    }

    [Fact]
    public void ProtocolException_CanBeCreatedWithMessage()
    {
        // Arrange
        var message = "Frame size violates protocol";

        // Act
        var ex = new ProtocolException(message);

        // Assert
        ex.Message.Should().Be(message);
        ex.InnerException.Should().BeNull();
    }

    [Fact]
    public void ProtocolException_CanBeCreatedWithMessageAndInnerException()
    {
        // Arrange
        var message = "Frame parsing failed";
        var innerEx = new IOException("Connection closed");

        // Act
        var ex = new ProtocolException(message, innerEx);

        // Assert
        ex.Message.Should().Be(message);
        ex.InnerException.Should().Be(innerEx);
    }

    [Fact]
    public void ProtocolException_IsInvalidOperationException()
    {
        // Arrange
        var ex = new ProtocolException("Test");

        // Assert
        ex.Should().BeAssignableTo<InvalidOperationException>();
    }
}
