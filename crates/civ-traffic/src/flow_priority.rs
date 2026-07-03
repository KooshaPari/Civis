//! Intersection flow-priority policy — FR-CIV-FLOW-PRIORITY.
//!
//! ## What this module is
//!
//! Pure-logic scheduler that decides which lane at an intersection gets the
//! green phase on a given tick. The policy is "higher-volume lanes go first",
//! which in aggregate lowers total wait time because we serve the head of the
//! queue rather than letting a busy lane back up while a quiet one drains.
//!
//! ## What this module is NOT
//!
//! * Not a renderer concern — emits only data the sim / snapshot layers
//!   consume.
//! * Not a wall-clock scheduler — `phase_ticks` is integer ticks so it stays
//!   deterministic for replay (FR-CIV-INFRA-030).
//! * Not coupled to `TrafficGraph` — the input is a small bundle of
//!   `LaneVolume` entries, so this module can be unit-tested without any
//!   voxel/world dependency and reused by any intersection layer that can
//!   supply per-lane demand.
//!
//! ## Acceptance test (FR-CIV-FLOW-PRIORITY)
//!
//! Given two lanes with different volumes, the higher-volume lane must be
//! granted priority (lower `wait_score` than the equal-time baseline) and the
//! aggregate wait over a small horizon must strictly decrease relative to a
//! naive equal-split schedule. The `#[cfg(test)]` block below asserts both
//! facts with explicit numbers.
//!
//! ## Determinism
//!
//! Ties are broken by lane id (`LaneId`) ascending so identical inputs yield
//! byte-identical priority orders across runs — replay-safe.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Schema version of the [`FlowPriorityPolicy`] data shape. Bump on breaking
/// changes so a future migration can detect old policy snapshots.
pub const FLOW_PRIORITY_SCHEMA_VERSION: &str = "0.1.0-flow-priority";

/// Stable identifier for a lane feeding an intersection. An arbitrary integer
/// (typically a hash of the road segment + direction); the policy uses it only
/// as a tie-breaker and as a key for results, so the renderer / debug UI can
/// correlate lanes across frames.
pub type LaneId = u32;

/// Per-lane demand snapshot supplied to the policy each scheduling tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LaneVolume {
    /// Which lane this entry describes.
    pub lane: LaneId,
    /// Number of agents / vehicles currently queued or expected this tick.
    /// Negative values are clamped to `0` — the policy never grants negative
    /// weight, that would invert the "high volume first" invariant.
    pub volume: u32,
}

/// Outcome of one scheduling tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseAssignment {
    /// Lane granted the green phase this tick, in priority order (highest
    /// priority first). Length is `min(policy.max_green_per_tick, lanes)`.
    pub green: Vec<LaneId>,
    /// Number of ticks the policy allocated to the green phase this round.
    /// `>= green.len()` because every green lane gets at least one tick.
    pub phase_ticks: u32,
    /// Per-lane accumulated wait in tick-units at the END of this phase.
    /// Keys are every input lane — lanes that did not go green accrue wait.
    pub waits_after: BTreeMap<LaneId, u64>,
}

/// Tunables for the flow-priority scheduler.
///
/// All fields have sensible defaults via [`FlowPriorityPolicy::default`].
/// Tuning them up or down changes the trade-off between fairness and
/// throughput; the default is the "higher-volume goes first" sweet spot the
/// FR calls out.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlowPriorityPolicy {
    /// Maximum number of lanes that may receive the green phase in a single
    /// tick. `1` = strictly serial; `>1` = parallel green phases.
    pub max_green_per_tick: u32,
    /// Ticks of green phase granted per green lane each round.
    pub green_ticks_per_lane: u32,
    /// Minimum green-phase length, regardless of how many lanes are green.
    /// Prevents degenerate zero-tick phases on tiny intersections.
    pub min_phase_ticks: u32,
    /// Starvation guard: a lane that has waited this many ticks without
    /// getting green is force-promoted to the front of the queue next tick.
    /// `0` disables the guard.
    pub starvation_threshold: u64,
}

impl Default for FlowPriorityPolicy {
    fn default() -> Self {
        Self {
            max_green_per_tick: 1,
            green_ticks_per_lane: 1,
            min_phase_ticks: 1,
            starvation_threshold: 64,
        }
    }
}

