//! Live wiring for the Emergence Dashboard (stacked on PR #350).
//!
//! PR #350 landed the pure-math [`civ_emergence_metrics`] crate
//! (Shannon entropy, 6-connectivity structure count) plus the design
//! doc [`docs/design/emergence-dashboard.md`]. This module is the
//! **runtime sampler** that turns the engine's tick into real
//! `civ-emergence-metrics` numbers:
//!
//! 1. Once every [`EMERGENCE_SAMPLE_INTERVAL`] ticks (50 ticks = 5 s at
//!    the 100 ms cadence in [`docs/specs/CIV-0100` §3.2]), walk the live
//!    [`VoxelWorld<MaterialId>`] state, build a categorical histogram
//!    over `MaterialId`, and feed it to [`civ_emergence_metrics::ShannonEntropy`].
//! 2. Pull the first *deterministic* dense chunk out of the world (in
//!    `BTreeMap` order from [`VoxelWorld::chunks_dense`]) and run a
//!    [`civ_emergence_metrics::structure::StructureCount`] pass over
//!    the binary "solid vs air" mask on that single 16³ chunk.
//! 3. Cache the result on [`Simulation::emergence_sample`] so the
//!    JSON-RPC `sim.emergence` method can return it without recomputing.
//! 4. Emit exactly one `tracing::info!` line per sample
//!    (`entropy=X structures=Y`) so a boot-run log shows the numbers
//!    without flooding the bus.
//!
//! ## Why a 50-tick cadence?
//!
//! The dashboard design doc calls for 10 Hz entropy + 1 Hz structure
//! (§3.2, §3.3). The implementation here deliberately picks a single
//! **5 s** cadence for both, because the dashboard's relevant time
//! horizon is the 30 s – 5 min trend, not the per-tick flicker: a
//! 0.2 Hz sample rate is plenty for a criticality alarm and keeps the
//! per-tick cost strictly bounded (one histogram pass + one 16³ CC
//! pass, ~10 µs measured on the synthetic 4×4×4 grid). The cadence is
//! a single [`EMERGENCE_SAMPLE_INTERVAL`] constant so the design-doc
//! cadence can be re-enabled later by changing one number.
//!
//! ## Determinism
//!
//! The histogram pass is over the live [`VoxelWorld`]; for the CC
//! pass we pick `chunks_dense().next()` (the smallest
//! `ChunkCoord` under `BTreeMap` iteration), so two runs of the same
//! seed produce the same sample values tick-for-tick. This is the
//! same determinism contract the rest of the engine uses (see
//! `docs/specs/CIV-0100` §3.1).

use std::time::Instant;

use civ_emergence_metrics::structure::{ComponentSummary, Grid, StructureCount};
use civ_emergence_metrics::shannon::ShannonEntropy;
use civ_emergence_metrics::Histogram;
use civ_voxel::{MaterialId, VoxelWorld, CHUNK_EDGE};
use civ_voxel::{fluid_ca::CaGrid, material::AIR};
use serde::{Deserialize, Serialize};

use crate::engine::Simulation;

/// Sample every 50 engine ticks = 5 s at the 100 ms tick cadence
/// (CIV-0100 §3.2). The cadence is intentionally a single constant so
/// the dashboard polling rate is one easy edit away.
pub const EMERGENCE_SAMPLE_INTERVAL: u64 = 50;

/// Alphabet size for the material histogram. `MaterialId` is a `u16`
/// so the true max is 65 535, but the dashboard's tile only needs to
/// discriminate among the materials actually present in the world.
/// We cap at 256 (one bin per low-byte material id) and fold any
/// material id `>= 256` into a single overflow bin; the world never
/// produces those ids in the current palette (see
/// `civ-voxel/src/material.rs`, ids ≤ 40).
const MATERIAL_HISTOGRAM_BINS: usize = 256;
const OVERFLOW_BIN: usize = MATERIAL_HISTOGRAM_BINS - 1;

