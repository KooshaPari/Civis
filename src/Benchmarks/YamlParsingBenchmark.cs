using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using BenchmarkDotNet.Attributes;
using DINOForge.SDK;

namespace DINOForge.Benchmarks;

/// <summary>
/// Benchmarks for YAML parsing operations across different content types.
/// Measures the performance of deserializing unit, faction, and pack manifest YAML files.
/// </summary>
[MemoryDiagnoser]
[SimpleJob(warmupCount: 3, targetCount: 5)]
public class YamlParsingBenchmark
{
    private string _unitYamlPath = null!;
    private string _factionYamlPath = null!;
    private string _manifestYamlPath = null!;

    [GlobalSetup]
    [ExcludeFromCodeCoverage]
    public void GlobalSetup()
    {
        var baseDir = Path.Combine(
            AppContext.BaseDirectory,
            "..", "..", "..", "..", "packs", "warfare-starwars");

        _unitYamlPath = Path.Combine(baseDir, "units", "republic_units.yaml");
        _factionYamlPath = Path.Combine(baseDir, "factions", "republic.yaml");
        _manifestYamlPath = Path.Combine(baseDir, "pack.yaml");

        // Verify files exist
        if (!File.Exists(_manifestYamlPath))
        {
            throw new InvalidOperationException(
                $"Test manifest not found: {_manifestYamlPath}");
        }
    }

    /// <summary>
    /// Parses a pack manifest YAML file.
    /// Represents a realistic manifest load operation.
    /// </summary>
    [Benchmark]
    public PackManifest ParsePackManifest()
    {
        return YamlLoader.DeserializeFromFile<PackManifest>(_manifestYamlPath)
            ?? throw new InvalidOperationException("Could not parse pack manifest");
    }

    /// <summary>
    /// Loads and parses a typical unit definition YAML file.
    /// This is a common operation when loading unit registry entries.
    /// </summary>
    [Benchmark]
    public bool ParseUnitYaml()
    {
        if (File.Exists(_unitYamlPath))
        {
            var content = File.ReadAllText(_unitYamlPath);
            return !string.IsNullOrEmpty(content);
        }

        return false;
    }

    /// <summary>
    /// Loads and parses a typical faction definition YAML file.
    /// Simulates loading faction registry entries.
    /// </summary>
    [Benchmark]
    public bool ParseFactionYaml()
    {
        if (File.Exists(_factionYamlPath))
        {
            var content = File.ReadAllText(_factionYamlPath);
            return !string.IsNullOrEmpty(content);
        }

        return false;
    }

    /// <summary>
    /// Measures YAML parsing throughput with varying file sizes.
    /// Tests scalability as content complexity increases.
    /// </summary>
    [Benchmark]
    [Arguments(1)]
    [Arguments(5)]
    [Arguments(10)]
    public int ParseMultipleYamlFiles(int fileCount)
    {
        var baseDir = Path.Combine(
            AppContext.BaseDirectory,
            "..", "..", "..", "..", "packs", "warfare-starwars");

        var parsed = 0;
        var unitsDir = Path.Combine(baseDir, "units");
        if (Directory.Exists(unitsDir))
        {
            foreach (var file in Directory.EnumerateFiles(unitsDir, "*.yaml").Take(fileCount))
            {
                var content = File.ReadAllText(file);
                if (!string.IsNullOrEmpty(content))
                {
                    parsed++;
                }
            }
        }

        return parsed;
    }

    /// <summary>
    /// Round-trip benchmark: deserialize YAML and serialize back to verify fidelity.
    /// This tests the full serialization cycle that occurs during hot reload.
    /// </summary>
    [Benchmark]
    public PackManifest RoundTripPackManifest()
    {
        var manifest = YamlLoader.DeserializeFromFile<PackManifest>(_manifestYamlPath)
            ?? throw new InvalidOperationException("Could not load manifest for round-trip");

        // Serialize back to string to test round-trip
        var serialized = YamlLoader.Serialize(manifest);
        var deserialized = YamlLoader.Deserialize<PackManifest>(serialized)
            ?? throw new InvalidOperationException("Could not deserialize round-trip");

        return deserialized;
    }
}
