//! Population migration phase — stress-driven cluster flows with cultural blending.
//!
//! Runs every [`MIGRATION_CADENCE`] ticks via [`Simulation::phase_migration`].
//! Translates per-cluster ECS state into [`ClusterMigration`]s, delegates to
//! [`civ_emergence_migration::MigrationEngine`], then writes population transfers
//! and cultural blends back into the simulation's emergence state.
//!
//! # Population conservation
//! All population changes are applied as re-assignments of [`ClusterMember`]
//! components on existing ECS entities: emigrants leave their source cluster and
//! arrive at their destination cluster.  The [`MigrationEngine`] guarantees that
//! no more migrants leave a cluster than its current headcount, so the global
//! civilian count is conserved.
//!
//! # Cadence
//! One migration evaluation every 10 ticks keeps computational cost low while
//! still producing meaningful flow over a session.

use std::collections::BTreeMap;

use civ_agents::{culture::CultureProfile, ClusterId, ClusterMember};
use civ_emergence_migration::{
    ClusterMigration, MigrationConfig, MigrationEngine, MigrationOpportunity, MigrationStress,
    ATTR_DIM,
};
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

use crate::engine::Simulation;

/// Tick cadence: migration runs once per this many ticks.
const MIGRATION_CADENCE: u64 = 10;

impl Simulation {
    /// Stress-driven population-migration phase (FR-CIV-EMERGENCE-MIGRATION).
    ///
    /// Skips silently on ticks that are not multiples of [`MIGRATION_CADENCE`].
    /// Safe to call unconditionally from [`Simulation::tick`].
    pub(crate) fn phase_migration(&mut self) {
        if self.state.tick % MIGRATION_CADENCE != 0 {
            return;
        }

        let mut engine = self.build_migration_engine();
        if engine.len() < 2 {
            return;
        }

        let tick_seed = self.state.tick ^ self.state.rng_seed;
        let mut rng = ChaCha8Rng::seed_from_u64(tick_seed);
        let report = engine.tick(&mut rng);

        // Apply population transfers by re-assigning ClusterMember components.
        for flow in &report.flows {
            let from_id = ClusterId(flow.from.0);
            let to_id = ClusterId(flow.to.0);
            let to_move = flow.count;

            let movers: Vec<hecs::Entity> = self
                .world
                .query::<&ClusterMember>()
                .iter()
                .filter_map(|(e, m)| {
                    if m.cluster == from_id {
                        Some(e)
                    } else {
                        None
                    }
                })
                .take(to_move as usize)
                .collect();

            for entity in movers {
                let _ = self
                    .world
                    .insert_one(entity, ClusterMember { cluster: to_id });
            }
        }

        // Sync cultural blends back: read updated cluster culture from engine.
        for flow in &report.flows {
            let dest_key = flow.to.0;
            if let Some(post) = engine.cluster(flow.to) {
                if let Some(profile) = self.emergence.cluster_cultures.get_mut(&dest_key) {
                    for i in 0..ATTR_DIM {
                        profile.traits[i] = post.culture[i].clamp(0.0, 1.0);
                    }
                }
            }
        }

        if report.total_moved > 0 {
            tracing::debug!(
                tick = self.state.tick,
                migrated = report.total_moved,
                flows = report.flows.len(),
                "phase_migration: {} people moved across {} cluster flows",
                report.total_moved,
                report.flows.len(),
            );
        }
    }

    /// Assemble a [`MigrationEngine`] populated with per-cluster state from ECS.
    fn build_migration_engine(&self) -> MigrationEngine {
        // Count members per cluster from ECS.
        let mut cluster_pops: BTreeMap<u64, u64> = BTreeMap::new();
        for (_, member) in self.world.query::<&ClusterMember>().iter() {
            *cluster_pops.entry(member.cluster.0).or_insert(0) += 1;
        }

        let global_unrest_stress = (self.state.unrest as f32 / 10_000.0_f32).clamp(0.0, 1.0);
        let mut engine = MigrationEngine::new(MigrationConfig::default());

        for (&cluster_id, &pop) in &cluster_pops {
            let mig_id = civ_emergence_migration::ClusterId(cluster_id);

            let culture_arr: [f32; ATTR_DIM] = self
                .emergence
                .cluster_cultures
                .get(&cluster_id)
                .map(|p| culture_to_attr(p))
                .unwrap_or([0.5; ATTR_DIM]);

            // Max belief deviation from neutral (0.5) → per-cluster stress.
            let belief_stress = self
                .emergence
                .cluster_beliefs
                .get(&cluster_id)
                .map(|b| {
                    b.iter()
                        .map(|v| (v - 0.5_f32).abs() * 2.0)
                        .fold(0.0_f32, f32::max)
                })
                .unwrap_or(0.3);

            let stress_val =
                (global_unrest_stress * 0.4 + belief_stress * 0.6).clamp(0.0, 1.0);
            let openness = culture_arr[0];
            let opportunity_val =
                ((1.0 - stress_val) * 0.7 + openness * 0.3).clamp(0.0, 1.0);

            // ponytail: carrying capacity estimated as 2× current population when
            // not tracked; upgrade when civ-voxel exposes per-cluster capacity.
            let capacity = (pop * 2).max(1);
            let spare = capacity - pop.min(capacity);
            let capacity_score = spare as f32 / capacity as f32;

            let mut cluster = ClusterMigration::new(mig_id, pop, capacity);
            cluster.stress = MigrationStress {
                scarcity: stress_val,
                disaster_severity: 0.0,
                war_intensity: 0.0,
                overpopulation_ratio: pop as f32 / capacity as f32,
            };
            cluster.opportunity = MigrationOpportunity {
                surplus: opportunity_val,
                safety: 1.0 - stress_val,
                capacity: capacity_score,
            };
            cluster.culture = culture_arr;

            engine.upsert(cluster);
        }

        engine
    }
}

