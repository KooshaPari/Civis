//! Desire-path emergence for FR-CIV-ROAD-900.
//!
//! Repeated agent traversal between two world cells accrues a *path weight*.
//! Once the weight crosses the road threshold the edge is promoted to an
//! emergent road segment. Unused paths decay each tick so forgotten trails
//! return to bare ground — preventing the graph from bloating with stale
//! "almost roads" forever.
//!
//! This is the **pure-logic core** of the desire-path feedback loop. It is
//! deliberately storage-agnostic and has no Bevy dependency: it can be driven
//! from any caller that can call `record_traversal` / `tick_decay` (sim
//! scheduler, replay scrubber, test harness). Renderers read the resulting
//! [`PathState`] to style the network.
//!
//! See `docs/specs/requirements/FR-CIV-ROAD.md` for the full requirement.
//!
//! ## Algorithm
//!
//! 1. [`DesirePathTracker::record_traversal`] adds `weight` to the canonical
//!    undirected edge between two world cells.
//! 2. [`DesirePathTracker::tick_decay`] applies a multiplicative decay factor
//!    per tick to every tracked edge. Edges that drop below
//!    [`DesirePathConfig::forget_threshold`] are forgotten entirely.
//! 3. [`DesirePathTracker::kind_for`] reads the current accumulated weight and
//!    maps it to a [`PathState`]: `None -> Trail -> Road`.
//!
//! All updates are deterministic for a fixed `(event order, config)` pair,
//! which the acceptance test pins down.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Rung on the emergent desire-path ladder. Ordered weakest -> strongest; the
/// `Ord` derive gives a stable promotion rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PathState {
    /// Bare ground; no desire trail yet.
    None,
    /// Foot-worn trail (weight above trail threshold but below road threshold).
    Trail,
    /// Promoted road segment (weight above road threshold).
    Road,
}

impl PathState {
    /// Highest rung an edge with `weight` accumulated may hold, given the
    /// supplied thresholds.
    #[must_use]
    pub fn for_weight(weight: f32, trail_threshold: f32, road_threshold: f32) -> Self {
        if weight >= road_threshold {
            PathState::Road
        } else if weight >= trail_threshold {
            PathState::Trail
        } else {
            PathState::None
        }
    }

    /// `true` once the edge has been promoted to a [`PathState::Road`].
    #[must_use]
    pub fn is_road(self) -> bool {
        matches!(self, PathState::Road)
    }
}

/// Tunable parameters for the desire-path feedback loop. Defaults follow the
/// Manor Lords desire-line cadence: a few trips make a trail, sustained use
/// promotes to road, a long quiet period erases the trace entirely.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DesirePathConfig {
    /// Weight above which an edge is considered a Trail.
    pub trail_threshold: f32,
    /// Weight above which an edge is promoted to Road.
    pub road_threshold: f32,
    /// Per-tick multiplicative decay factor (e.g. `0.95` keeps 95% of weight
    /// each tick). Must be in `(0.0, 1.0]` — values `>= 1.0` disable decay.
    pub decay_factor: f32,
    /// Edges whose weight drops below this absolute value are forgotten
    /// (removed from the map so iteration stays bounded).
    pub forget_threshold: f32,
}

impl Default for DesirePathConfig {
    fn default() -> Self {
        Self {
            trail_threshold: 8.0,
            road_threshold: 32.0,
            decay_factor: 0.95,
            forget_threshold: 0.01,
        }
    }
}

impl DesirePathConfig {
    /// Apply one tick of decay to `weight`, returning the post-decay value.
    /// Values at or below [`Self::forget_threshold`] collapse to `0.0`.
    #[must_use]
    pub fn apply_decay(&self, weight: f32) -> f32 {
        if weight <= self.forget_threshold {
            return 0.0;
        }
        let decayed = weight * self.decay_factor;
        if decayed <= self.forget_threshold {
            0.0
        } else {
            decayed
        }
    }
}

/// Canonical undirected edge key between two world cells. Endpoints are
/// stored in sorted order so `(a, b)` and `(b, a)` map to the same segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DesireEdgeKey {
    /// Lower-sorted endpoint `(x, y, z)`.
    pub a: (i64, i64, i64),
    /// Higher-sorted endpoint `(x, y, z)`.
    pub b: (i64, i64, i64),
}

impl DesireEdgeKey {
    /// Build a canonical undirected edge key from two world coords.
    #[must_use]
    pub fn new(from: (i64, i64, i64), to: (i64, i64, i64)) -> Self {
        if from <= to {
            Self { a: from, b: to }
        } else {
            Self { a: to, b: from }
        }
    }
}

/// Per-edge desire-path bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DesireEdge {
    /// Current accumulated path weight (post-decay).
    pub weight: f32,
    /// Last tick on which the edge received traffic (useful for diagnostics /
    /// "longest unused" queries).
    pub last_used_tick: u64,
}

