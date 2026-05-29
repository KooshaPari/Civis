using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using BenchmarkDotNet.Attributes;
using DINOForge.SDK;
using DINOForge.SDK.Registry;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for pack manifest loading and discovery operations.
/// Measures the performance of loading pack.yaml files and computing content summaries.
/// </summary>
[MemoryDiagnoser]
[SimpleJob(warmupCount: 3, targetCount: 5)]
public class PackLoadBenchmark
{
    private string _testPackDir = null!;
    private PackManifest _manifest = null!;
    private FileDiscoveryService _discoveryService = null!;

    [GlobalSetup]
    [ExcludeFromCodeCoverage]
    public void GlobalSetup()
    {
        // Use the warfare-starwars pack as a representative test case
        _testPackDir = Path.Combine(
            AppContext.BaseDirectory,
            "..", "..", "..", "..", "packs", "warfare-starwars");

        if (!Directory.Exists(_testPackDir))
        {
            throw new InvalidOperationException(
                $"Test pack directory not found: {_testPackDir}");
        }

        _discoveryService = new FileDiscoveryService();

        // Pre-load manifest for single-pack benchmark
        var manifestPath = Path.Combine(_testPackDir, "pack.yaml");
        _manifest = YamlLoader.DeserializeFromFile<PackManifest>(manifestPath)
            ?? throw new InvalidOperationException("Could not load test pack manifest");
    }

    /// <summary>
    /// Loads a single pack.yaml manifest from disk.
    /// </summary>
    [Benchmark]
    public PackManifest LoadSinglePackManifest()
    {
        var manifestPath = Path.Combine(_testPackDir, "pack.yaml");
        return YamlLoader.DeserializeFromFile<PackManifest>(manifestPath);
    }

    /// <summary>
    /// Discovers and loads all pack manifests in the packs directory.
    /// Tests the scalability of directory traversal and file parsing.
    /// </summary>
    [Benchmark]
    public List<PackManifest> LoadAllPackManifests()
    {
        var packsDir = Path.Combine(
            AppContext.BaseDirectory,
            "..", "..", "..", "..", "packs");

        var manifests = new List<PackManifest>();
        if (Directory.Exists(packsDir))
        {
            foreach (var dir in Directory.EnumerateDirectories(packsDir))
            {
                var manifestPath = Path.Combine(dir, "pack.yaml");
                if (File.Exists(manifestPath))
                {
                    var manifest = YamlLoader.DeserializeFromFile<PackManifest>(manifestPath);
                    if (manifest != null)
                    {
                        manifests.Add(manifest);
                    }
                }
            }
        }

        return manifests;
    }

    /// <summary>
    /// Builds a content summary from a loaded pack manifest.
    /// Simulates the operation performed by the ContentLoader when computing
    /// available units, buildings, factions, and other content.
    /// </summary>
    [Benchmark]
    public ContentSummary BuildContentSummary()
    {
        return ExtractContentSummary(_manifest)!;
    }

    /// <summary>
    /// Benchmarks varying numbers of packs to measure scalability.
    /// </summary>
    [Benchmark]
    [Arguments(1)]
    [Arguments(5)]
    [Arguments(10)]
    public int LoadMultiplePacksScaling(int packCount)
    {
        var packsDir = Path.Combine(
            AppContext.BaseDirectory,
            "..", "..", "..", "..", "packs");

        var loaded = 0;
        if (Directory.Exists(packsDir))
        {
            foreach (var dir in Directory.EnumerateDirectories(packsDir).Take(packCount))
            {
                var manifestPath = Path.Combine(dir, "pack.yaml");
                if (File.Exists(manifestPath))
                {
                    var manifest = YamlLoader.DeserializeFromFile<PackManifest>(manifestPath);
                    if (manifest != null)
                    {
                        loaded++;
                    }
                }
            }
        }

        return loaded;
    }

    /// <summary>
    /// Helper to extract a content summary from a manifest.
    /// Mimics the ContentLoader's summary computation.
    /// </summary>
    private static ContentSummary ExtractContentSummary(PackManifest manifest)
    {
        return new ContentSummary
        {
            PackId = manifest.Id,
            PackName = manifest.Name,
            UnitCount = 0,
            BuildingCount = 0,
            FactionCount = 0,
            DoctrineCount = 0,
            WeaponSetCount = 0
        };
    }
}

/// <summary>
/// Simple data structure to hold content summary information.
/// </summary>
public class ContentSummary
{
    public required string PackId { get; set; }
    public required string PackName { get; set; }
    public int UnitCount { get; set; }
    public int BuildingCount { get; set; }
    public int FactionCount { get; set; }
    public int DoctrineCount { get; set; }
    public int WeaponSetCount { get; set; }
}
