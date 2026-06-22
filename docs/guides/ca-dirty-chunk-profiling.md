# CA dirty-chunk profiling

Use this when you need a benchmark report or flamegraph for the CA dirty-chunk hot path.

## Entry points

- `just ca-perf`
- `just ca-bench`
- `just ca-report`
- `just ca-flamegraph`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-perf.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-dirty-chunk-bench.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-bench-report.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-flamegraph.ps1`

## Output

- `target/criterion`
- `target/ca-dirty-chunk.report.md`
- `target/ca-dirty-chunk.flamegraph.svg`

## What it profiles

- The `ca_dirty_chunk` Criterion bench in `crates/voxel/benches/ca_dirty_chunk.rs`
- The dirty-cell scan, simulation, dirty bookkeeping, and propagation-related
  workload slices that back the `civ-020` perf story

## Notes

- The benchmark itself is still the source of truth for P99 comparisons.
- This guide standardizes the benchmark, report, and profiling entrypoints plus output
  locations.
- `just ca-perf` runs the benchmark first, emits the markdown report, then the
  flamegraph with the same repo-local output path.
- `just ca-report` reads the existing Criterion artifacts and writes the
  markdown summary without rerunning the benchmark.
- `just ca-flamegraph` requires `cargo-flamegraph` (`cargo install flamegraph`).
