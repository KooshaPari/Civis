//! Legend memory decay (FR-CIV-LEGEND-DECAY).
//!
//! The saga graph records *what* happened; this module models *how long it is
//! remembered*. Each [`LegendEntry`] carries a **prominence** score in `0..=1`
//! that:
//!
//! * **decays exponentially** each epoch at a configurable rate (per
//!   [`LegendsConfig::decay`]) — old legends fade into hearsay and eventually
//!   into oblivion, mirroring real oral history.
//! * **rises on reinforcement** when a new related event lands — recent
//!   activity bumps prominence back up (capped at `1.0`).
//!
//! This is purely a memory/salience layer. It never authors sim outcomes and
//! never mutates the saga graph's [`crate::graph::SagaGraph`] structure —
//! callers (the narrator, the rumor mill, the inspector decay-order view)
//! read prominence out of the [`ProminenceTracker`] and render accordingly.
//!
//! Charter: see `docs/design/legends-engine.md` §6 (memory decay).
//! Spec: **FR-CIV-LEGEND-DECAY** — a legend's prominence decays over time
//! unless reinforced by new events.
//!
//! ADDITIVE: this module does not touch any existing engine logic. It lives
//! alongside it as an opt-in tracker.

use std::collections::HashMap;

use crate::ids::LegendEventId;
use crate::model::LegendEntry;

/// Configuration for the decay engine (FR-CIV-LEGEND-DECAY).
///
/// Values are tuned; nothing here is a charter *outcome* — it only tunes
/// "how fast legends fade", not "what happens in the sim".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecayConfig {
    /// Per-epoch multiplicative decay factor in `(0.0, 1.0]`.
    /// `0.9` ⇒ after ~22 epochs a forgotten legend is at ~10% of peak;
    /// `1.0` ⇒ no decay (legend is immortal); `0.0` ⇒ instant oblivion.
    pub decay_rate: f32,
    /// Lower bound for prominence (never falls below this).
    /// `0.0` ⇒ legends can be fully forgotten; `>0.0` ⇒ a residual cultural
    /// footprint (rumors + monuments) survives.
    pub floor: f32,
    /// Increment applied per reinforcement event, clamped into `(0.0, 1.0]`.
    pub reinforcement_delta: f32,
}

impl Default for DecayConfig {
    fn default() -> Self {
        // Mirrors `LegendsConfig::default().decay = 0.9` so the two decay
        // surfaces stay in lockstep unless callers override.
        DecayConfig {
            decay_rate: 0.9,
            floor: 0.0,
            reinforcement_delta: 0.2,
        }
    }
}

/// Per-legend memory state. `epoch` is the most recent epoch the tracker
/// observed for this legend (used to advance decay across elapsed epochs in
/// one shot).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prominence {
    /// Current remembered salience in `0..=1`.
    pub value: f32,
    /// Epoch at which `value` is the *post-tick* / *post-reinforcement* level.
    pub epoch: u64,
}

impl Prominence {
    fn new(value: f32, epoch: u64) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            epoch,
        }
    }
}

/// Tracks per-legend prominence with decay-by-elapsed-epoch + reinforcement.
///
/// Usage:
/// ```ignore
/// let mut t = ProminenceTracker::default();
/// let p = t.observe(&legend_entry, 0, /* matched = */ false, &DecayConfig::default());
/// assert!(p.value <= 1.0);
/// ```
///
/// The tracker is **not** part of [`crate::graph::SagaGraph`]; it is a
/// standalone adjunct the narrator + inspector query on demand.
#[derive(Debug, Clone, Default)]
pub struct ProminenceTracker {
    /// legend id → prominence
    by_legend: HashMap<LegendEventId, Prominence>,
}

impl ProminenceTracker {
    /// Construct a fresh, empty tracker.
    pub fn new() -> Self {
        Self {
            by_legend: HashMap::new(),
        }
    }

    /// Number of legends currently tracked.
    pub fn len(&self) -> usize {
        self.by_legend.len()
    }

    /// True iff no legends are tracked.
    pub fn is_empty(&self) -> bool {
        self.by_legend.is_empty()
    }

    /// Current prominence for a legend id, if any.
    pub fn get(&self, id: LegendEventId) -> Option<Prominence> {
        self.by_legend.get(&id).copied()
    }

