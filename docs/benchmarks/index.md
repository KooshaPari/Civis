---
title: Performance Benchmarks
description: BenchmarkDotNet results for DINOForge SDK and Runtime
---

# Performance Benchmarks

This section tracks performance metrics for critical DINOForge paths using [BenchmarkDotNet](https://benchmarkdotnet.org/).

## Overview

We maintain a comprehensive benchmark suite covering three core areas:

1. **Pack Loading** - Manifest discovery, YAML deserialization, content summarization
2. **YAML Parsing** - Unit, faction, and pack definition parsing at scale
3. **Patch Operations** - Stat override application, wildcard matching, compound patches

## Running Benchmarks Locally

```bash
cd src/Benchmarks
dotnet run -c Release
```

Results are generated in `BenchmarkDotNet.Artifacts/results/`.

## Benchmark Classes

### PackLoadBenchmark

Measures the performance of loading pack manifests and computing content summaries.

**Methods:**
- `LoadSinglePackManifest` - Load a single pack.yaml from disk
- `LoadAllPackManifests` - Discover and load all packs in the packs/ directory
- `BuildContentSummary` - Compute content summary from a manifest
- `LoadMultiplePacksScaling` - Scaling test with [1, 5, 10] packs

**Typical Performance (Baseline):**
- Single manifest load: ~2-5ms
- All packs discovery: ~20-50ms (varies with packs count)
- Content summary: <1ms
- Full pack load (10 packs): ~30-70ms

### YamlParsingBenchmark

Measures YAML deserialization across different content types.

**Methods:**
- `ParsePackManifest` - Parse a full pack.yaml file
- `ParseUnitYaml` - Parse a typical unit definition
- `ParseFactionYaml` - Parse a faction definition
- `ParseMultipleYamlFiles` - Scaling test with [1, 5, 10] files
- `RoundTripPackManifest` - Full serialize/deserialize cycle

**Typical Performance (Baseline):**
- Pack manifest: ~2-3ms
- Unit definition: <1ms
- Faction definition: <1ms
- Round-trip: ~3-4ms

### PatchOperationsBenchmark

Measures stat override and patch operation application.

**Methods:**
- `ApplySingleReplacePatch` - Apply one stat override
- `ApplyMixedPatchSet` - Apply 4 typical balance patches
- `ApplyScalingPatchSet` - Scaling test with [1, 5, 10, 20] patches
- `ApplyWildcardMultiplyPatch` - Complex path matching (wildcard)
- `ApplyFullBalancePatchScenario` - Full realistic patch scenario

**Typical Performance (Baseline):**
- Single replace: <1ms
- 4-patch set: <1ms
- Wildcard multiply: 1-2ms
- Full scenario: 1-2ms

## CI Integration

Benchmarks run nightly via GitHub Actions (`.github/workflows/perf-benchmarks.yml`):

```yaml
schedule:
  - cron: '0 2 * * *'  # 2 AM UTC daily
```

Results are uploaded as artifacts and (on PRs) posted as PR comments for visibility.

## Performance Gates

The benchmark suite establishes performance baselines for critical paths:

- **Target:** No more than 20% regression on existing benchmarks
- **Threshold:** If a single benchmark degrades >20%, investigate and optimize
- **Exemption:** New benchmarks have no baseline and are tracked for the first run

## Viewing Results

### On GitHub
1. Go to Actions → Performance Benchmarks
2. Select the latest run
3. Download "benchmark-results" artifact
4. Open `BenchmarkDotNet.Artifacts/results/` for full reports

### Local Runs
BenchmarkDotNet generates HTML, markdown, and CSV reports:

```
src/Benchmarks/BenchmarkDotNet.Artifacts/
  results/
    {timestamp}_{benchmark_class}-report-full.md
    {timestamp}_{benchmark_class}-report-github.md
    {timestamp}_{benchmark_class}-report.html
```

## Key Metrics

Each benchmark report includes:

| Metric | Meaning |
|--------|---------|
| **Mean** | Average execution time |
| **Median** | 50th percentile (stable execution) |
| **StdDev** | Standard deviation (consistency) |
| **Min/Max** | Outlier bounds |
| **Allocated** | Memory allocated during run |
| **Gen0/Gen1/Gen2** | Garbage collection events |

Lower memory allocation and fewer GC events indicate better performance.

## Adding New Benchmarks

To add a new benchmark class:

1. Create a new file in `src/Benchmarks/`
2. Decorate class with `[MemoryDiagnoser]` and `[SimpleJob(...)]`
3. Implement `[GlobalSetup]` and benchmark `[Benchmark]` methods
4. Use `[Arguments(...)]` for parameterized scaling tests
5. Run locally to verify: `dotnet run -c Release`
6. Commit alongside CHANGELOG update

## References

- [BenchmarkDotNet Documentation](https://benchmarkdotnet.org/)
- [DINOForge SDK Documentation](../README.md)
- [Performance Tips](https://benchmarkdotnet.org/articles/features/statistics.html)
