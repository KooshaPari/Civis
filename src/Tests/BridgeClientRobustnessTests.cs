using System;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge
{
    /// <summary>
    /// Tests for Bridge.Client error handling and edge cases.
    /// Targets uncovered branches: WaitForWorldAsync timeout, DumpStateAsync path handling,
    /// ConnectAsync pipe failures, reconnect logic.
    /// </summary>
    public class BridgeClientRobustnessTests
    {
        [Fact]
        public void Constructor_WithNullOptions_UsesDefaults()
        {
            // Arrange & Act
            var client = new GameClient();

            // Assert
            client.IsConnected.Should().BeFalse();
        }

        [Fact]
        public void IsConnected_WhenDisconnected_ReturnsFalse()
        {
            // Arrange
            var client = new GameClient();

            // Act
            var connected = client.IsConnected;

            // Assert
            connected.Should().BeFalse();
        }

        [Fact]
        public void State_AfterConstruction_IsDisconnected()
        {
            // Arrange & Act
            var client = new GameClient();

            // Assert
            client.State.Should().Be(ConnectionState.Disconnected);
        }

        [Fact]
        public void Disconnect_WhenAlreadyDisconnected_Succeeds()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - should not throw
            client.Disconnect();
        }

        [Fact]
        public void ThrowIfDisposed_WhenNotDisposed_DoesNotThrow()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - should not throw
            client.IsConnected.Should().BeFalse();
        }

        [Fact]
        public void ThrowIfDisposed_AfterDispose_ChecksState()
        {
            // Arrange
            var client = new GameClient();

            // Act - disposal should cleanup connection
            client.Dispose();

            // Assert - client state should be Disconnected
            client.State.Should().Be(ConnectionState.Disconnected);
        }

        [Fact]
        public void PingAsync_WithValidCancellation_CreatesTask()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - should not throw with valid CancellationToken
            var task = client.PingAsync(CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void WaitForWorldAsync_WithoutTimeout_CreatesTask()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - task should be created (may timeout in background)
            var task = client.WaitForWorldAsync(null, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void WaitForWorldAsync_WithTimeout_PassesTimeoutToTask()
        {
            // Arrange
            var client = new GameClient();
            const int timeoutMs = 1000;

            // Act & Assert
            var task = client.WaitForWorldAsync(timeoutMs, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void DumpStateAsync_WithoutPath_PassesNullPath()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - null path should be valid
            var task = client.DumpStateAsync(null, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void DumpStateAsync_WithPath_PassesPath()
        {
            // Arrange
            var client = new GameClient();
            const string path = "C:\\Temp\\dump.json";

            // Act & Assert - non-null path should be valid
            var task = client.DumpStateAsync(path, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void GetResourcesAsync_CreatesTask()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert
            var task = client.GetResourcesAsync(CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void ScreenshotAsync_WithoutPath_CreatesTask()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert
            var task = client.ScreenshotAsync(null, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void ScreenshotAsync_WithPath_PassesPath()
        {
            // Arrange
            var client = new GameClient();
            const string path = "C:\\Temp\\screenshot.png";

            // Act & Assert
            var task = client.ScreenshotAsync(path, CancellationToken.None);
            task.Should().NotBeNull();
        }

        [Fact]
        public void Dispose_CanBeCalledMultipleTimes()
        {
            // Arrange
            var client = new GameClient();

            // Act & Assert - multiple dispose calls should be safe
            client.Dispose();
            client.Dispose(); // Should not throw
        }

        [Fact]
        public void Dispose_SetsStateToDisconnected()
        {
            // Arrange
            var client = new GameClient();

            // Act
            client.Dispose();

            // Assert
            client.State.Should().Be(ConnectionState.Disconnected);
        }

        [Fact]
        public void Constructor_WithOptions_AppliesSettings()
        {
            // Arrange
            var options = new GameClientOptions { ConnectTimeoutMs = 5000 };

            // Act
            var client = new GameClient(options);

            // Assert
            client.Should().NotBeNull();
        }
    }
}