    /// Advance decay for a single legend to `current_epoch` and optionally
    /// reinforce it (when `matched == true`).
    ///
    /// * If the legend is **not yet tracked**, this seeds it at
    ///   `decay_config.reinforcement_delta` (a freshly seen legend has some
    ///   salience) if `matched`, else at `0.0`.
    /// * If the legend **is tracked**, decay is applied across the elapsed
    ///   epoch delta (`decay_rate^elapsed`), then `reinforcement_delta` is
    ///   added if `matched`. Final value is clamped to `[floor, 1.0]`.
    ///
    /// Returns the post-tick prominence snapshot.
    pub fn observe(
        &mut self,
        legend: &LegendEntry,
        current_epoch: u64,
        matched: bool,
        decay_config: &DecayConfig,
    ) -> Prominence {
        let id = legend.id;
        let next = match self.by_legend.get(&id).copied() {
            None => {
                // Seed: brand-new legend, salience starts low unless we already
                // matched (a reinforcing event saw it).
                let seed = if matched { legend.importance } else { 0.0 };
                Prominence::new(seed, current_epoch)
            }
            Some(prev) => {
                // Decay across elapsed epochs.
                let elapsed = current_epoch.saturating_sub(prev.epoch);
                let decayed = apply_decay(prev.value, elapsed, decay_config);
                let reinforced = if matched {
                    (decayed + decay_config.reinforcement_delta).min(1.0)
                } else {
                    decayed
                };
                let clamped = reinforced.max(decay_config.floor).min(1.0);
                Prominence::new(clamped, current_epoch)
            }
        };
        self.by_legend.insert(id, next);
        next
    }

    /// Explicitly prune legends below the supplied floor. Returns the ids
    /// pruned. Idempotent.
    pub fn prune_below(&mut self, floor: f32) -> Vec<LegendEventId> {
        let before: Vec<_> = self
            .by_legend
            .iter()
            .filter_map(|(k, v)| if v.value < floor { Some(*k) } else { None })
            .collect();
        for id in &before {
            self.by_legend.remove(id);
        }
        before
    }

    /// Decay *all* tracked legends to `current_epoch` without reinforcement.
    /// Convenience for end-of-epoch sweeps when no events were matched.
    pub fn sweep(&mut self, current_epoch: u64, decay_config: &DecayConfig) {
        let ids: Vec<_> = self.by_legend.keys().copied().collect();
        for id in ids {
            if let Some(prev) = self.by_legend.get(&id).copied() {
                let elapsed = current_epoch.saturating_sub(prev.epoch);
                let decayed = apply_decay(prev.value, elapsed, decay_config);
                let clamped = decayed.max(decay_config.floor).min(1.0);
                self.by_legend
                    .insert(id, Prominence::new(clamped, current_epoch));
            }
        }
    }
}