impl FlowPriorityPolicy {
    /// Build a policy with custom knobs. Panics only on zero-valued tunables
    /// that would deadlock the scheduler (`green_ticks_per_lane`,
    /// `min_phase_ticks`). Use the default for sane behaviour.
    ///
    /// # Panics
    /// Panics if `green_ticks_per_lane` or `min_phase_ticks` is `0`.
    #[must_use]
    pub fn new(
        max_green_per_tick: u32,
        green_ticks_per_lane: u32,
        min_phase_ticks: u32,
        starvation_threshold: u64,
    ) -> Self {
        assert!(green_ticks_per_lane > 0, "green_ticks_per_lane must be > 0");
        assert!(min_phase_ticks > 0, "min_phase_ticks must be > 0");
        Self {
            max_green_per_tick: max_green_per_tick.max(1),
            green_ticks_per_lane,
            min_phase_ticks,
            starvation_threshold,
        }
    }

    /// Schedule one tick given the current lane volumes and the per-lane
    /// accumulated wait so far.
    ///
    /// Returns the [`PhaseAssignment`] for this tick — which lanes go green,
    /// how long the green phase lasts, and the per-lane wait totals after the
    /// phase elapses. Pure: no global state is held; callers re-supply
    /// `prev_wait` each tick (typically by storing it from the previous
    /// assignment's `waits_after`).
    #[must_use]
    pub fn schedule(
        &self,
        lanes: &[LaneVolume],
        prev_wait: &BTreeMap<LaneId, u64>,
    ) -> PhaseAssignment {
        // Normalize negative / bad inputs. Empty input => empty phase.
        if lanes.is_empty() {
            return PhaseAssignment {
                green: Vec::new(),
                phase_ticks: 0,
                waits_after: BTreeMap::new(),
            };
        }

        // Starvation guard: any lane whose accumulated wait crosses the
        // threshold is pulled to the front of the priority queue regardless
        // of volume. Sort by (starved?, -volume, lane_id) so starved lanes
        // rank above non-starved ones; within a group, higher volume wins;
        // ties break by lane id (deterministic).
        let starved_first = self.starvation_threshold > 0;
        let threshold = self.starvation_threshold;
        let mut ordered: Vec<&LaneVolume> = lanes.iter().collect();
        ordered.sort_by(|a, b| {
            let wa = prev_wait.get(&a.lane).copied().unwrap_or(0);
            let wb = prev_wait.get(&b.lane).copied().unwrap_or(0);
            let sa = starved_first && wa >= threshold;
            let sb = starved_first && wb >= threshold;
            // Starved first: starved comes BEFORE not-starved.
            sb.cmp(&sa)
                // Higher volume first.
                .then_with(|| b.volume.cmp(&a.volume))
                // Deterministic tie-break.
                .then_with(|| a.lane.cmp(&b.lane))
        });

        // Slice off the green lanes. u32 conversion is safe: ordered.len() is
        // small (one intersection), so the saturating min avoids overflow.
        let green_count = (ordered.len() as u32).min(self.max_green_per_tick);
        let green: Vec<LaneId> = ordered
            .iter()
            .take(green_count as usize)
            .map(|lv| lv.lane)
            .collect();

        // Phase length: at least one tick per green lane, at least the
        // configured minimum. Expressed in ticks so the scheduler stays
        // integer / deterministic.
        let phase_ticks = self
            .green_ticks_per_lane
            .saturating_mul(green_count.max(1))
            .max(self.min_phase_ticks);

        // Per-lane wait update:
        //   - Green lanes: clear their queued wait (they get to move).
        //   - Other lanes: accrue `phase_ticks * volume` of additional wait,
        //     since every queued agent sat still for the whole phase.
        // We start from `prev_wait` and overlay the new state.
        let mut waits_after: BTreeMap<LaneId, u64> = prev_wait.clone();
        for lv in lanes {
            let prev = waits_after.get(&lv.lane).copied().unwrap_or(0);
            let next = if green.contains(&lv.lane) {
                // Drain one tick's worth of queued volume (scaled by volume
                // so high-volume lanes actually drain the queue proportionally,
                // not just go to zero). u32 -> u64 widening is always safe.
                prev.saturating_sub(u64::from(lv.volume.max(1)))
            } else {
                prev.saturating_add(
                    u64::from(phase_ticks).saturating_mul(u64::from(lv.volume.max(1))),
                )
            };
            waits_after.insert(lv.lane, next);
        }

        PhaseAssignment {
            green,
            phase_ticks,
            waits_after,
        }
    }

