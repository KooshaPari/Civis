//! Emergence-migration crate — stress-driven population flows between clusters.
//!
//! # Design
//! Each tick a caller assembles [`ClusterSnapshot`]s from live engine state,
//! passes them to [`MigrationEngine::process`], and receives a
//! [`MigrationResult`] containing population transfers and cultural blend
//! deltas.  The engine guarantees **population conservation**: total population
//! in == total population out.
//!
//! # Cultural blending
//! When a cohort migrates from cluster *A* to cluster *B*, the destination
//! culture vector receives a weighted nudge toward the source: the blend weight
//! is proportional to the fraction of incoming migrants relative to the
//! destination's existing population, clamped to [`MAX_BLEND_WEIGHT`].

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Maximum cultural blend weight per migration step (prevents full takeover).
const MAX_BLEND_WEIGHT: f32 = 0.05;

/// Number of culture dimensions (matches PSYCHE_DIM in agents crate).
pub const CULTURE_DIM: usize = 4;

/// A point-in-time snapshot of one population cluster.
///
/// All population values are signed integers; the engine treats negative values
/// as zero. Stress is a 0.0–1.0 normalised pressure: 0 = stable, 1 = crisis.
/// Opportunity is the inverse: how attractive this cluster is as a destination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterSnapshot {
    /// Stable cluster identifier.
    pub id: u64,
    /// Current population headcount (must be ≥ 0 after clamping).
    pub population: u64,
    /// Stress pressure \[0, 1\]. High stress = more emigrants.
    pub stress: f32,
    /// Opportunity score \[0, 1\]. High opportunity = more immigrants.
    pub opportunity: f32,
    /// Culture profile vector (length [`CULTURE_DIM`]).
    pub culture: [f32; CULTURE_DIM],
}

/// One directional population transfer from cluster `from_id` to `to_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationTransfer {
    pub from_id: u64,
    pub to_id: u64,
    /// Number of people moving.  Always ≥ 1.
    pub count: u64,
    /// Culture of the migrating cohort (snapshot of source at migration time).
    pub cohort_culture: [f32; CULTURE_DIM],
}

/// Cultural blend delta to apply to a destination cluster after migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultureBlendDelta {
    /// Cluster whose culture vector should be nudged.
    pub cluster_id: u64,
    /// Additive delta to apply element-wise (caller clamps result to \[0, 1\]).
    pub delta: [f32; CULTURE_DIM],
}

/// Result of one call to [`MigrationEngine::process`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MigrationResult {
    /// Ordered list of population transfers to apply.
    pub transfers: Vec<PopulationTransfer>,
    /// Cultural blend nudges for each destination that received migrants.
    pub culture_blends: Vec<CultureBlendDelta>,
    /// Total population that moved this step (sum of transfer counts).
    pub total_migrated: u64,
}

/// Deterministic migration engine.
///
/// Stateless between calls; inject the same seed each tick for reproducibility.
pub struct MigrationEngine {
    /// Fraction of a stressed cluster's population that migrates per step.
    /// Default: 2 % (0.02).
    pub emigration_rate: f32,
}

impl Default for MigrationEngine {
    fn default() -> Self {
        Self {
            emigration_rate: 0.02,
        }
    }
}

impl MigrationEngine {
    /// Create an engine with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Process one migration step.
    ///
    /// # Arguments
    /// * `clusters`  – Current state of every cluster in the simulation.
    /// * `tick_seed` – Deterministic seed derived from `tick ^ sim_seed` so
    ///   migration is reproducible but tick-varied.
    ///
    /// # Guarantees
    /// `sum(input.population) == sum(output after applying transfers)`.
    /// In practice: `sum(transfer.count for from X) <= cluster[X].population`.
    pub fn process(&self, clusters: &[ClusterSnapshot], tick_seed: u64) -> MigrationResult {
        if clusters.len() < 2 {
            return MigrationResult::default();
        }

        let mut rng = ChaCha8Rng::seed_from_u64(tick_seed);

        // Build index of clusters with opportunity (potential destinations).
        let destinations: Vec<&ClusterSnapshot> = clusters
            .iter()
            .filter(|c| c.opportunity > 0.0)
            .collect();

        if destinations.is_empty() {
            return MigrationResult::default();
        }

        let mut transfers: Vec<PopulationTransfer> = Vec::new();
        let mut culture_blends: Vec<CultureBlendDelta> = Vec::new();
        let mut total_migrated: u64 = 0;

        for source in clusters.iter().filter(|c| c.stress > 0.0 && c.population > 0) {
            // Emigrants proportional to stress × emigration_rate.
            let raw = (source.population as f32 * source.stress * self.emigration_rate).floor()
                as u64;
            let emigrants = raw.min(source.population);
            if emigrants == 0 {
                continue;
            }

            // Pick destination by opportunity-weighted selection.
            let dest = pick_destination(&mut rng, &destinations, source.id);
            if dest.id == source.id {
                continue;
            }

            // Cultural blend: weight = migrants / (dest_pop + migrants), capped.
            let blend_weight = if dest.population > 0 {
                (emigrants as f32 / (dest.population + emigrants) as f32).min(MAX_BLEND_WEIGHT)
            } else {
                MAX_BLEND_WEIGHT
            };

            let mut delta = [0.0f32; CULTURE_DIM];
            for i in 0..CULTURE_DIM {
                delta[i] = (source.culture[i] - dest.culture[i]) * blend_weight;
            }

            transfers.push(PopulationTransfer {
                from_id: source.id,
                to_id: dest.id,
                count: emigrants,
                cohort_culture: source.culture,
            });

            culture_blends.push(CultureBlendDelta {
                cluster_id: dest.id,
                delta,
            });

            total_migrated += emigrants;
        }

        MigrationResult {
            transfers,
            culture_blends,
            total_migrated,
        }
    }
}