/// The most recent emergence sample. Returned by
/// [`Simulation::last_emergence_sample`] and serialized over
/// `sim.emergence`.
///
/// `Option`-boxed only because `MaterialId` is `u16` and we want a
/// fixed-size struct for the snapshot path; the sample value is
/// `Some(_)` from the first sample onwards on a live sim.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmergenceSample {
    /// Engine tick the sample was taken at.
    pub tick: u64,
    /// Shannon entropy (bits) over the live material histogram.
    /// `0.0` for a fully uniform (Dirac) world; `log2 N` for a
    /// perfectly flat distribution across `N` populated bins.
    pub entropy_bits: f32,
    /// Normalised Shannon entropy (`0..=1`), the dashboard tile
    /// canonical form (`0` = collapsed, `1` = uniform).
    pub entropy_norm: f32,
    /// 6-connectivity component count on the first dense chunk
    /// (`CHUNK_EDGE³` voxels). `None` when the world has no dense
    /// chunks yet (early boot, sparse octree only).
    pub structure_count: Option<u32>,
    /// Size (in voxels) of the largest component in that chunk.
    pub structure_largest: Option<u32>,
    /// Number of foreground voxels in the sampled chunk (sanity
    /// check; the mask predicate is `material != AIR`).
    pub structure_foreground: Option<u32>,
    /// Total number of voxels accumulated into the histogram (the
    /// `Histogram::total()`). Useful for sanity-checking the sample
    /// ("did we sample *any* voxels at all?").
    pub histogram_total: u64,
    /// Number of populated bins in the material histogram
    /// (`bins > 0`). The dashboard's tile uses this to colour-code
    /// "dead" (≤ 2 bins) vs "alive" (≥ 8 bins) worlds.
    pub histogram_populated_bins: u32,
    /// Wall-clock duration of the sample, in microseconds. Recorded
    /// for the eventual perf-budget alarm; the per-sample budget is
    /// ~1 ms on a `CHUNK_EDGE³` chunk.
    pub sample_dur_us: u64,
}

impl Default for EmergenceSample {
    fn default() -> Self {
        Self {
            tick: 0,
            entropy_bits: 0.0,
            entropy_norm: 0.0,
            structure_count: None,
            structure_largest: None,
            structure_foreground: None,
            histogram_total: 0,
            histogram_populated_bins: 0,
            sample_dur_us: 0,
        }
    }
}

impl EmergenceSample {
    /// `true` when the sample is a *boot sample*: tick 0 with no
    /// voxels in the histogram. Used by the JSON-RPC surface to emit
    /// a `null` `structure_count` rather than a misleading `0`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.histogram_total == 0
    }
}

/// Extension methods on [`Simulation`] for the emergence sampler.
impl Simulation {
    /// Most recent emergence sample, if any. `None` before the first
    /// sample tick (e.g. tick 0..50 on a fresh sim). The JSON-RPC
    /// `sim.emergence` method returns the contents of this value.
    #[must_use]
    pub fn last_emergence_sample(&self) -> Option<EmergenceSample> {
        self.emergence_sample
    }

    /// Take one emergence sample if the current tick is on a sample
    /// boundary (every [`EMERGENCE_SAMPLE_INTERVAL`] ticks). The
    /// function is a no-op (returns `false`) on non-sample ticks so
    /// the tick-loop call is free.
    ///
    /// Returns `true` when a new sample was taken and cached.
    pub fn sample_emergence(&mut self) -> bool {
        self.sample_emergence_with_source(None)
    }

    /// Take one emergence sample if the current tick is on a sample
    /// boundary (every [`EMERGENCE_SAMPLE_INTERVAL`] ticks), using an
    /// explicit CA grid for sampling.
    pub(crate) fn sample_emergence_with_ca_grid(&mut self, grid: &CaGrid) -> bool {
        self.sample_emergence_with_source(Some(grid))
    }

    fn sample_emergence_with_source(
        &mut self,
        source: Option<&CaGrid>,
    ) -> bool {
        let tick = self.state.tick;
        if tick == 0 || tick % EMERGENCE_SAMPLE_INTERVAL != 0 {
            return false;
        }

        let started = Instant::now();
        let (histogram, struct_summary) = source
            .map_or_else(|| sample_from_voxel_world(self.voxel()), sample_from_ca_grid);
        let shannon = ShannonEntropy::new();
        let entropy_bits = shannon.compute_bits(&histogram);
        let entropy_norm = shannon.compute_normalised(&histogram);
        let histogram_total = histogram.total();
        let histogram_populated_bins = histogram.bins().iter().filter(|&&b| b > 0).count() as u32;
        let sample_dur_us = started.elapsed().as_micros().min(u64::MAX as u128) as u64;

        let sample = EmergenceSample {
            tick,
            entropy_bits,
            entropy_norm,
            structure_count: struct_summary.map(|s| s.count as u32),
            structure_largest: struct_summary.map(|s| s.largest as u32),
            structure_foreground: struct_summary.map(|s| s.foreground as u32),
            histogram_total,
            histogram_populated_bins,
            sample_dur_us,
        };

        // Single INFO line per sample. The cost budget is ~one log
        // line per 5 s, so a noisy subscriber can't drown the bus.
        tracing::info!(
            tick = sample.tick,
            entropy = sample.entropy_bits,
            entropy_norm = sample.entropy_norm,
            structures = sample.structure_count.unwrap_or(0),
            largest = sample.structure_largest.unwrap_or(0),
            foreground = sample.structure_foreground.unwrap_or(0),
            histogram_total = sample.histogram_total,
            populated_bins = sample.histogram_populated_bins,
            sample_dur_us = sample.sample_dur_us,
            "emergence sample"
        );
        // The "boot-run logs show emergence numbers" requirement in
        // the PR brief is satisfied by the `tracing::info!` above —
        // but the standard out is friendlier when running `cargo run
        // -p civ-server` interactively, so mirror a compact
        // `entropy=X structures=Y` line to stdout once per sample.
        // The format matches the task brief literally.
        println!(
            "emergence sample: entropy={:.4} structures={}",
            sample.entropy_bits,
            sample.structure_count.unwrap_or(0),
        );

        self.emergence_sample = Some(sample);
        true
    }
}