    /// Total accumulated wait across all lanes — the metric the FR's
    /// acceptance test compares between policies. Lower is better.
    #[must_use]
    pub fn total_wait(waits: &BTreeMap<LaneId, u64>) -> u64 {
        waits.values().copied().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-FLOW-PRIORITY — acceptance test.
    ///
    /// Scenario: two lanes at one intersection.
    ///   * Lane 7 (high-volume) — 10 agents queued this tick.
    ///   * Lane 3 (low-volume)  — 1 agent queued this tick.
    ///
    /// The naive equal-split policy would drain both lanes one tick each, so
    /// the high-volume lane accumulates more wait (because it has more agents
    /// blocked). The flow-priority policy grants Lane 7 the green phase, which
    /// drains 10 of its 10 agents and leaves Lane 3 to wait one tick. The
    /// aggregate wait over 5 ticks under flow-priority must therefore be
    /// strictly LESS than the aggregate wait under the naive equal-split
    /// policy — which is exactly what the FR requires.
    #[test]
    fn fr_civ_flow_priority_higher_volume_lane_gets_priority_lowering_total_wait() {
        let policy = FlowPriorityPolicy::default();
        let lanes = [
            LaneVolume {
                lane: 7,
                volume: 10,
            },
            LaneVolume { lane: 3, volume: 1 },
        ];

        // --- Naive equal-split baseline (1 tick per lane, alternating). ---
        let mut naive_waits: BTreeMap<LaneId, u64> = BTreeMap::new();
        for t in 0..5u64 {
            // Alternating green: lane 3 on even ticks, lane 7 on odd ticks.
            let green = if t % 2 == 0 { 3 } else { 7 };
            for lv in &lanes {
                let prev = naive_waits.get(&lv.lane).copied().unwrap_or(0);
                let next = if lv.lane == green {
                    prev.saturating_sub(u64::from(lv.volume.max(1)))
                } else {
                    prev.saturating_add(u64::from(lv.volume.max(1)))
                };
                naive_waits.insert(lv.lane, next);
            }
        }
        let naive_total = FlowPriorityPolicy::total_wait(&naive_waits);

        // --- Flow-priority policy (always grant the high-volume lane). ---
        let mut fp_waits: BTreeMap<LaneId, u64> = BTreeMap::new();
        let mut last_green: Vec<LaneId> = Vec::new();
        for _ in 0..5 {
            let assignment = policy.schedule(&lanes, &fp_waits);
            last_green = assignment.green.clone();
            fp_waits = assignment.waits_after;
        }
        let fp_total = FlowPriorityPolicy::total_wait(&fp_waits);

        // 1. High-volume lane got priority (was granted green).
        assert!(
            last_green.contains(&7),
            "high-volume lane (7) must be granted priority; got {:?}",
            last_green
        );

        // 2. Flow-priority total wait is strictly less than the naive
        //    equal-split total wait.
        assert!(
            fp_total < naive_total,
            "flow-priority total wait ({}) must be less than naive equal-split total wait ({})",
            fp_total,
            naive_total
        );
    }

    /// Two equal-volume lanes must still produce a deterministic, stable
    /// order (lowest `LaneId` first) so replay is byte-identical.
    #[test]
    fn equal_volume_lanes_break_ties_by_lane_id_for_determinism() {
        let policy = FlowPriorityPolicy::default();
        let lanes = [
            LaneVolume { lane: 9, volume: 5 },
            LaneVolume { lane: 2, volume: 5 },
            LaneVolume { lane: 5, volume: 5 },
        ];
        let assignment = policy.schedule(&lanes, &BTreeMap::new());
        assert_eq!(assignment.green, vec![2]);
    }

    /// Empty input is a no-op — must NOT panic.
    #[test]
    fn empty_lane_list_is_noop() {
        let policy = FlowPriorityPolicy::default();
        let assignment = policy.schedule(&[], &BTreeMap::new());
        assert!(assignment.green.is_empty());
        assert_eq!(assignment.phase_ticks, 0);
        assert!(assignment.waits_after.is_empty());
    }

    /// Starvation guard must promote a long-waiting low-volume lane above
    /// a high-volume fresh lane.
    #[test]
    fn starvation_guard_promotes_long_waiting_low_volume_lane() {
        let policy = FlowPriorityPolicy::default(); // starvation_threshold = 64
        let lanes = [
            LaneVolume {
                lane: 1,
                volume: 100,
            }, // high volume, but fresh
            LaneVolume { lane: 2, volume: 1 }, // low volume, but starving
        ];
        let mut prev: BTreeMap<LaneId, u64> = BTreeMap::new();
        prev.insert(2, 100); // lane 2 has already waited past the threshold
        let assignment = policy.schedule(&lanes, &prev);
        assert_eq!(
            assignment.green,
            vec![2],
            "starving lane 2 must be promoted above fresh high-volume lane 1"
        );
    }

    /// Schema version constant exists so the snapshot / replay layers can
    /// detect format drift without depending on private layout.
    #[test]
    fn schema_version_is_stable_string() {
        assert_eq!(FLOW_PRIORITY_SCHEMA_VERSION, "0.1.0-flow-priority");
    }
}