/// Opportunity-weighted random destination selection (excludes `exclude_id`).
fn pick_destination<'a>(
    rng: &mut ChaCha8Rng,
    destinations: &[&'a ClusterSnapshot],
    exclude_id: u64,
) -> &'a ClusterSnapshot {
    use rand::Rng as _;

    let eligible: Vec<&ClusterSnapshot> = destinations
        .iter()
        .copied()
        .filter(|c| c.id != exclude_id)
        .collect();

    if eligible.is_empty() {
        return destinations[0];
    }

    let eligible_opp: f32 = eligible.iter().map(|c| c.opportunity).sum();
    if eligible_opp <= 0.0 {
        return eligible[0];
    }

    let mut r: f32 = rng.gen::<f32>() * eligible_opp;
    for c in &eligible {
        r -= c.opportunity;
        if r <= 0.0 {
            return c;
        }
    }
    eligible[eligible.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cluster(id: u64, pop: u64, stress: f32, opportunity: f32) -> ClusterSnapshot {
        ClusterSnapshot {
            id,
            population: pop,
            stress,
            opportunity,
            culture: [0.5, 0.5, 0.5, 0.5],
        }
    }

    /// Population must be conserved: emigrants leave source, arrive at dest.
    #[test]
    fn population_conservation_two_clusters() {
        let engine = MigrationEngine::new();
        let clusters = vec![
            make_cluster(1, 1_000, 0.8, 0.1), // stressed source
            make_cluster(2, 500, 0.0, 0.9),   // stable destination
        ];
        let total_before: u64 = clusters.iter().map(|c| c.population).sum();

        let result = engine.process(&clusters, 42);

        // Apply transfers manually.
        let mut pops: std::collections::HashMap<u64, u64> =
            clusters.iter().map(|c| (c.id, c.population)).collect();
        for t in &result.transfers {
            *pops.get_mut(&t.from_id).unwrap() -= t.count;
            *pops.get_mut(&t.to_id).unwrap() += t.count;
        }
        let total_after: u64 = pops.values().sum();

        assert_eq!(
            total_before, total_after,
            "population must be conserved: before={total_before} after={total_after}"
        );
    }

    /// Stressed cluster should lose population to the surplus cluster over ticks.
    #[test]
    fn stressed_cluster_loses_population_over_ticks() {
        let engine = MigrationEngine::new();
        let mut clusters = vec![
            make_cluster(1, 2_000, 0.9, 0.0), // high-stress source
            make_cluster(2, 200, 0.0, 1.0),   // high-opportunity dest
        ];

        let initial_stressed = clusters[0].population;
        let mut current = clusters.clone();

        for tick in 0..20u64 {
            let result = engine.process(&current, tick.wrapping_mul(17) ^ 42);
            for t in &result.transfers {
                if let Some(src) = current.iter_mut().find(|c| c.id == t.from_id) {
                    src.population -= t.count;
                }
                if let Some(dst) = current.iter_mut().find(|c| c.id == t.to_id) {
                    dst.population += t.count;
                }
            }
        }

        assert!(
            current[0].population < initial_stressed,
            "stressed cluster should lose population: was={initial_stressed} now={}",
            current[0].population
        );
        assert!(
            current[1].population > clusters[1].population,
            "surplus cluster should gain population"
        );
    }

    /// Same seed must produce same result (determinism).
    #[test]
    fn deterministic_per_seed() {
        let engine = MigrationEngine::new();
        let clusters = vec![
            make_cluster(1, 1_000, 0.6, 0.2),
            make_cluster(2, 800, 0.1, 0.7),
            make_cluster(3, 1_200, 0.4, 0.5),
        ];

        let r1 = engine.process(&clusters, 99999);
        let r2 = engine.process(&clusters, 99999);

        assert_eq!(r1, r2, "same seed must produce identical migration result");
    }

    /// Single cluster: no migration possible.
    #[test]
    fn single_cluster_no_migration() {
        let engine = MigrationEngine::new();
        let clusters = vec![make_cluster(1, 500, 1.0, 1.0)];
        let result = engine.process(&clusters, 1);
        assert!(result.transfers.is_empty());
        assert_eq!(result.total_migrated, 0);
    }
}
