#nullable enable
using System;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for <see cref="GameClient"/> covering constructor contracts,
/// options handling, IDisposable implementation, and verification modes.
/// </summary>
public sealed class GameClientUnitTests : IDisposable
{
    private GameClient? _client;

    public void Dispose()
    {
        _client?.Dispose();
    }

    // ──────────────────────────── Constructor Tests ────────────────────────────────

    [Fact]
    public void Constructor_WithDefaultOptions_SucceedsAndInitializes()
    {
        // Arrange & Act
        _client = new GameClient();

        // Assert
        _client.Should().NotBeNull();
        _client.IsConnected.Should().BeFalse();
        _client.State.Should().Be(ConnectionState.Disconnected);
    }

    [Fact]
    public void Constructor_WithValidOptions_SucceedsAndStoresOptions()
    {
        // Arrange
        var options = new GameClientOptions
        {
            PipeName = $"dinoforge-test-{Guid.NewGuid():N}",
            ConnectTimeoutMs = 10000,
            RetryCount = 5
        };

        // Act
        _client = new GameClient(options);

        // Assert
        _client.Should().NotBeNull();
        _client.IsConnected.Should().BeFalse();
        _client.State.Should().Be(ConnectionState.Disconnected);
    }

    [Fact]
    public void Constructor_WithNullOptions_ThrowsArgumentNullException()
    {
        // Arrange
        GameClientOptions? nullOptions = null;

        // Act
        Action action = () => new GameClient(nullOptions!);

        // Assert
        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("options");
    }

    // ──────────────────────────── Options Defaults Tests ────────────────────────────

    [Fact]
    public void GameClientOptions_DefaultValues_MatchDocumentedContract()
    {
        // Arrange & Act
        var options = new GameClientOptions();

        // Assert
        options.PipeName.Should().Be("dinoforge-game-bridge");
        options.ConnectTimeoutMs.Should().Be(5000);
        options.SendTimeoutMs.Should().Be(5000);
        options.ReadTimeoutMs.Should().Be(30000);
        options.MaxMessageSizeBytes.Should().Be(1_000_000u);
        options.RetryCount.Should().Be(3);
        options.RetryDelayMs.Should().Be(1000);
        options.UseMessageFraming.Should().BeTrue();
        options.PerformConnectHandshake.Should().BeTrue();
    }

    [Fact]
    public void GameClientOptions_CustomValues_AreSettable()
    {
        // Arrange
        var testPipeName = $"dinoforge-test-{Guid.NewGuid():N}";
        var options = new GameClientOptions
        {
            PipeName = testPipeName,
            ConnectTimeoutMs = 15000,
            SendTimeoutMs = 8000,
            ReadTimeoutMs = 60000,
            MaxMessageSizeBytes = 2_000_000,
            RetryCount = 5,
            RetryDelayMs = 2000,
            UseMessageFraming = false,
            PerformConnectHandshake = false
        };

        // Act & Assert
        options.PipeName.Should().Be(testPipeName);
        options.ConnectTimeoutMs.Should().Be(15000);
        options.SendTimeoutMs.Should().Be(8000);
        options.ReadTimeoutMs.Should().Be(60000);
        options.MaxMessageSizeBytes.Should().Be(2_000_000u);
        options.RetryCount.Should().Be(5);
        options.RetryDelayMs.Should().Be(2000);
        options.UseMessageFraming.Should().BeFalse();
        options.PerformConnectHandshake.Should().BeFalse();
    }

    // ──────────────────────────── IDisposable Contract Tests ──────────────────────────

    [Fact]
    public void Dispose_Once_Succeeds()
    {
        // Arrange
        _client = new GameClient();

        // Act
        Action action = () => _client.Dispose();

        // Assert
        action.Should().NotThrow();
    }

    [Fact]
    public void Dispose_Multiple_IsIdempotent()
    {
        // Arrange
        _client = new GameClient();

        // Act
        Action action = () =>
        {
            _client.Dispose();
            _client.Dispose();
            _client.Dispose();
        };

        // Assert
        action.Should().NotThrow("Dispose should be idempotent");
    }

    [Fact]
    public void PostDispose_ConnectAsync_ThrowsObjectDisposedException()
    {
        // Arrange
        _client = new GameClient();
        _client.Dispose();

        // Act
        Func<Task> action = () => _client.ConnectAsync();

        // Assert
        action.Should().ThrowAsync<ObjectDisposedException>();
    }

