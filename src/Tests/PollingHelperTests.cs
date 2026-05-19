#nullable enable
using System;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.SDK.Concurrency;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Tests for PollingHelper retry-with-backoff utility.
/// Pattern #113 D4 (polling primitives) — covers exponential backoff, timeout, and cancellation.
/// </summary>
public class PollingHelperTests
{
    [Fact]
    public async Task RetryUntilAsync_SucceedsOnFirstProbe()
    {
        // Arrange
        var expected = new object();
        var callCount = 0;

        object? Probe()
        {
            callCount++;
            return expected;
        }

        // Act
        var result = await PollingHelper.RetryUntilAsync(Probe, timeout: TimeSpan.FromSeconds(5));

        // Assert
        result.Should().BeSameAs(expected);
        callCount.Should().Be(1);
    }

    [Fact]
    public async Task RetryUntilAsync_SucceedsOnNthProbe()
    {
        // Arrange
        var expected = new object();
        var callCount = 0;

        object? Probe()
        {
            callCount++;
            return callCount >= 3 ? expected : null;
        }

        // Act
        var result = await PollingHelper.RetryUntilAsync(
            Probe,
            timeout: TimeSpan.FromSeconds(5),
            initialDelay: TimeSpan.FromMilliseconds(10),
            maxDelay: TimeSpan.FromMilliseconds(50));

        // Assert
        result.Should().BeSameAs(expected);
        callCount.Should().Be(3);
    }

    [Fact]
    public async Task RetryUntilAsync_ReturnsNullOnTimeout()
    {
        // Arrange
        var callCount = 0;

        object? Probe()
        {
            callCount++;
            return null; // Never succeeds
        }

        // Act
        var result = await PollingHelper.RetryUntilAsync(
            Probe,
            timeout: TimeSpan.FromMilliseconds(100),
            initialDelay: TimeSpan.FromMilliseconds(30),
            maxDelay: TimeSpan.FromMilliseconds(30));

        // Assert
        result.Should().BeNull();
        callCount.Should().BeGreaterThanOrEqualTo(2); // Multiple attempts before timeout
    }

    [Fact]
    public async Task RetryUntilAsync_ThrowsOnCancellation()
    {
        // Arrange
        using var cts = new CancellationTokenSource();
        var callCount = 0;

        object? Probe()
        {
            callCount++;
            return null;
        }

        // Act
        cts.CancelAfter(TimeSpan.FromMilliseconds(50));
        var act = () => PollingHelper.RetryUntilAsync(
            Probe,
            timeout: TimeSpan.FromSeconds(10),
            initialDelay: TimeSpan.FromMilliseconds(20),
            ct: cts.Token);

        // Assert
        await act.Should().ThrowAsync<OperationCanceledException>();
        callCount.Should().BeGreaterThanOrEqualTo(1);
    }

    [Fact]
    public async Task RetryUntilTrueAsync_SucceedsOnFirstProbe()
    {
        // Arrange
        var callCount = 0;

        bool Probe()
        {
            callCount++;
            return true;
        }

        // Act
        var result = await PollingHelper.RetryUntilTrueAsync(Probe, timeout: TimeSpan.FromSeconds(5));

        // Assert
        result.Should().BeTrue();
        callCount.Should().Be(1);
    }

    [Fact]
    public async Task RetryUntilTrueAsync_SucceedsOnNthProbe()
    {
        // Arrange
        var callCount = 0;

        bool Probe()
        {
            callCount++;
            return callCount >= 3;
        }

        // Act
        var result = await PollingHelper.RetryUntilTrueAsync(
            Probe,
            timeout: TimeSpan.FromSeconds(5),
            initialDelay: TimeSpan.FromMilliseconds(10),
            maxDelay: TimeSpan.FromMilliseconds(50));

        // Assert
        result.Should().BeTrue();
        callCount.Should().Be(3);
    }

    [Fact]
    public async Task RetryUntilTrueAsync_ReturnsFalseOnTimeout()
    {
        // Arrange
        var callCount = 0;

        bool Probe()
        {
            callCount++;
            return false; // Never succeeds
        }

        // Act
        var result = await PollingHelper.RetryUntilTrueAsync(
            Probe,
            timeout: TimeSpan.FromMilliseconds(100),
            initialDelay: TimeSpan.FromMilliseconds(30),
            maxDelay: TimeSpan.FromMilliseconds(30));

        // Assert
        result.Should().BeFalse();
        callCount.Should().BeGreaterThanOrEqualTo(2);
    }

    [Fact]
    public async Task RetryUntilTrueAsync_ThrowsOnCancellation()
    {
        // Arrange
        using var cts = new CancellationTokenSource();
        var callCount = 0;

        bool Probe()
        {
            callCount++;
            return false;
        }

        // Act
        cts.CancelAfter(TimeSpan.FromMilliseconds(50));
        var act = () => PollingHelper.RetryUntilTrueAsync(
            Probe,
            timeout: TimeSpan.FromSeconds(10),
            initialDelay: TimeSpan.FromMilliseconds(20),
            ct: cts.Token);

        // Assert
        await act.Should().ThrowAsync<OperationCanceledException>();
        callCount.Should().BeGreaterThanOrEqualTo(1);
    }

    [Fact]
    public async Task RetryUntilAsync_ThrowsOnNullProbe()
    {
        // Act
        var act = () => PollingHelper.RetryUntilAsync<object>(
            null!,
            timeout: TimeSpan.FromSeconds(5));

        // Assert
        await act.Should().ThrowAsync<ArgumentNullException>();
    }

    [Fact]
    public async Task RetryUntilTrueAsync_ThrowsOnNullProbe()
    {
        // Act
        var act = () => PollingHelper.RetryUntilTrueAsync(
            null!,
            timeout: TimeSpan.FromSeconds(5));

        // Assert
        await act.Should().ThrowAsync<ArgumentNullException>();
    }

    [Fact]
    public async Task RetryUntilAsync_RespectsExponentialBackoff()
    {
        // Arrange
        var probeCount = 0;
        var sw = System.Diagnostics.Stopwatch.StartNew();

        object? Probe()
        {
            probeCount++;
            return probeCount >= 4 ? new object() : null;
        }

        // Act
        // initialDelay=20ms, backoffFactor=2.0
        // Attempts: 1 (now), delay 20ms, 2 (20ms), delay 40ms, 3 (60ms), delay 80ms, 4 (140ms+)
        var result = await PollingHelper.RetryUntilAsync(
            Probe,
            timeout: TimeSpan.FromSeconds(5),
            initialDelay: TimeSpan.FromMilliseconds(20),
            backoffFactor: 2.0,
            maxDelay: TimeSpan.FromMilliseconds(200));

        sw.Stop();

        // Assert
        result.Should().NotBeNull();
        probeCount.Should().Be(4);
        // Should take roughly 140ms minimum (20 + 40 + 80)
        sw.ElapsedMilliseconds.Should().BeGreaterThanOrEqualTo(140);
    }
}
