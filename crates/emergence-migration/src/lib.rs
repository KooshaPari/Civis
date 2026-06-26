//! civ-emergence-migration — emergent population migration substrate.
//!
//! Implements `FR-CIV-MIGRATION` from `docs/traceability/fr-emergence-matrix.md`.
//! Population flow is **emergent**: it is computed each tick from cluster state
//! (scarcity, disasters, war, overpopulation vs. surplus, safety, capacity) — it
//! is never scripted. Migrants carry culture/language/belief attributes that blend
//! into the destination, which **counters** language/religion divergence. Disasters
//! and wars raise refugee surges that decay over time after the event ends.
//!
//! Determinism is sacred (ADR-008): every stochastic step takes a seeded
//! [`rand_chacha::ChaCha8Rng`]; the same seed + same state ⇒ identical outcome.
//!
//! Traceability:
//! - Feature 1 push/pull engine — `FR-CIV-MIGRATION` (push/pull)
//! - Feature 2 settlement reshaping — `FR-CIV-MIGRATION` (resettlement)
//! - Feature 3 cultural mixing / counter-divergence — couples to `FR-CIV-LANG-*`,
//!   `FR-CIV-REL-*`, `FR-CIV-CULT-002`
//! - Feature 4 refugee surges — couples to `FR-CIV-CLIMATE-002`, `FR-CIV-DIPLO-004`

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version for save/replay compatibility.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Dimension of culture/language/belief attribute vectors (mirrors agents `PSYCHE_DIM`).
pub const ATTR_DIM: usize = 4;

/// A culture/language/belief attribute vector, each component normalized to `[0, 1]`.
pub type AttrVector = [f32; ATTR_DIM];

/// Stable cluster identifier (mirrors `civ_agents::ClusterId`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClusterId(pub u64);

// ---------------------------------------------------------------------------
// Feature 1: Push/Pull migration engine — stress & opportunity factors
// ---------------------------------------------------------------------------

/// Factors that **push** population out of a cluster (higher ⇒ more outflow).
///
/// All fields are normalized signals in `[0, 1]` except `overpopulation_ratio`,
/// which is `population / carrying_capacity` and may exceed `1.0`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MigrationStress {
    /// Resource scarcity score `[0, 1]` (1.0 = total scarcity / famine).
    pub scarcity: f32,
    /// Active disaster severity `[0, 1]` (flood/storm/drought/wildfire/etc).
    pub disaster_severity: f32,
    /// War intensity `[0, 1]` drawn from diplomacy war drains.
    pub war_intensity: f32,
    /// Overpopulation ratio = population / carrying_capacity (≥ 0, may exceed 1).
    pub overpopulation_ratio: f32,
}

impl Default for MigrationStress {
    fn default() -> Self {
        Self {
            scarcity: 0.0,
            disaster_severity: 0.0,
            war_intensity: 0.0,
            overpopulation_ratio: 0.0,
        }
    }
}

impl MigrationStress {
    /// Aggregate push pressure in `[0, 1]`.
    ///
    /// Scarcity, disaster, and war contribute directly; overpopulation contributes
    /// only its excess above carrying capacity (so an under-capacity cluster adds
    /// no overpopulation push).
    #[must_use]
    pub fn pressure(&self) -> f32 {
        let overcrowd = (self.overpopulation_ratio - 1.0).max(0.0);
        let raw = self.scarcity + self.disaster_severity + self.war_intensity + overcrowd;
        clamp01(raw / 4.0)
    }
}

/// Factors that **pull** population into a cluster (higher ⇒ more inflow attraction).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MigrationOpportunity {
    /// Economic surplus score `[0, 1]` (food/goods available above local need).
    pub surplus: f32,
    /// Safety score `[0, 1]` (1.0 = no disaster/war pressure).
    pub safety: f32,
    /// Spare capacity score `[0, 1]` = remaining_capacity / carrying_capacity.
    pub capacity: f32,
}

impl Default for MigrationOpportunity {
    fn default() -> Self {
        Self {
            surplus: 0.0,
            safety: 1.0,
            capacity: 0.0,
        }
    }
}

impl MigrationOpportunity {
    /// Aggregate pull attractiveness in `[0, 1]`.
    ///
    /// A cluster with no spare capacity cannot attract migrants regardless of
    /// surplus/safety, so capacity acts as a gating multiplier.
    #[must_use]
    pub fn attractiveness(&self) -> f32 {
        let base = (self.surplus + self.safety) / 2.0;
        clamp01(base * clamp01(self.capacity))
    }
}

