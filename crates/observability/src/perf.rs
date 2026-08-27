//! Per-phase simulation metrics for Prometheus export.
//!
//! All metrics use the `civ_` prefix to match the Grafana dashboard queries.
//! Call [`SimMetrics::new`] to create and register all gauges/histograms on
//! a custom [`prometheus::Registry`], then call [`SimMetrics::record`] after
//! each simulation tick to populate them from the live world state.

use prometheus::{Gauge, Histogram, HistogramOpts, IntGauge, Registry};

/// Human-readable metric names (shared between registration and Grafana).
pub const TICK_DURATION: &str = "civ_tick_duration_seconds";
pub const ENTITY_COUNT: &str = "civ_entity_count";
pub const FACTION_COUNT: &str = "civ_faction_count";
pub const BUILDING_COUNT: &str = "civ_building_count";
pub const ECONOMY_TREASURY: &str = "civ_economy_treasury";
pub const DIPLOMACY_TREATIES: &str = "civ_diplomacy_treaties";
pub const EMERGENCE_ENTROPY: &str = "civ_emergence_entropy";

/// Set of per-phase metrics registered on a Prometheus [`Registry`].
///
/// Created once during bridge startup and recorded every tick.
pub struct SimMetrics {
    /// Wall-clock duration of one simulation tick (seconds).
    pub tick_duration: Histogram,
    /// Total living entities (civilians + buildings + military units).
    pub entity_count: IntGauge,
    /// Number of active factions.
    pub faction_count: IntGauge,
    /// Number of buildings in the world.
    pub building_count: IntGauge,
    /// Sum of all faction treasuries (converted to f64).
    pub economy_treasury: Gauge,
    /// Number of active trade routes / diplomacy treaties.
    pub diplomacy_treaties: IntGauge,
    /// Latest emergence entropy value (0.0-1.0).
    pub emergence_entropy: Gauge,
}

/// Snapshot of per-phase values extracted from the simulation for metric
/// recording. Decouples the observability crate from engine types.
pub struct SimMetricSnapshot {
    /// Wall-clock seconds for one tick.
    pub tick_duration_secs: f64,
    /// Total living entities.
    pub entity_count: i64,
    /// Number of active factions.
    pub faction_count: i64,
    /// Number of buildings.
    pub building_count: i64,
    /// Sum of all faction treasuries.
    pub economy_treasury: f64,
    /// Number of active trade routes / diplomacy treaties.
    pub diplomacy_treaties: i64,
    /// Latest emergence entropy (0.0-1.0).
    pub emergence_entropy: f64,
}

impl SimMetrics {
    /// Create and register all per-phase metrics on the given `registry`.
    pub fn new(registry: &Registry) -> Result<Self, String> {
        let tick_duration = Histogram::with_opts(
            HistogramOpts::new(TICK_DURATION, "Wall-clock seconds for one simulation tick")
                .buckets(vec![
                    0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
                ]),
        )
        .map_err(|e| format!("create {TICK_DURATION}: {e}"))?;

        let entity_count = IntGauge::new(ENTITY_COUNT, "Total living entities in the simulation")
            .map_err(|e| format!("create {ENTITY_COUNT}: {e}"))?;
        let faction_count = IntGauge::new(FACTION_COUNT, "Number of active factions")
            .map_err(|e| format!("create {FACTION_COUNT}: {e}"))?;
        let building_count = IntGauge::new(BUILDING_COUNT, "Number of buildings in the world")
            .map_err(|e| format!("create {BUILDING_COUNT}: {e}"))?;
        let economy_treasury = Gauge::new(ECONOMY_TREASURY, "Sum of all faction treasuries")
            .map_err(|e| format!("create {ECONOMY_TREASURY}: {e}"))?;
        let diplomacy_treaties = IntGauge::new(
            DIPLOMACY_TREATIES,
            "Number of active trade routes (diplomacy treaties)",
        )
        .map_err(|e| format!("create {DIPLOMACY_TREATIES}: {e}"))?;
        let emergence_entropy = Gauge::new(
            EMERGENCE_ENTROPY,
            "Latest emergence entropy value (0.0 - 1.0)",
        )
        .map_err(|e| format!("create {EMERGENCE_ENTROPY}: {e}"))?;

        // Register all metrics on the provided registry.
        registry
            .register(Box::new(tick_duration.clone()))
            .map_err(|e| format!("register {TICK_DURATION}: {e}"))?;
        registry
            .register(Box::new(entity_count.clone()))
            .map_err(|e| format!("register {ENTITY_COUNT}: {e}"))?;
        registry
            .register(Box::new(faction_count.clone()))
            .map_err(|e| format!("register {FACTION_COUNT}: {e}"))?;
        registry
            .register(Box::new(building_count.clone()))
            .map_err(|e| format!("register {BUILDING_COUNT}: {e}"))?;
        registry
            .register(Box::new(economy_treasury.clone()))
            .map_err(|e| format!("register {ECONOMY_TREASURY}: {e}"))?;
        registry
            .register(Box::new(diplomacy_treaties.clone()))
            .map_err(|e| format!("register {DIPLOMACY_TREATIES}: {e}"))?;
        registry
            .register(Box::new(emergence_entropy.clone()))
            .map_err(|e| format!("register {EMERGENCE_ENTROPY}: {e}"))?;

        Ok(Self {
            tick_duration,
            entity_count,
            faction_count,
            building_count,
            economy_treasury,
            diplomacy_treaties,
            emergence_entropy,
        })
    }

    /// Record per-phase metrics from a snapshot of simulation values.
    ///
    /// Call this after `sim.tick()` completes, while the simulation lock
    /// is still held. Build the snapshot from the live `Simulation` in the
    /// caller (server crate) to avoid a circular dependency on `civ_engine`.
    pub fn record(&self, snapshot: &SimMetricSnapshot) {
        self.tick_duration.observe(snapshot.tick_duration_secs);
        self.entity_count.set(snapshot.entity_count);
        self.faction_count.set(snapshot.faction_count);
        self.building_count.set(snapshot.building_count);
        self.economy_treasury.set(snapshot.economy_treasury);
        self.diplomacy_treaties.set(snapshot.diplomacy_treaties);
        self.emergence_entropy.set(snapshot.emergence_entropy);
    }
}
