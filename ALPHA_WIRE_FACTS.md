# ALPHA_WIRE_FACTS — power-law α wiring facts (read-only, no analysis)

## 1. PowerLawFit signature & input

- `civ-emergence-metrics/src/power_law.rs:37` — `pub struct PowerLawFit;` (`#[derive(Default)]`)
- L42-44 — `pub fn new() -> Self` (constructor)
- L53 — `pub fn compute_rank_frequency(&self, input: &Histogram) -> PowerLawResult`
- L99-105 — `impl Metric for PowerLawFit`, `NAME = "power_law_alpha"`; `Metric::compute(&Histogram) -> f32` returns `alpha` only
- Input: `&Histogram` (rank-frequency bin counts). L5-12 doc names use cases: "city sizes, trade volumes, cluster populations" (distribution of size-`k` over `N` entities)
- Return: `PowerLawResult { alpha: f32, r_squared: f32 }` (L27-33). Real power law ≈ α∈2..3 + R²>0.95
- Degenerate: `alpha=0.0, r_squared=0.0` when `n<2` non-empty bins (L63-68, L84-89)

## 2. EmergenceSample struct (existing fields)

`engine/src/emergence_metrics.rs:145-196` — `pub struct EmergenceSample` fields:
`tick:u64` L148 · `entropy_bits:f32` L152 · `entropy_norm:f32` L155 · `structure_count:Option<u32>` L159 · `structure_largest:Option<u32>` L161 · `structure_foreground:Option<u32>` L164 · `histogram_total:u64` L168 · `histogram_populated_bins:u32` L172 · `sample_dur_us:u64` L176 · `dashboard:EmergenceDashboard` L185 · `branching_sigma:f32` L187 · `branching_sigma_score:f32` L189 · `branching_window:u32` L191 · `avalanches_closed:u64` L193 · `branching_regime:BranchingRegime` L195

Cache: `Simulation::emergence_sample`, read via `last_emergence_sample()` (L236-245).

## 3. Where populated (cadence)

- `EMERGENCE_SAMPLE_INTERVAL: u64 = 50` at `engine/src/emergence_metrics.rs:65` — every 50 ticks
- Builder: `fn sample_emergence_with_source(&mut self, source: Option<&CaGrid>) -> bool` at L395-494; no-op when `tick==0 || tick%EMERGENCE_SAMPLE_INTERVAL!=0` (L397-399)
- Public wrappers: `sample_emergence` L384 → `sample_emergence_with_ca_grid` L391 → `sample_emergence_with_source`
- `EmergenceSample { … }` constructed L424-440, stored L476: `self.emergence_sample = Some(sample);`

## 4. Distribution already in `sample_emergence_with_source` that could feed PowerLawFit

- `compute_dashboard(self)` called at L421; builds `cluster_sizes: Vec<u32>` at L631-635
- Source: fold `&ClusterMember` → `BTreeMap<u64, u32>` (cluster id → member count), L631-634
- Output: `cluster_sizes: Vec<u32> = cluster_pop.values().copied().collect();` (L635) — per-cluster population
- Only per-entity-size distribution built every sample boundary. Doc explicitly names "cluster populations" as power-law input (`power_law.rs:6`)
- Note: `PowerLawFit::compute_rank_frequency` consumes `&Histogram`, not `&[u32]` — `cluster_sizes` would need conversion via e.g. `Histogram::from_counts`

## 5. SCHEMA_VERSION

- `civ-emergence-metrics/src/lib.rs:55` — `pub const SCHEMA_VERSION: &str = "0.4.0-branching-ratio";`
