#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using DINOForge.SDK.Assets;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// PERF-004: Asset swap registry operations under latency targets.
/// Tests asset registration and lookup latencies without requiring the game process.
/// Uses a simple in-memory dictionary to verify that asset lookups meet performance targets.
/// </summary>
[Trait("Category", "Performance")]
public class AssetSwapLatencyTests
{
    [Fact]
    public void AssetSwapRegistry_DictionaryLookup_Under1Ms()
    {
        // Arrange — use a simple dictionary as a mock asset registry
        Dictionary<string, AssetSwapRequest> assetMap = new(StringComparer.OrdinalIgnoreCase);

        // Pre-populate with 100 assets
        for (int i = 0; i < 100; i++)
        {
            string bundleId = $"asset-{i:D3}";
            assetMap[bundleId] = new AssetSwapRequest(
                $"addr/unit-{i}.prefab",
                $"/path/mod-{i}.bundle",
                $"unit-{i}");
        }

        List<long> latencies = new();
        const int iterations = 100;

        // Act — measure lookup latencies
        for (int i = 0; i < iterations; i++)
        {
            string bundleId = $"asset-{i % 100:D3}";
            Stopwatch sw = Stopwatch.StartNew();
            bool found = assetMap.TryGetValue(bundleId, out var request);
            sw.Stop();

            found.Should().BeTrue("asset lookup should succeed for pre-registered asset");
            latencies.Add(sw.ElapsedMilliseconds);
        }

        // Assert — p99 latency must be under 1ms
        latencies.Sort();
        long p99 = latencies[(int)Math.Ceiling(latencies.Count * 0.99) - 1];

        p99.Should().BeLessThan(1,
            because: "asset registry dictionary lookup p99 latency must be under 1ms (actual: {0}ms)", p99);
    }

    [Fact]
    public void AssetSwapRegistry_BulkRegister_Under10Ms()
    {
        // Arrange
        Dictionary<string, AssetSwapRequest> assetMap = new(StringComparer.OrdinalIgnoreCase);
        const int assetCount = 500;

        // Act — measure bulk registration latency
        Stopwatch sw = Stopwatch.StartNew();
        for (int i = 0; i < assetCount; i++)
        {
            string bundleId = $"bulk-asset-{i:D4}";
            assetMap[bundleId] = new AssetSwapRequest(
                $"addr/bulk-unit-{i}.prefab",
                $"/path/bulk-mod-{i}.bundle",
                $"bulk-unit-{i}");
        }
        sw.Stop();

        // Assert — total bulk registration of 500 assets must complete in under 10ms
        sw.ElapsedMilliseconds.Should().BeLessThan(10,
            because: "bulk registration of 500 assets must complete under 10ms (actual: {0}ms)",
            sw.ElapsedMilliseconds);

        // Verify all assets were registered
        assetMap.Keys.Count.Should().Be(assetCount,
            because: "all bulk-registered assets should be present in the dictionary");
    }
}