/// Storage-agnostic desire-path tracker. Drives the emergent road network in
/// `civ-traffic`; this is the pure-logic kernel so it can be reused by replay
/// scrubbers, web dashboards, and tests without dragging in the full graph
/// type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesirePathTracker {
    edges: BTreeMap<DesireEdgeKey, DesireEdge>,
    config: DesirePathConfig,
    tick: u64,
}

impl Default for DesirePathTracker {
    fn default() -> Self {
        Self::with_config(DesirePathConfig::default())
    }
}

impl DesirePathTracker {
    /// Empty tracker with the given config.
    #[must_use]
    pub fn with_config(config: DesirePathConfig) -> Self {
        Self {
            edges: BTreeMap::new(),
            config,
            tick: 0,
        }
    }

    /// Replace the configuration. Existing edges keep their weights; the new
    /// thresholds/decay take effect on the next [`Self::tick_decay`] /
    /// [`Self::record_traversal`] call.
    pub fn set_config(&mut self, config: DesirePathConfig) {
        self.config = config;
    }

    /// Active configuration.
    #[must_use]
    pub fn config(&self) -> DesirePathConfig {
        self.config
    }

    /// Number of edges currently tracked (diagnostics / UI counters).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Accumulate `weight` of traffic on the edge between `from` and `to`,
    /// stamping the edge with the current tick. Returns the resulting
    /// [`PathState`] for that edge. Self-loops and non-positive weights are
    /// no-ops (no edge, no state change).
    pub fn record_traversal(
        &mut self,
        from: (i64, i64, i64),
        to: (i64, i64, i64),
        weight: f32,
    ) -> PathState {
        if from == to || weight <= 0.0 || !weight.is_finite() {
            return PathState::for_weight(
                self.edges
                    .get(&DesireEdgeKey::new(from, to))
                    .map_or(0.0, |e| e.weight),
                self.config.trail_threshold,
                self.config.road_threshold,
            );
        }
        let key = DesireEdgeKey::new(from, to);
        let entry = self.edges.entry(key).or_insert(DesireEdge {
            weight: 0.0,
            last_used_tick: self.tick,
        });
        entry.weight += weight;
        entry.last_used_tick = self.tick;
        self.kind_for(key)
    }

    /// Apply one tick of decay to every tracked edge, forgetting any that
    /// collapse to `0.0`. Returns the number of edges forgotten this tick
    /// (useful for telemetry / "trail lost" UI events).
    pub fn tick_decay(&mut self) -> usize {
        self.tick = self.tick.saturating_add(1);
        let config = self.config;
        let mut forgotten = 0usize;
        self.edges.retain(|_, edge| {
            edge.weight = config.apply_decay(edge.weight);
            if edge.weight <= 0.0 {
                forgotten += 1;
                false
            } else {
                true
            }
        });
        forgotten
    }

    /// Current [`PathState`] for the edge between `from` and `to`, or
    /// [`PathState::None`] if the edge is not tracked.
    #[must_use]
    pub fn state_between(&self, from: (i64, i64, i64), to: (i64, i64, i64)) -> PathState {
        let weight = self
            .edges
            .get(&DesireEdgeKey::new(from, to))
            .map_or(0.0, |e| e.weight);
        PathState::for_weight(
            weight,
            self.config.trail_threshold,
            self.config.road_threshold,
        )
    }

    /// Current accumulated weight for the edge between `from` and `to`, or
    /// `0.0` if the edge is not tracked.
    #[must_use]
    pub fn weight_between(&self, from: (i64, i64, i64), to: (i64, i64, i64)) -> f32 {
        self.edges
            .get(&DesireEdgeKey::new(from, to))
            .map_or(0.0, |e| e.weight)
    }

    /// Current [`PathState`] for an already-keyed edge.
    #[must_use]
    pub fn kind_for(&self, key: DesireEdgeKey) -> PathState {
        let weight = self.edges.get(&key).map_or(0.0, |e| e.weight);
        PathState::for_weight(
            weight,
            self.config.trail_threshold,
            self.config.road_threshold,
        )
    }

    /// Number of edges currently at or above `min` (renderer LOD / stats).
    #[must_use]
    pub fn count_at_least(&self, min: PathState) -> usize {
        self.edges
            .values()
            .filter(|e| {
                PathState::for_weight(
                    e.weight,
                    self.config.trail_threshold,
                    self.config.road_threshold,
                ) >= min
            })
            .count()
    }

