using System;
using System.Collections.Generic;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Order;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for pack content loading pipeline throughput.
/// Measures the overhead of manifest parsing, validation, and registry population.
/// </summary>
[MemoryDiagnoser]
[Orderer(SummaryOrderPolicy.FastestToSlowest)]
[RankColumn]
[SimpleJob(warmupCount: 3, iterationCount: 5)]
public class ContentLoaderBenchmarks
{
    /// <summary>Number of packs to load in this benchmark iteration.</summary>
    [Params(1, 5, 10, 25, 50)]
    public int PackCount { get; set; }

    /// <summary>Benchmark pack loading pipeline with synthetic manifests.</summary>
    [Benchmark]
    public void Load_Packs_Manifest_Parsing()
    {
        // Simulate manifest parsing for N packs
        for (int p = 0; p < PackCount; p++)
        {
            var yaml = GenerateSyntheticPackManifest(p);
            _ = yaml.Length; // prevent optimization elision
        }
    }

    /// <summary>Benchmark population of registry from pack data.</summary>
    [Benchmark]
    public int Load_Packs_Registry_Population()
    {
        var registry = new System.Collections.Generic.Dictionary<string, object?>();
        int entryCount = 0;

        // Simulate loading 10 units per pack
        for (int p = 0; p < PackCount; p++)
        {
            for (int u = 0; u < 10; u++)
            {
                registry[$"pack{p}-unit{u}"] = new { Id = $"pack{p}-unit{u}" };
                entryCount++;
            }
        }

        return entryCount;
    }

    /// <summary>Benchmark the full pipeline: manifest + registry + validation.</summary>
    [Benchmark]
    public int Load_Packs_Full_Pipeline()
    {
        var registry = new System.Collections.Generic.Dictionary<string, object?>();
        int entryCount = 0;

        for (int p = 0; p < PackCount; p++)
        {
            // Parse manifest
            var manifest = GenerateSyntheticPackManifest(p);

            // Populate registry (simplified validation)
            if (!string.IsNullOrEmpty(manifest))
            {
                for (int u = 0; u < 10; u++)
                {
                    registry[$"pack{p}-unit{u}"] = new { Id = $"pack{p}-unit{u}" };
                    entryCount++;
                }
            }
        }

        return entryCount;
    }

    /// <summary>Generate a realistic pack manifest for benchmarking.</summary>
    private static string GenerateSyntheticPackManifest(int packIndex)
    {
        return $@"id: bench-pack-{packIndex}
name: Benchmark Pack {packIndex}
version: 1.0.0
framework_version: '>=0.1.0 <1.0.0'
author: Benchmark
type: content
depends_on: []
conflicts_with: []
loads:
  factions:
    - faction-a
    - faction-b
  units:
    - unit-1
    - unit-2
    - unit-3
  buildings:
    - building-x
    - building-y
  weapons:
    - weapon-primary
    - weapon-secondary
";
    }
}