/// Convert a [`CultureProfile`] traits array into a fixed `[f32; ATTR_DIM]` vector.
fn culture_to_attr(profile: &CultureProfile) -> [f32; ATTR_DIM] {
    let t = profile.traits;
    [
        t.get(0).copied().unwrap_or(0.5),
        t.get(1).copied().unwrap_or(0.5),
        t.get(2).copied().unwrap_or(0.5),
        t.get(3).copied().unwrap_or(0.5),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Simulation;

    /// Stressed cluster (high unrest) should not gain members from migration.
    #[test]
    fn migration_stressed_cluster_does_not_gain() {
        let mut sim = Simulation::with_seed(42);

        let entities: Vec<hecs::Entity> = sim
            .world
            .query::<&civ_agents::Civilian>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let mid = entities.len() / 2;
        for &e in &entities[..mid] {
            let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(1) });
        }
        for &e in &entities[mid..] {
            let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(2) });
        }

        sim.emergence.cluster_cultures.insert(
            1,
            CultureProfile::new([0.9_f32, 0.5, 0.5, 0.5]),
        );
        sim.emergence.cluster_cultures.insert(
            2,
            CultureProfile::new([0.1_f32, 0.5, 0.5, 0.5]),
        );

        // High global unrest drives stress up.
        sim.state.unrest = 8_000;

        let c1_before = sim
            .world
            .query::<&ClusterMember>()
            .iter()
            .filter(|(_, m)| m.cluster == ClusterId(1))
            .count();

        for _ in 0..50 {
            sim.phase_migration();
            sim.state.tick += 1;
        }

        let c1_after = sim
            .world
            .query::<&ClusterMember>()
            .iter()
            .filter(|(_, m)| m.cluster == ClusterId(1))
            .count();

        assert!(
            c1_after <= c1_before,
            "stressed cluster should not gain members: before={c1_before} after={c1_after}"
        );
    }

    /// Total ClusterMember count must not change across migration ticks.
    #[test]
    fn migration_conserves_total_population() {
        let mut sim = Simulation::with_seed(7);

        let entities: Vec<hecs::Entity> = sim
            .world
            .query::<&civ_agents::Civilian>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        let total = entities.len();
        let mid = total / 2;
        for &e in &entities[..mid] {
            let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(10) });
        }
        for &e in &entities[mid..] {
            let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(20) });
        }
        sim.state.unrest = 5_000;

        for _ in 0..30 {
            sim.phase_migration();
            sim.state.tick += 1;
        }

        let after_count = sim
            .world
            .query::<&ClusterMember>()
            .iter()
            .count();

        assert_eq!(
            total, after_count,
            "ClusterMember count must be conserved: before={total} after={after_count}"
        );
    }

    /// Same seed must produce identical cluster distribution after N ticks.
    #[test]
    fn migration_deterministic_per_seed() {
        fn run_sim(seed: u64) -> (usize, usize) {
            let mut sim = Simulation::with_seed(seed);
            let entities: Vec<hecs::Entity> = sim
                .world
                .query::<&civ_agents::Civilian>()
                .iter()
                .map(|(e, _)| e)
                .collect();
            let mid = entities.len() / 2;
            for &e in &entities[..mid] {
                let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(1) });
            }
            for &e in &entities[mid..] {
                let _ = sim.world.insert_one(e, ClusterMember { cluster: ClusterId(2) });
            }
            sim.state.unrest = 3_000;

            for _ in 0..20 {
                sim.phase_migration();
                sim.state.tick += 1;
            }

            let c1 = sim
                .world
                .query::<&ClusterMember>()
                .iter()
                .filter(|(_, m)| m.cluster == ClusterId(1))
                .count();
            let c2 = sim
                .world
                .query::<&ClusterMember>()
                .iter()
                .filter(|(_, m)| m.cluster == ClusterId(2))
                .count();
            (c1, c2)
        }

        let r1 = run_sim(42);
        let r2 = run_sim(42);
        assert_eq!(r1, r2, "migration must be deterministic: {r1:?} vs {r2:?}");
    }
}
