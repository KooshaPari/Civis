# CA dirty-chunk profiling

Use this when you need a flamegraph for the CA dirty-chunk hot path.

## Entry points

- `just ca-perf`
- `just ca-bench`
- `just ca-flamegraph`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-perf.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-dirty-chunk-bench.ps1`
- `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-flamegraph.ps1`

## Output

- `target/criterion`
- `target/ca-dirty-chunk.flamegraph.svg`
- GitHub Actions manual quality sweep uploads both as artifacts.

## What it profiles

- The `ca_dirty_chunk` Criterion bench in `crates/voxel/benches/ca_dirty_chunk.rs`
- The dirty-cell scan, simulation, dirty bookkeeping, and propagation-related
  workload slices that back the `civ-020` perf story

## Notes

- The benchmark itself is still the source of truth for P99 comparisons.
- This guide standardizes the benchmark and profiling entrypoints plus output
  locations.
- `just ca-perf` runs the benchmark first, then the flamegraph with the same
  repo-local output path.
- `just ca-flamegraph` requires `cargo-flamegraph` (`cargo install flamegraph`).
