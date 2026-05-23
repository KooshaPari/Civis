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

    [Fact(Skip = "#397 — closure-gate hang (iter-92/93), skip until testhost stabilizes")]
    public async Task ConnectAsync_WithoutTimeout_UsesOptionsDefault()
    {
        // Arrange
        var options = new GameClientOptions { ConnectTimeoutMs = 3000 };
        var client = new GameClient(options);
        var sw = System.Diagnostics.Stopwatch.StartNew();

        // Act & Assert
        // Pipe doesn't exist, will fail, but should respect default timeout
        await Assert.ThrowsAsync<GameClientException>(
            async () => await client.ConnectAsync(CancellationToken.None).ConfigureAwait(true));

        sw.Stop();
        // Should fail within 4 seconds (3s timeout + overhead)
        sw.ElapsedMilliseconds.Should().BeLessThan(4500);
        client.Dispose();
    }

    [Fact(Skip = "#543 — flaky with wave-2 bounded-retry timeouts (iter-143), can take 20+ min when pipe connect succeeds then handshake retries; skip until pipe-name isolation lands")]
    public async Task ConnectAsync_WithCustomTimeout_UsesProvidedValue()
    {
        // Arrange
        var options = new GameClientOptions { ConnectTimeoutMs = 10000 };
        var client = new GameClient(options);
        var customTimeout = TimeSpan.FromMilliseconds(1000);
        var sw = System.Diagnostics.Stopwatch.StartNew();

        // Act & Assert
        // Pipe doesn't exist, will fail, but should respect custom timeout
        await Assert.ThrowsAsync<GameClientException>(
            async () => await client.ConnectAsync(customTimeout, CancellationToken.None).ConfigureAwait(true));

        sw.Stop();
        // Should fail quickly (within 2 seconds), proving custom timeout was used
        sw.ElapsedMilliseconds.Should().BeLessThan(2500);
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
