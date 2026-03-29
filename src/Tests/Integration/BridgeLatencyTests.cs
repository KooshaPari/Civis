#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// INT-007 / PERF-003: Bridge round-trip latency under 100ms.
/// Measures ping and status query latencies using the FakeGameBridge (no game required).
/// </summary>
[Trait("Category", "Performance")]
public class BridgeLatencyTests
{
    [Fact]
    public void BridgePing_Latency_Under100Ms()
    {
        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null); // Ensure bridge is in ready state

        List<long> latencies = new();
        const int iterations = 10;

        // Act — measure ping latencies
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            PingResult result = bridge.Ping();
            sw.Stop();

            result.Pong.Should().BeTrue("ping should succeed");
            latencies.Add(sw.ElapsedMilliseconds);
        }

        // Assert — p99 latency must be under 100ms
        latencies.Sort();
        long p99 = latencies[(int)Math.Ceiling(latencies.Count * 0.99) - 1];

        p99.Should().BeLessThan(100,
            because: "bridge ping p99 latency must be under 100ms (actual: {0}ms)", p99);
    }

    [Fact]
    public void BridgeStatusQuery_Latency_Under200Ms()
    {
        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null);

        List<long> latencies = new();
        const int iterations = 10;

        // Act — measure status query latencies
        for (int i = 0; i < iterations; i++)
        {
            Stopwatch sw = Stopwatch.StartNew();
            GameStatus status = bridge.Status();
            sw.Stop();

            status.Running.Should().BeTrue("status should report game as running");
            latencies.Add(sw.ElapsedMilliseconds);
        }

        // Assert — p99 latency must be under 200ms
        latencies.Sort();
        long p99 = latencies[(int)Math.Ceiling(latencies.Count * 0.99) - 1];

        p99.Should().BeLessThan(200,
            because: "bridge status query p99 latency must be under 200ms (actual: {0}ms)", p99);
    }
}