/// Full per-cluster migration state used by the engine each tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClusterMigration {
    /// Cluster identity.
    pub id: ClusterId,
    /// Current population count.
    pub population: u64,
    /// Carrying capacity (soft cap; overpopulation pushes out).
    pub carrying_capacity: u64,
    /// Push factors.
    pub stress: MigrationStress,
    /// Pull factors.
    pub opportunity: MigrationOpportunity,
    /// Cultural meme vector.
    pub culture: AttrVector,
    /// Language drift vector.
    pub language: AttrVector,
    /// Belief / religion vector.
    pub belief: AttrVector,
    /// Active refugee surge multiplier (≥ 1.0). Decays toward 1.0 each tick.
    pub surge: f32,
}

impl ClusterMigration {
    /// Construct a cluster with neutral culture and no active surge.
    #[must_use]
    pub fn new(id: ClusterId, population: u64, carrying_capacity: u64) -> Self {
        Self {
            id,
            population,
            carrying_capacity,
            stress: MigrationStress::default(),
            opportunity: MigrationOpportunity::default(),
            culture: [0.5; ATTR_DIM],
            language: [0.5; ATTR_DIM],
            belief: [0.5; ATTR_DIM],
            surge: 1.0,
        }
    }

    /// Effective push pressure including any active refugee surge.
    #[must_use]
    pub fn effective_push(&self) -> f32 {
        clamp01(self.stress.pressure() * self.surge)
    }

    /// Effective pull attractiveness (surge does not amplify pull).
    #[must_use]
    pub fn effective_pull(&self) -> f32 {
        self.opportunity.attractiveness()
    }
}

// ---------------------------------------------------------------------------
// Engine configuration & flow records
// ---------------------------------------------------------------------------

/// Tunable parameters for the migration engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MigrationConfig {
    /// Maximum fraction of a cluster's population that may emigrate in one tick.
    pub max_emigration_fraction: f32,
    /// Surge multiplier applied when a disaster strikes.
    pub disaster_surge_multiplier: f32,
    /// Surge multiplier applied when a war is active.
    pub war_surge_multiplier: f32,
    /// Per-tick exponential decay applied to the surge toward 1.0 (`[0, 1)`).
    pub surge_decay: f32,
    /// Blend rate `[0, 1]` for migrant attributes mixing into the destination.
    pub culture_blend_rate: f32,
    /// Stochastic jitter `[0, 1]` applied to per-pair flow (0 = fully deterministic split).
    pub flow_jitter: f32,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            max_emigration_fraction: 0.10,
            disaster_surge_multiplier: 3.0,
            war_surge_multiplier: 2.5,
            surge_decay: 0.20,
            culture_blend_rate: 0.25,
            flow_jitter: 0.05,
        }
    }
}

/// A single resolved migration flow from one cluster to another this tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationFlow {
    /// Origin cluster.
    pub from: ClusterId,
    /// Destination cluster.
    pub to: ClusterId,
    /// Number of people who moved.
    pub count: u64,
}

/// Outcome of one migration tick: the resolved flows plus aggregate counters.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MigrationReport {
    /// All non-zero flows, sorted deterministically by (from, to).
    pub flows: Vec<MigrationFlow>,
    /// Total people moved across all flows.
    pub total_moved: u64,
}

impl MigrationReport {
    /// Total migrants that left a given cluster this tick.
    #[must_use]
    pub fn emigrants_from(&self, id: ClusterId) -> u64 {
        self.flows
            .iter()
            .filter(|f| f.from == id)
            .map(|f| f.count)
            .sum()
    }

    /// Total migrants that arrived at a given cluster this tick.
    #[must_use]
    pub fn immigrants_to(&self, id: ClusterId) -> u64 {
        self.flows
            .iter()
            .filter(|f| f.to == id)
            .map(|f| f.count)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// Feature 4: Refugee surges — event triggers
// ---------------------------------------------------------------------------

/// An exogenous event that triggers a refugee surge on a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurgeEvent {
    /// Disaster (flood/storm/drought/wildfire/quake/…) struck this cluster.
    Disaster {
        /// Affected cluster.
        cluster: ClusterId,
        /// Disaster severity `[0, 1]`.
        severity: f32,
    },
    /// War became active on this cluster.
    War {
        /// Affected cluster.
        cluster: ClusterId,
        /// War intensity `[0, 1]`.
        intensity: f32,
    },
}

// ---------------------------------------------------------------------------
// The migration engine
// ---------------------------------------------------------------------------

/// Stateful, deterministic migration engine over a set of clusters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationEngine {
    /// Clusters keyed by id (BTreeMap ⇒ deterministic iteration order).
    clusters: BTreeMap<ClusterId, ClusterMigration>,
    /// Engine configuration.
    pub config: MigrationConfig,
}