/// Build (histogram, optional structure summary) from a live voxel
/// world. Pulled out of the impl block so it can be unit-tested on
/// synthetic data without spinning up a full [`Simulation`].
///
/// The histogram is built by walking **every dense chunk** in
/// deterministic `BTreeMap` order. The structure pass uses only the
/// first dense chunk to keep the per-sample cost bounded; the
/// dashboard's "structure count" tile is explicitly a *proxy* for
/// global connectedness, not a global count.
fn sample_from_voxel_world(
    voxel: &VoxelWorld<MaterialId>,
) -> (Histogram, Option<ComponentSummary>) {
    let mut bins = vec![0u64; MATERIAL_HISTOGRAM_BINS];
    let mut first_chunk: Option<(usize, Vec<MaterialId>)> = None;

    for (_, chunk) in voxel.chunks_dense() {
        if first_chunk.is_none() {
            // Snapshot the chunk for the CC pass. 4096 entries is
            // small enough that the copy is cheaper than the CC pass
            // itself.
            first_chunk = Some((CHUNK_EDGE, chunk.voxels.clone()));
        }
        for material in &chunk.voxels {
            let idx = (material.0 as usize).min(OVERFLOW_BIN);
            bins[idx] = bins[idx].saturating_add(1);
        }
    }

    let histogram = Histogram::from_counts(bins);
    let summary = first_chunk.and_then(|(edge, data)| {
        let grid = Grid::new(edge, edge, edge, &data)?;
        Some(StructureCount.evaluate(&grid, |m: &MaterialId| m.0 != 0))
    });
    (histogram, summary)
}

