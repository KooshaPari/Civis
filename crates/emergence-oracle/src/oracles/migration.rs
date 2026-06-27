//! FR-EMG-009: Migration emergence oracle.
//!
//! Validates that `civ-emergence-migration` moves population from a stressed
//! origin cluster to an attractive destination cluster in a constructed
//! two-cluster case.
//!
//! Measurement: total people moved in one seeded migration tick.
//! Threshold: > 0 people moved, with origin population decreasing and
//! destination population increasing.

use crate::{FeatureOracle, OracleVerdict};
use civ_emergence_migration::{
    ClusterId, ClusterMigration, MigrationConfig, MigrationEngine, MigrationOpportunity,
};
use civ_engine::Simulation;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub struct MigrationOracle;

impl FeatureOracle for MigrationOracle {
    fn fr_id(&self) -> &str {
        "FR-EMG-009"
    }

    fn check(&self, _sim: &Simulation) -> OracleVerdict {
        let mut engine = MigrationEngine::new(MigrationConfig::default());

        let mut stressed = ClusterMigration::new(ClusterId(1), 1000, 1000);
        stressed.stress.scarcity = 0.9;
        stressed.stress.overpopulation_ratio = 1.0;
        stressed.opportunity = MigrationOpportunity {
            surplus: 0.0,
            safety: 0.2,
            capacity: 0.0,
        };
        engine.upsert(stressed);

        let mut rich = ClusterMigration::new(ClusterId(2), 200, 5000);
        rich.opportunity = MigrationOpportunity {
            surplus: 0.9,
            safety: 1.0,
            capacity: 0.9,
        };
        engine.upsert(rich);

        let before_origin = engine.cluster(ClusterId(1)).expect("origin cluster missing").population;
        let before_dest = engine
            .cluster(ClusterId(2))
            .expect("destination cluster missing")
            .population;

        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let report = engine.tick(&mut rng);

        let after_origin = engine.cluster(ClusterId(1)).expect("origin cluster missing").population;
        let after_dest = engine
            .cluster(ClusterId(2))
            .expect("destination cluster missing")
            .population;

        let measured = report.total_moved as f64;
        let threshold = 1.0;
        let passed = report.total_moved > 0 && after_origin < before_origin && after_dest > before_dest;

        OracleVerdict {
            fr_id: self.fr_id().to_string(),
            passed,
            measured,
            threshold,
            detail: format!(
                "Migration emergence: moved={} origin_pop={}->{} dest_pop={}->{} flows={} in 2-cluster stress->opportunity case",
                report.total_moved,
                before_origin,
                after_origin,
                before_dest,
                after_dest,
                report.flows.len()
            ),
        }
    }
}