impl MigrationEngine {
    /// Create an empty engine with default config.
    #[must_use]
    pub fn new(config: MigrationConfig) -> Self {
        Self {
            clusters: BTreeMap::new(),
            config,
        }
    }

    /// Insert or replace a cluster.
    pub fn upsert(&mut self, cluster: ClusterMigration) {
        self.clusters.insert(cluster.id, cluster);
    }

    /// Read-only access to a cluster.
    #[must_use]
    pub fn cluster(&self, id: ClusterId) -> Option<&ClusterMigration> {
        self.clusters.get(&id)
    }

    /// Number of clusters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Whether the engine has any clusters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    /// **Feature 4** — apply a refugee surge from an exogenous event.
    ///
    /// Disasters and wars raise the affected cluster's surge multiplier (taking the
    /// max so the worst concurrent event wins) and also feed the relevant stress
    /// factor so the surge has a real push to amplify.
    pub fn apply_event(&mut self, event: SurgeEvent) {
        let (id, mult, set_stress): (ClusterId, f32, fn(&mut MigrationStress, f32)) = match event {
            SurgeEvent::Disaster { cluster, severity } => (
                cluster,
                1.0 + (self.config.disaster_surge_multiplier - 1.0) * clamp01(severity),
                |s, v| s.disaster_severity = s.disaster_severity.max(v),
            ),
            SurgeEvent::War { cluster, intensity } => (
                cluster,
                1.0 + (self.config.war_surge_multiplier - 1.0) * clamp01(intensity),
                |s, v| s.war_intensity = s.war_intensity.max(v),
            ),
        };
        let sev = match event {
            SurgeEvent::Disaster { severity, .. } => severity,
            SurgeEvent::War { intensity, .. } => intensity,
        };
        if let Some(c) = self.clusters.get_mut(&id) {
            c.surge = c.surge.max(mult);
            set_stress(&mut c.stress, clamp01(sev));
        }
    }

    /// **Feature 4** — decay every cluster's surge one step toward 1.0.
    ///
    /// Called automatically at the end of [`MigrationEngine::tick`]; exposed for
    /// tests of surge decay in isolation.
    pub fn decay_surges(&mut self) {
        let keep = 1.0 - clamp01(self.config.surge_decay);
        for c in self.clusters.values_mut() {
            c.surge = 1.0 + (c.surge - 1.0) * keep;
            if c.surge < 1.0 + 1e-4 {
                c.surge = 1.0;
            }
        }
    }

    /// Advance one migration tick.
    ///
    /// 1. **Feature 1** computes net push/pull and resolves flows from high-stress
    ///    origins to high-opportunity destinations (proportional to attractiveness).
    /// 2. **Feature 2** applies arrivals/departures to cluster populations.
    /// 3. **Feature 3** blends migrant culture/language/belief into destinations,
    ///    reducing divergence.
    /// 4. **Feature 4** decays refugee surges.
    ///
    /// Deterministic for a fixed `rng` seed and cluster state.
    pub fn tick(&mut self, rng: &mut ChaCha8Rng) -> MigrationReport {
        let flows = self.compute_flows(rng);
        self.apply_flows(&flows);
        self.decay_surges();

        let total_moved = flows.iter().map(|f| f.count).sum();
        MigrationReport { flows, total_moved }
    }