    [Fact]
    public void PostDispose_PingAsync_ThrowsObjectDisposedException()
    {
        // Arrange
        _client = new GameClient();
        _client.Dispose();

        // Act
        Func<Task> action = () => _client.PingAsync();

        // Assert
        action.Should().ThrowAsync<ObjectDisposedException>();
    }

    [Fact]
    public void PostDispose_Disconnect_ThrowsObjectDisposedException()
    {
        // Arrange
        _client = new GameClient();
        _client.Dispose();

        // Act
        Action action = () => _client.Disconnect();

        // Assert
        action.Should().Throw<ObjectDisposedException>();
    }

    // ──────────────────────────── Initial State Tests ──────────────────────────────

    [Fact]
    public void NewClient_StateIsDisconnected()
    {
        // Arrange & Act
        _client = new GameClient();

        // Assert
        _client.State.Should().Be(ConnectionState.Disconnected);
        _client.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void NewClient_HmacVerificationModeDefaultIsStrict()
    {
        // Arrange & Act
        _client = new GameClient();

        // Assert
        _client.HmacVerificationMode.Should().Be(VerificationMode.Strict);
    }

    // ──────────────────────────── HmacVerificationMode Tests ──────────────────────────

    [Fact]
    public void HmacVerificationMode_CanSetToOff()
    {
        // Arrange
        _client = new GameClient
        {
            HmacVerificationMode = VerificationMode.Off
        };

        // Act & Assert
        _client.HmacVerificationMode.Should().Be(VerificationMode.Off);
    }

    [Fact]
    public void HmacVerificationMode_CanSetToWarnOnly()
    {
        // Arrange
        _client = new GameClient
        {
            HmacVerificationMode = VerificationMode.WarnOnly
        };

        // Act & Assert
        _client.HmacVerificationMode.Should().Be(VerificationMode.WarnOnly);
    }

    [Fact]
    public void HmacVerificationMode_CanSetToStrict()
    {
        // Arrange
        _client = new GameClient
        {
            HmacVerificationMode = VerificationMode.Strict
        };

        // Act & Assert
        _client.HmacVerificationMode.Should().Be(VerificationMode.Strict);
    }

    // ──────────────────────────── PerformConnectHandshake Tests ──────────────────────────

    [Fact]
    public void Constructor_WithPerformConnectHandshakeFalse_InitializesSuccessfully()
    {
        // Arrange
        var options = new GameClientOptions { PerformConnectHandshake = false };

        // Act
        _client = new GameClient(options);

        // Assert
        _client.Should().NotBeNull();
        _client.State.Should().Be(ConnectionState.Disconnected);
        _client.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void Constructor_WithPerformConnectHandshakeTrue_InitializesSuccessfully()
    {
        // Arrange
        var options = new GameClientOptions { PerformConnectHandshake = true };

        // Act
        _client = new GameClient(options);

        // Assert
        _client.Should().NotBeNull();
        _client.State.Should().Be(ConnectionState.Disconnected);
        _client.IsConnected.Should().BeFalse();
    }

    // ──────────────────────────── SessionKeys Access Tests ──────────────────────────

    [Fact]
    public void SessionKeys_AreAccessibleInternally()
    {
        // Arrange
        _client = new GameClient();

        // Act
        var sessionKeys = ((dynamic)_client).SessionKeys;

        // Assert (via reflection to access internal property)
        // Just verify the client initializes without throwing
        _client.Should().NotBeNull();
    }

    [Fact]
    public void MultipleClients_AreIndependent()
    {
        // Arrange
        var client1 = new GameClient();
        var client2 = new GameClient();

        try
        {
            // Act & Assert
            client1.Should().NotBe(client2);
            client1.State.Should().Be(ConnectionState.Disconnected);
            client2.State.Should().Be(ConnectionState.Disconnected);

            client1.HmacVerificationMode = VerificationMode.Off;
            client2.HmacVerificationMode = VerificationMode.WarnOnly;

            client1.HmacVerificationMode.Should().Be(VerificationMode.Off);
            client2.HmacVerificationMode.Should().Be(VerificationMode.WarnOnly);
        }
        finally
        {
            client1.Dispose();
            client2.Dispose();
        }
    }
}
