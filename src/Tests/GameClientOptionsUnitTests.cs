#nullable enable
using System;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for <see cref="GameClientOptions"/> configuration.
/// Validates defaults, property mutations, and semantic invariants.
/// </summary>
public sealed class GameClientOptionsUnitTests
{
    [Fact]
    public void Constructor_ProducedExpectedDefaults()
    {
        // Arrange & Act
        var options = new GameClientOptions();

        // Assert
        options.PipeName.Should().Be("dinoforge-game-bridge");
        options.ConnectTimeoutMs.Should().Be(5000);
        options.SendTimeoutMs.Should().Be(5000);
        options.ReadTimeoutMs.Should().Be(30000);
        options.MaxMessageSizeBytes.Should().Be(1_000_000);
        options.RetryCount.Should().Be(3);
        options.RetryDelayMs.Should().Be(1000);
        options.UseMessageFraming.Should().BeTrue();
        options.PerformConnectHandshake.Should().BeTrue();
    }

    [Fact]
    public void PipeName_CanBeSetAndRetrieved()
    {
        // Arrange
        var options = new GameClientOptions();
        var pipeName = $"dinoforge-test-{Guid.NewGuid():N}";

        // Act
        options.PipeName = pipeName;

        // Assert
        options.PipeName.Should().Be(pipeName);
    }

    [Fact]
    public void ConnectTimeoutMs_CanBeSetAndRetrieved()
    {
        // Arrange
        var options = new GameClientOptions();

        // Act
        options.ConnectTimeoutMs = 10000;

        // Assert
        options.ConnectTimeoutMs.Should().Be(10000);
    }

    [Fact]
    public void RetryCountAndDelay_CanBeSetAndRetrieved()
    {
        // Arrange
        var options = new GameClientOptions();

        // Act
        options.RetryCount = 5;
        options.RetryDelayMs = 2000;

        // Assert
        options.RetryCount.Should().Be(5);
        options.RetryDelayMs.Should().Be(2000);
    }

    [Fact]
    public void UseMessageFraming_DefaultTrueAndCanToggle()
    {
        // Arrange
        var options = new GameClientOptions();

        // Act
        options.UseMessageFraming.Should().BeTrue();
        options.UseMessageFraming = false;

        // Assert
        options.UseMessageFraming.Should().BeFalse();
    }

    [Fact]
    public void PerformConnectHandshake_DefaultTruePerPhase4c()
    {
        // Arrange & Act
        var options = new GameClientOptions();

        // Assert — Per iter-94 #311: default is true (Phase 4c completion)
        options.PerformConnectHandshake.Should().BeTrue();
    }

    [Fact]
    public void PerformConnectHandshake_CanBeDisabledForLegacyServers()
    {
        // Arrange
        var options = new GameClientOptions();

        // Act
        options.PerformConnectHandshake = false;

        // Assert
        options.PerformConnectHandshake.Should().BeFalse();
    }

    [Fact]
    public void MaxMessageSize_CanBeSetToCustomValue()
    {
        // Arrange
        var options = new GameClientOptions();

        // Act
        options.MaxMessageSizeBytes = 5_000_000;

        // Assert
        options.MaxMessageSizeBytes.Should().Be(5_000_000);
    }
}