    /// **Feature 1** — compute emergent flows for this tick (no mutation).
    fn compute_flows(&self, rng: &mut ChaCha8Rng) -> Vec<MigrationFlow> {
        // Destinations ranked by pull; iterate deterministically.
        let ids: Vec<ClusterId> = self.clusters.keys().copied().collect();
        let pulls: Vec<(ClusterId, f32)> = ids
            .iter()
            .map(|id| (*id, self.clusters[id].effective_pull()))
            .filter(|(_, p)| *p > 0.0)
            .collect();
        let total_pull: f32 = pulls.iter().map(|(_, p)| *p).sum();

        let mut flows: Vec<MigrationFlow> = Vec::new();
        if total_pull <= 0.0 {
            return flows;
        }

        for origin_id in &ids {
            let origin = &self.clusters[origin_id];
            let push = origin.effective_push();
            if push <= 0.0 || origin.population == 0 {
                continue;
            }
            // Net flow = push gated by max emigration fraction.
            let leaving =
                (origin.population as f32 * self.config.max_emigration_fraction * push) as u64;
            if leaving == 0 {
                continue;
            }

            // Distribute leavers across destinations proportional to pull, excluding
            // the origin itself. Deterministic largest-remainder allocation.
            let mut remaining = leaving;
            let dests: Vec<(ClusterId, f32)> =
                pulls.iter().copied().filter(|(d, _)| d != origin_id).collect();
            let dest_pull: f32 = dests.iter().map(|(_, p)| *p).sum();
            if dest_pull <= 0.0 {
                continue;
            }

            let mut shares: Vec<(ClusterId, u64, f32)> = Vec::with_capacity(dests.len());
            for (dest_id, pull) in &dests {
                let frac = pull / dest_pull;
                let exact = leaving as f32 * frac;
                // Optional deterministic jitter keeps ties from always favoring low ids.
                let jitter = if self.config.flow_jitter > 0.0 {
                    1.0 + (rng.gen::<f32>() - 0.5) * 2.0 * self.config.flow_jitter
                } else {
                    1.0
                };
                let count = (exact * jitter).floor().max(0.0) as u64;
                shares.push((*dest_id, count, exact - count as f32));
            }

            // Assign floors, then hand out the remainder by largest fractional part.
            let assigned: u64 = shares.iter().map(|(_, c, _)| *c).sum();
            let mut leftover = leaving.saturating_sub(assigned);
            shares.sort_by(|a, b| {
                b.2.partial_cmp(&a.2)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.0.cmp(&b.0))
            });
            for share in shares.iter_mut() {
                if leftover == 0 {
                    break;
                }
                share.1 += 1;
                leftover -= 1;
            }

            for (dest_id, count, _) in shares {
                let count = count.min(remaining);
                if count == 0 {
                    continue;
                }
                remaining -= count;
                flows.push(MigrationFlow {
                    from: *origin_id,
                    to: dest_id,
                    count,
                });
            }
        }

        // Deterministic ordering for stable reports/replay.
        flows.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        flows
    }

    /// **Feature 2 + 3** — apply flows: reshape populations and blend culture.
    fn apply_flows(&mut self, flows: &[MigrationFlow]) {
        for flow in flows {
            // Snapshot the migrant cohort's attributes from the origin.
            let (mig_culture, mig_language, mig_belief) = {
                let origin = match self.clusters.get_mut(&flow.from) {
                    Some(c) => c,
                    None => continue,
                };
                let count = flow.count.min(origin.population);
                origin.population -= count; // Feature 2: departures
                (origin.culture, origin.language, origin.belief)
            };

            if let Some(dest) = self.clusters.get_mut(&flow.to) {
                let prev_pop = dest.population;
                dest.population += flow.count; // Feature 2: arrivals

                // Feature 3: blend migrant attributes into destination, weighted by
                // the size of the arriving cohort relative to the new total. This
                // moves the destination's vectors toward the migrants', countering
                // divergence between source and destination.
                let cohort_weight = if dest.population > 0 {
                    (flow.count as f32 / dest.population as f32).min(1.0)
                } else {
                    0.0
                };
                let blend = clamp01(self.config.culture_blend_rate * (cohort_weight + 0.5));
                blend_into(&mut dest.culture, &mig_culture, blend);
                blend_into(&mut dest.language, &mig_language, blend);
                blend_into(&mut dest.belief, &mig_belief, blend);
                let _ = prev_pop;
            }
        }
    }

    /// Divergence between two clusters' language vectors (Euclidean, `[0, 1]`-ish).
    #[must_use]
    pub fn language_divergence(&self, a: ClusterId, b: ClusterId) -> Option<f32> {
        let (ca, cb) = (self.clusters.get(&a)?, self.clusters.get(&b)?);
        Some(vector_distance(&ca.language, &cb.language))
    }

    /// Divergence between two clusters' belief/religion vectors.
    #[must_use]
    pub fn belief_divergence(&self, a: ClusterId, b: ClusterId) -> Option<f32> {
        let (ca, cb) = (self.clusters.get(&a)?, self.clusters.get(&b)?);
        Some(vector_distance(&ca.belief, &cb.belief))
    }
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