/// Apply per-epoch multiplicative decay across `elapsed` epochs.
///
/// `elapsed = 0` is a no-op. `elapsed` saturates at `u64::MAX - prev.epoch`
/// in the caller, so we just guard against `decay_rate <= 0.0` (treated as
/// instant floor) and `decay_rate > 1.0` (clamped to 1.0 — no decay).
fn apply_decay(value: f32, elapsed: u64, decay_config: &DecayConfig) -> f32 {
    if elapsed == 0 {
        return value;
    }
    let rate = decay_config.decay_rate.clamp(0.0, 1.0);
    let factor = rate.powi(elapsed as i32);
    (value * factor).max(decay_config.floor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Epoch, LegendEntityId, NameRef, Provenance, RegionId, SourceCrate};
    use crate::model::{EventKind, Tag};
    use smallvec::SmallVec;

    fn make_legend(id: u64) -> LegendEntry {
        LegendEntry {
            id: LegendEventId(id),
            name: Some(NameRef(id)),
            event_id: LegendEventId(id),
            principal_entity: LegendEntityId(id),
            epoch: Epoch(0),
            importance: 0.8,
            event_kind: EventKind::Battle,
            region: Some(RegionId(0)),
            participants: SmallVec::new(),
            provenance: Provenance::Lived,
        }
    }

    #[test]
    fn prominence_falls_over_time() {
        // Spec: FR-CIV-LEGEND-DECAY — prominence decays unless reinforced.
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig {
            decay_rate: 0.9,
            floor: 0.0,
            reinforcement_delta: 0.0, // explicit: no later reinforcement
        };

        // Seed legend 0 at epoch 0 with a single reinforcement (matched=true).
        let legend = make_legend(0);
        let p0 = t.observe(&legend, 0, true, &cfg);
        assert!(
            (p0.value - legend.importance).abs() < 1e-6,
            "seed should equal legend importance, got {}",
            p0.value
        );

        // Advance 10 epochs with no reinforcement → value must fall.
        let p1 = t.observe(&legend, 10, false, &cfg);
        assert!(
            p1.value < p0.value,
            "prominence must fall: p0={}, p1={}",
            p0.value,
            p1.value
        );
        // Factor = 0.9^10 ≈ 0.3487
        let expected = (p0.value * 0.9f32.powi(10)).max(cfg.floor).min(1.0);
        assert!(
            (p1.value - expected).abs() < 1e-3,
            "expected ~{}, got {}",
            expected,
            p1.value
        );
    }

    #[test]
    fn prominence_rises_on_reinforcement() {
        // Spec: FR-CIV-LEGEND-DECAY — reinforcement restores prominence.
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig {
            decay_rate: 0.9,
            floor: 0.0,
            reinforcement_delta: 0.3,
        };

        let legend = make_legend(42);

        // Seed at epoch 0 with a reinforcement.
        let p0 = t.observe(&legend, 0, true, &cfg);
        assert!((p0.value - legend.importance).abs() < 1e-6);

        // Wait 20 epochs → should decay substantially.
        let p1 = t.observe(&legend, 20, false, &cfg);
        assert!(p1.value < p0.value);

        // Reinforcement lands → should rise again (above p1).
        let p2 = t.observe(&legend, 20, true, &cfg);
        assert!(
            p2.value > p1.value,
            "reinforcement must raise prominence: p1={}, p2={}",
            p1.value,
            p2.value
        );
        // Should also not exceed the seeded peak beyond the cap.
        assert!(p2.value <= 1.0);
        // And not exceed the seeded value plus a reinforcement step
        // (decayed-from-seed path: p0 * 0.9^20 + 0.3, clamped to 1.0).
        let expected = (p0.value * 0.9f32.powi(20) + 0.3).min(1.0);
        assert!(
            (p2.value - expected).abs() < 1e-3,
            "expected ~{}, got {}",
            expected,
            p2.value
        );
    }

    #[test]
    fn reinforcement_caps_at_one() {
        // Repeated reinforcement must clamp at 1.0, never overshoot.
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig::default();
        let legend = make_legend(7);

        // Hammer it 100 times at the same epoch → must saturate at 1.0.
        for _ in 0..100 {
            t.observe(&legend, 0, true, &cfg);
        }
        let p = t.get(LegendEventId(7)).expect("tracked");
        assert!(
            (p.value - 1.0).abs() < 1e-6,
            "must clamp to 1.0, got {}",
            p.value
        );
    }

    #[test]
    fn floor_prevents_total_oblivion() {
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig {
            decay_rate: 0.5,
            floor: 0.05, // a small but nonzero cultural footprint
            reinforcement_delta: 0.5,
        };
        let legend = make_legend(99);

        // Reinforce once, then let it decay for 1000 epochs. Without the
        // floor it would hit 0; with the floor it must stay at >= 0.05.
        t.observe(&legend, 0, true, &cfg);
        let p = t.observe(&legend, 1000, false, &cfg);
        assert!(
            p.value >= cfg.floor - 1e-6,
            "floor must hold; got {}",
            p.value
        );
        assert!(p.value <= 1.0);
    }

    #[test]
    fn sweep_decays_uniformly() {
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig {
            decay_rate: 0.8,
            floor: 0.0,
            reinforcement_delta: 0.4,
        };

        // Seed three legends at epoch 0, each reinforced.
        for id in [1u64, 2, 3] {
            let legend = make_legend(id);
            t.observe(&legend, 0, true, &cfg);
        }

        // Sweep to epoch 5 with no matches → all three must decay equally.
        t.sweep(5, &cfg);
        let v1 = t.get(LegendEventId(1)).unwrap().value;
        let v2 = t.get(LegendEventId(2)).unwrap().value;
        let v3 = t.get(LegendEventId(3)).unwrap().value;
        assert!((v1 - v2).abs() < 1e-6);
        assert!((v2 - v3).abs() < 1e-6);
        // Seeded legend importance decays uniformly across elapsed epochs.
        let expected = (make_legend(1).importance * 0.8f32.powi(5)).max(0.0);
        assert!(
            (v1 - expected).abs() < 1e-3,
            "expected ~{}, got {}",
            expected,
            v1
        );
    }

    #[test]
    fn prune_below_drops_forgotten_legends() {
        let mut t = ProminenceTracker::new();
        let cfg = DecayConfig {
            decay_rate: 0.1, // very fast decay
            floor: 0.0,
            reinforcement_delta: 0.0,
        };
        let strong = make_legend(1);
        let weak = make_legend(2);

        // Seed `strong` with a non-decaying baseline (we'll bypass via manual
        // insert by hammering at epoch 0 with a high delta… simplest: just
        // set reinforcement_delta high enough that one hit exceeds floor).
        let cfg_high = DecayConfig {
            reinforcement_delta: 0.9,
            ..cfg
        };
        t.observe(&strong, 0, true, &cfg_high);
        t.observe(&weak, 0, true, &cfg_high);

        // Wait many epochs; both should decay close to floor.
        t.observe(&strong, 50, false, &cfg);
        t.observe(&weak, 50, false, &cfg);

        // Prune below 0.01 → both should be removed (decayed to floor=0).
        let pruned = t.prune_below(0.01);
        assert!(
            pruned.contains(&LegendEventId(1)),
            "expected strong legend pruned: {:?}",
            pruned
        );
        assert!(t.is_empty(), "tracker should be empty after pruning");
    }

    // --- silence dead-code warnings on fields/methods not exercised here ---

    #[allow(dead_code)]
    fn _unused_anchors() {
        // Touching private module items to keep the model field references
        // honest in tests above.
        let _t: Tag = "unused".to_string();
        let _s: SourceCrate = SourceCrate::Agents;
    }
}
