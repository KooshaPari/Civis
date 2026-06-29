//! SimSnapshot construction from engine state via a trait bridge.
//!
//! Holocron doesn't depend on `civ_engine` (that would create a circular
//! dependency). Instead, callers implement the `SimSnapshotSource`
//! trait against their own engine state and call `snapshot_from_source`.
//!
//! The engine crate implements `SimSnapshotSource` for `Simulation`
//! (in `crates/holocron` adapter glue), so the runtime side is just:
//!
//! ```ignore
//! // in engine tick:
//! let snap: SimSnapshot = (&sim).into();
//! panel.push_snapshot(snap);
//! panel.render();
//! ```

use crate::rank::{EraKind, FactionStance, SimSnapshot};

/// Source of sim-state scalars that holocron consumes for ranking.
///
/// Implement this against any engine/runtime state. The contract is
/// "best-effort" — return `0` / `Default::default()` for any field
/// you don't track; the ranker gracefully degrades.
pub trait SimSnapshotSource {
    /// Current sim tick (monotonic). 0 if unknown.
    fn tick(&self) -> u64;
    /// Number of active disasters. 0 if unknown.
    fn active_disasters(&self) -> u32;
    /// Dominant era. `Default::default()` if unknown.
    fn dominant_era(&self) -> EraKind {
        EraKind::default()
    }
    /// Per-pair faction stance.
    fn faction_relations(&self) -> Vec<(u32, u32, FactionStance)> {
        Vec::new()
    }
    /// Total living population. 0 if unknown.
    fn population(&self) -> u32;
    /// Market stress [0..1]. 0 if unknown.
    fn market_stress(&self) -> f32;
    /// Culture drift [0..1]. 0 if unknown.
    fn culture_drift(&self) -> f32;
}

/// Build a `SimSnapshot` from any source.
pub fn snapshot_from_source<S: SimSnapshotSource>(s: &S) -> SimSnapshot {
    SimSnapshot {
        tick: s.tick(),
        active_disasters: s.active_disasters(),
        dominant_era: s.dominant_era(),
        faction_relations: s.faction_relations(),
        population: s.population(),
        market_stress: s.market_stress().clamp(0.0, 1.0),
        culture_drift: s.culture_drift().clamp(0.0, 1.0),
    }
}

/// Adapter blanket impl: any reference-to-tick-bearing thing can be turned into
/// a snapshot via the trait — callers only implement the fields they care about.
impl<T> SimSnapshotSource for &T
where
    T: SimSnapshotSource,
{
    fn tick(&self) -> u64 {
        (*self).tick()
    }
    fn active_disasters(&self) -> u32 {
        (*self).active_disasters()
    }
    fn dominant_era(&self) -> EraKind {
        (*self).dominant_era()
    }
    fn faction_relations(&self) -> Vec<(u32, u32, FactionStance)> {
        (*self).faction_relations()
    }
    fn population(&self) -> u32 {
        (*self).population()
    }
    fn market_stress(&self) -> f32 {
        (*self).market_stress()
    }
    fn culture_drift(&self) -> f32 {
        (*self).culture_drift()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mock {
        tick: u64,
        disasters: u32,
        era: EraKind,
        tension: FactionStance,
        pop: u32,
        market: f32,
        drift: f32,
    }

    impl SimSnapshotSource for Mock {
        fn tick(&self) -> u64 {
            self.tick
        }
        fn active_disasters(&self) -> u32 {
            self.disasters
        }
        fn dominant_era(&self) -> EraKind {
            self.era
        }
        fn faction_relations(&self) -> Vec<(u32, u32, FactionStance)> {
            vec![(0, 1, self.tension)]
        }
        fn population(&self) -> u32 {
            self.pop
        }
        fn market_stress(&self) -> f32 {
            self.market
        }
        fn culture_drift(&self) -> f32 {
            self.drift
        }
    }

    #[test]
    fn snapshot_from_source_basic() {
        let m = Mock {
            tick: 42,
            disasters: 3,
            era: EraKind::Conflict,
            tension: FactionStance::AtWar,
            pop: 1200,
            market: 0.7,
            drift: 0.4,
        };
        let snap = snapshot_from_source(&m);
        assert_eq!(snap.tick, 42);
        assert_eq!(snap.active_disasters, 3);
        assert_eq!(snap.dominant_era, EraKind::Conflict);
        assert_eq!(snap.population, 1200);
        assert!((snap.market_stress - 0.7).abs() < 1e-6);
        assert!((snap.culture_drift - 0.4).abs() < 1e-6);
        assert_eq!(snap.faction_relations.len(), 1);
        assert_eq!(snap.faction_relations[0].2, FactionStance::AtWar);
    }

    #[test]
    fn snapshot_clamps_stress_drift_to_unit_interval() {
        struct OutOfRange;
        impl SimSnapshotSource for OutOfRange {
            fn tick(&self) -> u64 {
                0
            }
            fn active_disasters(&self) -> u32 {
                0
            }
            fn population(&self) -> u32 {
                0
            }
            fn market_stress(&self) -> f32 {
                5.0
            }
            fn culture_drift(&self) -> f32 {
                -0.3
            }
        }
        let snap = snapshot_from_source(&OutOfRange);
        assert_eq!(snap.market_stress, 1.0);
        assert_eq!(snap.culture_drift, 0.0);
    }

    #[test]
    fn snapshot_default_impls_for_optional_signals() {
        struct Minimal;
        impl SimSnapshotSource for Minimal {
            fn tick(&self) -> u64 {
                7
            }
            fn active_disasters(&self) -> u32 {
                0
            }
            fn population(&self) -> u32 {
                0
            }
        }
        let snap = snapshot_from_source(&Minimal);
        assert_eq!(snap.tick, 7);
        assert_eq!(snap.dominant_era, EraKind::default());
        assert!(snap.faction_relations.is_empty());
    }
}