/// Clamp a scalar to `[0, 1]`.
#[must_use]
pub fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Euclidean distance between two attribute vectors.
#[must_use]
pub fn vector_distance(a: &AttrVector, b: &AttrVector) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// Move `target` a `rate` fraction toward `source` (in place).
fn blend_into(target: &mut AttrVector, source: &AttrVector, rate: f32) {
    let r = clamp01(rate);
    for i in 0..ATTR_DIM {
        target[i] += (source[i] - target[i]) * r;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Two clusters: a high-stress origin and a high-opportunity destination.
    fn stress_to_opportunity() -> MigrationEngine {
        let mut eng = MigrationEngine::new(MigrationConfig::default());

        let mut stressed = ClusterMigration::new(ClusterId(1), 1000, 1000);
        stressed.stress.scarcity = 0.9;
        stressed.stress.overpopulation_ratio = 1.0; // population / capacity
        stressed.opportunity = MigrationOpportunity {
            surplus: 0.0,
            safety: 0.2,
            capacity: 0.0,
        };
        // Distinct culture so mixing is observable.
        stressed.culture = [0.9; ATTR_DIM];
        stressed.language = [0.9; ATTR_DIM];
        stressed.belief = [0.9; ATTR_DIM];
        eng.upsert(stressed);

        let mut rich = ClusterMigration::new(ClusterId(2), 200, 5000);
        rich.opportunity = MigrationOpportunity {
            surplus: 0.9,
            safety: 1.0,
            capacity: 0.9,
        };
        rich.culture = [0.1; ATTR_DIM];
        rich.language = [0.1; ATTR_DIM];
        rich.belief = [0.1; ATTR_DIM];
        eng.upsert(rich);

        eng
    }

    #[test]
    fn population_flows_from_stress_to_opportunity() {
        let mut eng = stress_to_opportunity();
        let before_origin = eng.cluster(ClusterId(1)).unwrap().population;
        let before_dest = eng.cluster(ClusterId(2)).unwrap().population;

        let report = eng.tick(&mut rng(7));

        let after_origin = eng.cluster(ClusterId(1)).unwrap().population;
        let after_dest = eng.cluster(ClusterId(2)).unwrap().population;

        assert!(report.total_moved > 0, "expected migration to occur");
        assert!(after_origin < before_origin, "stressed cluster should lose people");
        assert!(after_dest > before_dest, "opportunity cluster should gain people");
        // Conservation: nobody is created or destroyed.
        assert_eq!(before_origin + before_dest, after_origin + after_dest);
        assert_eq!(report.emigrants_from(ClusterId(1)), report.immigrants_to(ClusterId(2)));
    }

    #[test]
    fn no_flow_without_opportunity() {
        let mut eng = MigrationEngine::new(MigrationConfig::default());
        let mut a = ClusterMigration::new(ClusterId(1), 1000, 1000);
        a.stress.scarcity = 1.0;
        eng.upsert(a);
        let mut b = ClusterMigration::new(ClusterId(2), 1000, 1000);
        b.stress.scarcity = 1.0; // also stressed, zero opportunity
        eng.upsert(b);

        let report = eng.tick(&mut rng(1));
        assert_eq!(report.total_moved, 0, "no pull anywhere ⇒ no migration");
    }

    #[test]
    fn disaster_triggers_surge() {
        // Baseline: mild scarcity origin, attractive destination.
        let mut base = MigrationEngine::new(MigrationConfig::default());
        let mut o = ClusterMigration::new(ClusterId(1), 1000, 1000);
        o.stress.scarcity = 0.2;
        base.upsert(o.clone());
        let mut d = ClusterMigration::new(ClusterId(2), 100, 5000);
        d.opportunity = MigrationOpportunity { surplus: 0.9, safety: 1.0, capacity: 0.9 };
        base.upsert(d.clone());

        let baseline = base.clone().tick(&mut rng(3)).total_moved;

        // Same setup + a disaster on the origin should spike the flow.
        let mut surged = base;
        surged.apply_event(SurgeEvent::Disaster { cluster: ClusterId(1), severity: 1.0 });
        let spiked = surged.tick(&mut rng(3)).total_moved;

        assert!(
            spiked > baseline,
            "disaster surge should increase migration flow (baseline={baseline}, spiked={spiked})"
        );
    }

    #[test]
    fn war_triggers_surge() {
        let mut eng = MigrationEngine::new(MigrationConfig::default());
        let mut o = ClusterMigration::new(ClusterId(1), 1000, 1000);
        o.stress.scarcity = 0.1;
        eng.upsert(o);
        let mut d = ClusterMigration::new(ClusterId(2), 100, 5000);
        d.opportunity = MigrationOpportunity { surplus: 0.9, safety: 1.0, capacity: 0.9 };
        eng.upsert(d);

        let baseline = eng.clone().tick(&mut rng(9)).total_moved;
        eng.apply_event(SurgeEvent::War { cluster: ClusterId(1), intensity: 1.0 });
        let spiked = eng.tick(&mut rng(9)).total_moved;
        assert!(spiked > baseline, "war surge should increase migration");
    }

    #[test]
    fn surge_decays_over_time() {
        let mut eng = stress_to_opportunity();
        eng.apply_event(SurgeEvent::Disaster { cluster: ClusterId(1), severity: 1.0 });
        let initial = eng.cluster(ClusterId(1)).unwrap().surge;
        assert!(initial > 1.0);

        // Decay several times with no new events; surge should monotonically fall.
        let mut prev = initial;
        for _ in 0..5 {
            eng.decay_surges();
            let now = eng.cluster(ClusterId(1)).unwrap().surge;
            assert!(now <= prev, "surge must not increase while decaying");
            prev = now;
        }
        assert!(prev < initial, "surge should decay below its peak");
    }

    #[test]
    fn migration_mixes_culture_and_reduces_divergence() {
        let mut eng = stress_to_opportunity();
        let before = eng
            .language_divergence(ClusterId(1), ClusterId(2))
            .unwrap();
        let before_belief = eng.belief_divergence(ClusterId(1), ClusterId(2)).unwrap();

        // Run several ticks so migrants blend into the destination repeatedly.
        for _ in 0..5 {
            eng.tick(&mut rng(11));
        }

        let after = eng.language_divergence(ClusterId(1), ClusterId(2)).unwrap();
        let after_belief = eng.belief_divergence(ClusterId(1), ClusterId(2)).unwrap();
        assert!(
            after < before,
            "migration should reduce language divergence (before={before}, after={after})"
        );
        assert!(
            after_belief < before_belief,
            "migration should reduce religion/belief divergence"
        );
    }

    #[test]
    fn determinism_same_seed_same_outcome() {
        let mut a = stress_to_opportunity();
        let mut b = stress_to_opportunity();

        let mut ra = rng(42);
        let mut rb = rng(42);
        let mut report_a = Vec::new();
        let mut report_b = Vec::new();
        for _ in 0..10 {
            report_a.push(a.tick(&mut ra));
            report_b.push(b.tick(&mut rb));
        }
        assert_eq!(report_a, report_b, "same seed ⇒ identical migration outcome");
        assert_eq!(a, b, "same seed ⇒ identical engine state");
    }

    #[test]
    fn different_seed_may_differ_but_conserves_population() {
        let mut eng = stress_to_opportunity();
        let total_before: u64 = (1..=2)
            .map(|i| eng.cluster(ClusterId(i)).unwrap().population)
            .sum();
        for _ in 0..20 {
            eng.tick(&mut rng(123));
        }
        let total_after: u64 = (1..=2)
            .map(|i| eng.cluster(ClusterId(i)).unwrap().population)
            .sum();
        assert_eq!(total_before, total_after, "population is conserved across ticks");
    }

    #[test]
    fn stress_and_opportunity_aggregates_are_bounded() {
        let s = MigrationStress {
            scarcity: 1.0,
            disaster_severity: 1.0,
            war_intensity: 1.0,
            overpopulation_ratio: 5.0,
        };
        assert!((0.0..=1.0).contains(&s.pressure()));

        let o = MigrationOpportunity { surplus: 1.0, safety: 1.0, capacity: 1.0 };
        assert!((0.0..=1.0).contains(&o.attractiveness()));

        let capped = MigrationOpportunity { surplus: 1.0, safety: 1.0, capacity: 0.0 };
        assert_eq!(capped.attractiveness(), 0.0, "no capacity ⇒ no attraction");
    }
}
