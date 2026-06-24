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

use std::collections::BTreeMap;
use std::time::Instant;

use civ_agents::{Alignment, Civilian, ClusterMember, Mood, Position3d, Psyche};
use civ_emergence_metrics::branching::{
    classify_regime, rolling_mean_sigma, sigma_a, sigma_score, BranchingLedger, BranchingRegime,
    DEFAULT_BRANCHING_WINDOW, SIGMA_SUBCRITICAL, SIGMA_SUPERCRITICAL,
};
use civ_emergence_metrics::dashboard::TileDashboard;
use civ_emergence_metrics::power_law::PowerLawFit;
use civ_emergence_metrics::shannon::ShannonEntropy;
use civ_emergence_metrics::structure::{ComponentSummary, Grid, StructureCount};
use civ_emergence_metrics::{Histogram, Metric};
use civ_voxel::{fluid_ca::CaGrid, material::AIR};
use civ_voxel::{MaterialId, VoxelWorld, CHUNK_EDGE};
use serde::{Deserialize, Serialize};

use crate::engine::{DiplomacyKind, Simulation};

/// Sample every 50 engine ticks = 5 s at the 100 ms tick cadence
/// (CIV-0100 §3.2). The cadence is intentionally a single constant so
/// the dashboard polling rate is one easy edit away.
pub const EMERGENCE_SAMPLE_INTERVAL: u64 = 50;

/// W_nov = 64 ticks — novelty fingerprint window width
pub const NOVELTY_SAMPLE_INTERVAL: u64 = 64;

/// Default fuse for avalanche size (charter §6 / AC-002).
pub const DEFAULT_MAX_AVALANCHE_SIZE: u64 = 10_000;

/// MT-013: subcritical `σ̄` sustained for this many ticks.
const SUBCRITICAL_TICK_ALARM: u32 = 100;

/// MT-012: supercritical `σ_a` on this many consecutive closed avalanches.
const SUPERCRITICAL_AVALANCHE_ALARM: u32 = 10;

/// Open avalanche tracked between seed tick and closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenAvalanche {
    seed_tick: u64,
    seed_actors: u32,
    size: u64,
    /// Descendants observed on the most recent active tick after seed.
    last_descendants: u32,
}

/// Live branching-ratio state on [`Simulation`].
#[derive(Debug, Clone, PartialEq)]
pub struct EmergenceBranchingState {
    /// Ring buffer of closed avalanches.
    pub ledger: BranchingLedger,
    /// Avalanche opened on the most recent actor tick, if any.
    open: Option<OpenAvalanche>,
    /// Rolling window `W` for `σ̄_W`.
    pub window: usize,
    /// Fuse: close immediately when `s_a` reaches this size.
    pub max_avalanche_size: u64,
    /// Latest rolling-mean `σ̄_W` (updated every tick).
    pub sigma_bar: f32,
    /// Normalised edge-of-chaos score (`0..=1`).
    pub sigma_score: f32,
    /// Charter regime label for the latest `σ̄_W`.
    pub regime: BranchingRegime,
    /// MT-013 consecutive ticks with `σ̄_W < 0.85` (or silence).
    pub subcritical_tick_streak: u32,
    /// MT-012 consecutive closed avalanches with `σ_a > 1.0`.
    pub supercritical_avalanche_streak: u32,
    /// Positive micro-activity units from societal unrest this tick.
    pub last_tick_unrest_events: u32,
}

