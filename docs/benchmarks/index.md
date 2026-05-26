---
title: Performance Benchmarks
description: BenchmarkDotNet results for DINOForge SDK and Runtime
---

# Performance Benchmarks

Performance benchmarks run on-demand via GitHub Actions to track ContentLoader, Registry, and ECS system performance across releases.

<script setup>
import { ref, onMounted } from 'vue'
const data = ref(null)
const baseline = ref(null)
onMounted(async () => {
  try {
    const r = await fetch('/Dino/benchmarks/latest.json')
    data.value = await r.json()
    const b = await fetch('/Dino/benchmarks/baseline.json')
    baseline.value = await b.json()
  } catch {}
})
</script>

<div v-if="data && baseline">

## Latest Run

**Timestamp:** {{ new Date(data.timestamp).toLocaleString() }}
**Status:** {{ data.status }}

### Regression Gate

Performance regression gate: **>10% slower than baseline = CI failure**

If a benchmark degrades more than 10% from the baseline, the workflow fails and prevents merge until performance is restored.

</div>
<div v-else>

## Latest Run

Performance benchmark results will appear here after the next benchmark run.

</div>

## Benchmark Coverage

We measure:

- **ContentLoader.LoadSimplePack** — Time to load a basic pack (10 units, 5 buildings)
- **ContentLoader.LoadComplexPack** — Time to load a full pack (100+ definitions, cross-references)
- **Registry.QueryUnits** — Time to query 1000 units by faction + role
- **Registry.RegisterBulkUnits** — Time to register 100 units with conflict detection

## Running Locally

```bash
dotnet run --project src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj -- --configuration Release
```

Results appear in `BenchmarkDotNet.Artifacts/results/`.

## Baseline Management

- **First run**: Creates `docs/benchmarks/baseline.json` as the baseline
- **Subsequent runs**: Compare against baseline; >10% regression fails the workflow
- **On success**: Baseline is updated with new results (moving target)

To reset baseline:

```bash
rm docs/benchmarks/baseline.json
```

The next benchmark run will establish a new baseline.

## Historical Data

Benchmark history is available via GitHub Actions artifacts:
- Workflow: `.github/workflows/benchmarks.yml`
- Artifacts: Retained for 90 days
- Latest run: Available in `docs/benchmarks/latest-raw.json`

## CI Integration

Benchmarks run on Ubuntu latest (ubuntu-latest) with:
- .NET 11.0.x
- Release configuration
- Isolated from game instance (no ECS)

Benchmarks do NOT run on Windows (where game deployment occurs) to avoid interference from game processes.
