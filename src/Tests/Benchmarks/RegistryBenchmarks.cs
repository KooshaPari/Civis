using System;
using System.Collections.Generic;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Order;
using DINOForge.SDK.Registry;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for Registry{T} read and write performance under realistic loads.
/// </summary>
[MemoryDiagnoser]
[Orderer(SummaryOrderPolicy.FastestToSlowest)]
[RankColumn]
[SimpleJob(warmupCount: 3, iterationCount: 5)]
public class RegistryBenchmarks
{
    private Registry<TestRegistryEntry>? _smallRegistry;
    private Registry<TestRegistryEntry>? _largeRegistry;

    [GlobalSetup]
    public void Setup()
    {
        // Small registry: typical gameplay with ~100 units
        _smallRegistry = new Registry<TestRegistryEntry>();
        for (int i = 0; i < 100; i++)
        {
            _smallRegistry.Register(
                $"unit-{i}",
                new TestRegistryEntry { Id = $"unit-{i}", Name = $"Unit {i}" },
                RegistrySource.Pack,
                "base-game",
                i
            );
        }

        // Large registry: modded with 10+ packs, ~1000 entries
        _largeRegistry = new Registry<TestRegistryEntry>();
        for (int pack = 0; pack < 10; pack++)
        {
            for (int i = 0; i < 100; i++)
            {
                _largeRegistry.Register(
                    $"unit-{i}",
                    new TestRegistryEntry { Id = $"unit-{i}", Name = $"Unit {i}", Source = $"pack-{pack}" },
                    RegistrySource.Pack,
                    $"pack-{pack}",
                    i
                );
            }
        }
    }

    [Benchmark(Baseline = true)]
    [BenchmarkCategory("Read")]
    public TestRegistryEntry? GetById_Small()
    {
        return _smallRegistry!.Get("unit-50");
    }

    [Benchmark]
    [BenchmarkCategory("Read")]
    public TestRegistryEntry? GetById_Large()
    {
        return _largeRegistry!.Get("unit-500");
    }

    [Benchmark]
    [BenchmarkCategory("Read")]
    public IReadOnlyDictionary<string, RegistryEntry<TestRegistryEntry>> GetAll_Small()
    {
        return _smallRegistry!.All;
    }

    [Benchmark]
    [BenchmarkCategory("Read")]
    public IReadOnlyDictionary<string, RegistryEntry<TestRegistryEntry>> GetAll_Large()
    {
        return _largeRegistry!.All;
    }

    [Benchmark]
    [BenchmarkCategory("Read")]
    public bool Contains_Small()
    {
        return _smallRegistry!.Contains("unit-50");
    }

    [Benchmark]
    [BenchmarkCategory("Read")]
    public bool Contains_Large()
    {
        return _largeRegistry!.Contains("unit-500");
    }

    [Benchmark]
    [BenchmarkCategory("Write")]
    public void Register_1000Entries()
    {
        var registry = new Registry<TestRegistryEntry>();
        for (int i = 0; i < 1000; i++)
        {
            registry.Register(
                $"entry-{i}",
                new TestRegistryEntry { Id = $"entry-{i}", Name = $"Entry {i}" },
                RegistrySource.Pack,
                "bench-pack",
                i
            );
        }
    }

    [Benchmark]
    [BenchmarkCategory("Write")]
    public void Override_100Times()
    {
        var registry = new Registry<TestRegistryEntry>();

        // Pre-populate
        for (int i = 0; i < 100; i++)
        {
            registry.Register(
                $"entry-{i}",
                new TestRegistryEntry { Id = $"entry-{i}", Name = $"Entry {i}" },
                RegistrySource.Pack,
                "original-pack",
                100
            );
        }

        // Override entries with higher priority
        for (int i = 0; i < 100; i++)
        {
            registry.Override(
                $"entry-{i}",
                new TestRegistryEntry { Id = $"entry-{i}", Name = $"Override {i}" },
                RegistrySource.Pack,
                "override-pack",
                110
            );
        }
    }

    [Benchmark]
    [BenchmarkCategory("ConflictDetection")]
    public IReadOnlyList<RegistryConflict> DetectConflicts_100Entries_SamePriority()
    {
        var registry = new Registry<TestRegistryEntry>();

        // Create 100 entries, each with 3 registrations at equal priority (conflict)
        for (int i = 0; i < 100; i++)
        {
            registry.Register(
                $"conflicted-{i}",
                new TestRegistryEntry { Id = $"conflicted-{i}", Name = $"A" },
                RegistrySource.Pack,
                "pack-a",
                100
            );
            registry.Register(
                $"conflicted-{i}",
                new TestRegistryEntry { Id = $"conflicted-{i}", Name = $"B" },
                RegistrySource.Pack,
                "pack-b",
                100
            );
            registry.Register(
                $"conflicted-{i}",
                new TestRegistryEntry { Id = $"conflicted-{i}", Name = $"C" },
                RegistrySource.Pack,
                "pack-c",
                100
            );
        }

        return registry.DetectConflicts();
    }
}

/// <summary>Test entry type for registry benchmarks.</summary>
public sealed class TestRegistryEntry
{
    public required string Id { get; init; }
    public required string Name { get; init; }
    public string? Source { get; init; }
    public string? Description { get; init; }
}