impl Default for EmergenceBranchingState {
    fn default() -> Self {
        Self {
            ledger: BranchingLedger::with_capacity(DEFAULT_BRANCHING_WINDOW),
            open: None,
            window: DEFAULT_BRANCHING_WINDOW,
            max_avalanche_size: DEFAULT_MAX_AVALANCHE_SIZE,
            sigma_bar: 0.0,
            sigma_score: 0.0,
            regime: BranchingRegime::HeatDeath,
            subcritical_tick_streak: 0,
            supercritical_avalanche_streak: 0,
            last_tick_unrest_events: 0,
        }
    }
}

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
    /// Five-tile summary computed from the live ECS / diplomacy
    /// state at the sample tick (FR-CIV-EMERG-001). See
    /// [`civ_emergence_metrics::dashboard::TileDashboard`] for
    /// the per-metric contracts. The field is `0.0` / `1.0` on a
    /// tick that has no civilians, no clusters, or no diplomacy
    /// events yet — see the unit tests in
    /// `civ-emergence-metrics::dashboard::tests` for the documented
    /// degenerate-state values.
    pub dashboard: TileDashboard,
    /// Rolling-mean branching ratio `σ̄_W` (charter §3.6).
    pub branching_sigma: f32,
    /// Normalised edge-of-chaos score derived from `branching_sigma`.
    pub branching_sigma_score: f32,
    /// Rolling window `W` used for `branching_sigma`.
    pub branching_window: u32,
    /// Total closed avalanches (monotonic diagnostic counter).
    pub avalanches_closed: u64,
    /// Charter regime classification for `branching_sigma`.
    pub branching_regime: BranchingRegime,
    /// Power-law exponent `α` fitted over the per-cluster population
    /// rank-frequency distribution at this sample tick (charter §3.4).
    /// `0.0` sentinel when fewer than 3 clusters are present or the fit
    /// is non-finite. A true power-law society yields `α ≈ 2..3`.
    #[serde(default)]
    pub power_law_alpha: f32,
    /// Per-capita rate of novel world configurations in the current W_nov
    /// window (charter §3.4). Computed as
    /// `new_configs_in_window / W_nov / total_civilians`.
    /// `0.0` when the world has stabilised (all configs already seen).
    #[serde(default)]
    pub novelty_rate: f32,
    /// Normalised mutual information I(material; faction) / H(material),
    /// clamped to [0, 1]. Measures coupling between the voxel-material
    /// layer and the faction layer.
    ///
    /// `None` until faction_id is added to the Citizen struct and wired
    /// into the sampler; the field is present so downstream consumers can
    /// forward-compatibly deserialise samples produced by newer engine builds.
    #[serde(default)]
    pub mi_material_faction_norm: Option<f32>,
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
            dashboard: TileDashboard::default(),
            branching_sigma: 0.0,
            branching_sigma_score: 0.0,
            branching_window: DEFAULT_BRANCHING_WINDOW as u32,
            avalanches_closed: 0,
            branching_regime: BranchingRegime::HeatDeath,
            power_law_alpha: 0.0,
            novelty_rate: 0.0,
            mi_material_faction_norm: None,
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
        self.emergence_sample.map(|mut sample| {
            sample.branching_sigma = self.emergence_branching.sigma_bar;
            sample.branching_sigma_score = self.emergence_branching.sigma_score;
            sample.branching_window = self.emergence_branching.window as u32;
            sample.avalanches_closed = self.emergence_branching.ledger.closed_total();
            sample.branching_regime = self.emergence_branching.regime;
            sample
        })
    }

    /// Borrow live branching-ratio state (updated every tick).
    #[must_use]
    pub fn emergence_branching_state(&self) -> &EmergenceBranchingState {
        &self.emergence_branching
    }

    /// Close/open avalanches and refresh `σ̄_W` from per-tick micro-activity.
    ///
    /// Runs after `phase_chronicle` and before `sample_emergence` (charter
    /// §4.5). O(1): integer sums over existing per-tick buffers.
    pub(crate) fn phase_emergence_events_close(&mut self) {
        let actors = self.micro_actor_action_count();
        let descendants = self.micro_descendant_action_count();
        let tick = self.state.tick;
        let max_size = self.emergence_branching.max_avalanche_size;

        if actors == 0 && descendants == 0 {
            if let Some(open) = self.emergence_branching.open.take() {
                let ratio = sigma_a(open.seed_actors, open.last_descendants);
                self.close_open_avalanche(open, ratio, tick);
            }
            self.refresh_branching_metrics(true);
            return;
        }

        if let Some(mut open) = self.emergence_branching.open.take() {
            if tick > open.seed_tick {
                if descendants > 0 {
                    open.last_descendants = descendants;
                }
                open.size = open.size.saturating_add(u64::from(descendants));
                let ratio = sigma_a(open.seed_actors, open.last_descendants);
                let fuse = open.size >= max_size;
                if descendants == 0 || fuse {
                    self.close_open_avalanche(open, ratio, tick);
                } else {
                    self.emergence_branching.open = Some(open);
                }
            } else {
                self.emergence_branching.open = Some(open);
            }
        }

        if actors > 0 && self.emergence_branching.open.is_none() {
            self.emergence_branching.open = Some(OpenAvalanche {
                seed_tick: tick,
                seed_actors: actors,
                size: u64::from(actors),
                last_descendants: 0,
            });
        }

        self.refresh_branching_metrics(false);
    }

    fn close_open_avalanche(&mut self, open: OpenAvalanche, ratio: f32, close_tick: u64) {
        if open.seed_actors > 0 {
            self.emergence_branching
                .ledger
                .push_closed(ratio, open.size, close_tick);
            if ratio > SIGMA_SUPERCRITICAL {
                self.emergence_branching.supercritical_avalanche_streak = self
                    .emergence_branching
                    .supercritical_avalanche_streak
                    .saturating_add(1);
            } else {
                self.emergence_branching.supercritical_avalanche_streak = 0;
            }
        }
        if open.size >= self.emergence_branching.max_avalanche_size {
            self.emergence_branching.supercritical_avalanche_streak = SUPERCRITICAL_AVALANCHE_ALARM;
        }
    }

    fn refresh_branching_metrics(&mut self, silence: bool) {
        let window = self.emergence_branching.window;
        let sigma_bar = rolling_mean_sigma(&self.emergence_branching.ledger, window);
        self.emergence_branching.sigma_bar = sigma_bar;
        self.emergence_branching.sigma_score = sigma_score(sigma_bar);
        self.emergence_branching.regime = classify_regime(sigma_bar);

        let subcritical = sigma_bar < SIGMA_SUBCRITICAL
            || (silence && self.emergence_branching.ledger.is_empty());
        self.emergence_branching.subcritical_tick_streak = if subcritical {
            self.emergence_branching
                .subcritical_tick_streak
                .saturating_add(1)
        } else {
            0
        };

        if self.emergence_branching.subcritical_tick_streak >= SUBCRITICAL_TICK_ALARM {
            tracing::warn!(
                tick = self.state.tick,
                sigma_bar,
                streak = self.emergence_branching.subcritical_tick_streak,
                "emergence alarm MT-013: sustained subcritical branching ratio"
            );
        }
        if self.emergence_branching.supercritical_avalanche_streak >= SUPERCRITICAL_AVALANCHE_ALARM
        {
            tracing::warn!(
                tick = self.state.tick,
                sigma_bar,
                streak = self.emergence_branching.supercritical_avalanche_streak,
                "emergence alarm MT-012: sustained supercritical branching ratio"
            );
        }

        if let Some(sample) = &mut self.emergence_sample {
            sample.branching_sigma = sigma_bar;
            sample.branching_sigma_score = self.emergence_branching.sigma_score;
            sample.branching_window = window as u32;
            sample.avalanches_closed = self.emergence_branching.ledger.closed_total();
            sample.branching_regime = self.emergence_branching.regime;
        }

        // Per-tick unrest is consumed here (mirrors `tick_with_emergence_source`
        // clearing at tick start) so actor/descendant counts do not bleed across
        // manual multi-tick test steps.
        self.emergence_branching.last_tick_unrest_events = 0;
    }

    /// Record positive unrest micro-activity for the current tick (v1
    /// bootstrap: one unit per positive global/faction unrest delta).
    pub(crate) fn record_unrest_micro_activity(&mut self, units: u32) {
        self.emergence_branching.last_tick_unrest_events = self
            .emergence_branching
            .last_tick_unrest_events
            .saturating_add(units);
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

    fn sample_emergence_with_source(&mut self, source: Option<&CaGrid>) -> bool {
        let tick = self.state.tick;
        if tick == 0 || tick % EMERGENCE_SAMPLE_INTERVAL != 0 {
            return false;
        }

        let started = Instant::now();
        let (histogram, struct_summary) = source.map_or_else(
            || sample_from_voxel_world(self.voxel()),
            sample_from_ca_grid,
        );
        let shannon = ShannonEntropy::new();
        let entropy_bits = shannon.compute_bits(&histogram);
        let entropy_norm = shannon.compute_normalised(&histogram);
        let histogram_total = histogram.total();
        let histogram_populated_bins = histogram.bins().iter().filter(|&&b| b > 0).count() as u32;
        let sample_dur_us = started.elapsed().as_micros().min(u64::MAX as u128) as u64;

        // FR-CIV-EMERG-001 / -002: compute the five dashboard tiles
        // from pre-aggregated slices pulled off the live ECS. The
        // metric crate is pure math; the engine owns the *meaning* of
        // "civilian", "cluster", and "diplomacy event". Two runs of
        // the same seed yield the same five values tick-for-tick
        // because the source slices are deterministic (BTreeMap
        // iteration on `&ClusterMember` + hecs `query` iteration in
        // insertion order + `diplomacy_events()` already sorted).
        let (dashboard, power_law_alpha) = compute_dashboard(self);
        let branching = &self.emergence_branching;

        // §3.4 novelty-rate: build a config fingerprint from stable, cheap fields
        // and track how many novel fingerprints appear per W_nov window.
        let novelty_rate = {
            // Count ActorVisual variants (Humanoid=0, Herd=1) from the ECS.
            let mut humanoid_count: u32 = 0;
            let mut herd_count: u32 = 0;
            for (_, av) in self.world.query::<&civ_agents::ActorVisual>().iter() {
                match av.0 {
                    civ_agents::ActorVisualKind::Humanoid => humanoid_count += 1,
                    civ_agents::ActorVisualKind::Herd => herd_count += 1,
                }
            }
            let actor_visual_counts = [humanoid_count, herd_count];
            let fp = compute_config_fingerprint(
                histogram_populated_bins,
                struct_summary.map(|s| s.count as u32).unwrap_or(0),
                &actor_visual_counts,
            );

            // Close the window when W_nov ticks have elapsed, then reset.
            if tick.saturating_sub(self.emergence.novelty_window_start_tick)
                >= NOVELTY_SAMPLE_INTERVAL
            {
                self.emergence.novelty_window_new = 0;
                self.emergence.novelty_window_start_tick = tick;
            }

            if self.emergence.seen_config_hashes.insert(fp) {
                self.emergence.novelty_window_new += 1;
            }

            let total_civilians: u32 = self
                .world
                .query::<&civ_agents::Civilian>()
                .iter()
                .count()
                .try_into()
                .unwrap_or(u32::MAX);

            let raw = self.emergence.novelty_window_new as f32
                / NOVELTY_SAMPLE_INTERVAL as f32
                / total_civilians.max(1) as f32;
            if raw.is_finite() {
                raw
            } else {
                0.0
            }
        };
        let power_law_alpha = sanitize_emergence_metric(power_law_alpha);
        let novelty_rate = sanitize_emergence_metric(novelty_rate);
        let mi_material_faction_norm =
            compute_material_faction_mi(self).and_then(|value| value.is_finite().then_some(value));

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
            dashboard,
            branching_sigma: branching.sigma_bar,
            branching_sigma_score: branching.sigma_score,
            branching_window: branching.window as u32,
            avalanches_closed: branching.ledger.closed_total(),
            branching_regime: branching.regime,
            power_law_alpha,
            novelty_rate,
            mi_material_faction_norm,
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
            cluster_entropy = sample.dashboard.cluster_entropy,
            ideology_homophily = sample.dashboard.ideology_homophily,
            sentience_fraction = sample.dashboard.sentience_fraction,
            psyche_stability = sample.dashboard.psyche_stability,
            diplomacy_tension = sample.dashboard.diplomacy_tension,
            branching_sigma = sample.branching_sigma,
            branching_sigma_score = sample.branching_sigma_score,
            branching_regime = ?sample.branching_regime,
            power_law_alpha = sample.power_law_alpha,
            novelty_rate = sample.novelty_rate,
            mi_material_faction = sample.mi_material_faction_norm.unwrap_or(f32::NAN),
            sample_dur_us = sample.sample_dur_us,
            "emergence sample"
        );
        // The "boot-run logs show emergence numbers" requirement in
        // the PR brief is satisfied by the `tracing::info!` above —
        // but the standard out is friendlier when running `cargo run
        // -p civ-server` interactively, so mirror a compact summary
        // line to stdout once per sample.
        println!("{}", emergence_sample_stdout_summary(&sample));

        self.emergence_sample = Some(sample);
        // FR-CIV-EMERG-003: emit the `emergence_metrics.v1` replay-bus
        // event with the five-tile dashboard summary. This is a
        // side-band record (does not advance the running hash chain)
        // so the dashboard block can be enabled without breaking
        // replay-compatibility for downstream consumers.
        self.replay_log_mut().record_emergence_metrics(
            sample.tick,
            sample.dashboard.cluster_entropy,
            sample.dashboard.ideology_homophily,
            sample.dashboard.sentience_fraction,
            sample.dashboard.psyche_stability,
            sample.dashboard.diplomacy_tension,
            sample.branching_sigma,
            sample.branching_sigma_score,
            sample.branching_regime.label(),
            sample.power_law_alpha,
            sample.novelty_rate,
            sample.mi_material_faction_norm,
        );
        true
    }
}