    /// Iterate every tracked edge in deterministic `BTreeMap` order (so
    /// renderers can draw the emergent network without sorting).
    pub fn iter_edges(&self) -> impl Iterator<Item = (DesireEdgeKey, DesireEdge)> + '_ {
        self.edges.iter().map(|(k, v)| (*k, *v))
    }

    /// Total number of ticks elapsed (number of [`Self::tick_decay`] calls).
    #[must_use]
    pub fn current_tick(&self) -> u64 {
        self.tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-ROAD-900 acceptance: repeated traversal between two points
    /// accrues weight past the road threshold to form a road segment.
    #[test]
    fn repeated_traversal_raises_weight_past_threshold_to_form_road() {
        let mut t = DesirePathTracker::default();
        let a = (0, 0, 0);
        let b = (1, 0, 0);

        // Before any traffic the edge is bare ground.
        assert_eq!(t.state_between(a, b), PathState::None);

        // Five trips of weight 4 each = 20, above trail threshold (8) but below
        // road threshold (32) -> a Trail emerges.
        for _ in 0..5 {
            t.record_traversal(a, b, 4.0);
        }
        assert_eq!(t.state_between(a, b), PathState::Trail);
        assert!(!t.state_between(a, b).is_road());

        // Three more trips of weight 5 each push total to 35, above road
        // threshold (32) -> promotion to Road.
        for _ in 0..3 {
            t.record_traversal(a, b, 5.0);
        }
        assert!(t.state_between(a, b).is_road());
        assert_eq!(t.count_at_least(PathState::Road), 1);
    }

    /// FR-CIV-ROAD-900 acceptance: unused paths decay back to bare ground.
    #[test]
    fn unused_paths_decay_back_to_none() {
        let cfg = DesirePathConfig {
            trail_threshold: 8.0,
            road_threshold: 32.0,
            decay_factor: 0.5,
            forget_threshold: 0.5,
        };
        let mut t = DesirePathTracker::with_config(cfg);
        let a = (0, 0, 0);
        let b = (2, 0, 0);

        // Build a Trail.
        t.record_traversal(a, b, 10.0);
        assert_eq!(t.state_between(a, b), PathState::Trail);

        // Aggressive decay (factor 0.5) drops 10 -> 5 -> 2.5 -> 1.25 -> 0.625
        // -> forgotten (<= 0.5). After enough ticks the edge is gone.
        let mut ticks = 0;
        while t.edge_count() > 0 && ticks < 50 {
            t.tick_decay();
            ticks += 1;
        }
        assert_eq!(
            t.state_between(a, b),
            PathState::None,
            "unused path must decay back to None"
        );
        // Once weight collapses below the forget threshold the edge itself
        // must be removed so the map cannot bloat with stale near-zero rows.
        assert_eq!(t.edge_count(), 0);
    }

    /// Edges are undirected: (a, b) and (b, a) share the same weight.
    #[test]
    fn edges_are_undirected() {
        let mut t = DesirePathTracker::default();
        t.record_traversal((3, 0, 0), (4, 0, 0), 6.0);
        // Traversing the reverse direction adds to the SAME edge.
        t.record_traversal((4, 0, 0), (3, 0, 0), 6.0);
        assert_eq!(t.edge_count(), 1);
        assert_eq!(t.weight_between((3, 0, 0), (4, 0, 0)), 12.0);
    }

    /// Determinism: the same event order yields the same tracker.
    #[test]
    fn tracker_is_deterministic() {
        let build = || {
            let mut t = DesirePathTracker::default();
            for i in 0..16i64 {
                t.record_traversal((i % 4, 0, 0), ((i + 1) % 4, 0, 0), 3.0);
            }
            for _ in 0..5 {
                t.tick_decay();
            }
            t
        };
        assert_eq!(build(), build());
    }

    /// PathState::for_weight respects the thresholds exactly at the boundary.
    #[test]
    fn path_state_threshold_boundaries() {
        // Just below trail threshold -> None.
        assert_eq!(PathState::for_weight(7.99, 8.0, 32.0), PathState::None);
        // Exactly at trail threshold -> Trail.
        assert_eq!(PathState::for_weight(8.0, 8.0, 32.0), PathState::Trail);
        // Just below road threshold -> Trail.
        assert_eq!(PathState::for_weight(31.99, 8.0, 32.0), PathState::Trail);
        // At or above road threshold -> Road.
        assert_eq!(PathState::for_weight(32.0, 8.0, 32.0), PathState::Road);
        assert_eq!(PathState::for_weight(100.0, 8.0, 32.0), PathState::Road);
    }

    /// Self-loops and non-positive weights must never create edges or mutate
    /// state — they're no-ops the caller can blindly invoke per agent step.
    #[test]
    fn self_loops_and_zero_weight_are_noops() {
        let mut t = DesirePathTracker::default();
        let p = (1, 2, 3);
        assert_eq!(t.record_traversal(p, p, 10.0), PathState::None);
        assert_eq!(t.record_traversal(p, (4, 5, 6), 0.0), PathState::None);
        assert_eq!(t.record_traversal(p, (4, 5, 6), -1.0), PathState::None);
        assert_eq!(t.edge_count(), 0);
    }

    /// tick_decay stamps last_used_tick so callers can query "edge stale
    /// for N ticks" for diagnostics; we at least guarantee the tick counter
    /// advances and is observable.
    #[test]
    fn tick_decay_advances_tick_counter() {
        let mut t = DesirePathTracker::default();
        assert_eq!(t.current_tick(), 0);
        t.tick_decay();
        t.tick_decay();
        t.tick_decay();
        assert_eq!(t.current_tick(), 3);
    }
}
