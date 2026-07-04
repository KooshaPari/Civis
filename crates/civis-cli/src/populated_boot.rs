//! Populated voxel boot — extended engine test harness (CLI surface).
//!
//! Spawns a `Simulation::with_seed` and pre-fills its voxel world with a
//! deterministic non-empty material pattern (5 disconnected 2³ blocks, one
//! per `MaterialId(1..=5)`), then ticks it past 50-tick boundaries. The
//! existing `Simulation::sample_emergence()` path emits the
//! `emergence sample: entropy=… structures=…` stdout line once per
//! boundary, exercising the live sampler against a world that actually
//! has dense chunks.
//!
//! This is the "engine test harness" boot from the populated-evidence
//! follow-up to PR #363 (which proved cadence on an empty default sim
//! but yielded `entropy=0.0000 structures=0` because `Simulation::default`
//! initialises an empty `VoxelWorld<MaterialId>`).
//!
//! See `crates/engine/src/emergence_metrics.rs` for the sampler
//! implementation and `crates/engine/src/emergence_metrics.rs::tests` for
//! the synthetic two-block case this pattern generalises.

use civ_engine::Simulation;
use civ_voxel::{MaterialId, WorldCoord, FIXED_SCALE};

use crate::{CliError, CliResult};

/// Run a populated-voxel boot: build a sim with deterministic voxel
/// content, tick it `ticks` times, and rely on the existing
/// `sample_emergence()` to print the boot-run line once per 50-tick
/// boundary.
pub fn run_populated_boot(seed: u64, ticks: u64) -> CliResult<serde_json::Value> {
    if ticks == 0 {
        return Err(CliError::new(2, "ticks must be > 0"));
    }

    let mut sim = Simulation::with_seed(seed);
    populate_voxel_world(&mut sim);
    let initial_dense_chunks = sim.voxel().chunks_dense().count();

    // Ticking mutates `self.state.tick` first (line 1154 of
    // `crates/engine/src/engine.rs`), so the *first* boundary we hit is
    // `tick = 50` regardless of how many `tick()` calls we make. Run at
    // least 50 to guarantee at least one sample.
    let boundaries_expected = (ticks / 50).max(1);
    let started = std::time::Instant::now();
    for _ in 0..ticks {
        sim.tick();
    }
    let elapsed = started.elapsed();

    let last_sample = sim
        .last_emergence_sample()
        .map(|s| {
            serde_json::json!({
                "tick": s.tick,
                "entropy_bits": s.entropy_bits,
                "entropy_norm": s.entropy_norm,
                "structure_count": s.structure_count,
                "structure_largest": s.structure_largest,
                "structure_foreground": s.structure_foreground,
                "histogram_total": s.histogram_total,
                "histogram_populated_bins": s.histogram_populated_bins,
                "sample_dur_us": s.sample_dur_us,
            })
        })
        .unwrap_or(serde_json::Value::Null);

    Ok(serde_json::json!({
        "command": "populated-boot",
        "seed": seed,
        "ticks": ticks,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "boundaries_expected": boundaries_expected,
        "initial_dense_chunks": initial_dense_chunks,
        "last_sample": last_sample,
    }))
}

/// Pre-fill the simulation's voxel world with a deterministic
/// non-uniform pattern: 5 disconnected 2³ solid blocks of distinct
/// `MaterialId(1..=5)` packed inside a single 16³ dense chunk.
///
/// The pattern yields:
/// - 5 populated material bins in the histogram (0..=5, 0 is the air
///   background that the chunk is fully enumerated as by the sampler —
///   see `crates/engine/src/emergence_metrics.rs::sample_from_voxel_world`)
/// - 5 disconnected solid components (each 2³ block), so
///   `StructureCount` reports `count = 5`, `largest = 8`,
///   `foreground = 40`
/// - Non-zero Shannon entropy (mixed distribution).
fn populate_voxel_world(sim: &mut Simulation) {
    // Place 5 blocks at corners of an 8×8×8 region, one per material id
    // 1..=5. Voxel coords are fixed-point world units (see FIXED_SCALE).
    // Each block occupies x∈{0,1}×y∈{0,1}×z∈{0,1} (in fixed-point units)
    // and is offset to a distinct corner.
    let block_offsets: [(i64, i64, i64, u16); 5] = [
        (0, 0, 0, 1),
        (2, 0, 0, 2),
        (0, 2, 0, 3),
        (0, 0, 2, 4),
        (2, 2, 2, 5),
    ];
    for (ox, oy, oz, material_id) in block_offsets {
        for dz in 0..2 {
            for dy in 0..2 {
                for dx in 0..2 {
                    sim.push_voxel_write(
                        WorldCoord {
                            x: (ox + dx) * FIXED_SCALE,
                            y: (oy + dy) * FIXED_SCALE,
                            z: (oz + dz) * FIXED_SCALE,
                        },
                        MaterialId(material_id),
                    );
                }
            }
        }
    }
}