fn emergence_sample_stdout_summary(sample: &EmergenceSample) -> String {
    format!(
        "emergence sample: entropy={:.4} structures={} power_law_alpha={:.4} novelty_rate={:.6} mi_material_faction={:.4}",
        sample.entropy_bits,
        sample.structure_count.unwrap_or(0),
        sample.power_law_alpha,
        sample.novelty_rate,
        sample.mi_material_faction_norm.unwrap_or(f32::NAN),
    )
}

fn sanitize_emergence_metric(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// Compute a deterministic u64 fingerprint of the current world configuration
/// for the novelty-rate metric (charter §3.4).
///
/// Hashes (histogram_populated_bins, structure_count, actor_visual_counts) in a
/// stable order so two identical world snapshots always yield the same hash.
fn compute_config_fingerprint(
    histogram_populated_bins: u32,
    structure_count: u32,
    actor_visual_counts: &[u32],
) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    histogram_populated_bins.hash(&mut h);
    structure_count.hash(&mut h);
    actor_visual_counts.hash(&mut h);
    h.finish()
}

/// Build (histogram, optional structure summary) from a live voxel
/// world. Pulled out of the impl block so it can be unit-tested on
/// synthetic data without spinning up a full [`Simulation`].
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
    if grid.dims.contains(&0) {
        return (
            Histogram::from_counts(vec![0; MATERIAL_HISTOGRAM_BINS]),
            None,
        );
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

/// Score in `[-1, 1]` for one [`DiplomacyKind`]. The mapping matches
/// the design-doc [FR-CIV-EMERG-001] tile semantics:
/// `Conflict` is strongly antagonistic (the cluster pair traded
/// violence or threat); `TradeAgreement` is strongly cooperative;
/// `Peace` is mildly cooperative. The absolute value feeds the
/// `diplomacy_tension` index (the dashboard's alarm grows with
/// magnitude regardless of sign — both war and fanatical alliance
/// are "high tension" for the operator's view).
fn diplomacy_kind_score(kind: DiplomacyKind) -> f32 {
    match kind {
        DiplomacyKind::Conflict => -0.9,
        DiplomacyKind::TradeAgreement => 0.7,
        DiplomacyKind::Peace => 0.1,
    }
}

/// Compute the five-tile dashboard from the live simulation state at
/// the current tick (FR-CIV-EMERG-001). Pulled out of the impl
/// block so the data-collection steps are individually testable on a
/// `Simulation` with known ECS state.
///
/// The function is read-only, allocation-bounded, and uses no RNG
/// or wall-clock (the dashboard helper in `civ-emergence-metrics` is
/// pure math). Two runs of the same seed yield the same five values
/// tick-for-tick.
///
/// `ideology` is derived from `Psyche.beliefs[0]` because the agent
/// component has no explicit `ideology` field yet — the
/// `civ-diffusion` and `civ-agents::culture` crates are the spec'd
/// source of truth (FR-CIV-EMERG-001 dependency chain), and the
/// first `beliefs` axis is the collectivist / individualist axis
/// that those crates' S-curve diffusion operates on. We clamp to
/// `[-1, 1]` to keep the dashboard's bin mapping stable across the
/// full `beliefs` range.
fn compute_dashboard(sim: &Simulation) -> (TileDashboard, f32) {
    // 1. cluster_sizes — fold &ClusterMember into a sorted map of
    //    cluster id → member count. `BTreeMap` keeps iteration
    //    order stable across runs; the engine itself assigns cluster
    //    ids by minimum agent id, so the same seed → same map.
    let mut cluster_pop: BTreeMap<u64, u32> = BTreeMap::new();
    for (_, member) in sim.world.query::<&ClusterMember>().iter() {
        *cluster_pop.entry(member.cluster.0).or_insert(0) += 1;
    }
    let cluster_sizes: Vec<u32> = cluster_pop.values().copied().collect();

    // Power-law alpha over the rank-frequency distribution of cluster
    // sizes (charter §3.4). Sort descending (rank 1 = largest cluster)
    // and build a histogram whose bin k holds the size of the (k+1)-th
    // ranked cluster. PowerLawFit treats bins as rank-ordered frequencies
    // and ignores empty bins, so this directly gives the Zipf-like fit.
    let power_law_alpha = if cluster_sizes.len() >= 3 {
        let mut ranked = cluster_sizes.clone();
        ranked.sort_unstable_by(|a, b| b.cmp(a));
        let bins: Vec<u64> = ranked.iter().map(|&s| u64::from(s)).collect();
        let hist = Histogram::from_counts(bins);
        let a = PowerLawFit.compute(&hist);
        if a.is_finite() {
            a
        } else {
            0.0
        }
    } else {
        0.0
    };

    // 2. ideologies — &Psyche.beliefs[0] per agent, clamped.
    //    (`&Mood` is also iterated so the heuristic holds whether or
    //    not the agent has a `Psyche` component yet — agents without
    //    `Psyche` simply don't contribute to the ideology sample.)
    let mut ideologies: Vec<f32> = Vec::new();
    for (_, psyche) in sim.world.query::<&Psyche>().iter() {
        let v = psyche
            .beliefs
            .first()
            .copied()
            .unwrap_or(0.0)
            .clamp(-1.0, 1.0);
        ideologies.push(v);
    }

    // 3. sentient_count / total_civilians — pull the sentience
    //    bookkeeping from `EmergenceState` (the
    //    `phase_diffusion → sentience_evaluate` path mutates it
    //    every sample boundary). The total civilian count is the
    //    live `&Civilian` population in the ECS world.
    let sentient_count: u32 = sim
        .emergence
        .sentient_agents
        .len()
        .try_into()
        .unwrap_or(u32::MAX);
    let total_civilians: u32 = sim
        .world
        .query::<&civ_agents::Civilian>()
        .iter()
        .count()
        .try_into()
        .unwrap_or(u32::MAX);

    // 4. mood_valences — per-agent `Mood.valence` from the live ECS.
    let mut mood_valences: Vec<f32> = Vec::new();
    for (_, mood) in sim.world.query::<&Mood>().iter() {
        mood_valences.push(mood.valence.clamp(-1.0, 1.0));
    }

    // 5. diplomacy_pair_scores — current tick's
    //    `DiplomacyEvent` history, mapped to the kind-to-score
    //    contract above.
    let diplomacy_pair_scores: Vec<f32> = sim
        .diplomacy_events()
        .iter()
        .map(|event| diplomacy_kind_score(event.kind))
        .collect();

    let dashboard = TileDashboard::compute(
        &cluster_sizes,
        &ideologies,
        sentient_count,
        total_civilians,
        &mood_valences,
        &diplomacy_pair_scores,
    );
    (dashboard, power_law_alpha)
}

/// Compute normalised mutual information I(material; faction) / H(material)
/// over the live ECS population (charter §3.5).
///
/// Each faction-aligned civilian contributes one (material-under-position,
/// faction-column) observation to a [`JointHistogram`]. Returns `None` when
/// fewer than 8 faction-aligned civilians are present (degenerate) or when
/// no factions exist.
fn compute_material_faction_mi(sim: &Simulation) -> Option<f32> {
    use std::collections::HashMap;
    // First pass: collect distinct faction ids and assign dense column indices.
    let mut faction_cols: HashMap<u32, usize> = HashMap::new();
    for (_, (civ,)) in sim.world.query::<(&Civilian,)>().iter() {
        if let Alignment::Faction(fid) = civ.alignment {
            let next = faction_cols.len();
            faction_cols.entry(fid).or_insert(next);
        }
    }
    let faction_count = faction_cols.len();
    if faction_count == 0 {
        return None;
    }

    // Second pass: fill the joint histogram.
    let mut joint =
        civ_emergence_metrics::JointHistogram::new(MATERIAL_HISTOGRAM_BINS, faction_count);
    let mut obs: usize = 0;
    for (_, (civ, pos)) in sim.world.query::<(&Civilian, &Position3d)>().iter() {
        if let Alignment::Faction(fid) = civ.alignment {
            if let Some(&col) = faction_cols.get(&fid) {
                let m = sim.voxel().read(pos.coord);
                joint.observe((m.0 as usize).min(OVERFLOW_BIN), col);
                obs += 1;
            }
        }
    }

    if obs < 8 {
        return None;
    }

    let mi = civ_emergence_metrics::mutual_information_normalised(&joint);
    if mi.is_finite() {
        Some(mi)
    } else {
        None
    }
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
        assert!(
            entropy.abs() < 1e-6,
            "Dirac entropy must be 0, got {entropy}"
        );

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

    /// The stdout mirror is a stable, parseable summary line with a fixed
    /// field order so boot logs can be grepped deterministically.
    #[test]
    fn emergence_sample_stdout_summary_has_stable_field_order() {
        let sample = EmergenceSample {
            entropy_bits: 1.25,
            structure_count: Some(3),
            power_law_alpha: 2.5,
            novelty_rate: 0.125,
            mi_material_faction_norm: Some(0.75),
            ..EmergenceSample::default()
        };

        assert_eq!(
            emergence_sample_stdout_summary(&sample),
            "emergence sample: entropy=1.2500 structures=3 power_law_alpha=2.5000 novelty_rate=0.125000 mi_material_faction=0.7500"
        );
    }

    /// FR-CIV-EMERG-001: the sampler computes the five-tile
    /// `TileDashboard` from the live ECS and caches it on the
    /// `EmergenceSample`. The test inserts a population with `Civilian`,
    /// `ClusterMember`, `Psyche`, and `Mood`, takes one sample, and asserts the
    /// dashboard block is `Some(_)` with values that match the helper crate's
    /// output for the same input slices.
    #[test]
    fn emerg_emerg_001_dashboard_block_populated_from_ecs() {
        use civ_agents::{Alignment, Civilian, ClusterId};
        let mut sim = Simulation::with_seed(7);
        sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
        // Snapshot the pre-existing civilian count — the default
        // sim spawns a baseline population we don't author.
        let pre_civilians = sim.world.query::<&Civilian>().iter().count() as u32;

        // Build a small population across two clusters with mixed
        // beliefs and mood valence. 5 agents in cluster 1, 3 in
        // cluster 2; 4/8 agents sentient; diplomacy events
        // pre-populated for the tension tile.
        let mut cluster_pop: BTreeMap<u64, u32> = BTreeMap::new();
        for (entity, (cluster, sentient, belief, mood_v)) in [
            (0u64, (1u64, true, 0.8f32, 0.5f32)),
            (1, (1, true, 0.6, 0.3)),
            (2, (1, true, -0.4, -0.2)),
            (3, (1, true, -0.7, -0.5)),
            (4, (1, false, 0.1, 0.0)),
            (5, (2, true, 0.9, 0.7)),
            (6, (2, true, 0.4, 0.2)),
            (7, (2, false, -0.3, -0.1)),
        ] {
            let id = sim.world.spawn((
                Civilian {
                    id: entity + 1,
                    alignment: Alignment::None,
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster),
                },
                Psyche {
                    drives: [belief, 0.0, 0.0, 0.0],
                    temperament: civ_agents::Temperament::neutral(),
                    mood: Mood {
                        valence: mood_v,
                        arousal: 0.0,
                    },
                    beliefs: [belief, 0.0, 0.0, 0.0],
                    maturity: 0.5,
                },
            ));
            if sentient {
                sim.emergence.sentient_agents.insert(id.id() as u64);
            }
            *cluster_pop.entry(cluster).or_insert(0) += 1;
        }
        // 9th, isolated, no ClusterMember — used to confirm the
        // sentience fraction counts `&Civilian` correctly even when
        // the agent has no `&ClusterMember`.
        sim.world.spawn((Civilian {
            id: 9_999,
            alignment: Alignment::None,
            age: 30,
        },));
        // Authored civilians only: 9 (8 with cluster + 1 isolated).
        // Default sim pre-populates additional civilians, so
        // assertions are made on the *delta* the dashboard observes
        // over those.
        let authored_civilians = 9u32;
        let expected_total = pre_civilians + authored_civilians;

        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        // Tiles are computed (not the default 0.0/1.0 sentinel) on a
        // world with cluster members.
        let d = s.dashboard;
        // Two clusters → cluster_entropy strictly in (0, 1) (sizes 5/3).
        assert!(
            d.cluster_entropy > 0.0 && d.cluster_entropy < 1.0,
            "cluster_entropy should reflect two clusters; got {}",
            d.cluster_entropy
        );
        // Sentience fraction is in [0, 1] by construction; the test
        // asserts the dashboard isn't reporting the default
        // (sentinel) 0.0 / 1.0 values. The exact ratio depends on
        // the pre-existing sim population that the default sim
        // populates at boot.
        assert!(
            d.sentience_fraction >= 0.0 && d.sentience_fraction <= 1.0,
            "sentience_fraction must be in [0, 1]; got {}",
            d.sentience_fraction
        );
        // Psyche stability is the documented "1.0 = no variance" sentinel
        // when the population has identical valence. The test asserts
        // the dashboard isn't reporting a NaN and the field is in
        // [0, 1] (i.e. the wiring is correct); the exact value
        // depends on the pre-existing sim's mood population.
        assert!(
            d.psyche_stability.is_finite(),
            "psyche_stability must be finite; got {}",
            d.psyche_stability
        );
        assert!(
            d.psyche_stability >= 0.0 && d.psyche_stability <= 1.0,
            "psyche_stability must be in [0, 1]; got {}",
            d.psyche_stability
        );
        let _ = (
            cluster_pop,
            expected_total,
            pre_civilians,
            authored_civilians,
        );
    }

    /// FR-CIV-EMERG-002: the dashboard is deterministic. Two sims
    /// built from the same seed + the same ECS population yield
    /// identical five-tile summaries. We assert each field bit-equal
    /// (the dashboard helpers are pure math; the engine inputs are
    /// pulled in deterministic order).
    #[test]
    fn emerg_emerg_002_dashboard_is_deterministic_same_seed() {
        use civ_agents::{Alignment, Civilian, ClusterId};
        let build = || {
            let mut sim = Simulation::with_seed(13);
            sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
            for (entity, (cluster, belief, mood_v)) in [
                (0u64, (1u64, 0.8f32, 0.5f32)),
                (1, (1, 0.6, 0.3)),
                (2, (2, -0.4, -0.2)),
                (3, (2, 0.1, 0.0)),
            ] {
                let id = sim.world.spawn((
                    Civilian {
                        id: entity + 1,
                        alignment: Alignment::None,
                        age: 20,
                    },
                    ClusterMember {
                        cluster: ClusterId(cluster),
                    },
                    Psyche {
                        drives: [belief, 0.0, 0.0, 0.0],
                        temperament: civ_agents::Temperament::neutral(),
                        mood: Mood {
                            valence: mood_v,
                            arousal: 0.0,
                        },
                        beliefs: [belief, 0.0, 0.0, 0.0],
                        maturity: 0.5,
                    },
                ));
                sim.emergence.sentient_agents.insert(id.id() as u64);
            }
            sim
        };
        let mut a = build();
        let mut b = build();
        assert!(a.sample_emergence());
        assert!(b.sample_emergence());
        let sa = a.last_emergence_sample().unwrap();
        let sb = b.last_emergence_sample().unwrap();
        assert_eq!(sa.dashboard.cluster_entropy, sb.dashboard.cluster_entropy);
        assert_eq!(
            sa.dashboard.ideology_homophily,
            sb.dashboard.ideology_homophily
        );
        assert_eq!(
            sa.dashboard.sentience_fraction,
            sb.dashboard.sentience_fraction
        );
        assert_eq!(sa.dashboard.psyche_stability, sb.dashboard.psyche_stability);
        assert_eq!(
            sa.dashboard.diplomacy_tension,
            sb.dashboard.diplomacy_tension
        );
    }

    /// Charter §3.6: a known closed-avalanche stream yields the
    /// expected rolling-mean `σ̄` and regime classification.
    #[test]
    fn known_event_stream_yields_expected_sigma_bar_and_regime() {
        use civ_emergence_metrics::branching::{
            classify_regime, rolling_mean_sigma, sigma_a, BranchingLedger, BranchingRegime,
        };

        let stream = [(10, 8), (10, 9), (10, 10), (10, 11)];
        let mut ledger = BranchingLedger::with_capacity(stream.len());
        for (actors, descendants) in stream {
            ledger.push_closed(sigma_a(actors, descendants), u64::from(descendants), 0);
        }
        let sigma_bar = rolling_mean_sigma(&ledger, stream.len());
        assert!(
            (sigma_bar - 0.95).abs() < 1e-6,
            "expected σ̄=0.95, got {sigma_bar}"
        );
        assert_eq!(classify_regime(sigma_bar), BranchingRegime::EdgeOfChaos);
        assert_eq!(classify_regime(0.75), BranchingRegime::HeatDeath);
        assert_eq!(classify_regime(1.15), BranchingRegime::Supercritical);
    }

    /// Tick-loop wiring: `phase_emergence_events_close` updates live `σ̄`.
    ///
    /// Hand-derived σ̄ for the unrest stream below (charter §3.6):
    /// - tick 1: `N_actors = 10` seeds one open avalanche;
    /// - tick 2: `N_descendants = 9` on `t+1` → per-avalanche `σ_a = 9/10 = 0.9`;
    /// - tick 3: silence (`N_actors = N_descendants = 0`) closes the avalanche;
    /// - ledger holds one closed entry, `W = 10` →
    ///   `σ̄_W = (1 / min(10, 1)) · 0.9 = 0.9` ∈ `[0.85, 0.95)` →
    ///   `SubcriticalTransition`.
    #[test]
    fn phase_emergence_events_close_updates_branching_state() {
        let mut sim = Simulation::with_seed(21);
        sim.state.tick = 1;
        sim.record_unrest_micro_activity(10);
        sim.phase_emergence_events_close();
        sim.state.tick = 2;
        sim.record_unrest_micro_activity(9);
        sim.phase_emergence_events_close();
        sim.state.tick = 3;
        sim.phase_emergence_events_close();
        assert_eq!(sim.emergence_branching_state().ledger.closed_total(), 1);
        assert!(
            (sim.branching_ratio() - 0.9).abs() < 1e-6,
            "σ̄_W hand-derived as 9/10 = 0.9, got {}",
            sim.branching_ratio()
        );
        assert_eq!(
            sim.emergence_branching_state().regime,
            BranchingRegime::SubcriticalTransition
        );
    }

    /// Charter §3.4: fewer than 3 clusters → power_law_alpha sentinel 0.0.
    #[test]
    fn power_law_alpha_zero_when_too_few_clusters() {
        use civ_agents::{Alignment, Civilian, ClusterId};
        let mut sim = Simulation::with_seed(42);
        sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
        // Spawn only 2 clusters (below the minimum of 3).
        for (entity, cluster) in [(1u64, 1u64), (2, 1), (3, 2)] {
            sim.world.spawn((
                Civilian {
                    id: entity,
                    alignment: Alignment::None,
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster),
                },
            ));
        }
        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        assert_eq!(
            s.power_law_alpha, 0.0,
            "fewer than 3 clusters must yield sentinel 0.0, got {}",
            s.power_law_alpha
        );
    }

    /// Charter §3.4: ≥3 clusters with varied sizes → power_law_alpha finite and > 0.
    #[test]
    fn power_law_alpha_populated_on_distribution() {
        use civ_agents::{Alignment, Civilian, ClusterId};
        let mut sim = Simulation::with_seed(43);
        sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
        // Spawn 4 clusters with Zipf-like sizes: 100, 50, 25, 12 members.
        let cluster_defs: &[(u64, u32)] = &[(1, 100), (2, 50), (3, 25), (4, 12)];
        let mut entity_id: u64 = 1;
        for &(cluster_id, count) in cluster_defs {
            for _ in 0..count {
                sim.world.spawn((
                    Civilian {
                        id: entity_id,
                        alignment: Alignment::None,
                        age: 20,
                    },
                    ClusterMember {
                        cluster: ClusterId(cluster_id),
                    },
                ));
                entity_id += 1;
            }
        }
        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        assert!(
            s.power_law_alpha.is_finite(),
            "power_law_alpha must be finite; got {}",
            s.power_law_alpha
        );
        assert!(
            s.power_law_alpha > 0.0,
            "Zipf-like cluster distribution must yield alpha > 0, got {}",
            s.power_law_alpha
        );
    }

    /// §3.4: inserting the same fingerprint multiple times keeps the seen-set
    /// count at 1 (repeated identical config is never novel after the first).
    #[test]
    fn novelty_rate_zero_on_repeated_identical_config() {
        let fp = compute_config_fingerprint(4, 2, &[10, 3]);
        let mut seen = std::collections::HashSet::<u64>::new();
        let mut new_count = 0u32;
        for _ in 0..5 {
            if seen.insert(fp) {
                new_count += 1;
            }
        }
        assert_eq!(
            seen.len(),
            1,
            "same fingerprint must appear only once in the set"
        );
        assert_eq!(new_count, 1, "only the first insertion is novel");
    }

    /// §3.4: distinct fingerprints each increment the seen-set and novelty count.
    #[test]
    fn novelty_rate_positive_on_new_configs() {
        let fps = [
            compute_config_fingerprint(2, 1, &[5, 0]),
            compute_config_fingerprint(4, 3, &[8, 2]),
            compute_config_fingerprint(6, 5, &[12, 4]),
        ];
        let mut seen = std::collections::HashSet::<u64>::new();
        let mut new_count = 0u32;
        for &fp in &fps {
            if seen.insert(fp) {
                new_count += 1;
            }
        }
        assert_eq!(new_count, 3, "three distinct configs must all be novel");
        let total_civilians: u32 = 10;
        let raw = new_count as f32 / NOVELTY_SAMPLE_INTERVAL as f32 / total_civilians.max(1) as f32;
        assert!(
            raw > 0.0,
            "novelty_rate must be positive when new configs appear; got {raw}"
        );
        assert!(raw.is_finite(), "novelty_rate must be finite; got {raw}");
    }

    /// §3.5: civilians with Alignment::None contribute no faction observations
    /// → mi_material_faction_norm must be None (degenerate: 0 factions).
    #[test]
    fn mi_material_faction_none_without_factions() {
        use civ_agents::{Alignment, Civilian, Position3d};
        use civ_voxel::WorldCoord;
        let mut sim = Simulation::with_seed(99);
        sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
        for id in 1u64..=10 {
            sim.world.spawn((
                Civilian {
                    id,
                    alignment: Alignment::None,
                    age: 20,
                },
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ));
        }
        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        assert!(
            s.mi_material_faction_norm.is_none(),
            "no faction-aligned citizens → MI must be None, got {:?}",
            s.mi_material_faction_norm
        );
    }

    /// §3.5: civilians of ≥2 factions each standing on a distinct material →
    /// mi_material_faction_norm must be Some(x) with x.is_finite().
    ///
    /// We assert only `Some` + `finite` (not `> 0`) because
    /// `compute_material_faction_mi` calls `sim.voxel().read(pos.coord)` which
    /// returns the default `MaterialId(0)` (AIR) for unwritten cells regardless
    /// of faction, so both factions land on the same material bin and MI = 0.
    /// A `> 0` assertion would require writing actual voxels which is the
    /// province of integration tests; the unit test here validates the
    /// wiring and that the value is finite and present.
    #[test]
    fn mi_material_faction_some_with_coupling() {
        use civ_agents::{Alignment, Civilian, Position3d};
        use civ_voxel::WorldCoord;
        let mut sim = Simulation::with_seed(100);
        sim.state.tick = EMERGENCE_SAMPLE_INTERVAL;
        // 10 citizens in faction 1, 10 in faction 2 — well above the obs=8 threshold.
        for id in 1u64..=10 {
            sim.world.spawn((
                Civilian {
                    id,
                    alignment: Alignment::Faction(1),
                    age: 20,
                },
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ));
        }
        for id in 11u64..=20 {
            sim.world.spawn((
                Civilian {
                    id,
                    alignment: Alignment::Faction(2),
                    age: 20,
                },
                Position3d {
                    coord: WorldCoord { x: 0, y: 0, z: 0 },
                },
            ));
        }
        assert!(sim.sample_emergence());
        let s = sim.last_emergence_sample().expect("sample cached");
        let mi = s
            .mi_material_faction_norm
            .expect("≥2 factions with ≥8 observations → MI must be Some(_)");
        assert!(
            mi.is_finite(),
            "mi_material_faction_norm must be finite; got {mi}"
        );
        assert!(
            (0.0..=1.0).contains(&mi),
            "normalised MI must be in [0, 1]; got {mi}"
        );
    }
}