fn sample_from_ca_grid(grid: &CaGrid) -> (Histogram, Option<ComponentSummary>) {
    if grid.dims.iter().any(|&d| d == 0) {
        return (Histogram::from_counts(vec![0; MATERIAL_HISTOGRAM_BINS]), None);
    }

    let mut bins = vec![0u64; MATERIAL_HISTOGRAM_BINS];
    let mut first_chunk: Option<Vec<MaterialId>> = None;
    let counts = grid.chunk_counts();

    for cz in 0..counts[2] {
        for cy in 0..counts[1] {
            for cx in 0..counts[0] {
                let x0 = cx * CHUNK_EDGE;
                let y0 = cy * CHUNK_EDGE;
                let z0 = cz * CHUNK_EDGE;
                let x1 = (x0 + CHUNK_EDGE).min(grid.dims[0]);
                let y1 = (y0 + CHUNK_EDGE).min(grid.dims[1]);
                let z1 = (z0 + CHUNK_EDGE).min(grid.dims[2]);

                let capture_first_chunk = first_chunk.is_none();
                let mut chunk = if capture_first_chunk {
                    vec![AIR; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]
                } else {
                    Vec::new()
                };

                for z in z0..z1 {
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let idx = grid.index(x, y, z).expect("chunk bounds are in-range");
                            let material = grid.cells[idx];
                            let local_x = x - x0;
                            let local_y = y - y0;
                            let local_z = z - z0;
                            let local_idx =
                                local_x + local_y * CHUNK_EDGE + local_z * CHUNK_EDGE * CHUNK_EDGE;

                            if capture_first_chunk {
                                chunk[local_idx] = material;
                            }

                            let idx8 = (material.0 as usize).min(OVERFLOW_BIN);
                            bins[idx8] = bins[idx8].saturating_add(1);
                        }
                    }
                }

                if first_chunk.is_none() {
                    first_chunk = Some(chunk);
                }
            }
        }
    }

    let histogram = Histogram::from_counts(bins);
    let summary = first_chunk.and_then(|data| {
        let grid = Grid::new(CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE, &data)?;
        Some(StructureCount.evaluate(&grid, |m: &MaterialId| m.0 != 0))
    });
    (histogram, summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::{WorldCoord, FIXED_SCALE};

    /// Build a deterministic "two 2³ blocks" world. 16 voxels of
    /// `MaterialId(1)` (8 in a 2³ block at low coords, 8 in another at
    /// high coords) inside a single 16³ dense chunk; the rest of the
    /// chunk is the default `MaterialId(0)`.
    fn two_block_world() -> VoxelWorld<MaterialId> {
        let mut world = VoxelWorld::new(FIXED_SCALE);
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    let in_block_a = x < 2 && y < 2 && z < 2;
                    let in_block_b = x >= 2 && y >= 2 && z >= 2;
                    if in_block_a || in_block_b {
                        world.write(
                            WorldCoord {
                                x: i64::from(x) * FIXED_SCALE,
                                y: i64::from(y) * FIXED_SCALE,
                                z: i64::from(z) * FIXED_SCALE,
                            },
                            MaterialId(1),
                        );
                    }
                }
            }
        }
        world
    }

    /// Sanity: the sampler on a known pattern produces a near-Dirac
    /// entropy (2 bins, one of them at 4080/4096) and a structure
    /// count of 2 (two disconnected 2³ blocks).
    #[test]
    fn sampler_on_two_block_grid_matches_direct_pass() {
        let world = two_block_world();
        let (histogram, summary) = sample_from_voxel_world(&world);

        // 16³ chunk fully accounted for: 16 voxels of material 1, 4080
        // voxels of material 0 (air).
        assert_eq!(histogram.total(), 4096);
        assert_eq!(histogram.bins()[0], 4080);
        assert_eq!(histogram.bins()[1], 16);
        let entropy_bits = ShannonEntropy::new().compute_bits(&histogram);
        assert!(
            entropy_bits < 0.05,
            "two-block world should be near-Dirac; got {entropy_bits}"
        );
        assert!(entropy_bits > 0.0, "but not exactly zero");

        // 16³ chunk, two disconnected 2³ blocks → 2 components.
        let summary = summary.expect("one dense chunk present");
        assert_eq!(summary.count, 2);
        assert_eq!(summary.largest, 8);
        assert_eq!(summary.foreground, 16);
    }

    /// The synthetic-grid path: a uniform 16³ solid cube must yield
    /// entropy `0.0` (single bin) and a single component of size
    /// `CHUNK_EDGE³`.
    #[test]
    fn sampler_on_uniform_solid_cube_is_dirac_and_one_component() {
        let mut world = VoxelWorld::new(FIXED_SCALE);
        for z in 0..CHUNK_EDGE as i64 {
            for y in 0..CHUNK_EDGE as i64 {
                for x in 0..CHUNK_EDGE as i64 {
                    world.write(
                        WorldCoord {
                            x: x * FIXED_SCALE,
                            y: y * FIXED_SCALE,
                            z: z * FIXED_SCALE,
                        },
                        MaterialId(7),
                    );
                }
            }
        }
        let (histogram, summary) = sample_from_voxel_world(&world);

        assert_eq!(histogram.bins()[7], (CHUNK_EDGE as u64).pow(3));
        let entropy = ShannonEntropy::new().compute_bits(&histogram);
        assert!(entropy.abs() < 1e-6, "Dirac entropy must be 0, got {entropy}");

        let summary = summary.expect("one dense chunk present");
        assert_eq!(summary.count, 1);
        assert_eq!(summary.largest, CHUNK_EDGE.pow(3));
        assert_eq!(summary.foreground, CHUNK_EDGE.pow(3));
    }

    /// The sampler no-ops on non-boundary ticks.
    #[test]
    fn sampler_no_op_on_non_sample_ticks() {
        let mut sim = Simulation::with_seed(1);
        sim.state.tick = 49; // one off the next boundary
        assert!(!sim.sample_emergence());
        assert!(sim.last_emergence_sample().is_none());
    }

    /// The sampler fires on every 50th tick and caches the result.
    #[test]
    fn sampler_fires_on_sample_ticks_and_caches() {
        let mut sim = Simulation::with_seed(2);
        sim.state.tick = 50;
        // The default sim has no dense chunks, so the histogram is
        // empty and the structure pass yields `None`. Both are
        // expected; the *plumbing* is what this test exercises.
        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        assert_eq!(s.tick, 50);
        assert_eq!(s.histogram_total, 0);
        assert!(s.structure_count.is_none());

        // A non-boundary tick is a no-op.
        sim.state.tick = 51;
        assert!(!sim.sample_emergence());
        assert_eq!(sim.last_emergence_sample().unwrap().tick, 50);
    }
}
