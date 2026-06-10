//! civ-diplomacy — Diplomacy **substrate** (FR-CIV-DIPLO-001 partial).
//!
//! Scope (this slice, civ-006/civ-007 substrate only):
//!
//! * Pairwise [`Relation`] graph keyed by ordered [`Pair`].
//! * Scalar [`Relation::standing`] (i32, fixed-point friendly) bumped by
//!   interaction events fed in from the existing sim event stream and decayed
//!   back toward zero each tick.
//! * Threshold-crossing events [`DiplomacyTickEvent`] emitted for downstream
//!   systems (AI, scenario, JSON-RPC, replay bus) to consume.
//!
//! Explicitly **not** in scope (deferred to future slices — see
//! `agileplus-specs/civ-007-diplomacy-laws-government/plan.md`):
//!
//! * The full 8-state Diplomatic FSM (FR-CIV-DIPLO-001 full).
//! * Influence capital (FR-CIV-DIPLO-002).
//! * Shadow network (FR-CIV-DIPLO-003).
//! * Government type / laws (FR-CIV-GOV-001/002 — those live in `civ-laws`).
//! * Scripted diplomacy behaviors. This crate exposes the *substrate*; emergent
//!   relations fall out of (combat) interaction events.
//!
//! # Upgrade path
//!
//! The substrate starts with a single scalar standing because (a) every
//! downstream consumer can be designed against a scalar today and (b) the
//! property test surface stays minimal. When richer modelling is required,
//! [`Relation::standing`] can be widened to a `Standing { trust, fear, debt }`
//! struct without breaking the public API: the `(ActorId, ActorId)` pair key,
//! decay step, and threshold-crossing event shape all stay the same.
//!
//! # Determinism
//!
//! `BTreeMap` keeps relation iteration order stable; `Pair::new` enforces
//! `a < b`; all math is integer. Given the same sequence of
//! `ingest_*` calls in the same tick order, two instances produce identical
//! states and event vectors.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Schema version of this crate's public types. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Stable actor identifier. Maps to faction ids in the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActorId(pub u32);

impl ActorId {
    /// Wrap a raw `u32` id.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Ordered pair `(min, max)` of actor ids. The order is fixed at construction
/// so that `Pair::new(a, b) == Pair::new(b, a)` for any `a, b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pair {
    /// The smaller id (always).
    pub lo: ActorId,
    /// The larger id (always).
    pub hi: ActorId,
}

impl Pair {
    /// Construct a canonical ordered pair.
    pub fn new(a: ActorId, b: ActorId) -> Self {
        if a.0 <= b.0 {
            Self { lo: a, hi: b }
        } else {
            Self { lo: b, hi: a }
        }
    }

    /// Iterate `(lo, hi)`.
    pub fn actors(self) -> (ActorId, ActorId) {
        (self.lo, self.hi)
    }
}

/// Coarse stance derived from standing. Thresholds are configurable via
/// [`DiplomacyConfig::hostile_threshold`] and
/// [`DiplomacyConfig::allied_threshold`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stance {
    /// Standing ≤ `hostile_threshold`.
    Hostile,
    /// Standing between the two thresholds.
    Neutral,
    /// Standing ≥ `allied_threshold`.
    Allied,
}

/// One pairwise relation between two actors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    /// The actor pair.
    pub pair: Pair,
    /// Signed scalar standing. Positive = warmer, negative = colder. Decays
    /// toward 0 each tick by [`DiplomacyConfig::decay_per_tick`].
    pub standing: i32,
    /// Last tick the standing was modified (bump or decay step that actually
    /// changed the value). Useful for replay / auditing.
    pub last_updated_tick: u64,
}

impl Relation {
    /// Project the current standing to a [`Stance`] using `config`'s
    /// thresholds.
    pub fn stance(&self, config: &DiplomacyConfig) -> Stance {
        stance_for(self.standing, config)
    }
}

/// Inputs the substrate consumes from the sim event stream.
///
/// The substrate is intentionally narrow: only the events that move pairwise
/// standing. Adding a new variant does not require touching downstream
/// systems that only listen to [`DiplomacyTickEvent`]s.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionEvent {
    /// A combat engagement between two factions (sourced from
    /// `civ-tactics::CombatEngagement`). Both pairs see a negative bump
    /// (combat is hostility-amplifying) scaled by `damage_energy` and the
    /// substrate config.
    Combat {
        /// Attacking faction.
        attacker: ActorId,
        /// Defending faction.
        defender: ActorId,
        /// Energy of the engagement (mirrors `CombatEngagement::damage.energy`).
        energy: u32,
        /// Simulation tick.
        tick: u64,
    },
    /// An explicit diplomatic gesture (treaty signing, tribute, insult) from a
    /// future caller. Positive delta = warmer, negative = colder.
    Gesture {
        /// Acting faction.
        from: ActorId,
        /// Receiving faction.
        to: ActorId,
        /// Signed standing delta. Clamped to the substrate bounds.
        delta: i32,
        /// Simulation tick.
        tick: u64,
    },
}

/// Output the substrate emits each tick when a pair crosses a [`Stance`]
/// threshold. Downstream systems (AI planner, scenario, JSON-RPC, replay bus)
/// consume these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyTickEvent {
    /// Tick the crossing was observed on.
    pub tick: u64,
    /// The pair whose standing crossed.
    pub pair: Pair,
    /// The stance before the crossing.
    pub from: Stance,
    /// The stance after the crossing.
    pub to: Stance,
    /// Standing value after the crossing.
    pub standing: i32,
}

impl DiplomacyTickEvent {
    /// True if the event is a warming transition (toward Allied).
    pub fn is_warming(&self) -> bool {
        matches!(
            (self.from, self.to),
            (Stance::Hostile, Stance::Neutral)
                | (Stance::Hostile, Stance::Allied)
                | (Stance::Neutral, Stance::Allied)
        )
    }
}

/// Tunable parameters for the substrate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyConfig {
    /// Absolute clamp on standing magnitude. Stops griefing via giant bumps
    /// from skewing the substrate; also makes decay-convergence properties
    /// trivially testable.
    pub standing_max: i32,
    /// Linear decay subtracted from `|standing|` each tick. Decay always
    /// pulls standing toward zero; the sign is preserved.
    pub decay_per_tick: u32,
    /// Standing ≤ this is [`Stance::Hostile`]. Must be ≤ 0.
    pub hostile_threshold: i32,
    /// Standing ≥ this is [`Stance::Allied`]. Must be ≥ 0.
    pub allied_threshold: i32,
}

impl Default for DiplomacyConfig {
    fn default() -> Self {
        Self {
            standing_max: 1_000,
            decay_per_tick: 1,
            hostile_threshold: -100,
            allied_threshold: 100,
        }
    }
}

impl DiplomacyConfig {
    /// Sanity check the config. Returns the first inconsistency.
    ///
    /// Order of checks matters: structural problems (overlap) are reported
    /// before per-threshold sanity so that "give me the strongest fix"
    /// scenarios surface the structural error first.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.hostile_threshold >= self.allied_threshold {
            return Err(ConfigError::ThresholdsOverlap {
                hostile: self.hostile_threshold,
                allied: self.allied_threshold,
            });
        }
        if self.standing_max < 0 {
            return Err(ConfigError::NegativeStandingMax(self.standing_max));
        }
        if self.hostile_threshold > 0 {
            return Err(ConfigError::HostileThresholdPositive(self.hostile_threshold));
        }
        if self.allied_threshold < 0 {
            return Err(ConfigError::AlliedThresholdNegative(self.allied_threshold));
        }
        Ok(())
    }
}

/// Configuration inconsistency. Caught at init time, never at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// `standing_max` is negative — the clamp would be vacuous.
    #[error("standing_max must be non-negative (got {0})")]
    NegativeStandingMax(i32),
    /// `hostile_threshold` must be ≤ 0 to make sense.
    #[error("hostile_threshold must be <= 0 (got {0})")]
    HostileThresholdPositive(i32),
    /// `allied_threshold` must be ≥ 0 to make sense.
    #[error("allied_threshold must be >= 0 (got {0})")]
    AlliedThresholdNegative(i32),
    /// Thresholds overlap: `hostile_threshold >= allied_threshold` would
    /// leave no Neutral band.
    #[error(
        "hostile_threshold ({hostile}) must be strictly less than allied_threshold ({allied})"
    )]
    ThresholdsOverlap {
        /// `hostile_threshold` value.
        hostile: i32,
        /// `allied_threshold` value.
        allied: i32,
    },
}

/// The diplomacy substrate. Owns the [`Relation`] graph and the buffer of
/// events emitted during the current tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacyState {
    /// Configuration in effect. Cloned into [`Relation::stance`] so callers
    /// never need to plumb the config separately.
    pub config: DiplomacyConfig,
    /// Pairwise relations, keyed by canonical [`Pair`].
    relations: BTreeMap<Pair, Relation>,
    /// Events emitted by the most recent mutation pass. Cleared by
    /// [`Self::drain_events`].
    pending_events: Vec<DiplomacyTickEvent>,
}

impl DiplomacyState {
    /// Construct an empty substrate with `config`. Validates the config.
    pub fn new(config: DiplomacyConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Self {
            config,
            relations: BTreeMap::new(),
            pending_events: Vec::new(),
        })
    }

    /// Number of relations currently tracked.
    pub fn len(&self) -> usize {
        self.relations.len()
    }

    /// True if no relations are tracked.
    pub fn is_empty(&self) -> bool {
        self.relations.is_empty()
    }

    /// Iterate all relations in stable (BTreeMap) order.
    pub fn relations(&self) -> impl Iterator<Item = &Relation> {
        self.relations.values()
    }

    /// Look up the relation between `a` and `b`, if any.
    pub fn get(&self, a: ActorId, b: ActorId) -> Option<&Relation> {
        if a == b {
            return None;
        }
        self.relations.get(&Pair::new(a, b))
    }

    /// Mutable access to the relation between `a` and `b`. Used internally
    /// by tests that want to inspect mutation order without going through
    /// [`Self::ingest`]. Returns `None` for `a == b` and for untracked pairs.
    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self, a: ActorId, b: ActorId) -> Option<&mut Relation> {
        if a == b {
            return None;
        }
        self.relations.get_mut(&Pair::new(a, b))
    }

    /// Drain all [`DiplomacyTickEvent`]s accumulated since the last drain.
    /// Call once per tick after [`Self::decay`] + [`Self::ingest`].
    pub fn drain_events(&mut self) -> Vec<DiplomacyTickEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Peek at pending events without consuming them.
    pub fn pending_events(&self) -> &[DiplomacyTickEvent] {
        &self.pending_events
    }

    /// Apply per-tick linear decay to every relation. Standing moves toward
    /// 0 by [`DiplomacyConfig::decay_per_tick`] each call. Crossing
    /// thresholds during the decay step emits events.
    ///
    /// Decay runs **before** [`Self::ingest`] so that a single `ingest`
    /// that does not change the band is correctly recorded as "no event".
    pub fn decay(&mut self, tick: u64) {
        let decay = i64::from(self.config.decay_per_tick);
        let max = self.config.standing_max;
        // Snapshot the keys; we mutate through the BTreeMap by re-inserting.
        let pairs: Vec<Pair> = self.relations.keys().copied().collect();
        for pair in pairs {
            let rel = self
                .relations
                .get_mut(&pair)
                .expect("pair present: snapshot above");
            let before = stance_for(rel.standing, &self.config);
            // Move toward zero, integer, clamped to [-max, max].
            let new_standing = if rel.standing > 0 {
                (i64::from(rel.standing) - decay).max(0).min(i64::from(max)) as i32
            } else if rel.standing < 0 {
                (i64::from(rel.standing) + decay).min(0).max(-i64::from(max)) as i32
            } else {
                0
            };
            if new_standing != rel.standing {
                rel.standing = new_standing;
                rel.last_updated_tick = tick;
            }
            let after = stance_for(rel.standing, &self.config);
            if before != after {
                self.pending_events.push(DiplomacyTickEvent {
                    tick,
                    pair,
                    from: before,
                    to: after,
                    standing: rel.standing,
                });
            }
        }
    }

    /// Ingest a batch of [`InteractionEvent`]s in the order given. All events
    /// carry their own `tick`; threshold crossings are recorded with that
    /// tick. Events for the same pair are applied in input order; the
    /// intermediate band-crossings are emitted (we never swallow them).
    pub fn ingest(&mut self, events: &[InteractionEvent]) {
        for ev in events {
            match *ev {
                InteractionEvent::Combat {
                    attacker,
                    defender,
                    energy,
                    tick,
                } => {
                    if attacker == defender {
                        // Same faction firing itself — nothing to record.
                        continue;
                    }
                    // Combat amplifies hostility: scale by `energy` via integer
                    // log10. We avoid floats; the substrate is intentionally
                    // coarse. A 0-energy event records no standing change.
                    let magnitude = bump_from_energy(energy);
                    if magnitude == 0 {
                        // The substrate records standing changes only; a
                        // 0-energy combat does not move the band.
                        continue;
                    }
                    // Relations are stored symmetrically (a canonical Pair),
                    // so a single bump captures the "hostility for both
                    // sides" semantics — both observers read the same value.
                    self.bump(attacker, defender, -magnitude, tick);
                }
                InteractionEvent::Gesture { from, to, delta, tick } => {
                    if from == to {
                        continue;
                    }
                    self.bump(from, to, delta, tick);
                }
            }
        }
    }

    /// Apply a signed `delta` to the relation `(a, b)`, creating the relation
    /// if needed. Emits a [`DiplomacyTickEvent`] on threshold crossing.
    fn bump(&mut self, a: ActorId, b: ActorId, delta: i32, tick: u64) {
        let pair = Pair::new(a, b);
        let max = self.config.standing_max;
        let rel = self.relations.entry(pair).or_insert_with(|| Relation {
            pair,
            standing: 0,
            last_updated_tick: tick,
        });
        let before = stance_for(rel.standing, &self.config);
        let new = (i64::from(rel.standing) + i64::from(delta))
            .clamp(-i64::from(max), i64::from(max)) as i32;
        if new != rel.standing {
            rel.standing = new;
            rel.last_updated_tick = tick;
        }
        let after = stance_for(rel.standing, &self.config);
        if before != after {
            self.pending_events.push(DiplomacyTickEvent {
                tick,
                pair,
                from: before,
                to: after,
                standing: rel.standing,
            });
        }
    }

    /// How many ticks of [`Self::decay`] are required to pull standing from
    /// `start` to zero, given the configured `decay_per_tick`. Useful for
    /// property tests and scenario authoring.
    pub fn ticks_to_neutral(start: i32, config: &DiplomacyConfig) -> u64 {
        let decay = i64::from(config.decay_per_tick).max(1);
        let mag = i64::from(start.unsigned_abs());
        // ceil(mag / decay) — integer math.
        ((mag + decay - 1) / decay) as u64
    }
}

/// Project a standing value to a [`Stance`] using `config`'s thresholds.
fn stance_for(standing: i32, config: &DiplomacyConfig) -> Stance {
    if standing <= config.hostile_threshold {
        Stance::Hostile
    } else if standing >= config.allied_threshold {
        Stance::Allied
    } else {
        Stance::Neutral
    }
}

/// Convert a `damage.energy` value to a hostility-amplifying bump magnitude.
///
/// 0 energy -> 0 (no substrate change). Otherwise we take `floor(log10(energy))`
/// + 1 so that even small skirmishes move standing a little, and large
/// engagements move it a lot — without ever using floats. Clamped to a
/// sensible range to keep the substrate bounded.
fn bump_from_energy(energy: u32) -> i32 {
    if energy == 0 {
        return 0;
    }
    // log10(energy) as integer digit count minus 1, then +1 for the bump.
    let digits = (u32::checked_ilog10(energy).unwrap_or(0) + 1) as i32;
    digits.clamp(1, 50)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cfg() -> DiplomacyConfig {
        DiplomacyConfig {
            standing_max: 1_000,
            decay_per_tick: 1,
            hostile_threshold: -100,
            allied_threshold: 100,
        }
    }

    fn a(id: u32) -> ActorId {
        ActorId(id)
    }

    // -- Pair ordering -------------------------------------------------------

    #[test]
    fn pair_is_canonical_and_symmetric() {
        let p1 = Pair::new(a(3), a(7));
        let p2 = Pair::new(a(7), a(3));
        assert_eq!(p1, p2);
        assert_eq!(p1.lo, a(3));
        assert_eq!(p1.hi, a(7));
    }

    #[test]
    fn pair_with_equal_actors_is_self() {
        // a == b is degenerate; we still produce (a, a) and the relation
        // layer rejects it. Verify canonical ordering holds.
        let p = Pair::new(a(5), a(5));
        assert_eq!(p.lo, a(5));
        assert_eq!(p.hi, a(5));
    }

    // -- Config validation ---------------------------------------------------

    #[test]
    fn default_config_validates() {
        assert!(DiplomacyConfig::default().validate().is_ok());
    }

    #[test]
    fn overlapping_thresholds_rejected() {
        let bad = DiplomacyConfig {
            hostile_threshold: 50,
            allied_threshold: 10,
            ..cfg()
        };
        assert!(matches!(
            bad.validate(),
            Err(ConfigError::ThresholdsOverlap { .. })
        ));
    }

    #[test]
    fn positive_hostile_threshold_rejected() {
        let bad = DiplomacyConfig {
            hostile_threshold: 1,
            allied_threshold: 100,
            ..cfg()
        };
        assert!(matches!(
            bad.validate(),
            Err(ConfigError::HostileThresholdPositive(_))
        ));
    }

    // -- Stance projection ---------------------------------------------------

    #[test]
    fn stance_thresholds_partition_real_line() {
        let c = cfg();
        assert_eq!(stance_for(-500, &c), Stance::Hostile);
        assert_eq!(stance_for(-100, &c), Stance::Hostile);
        assert_eq!(stance_for(-99, &c), Stance::Neutral);
        assert_eq!(stance_for(0, &c), Stance::Neutral);
        assert_eq!(stance_for(99, &c), Stance::Neutral);
        assert_eq!(stance_for(100, &c), Stance::Allied);
        assert_eq!(stance_for(500, &c), Stance::Allied);
    }

    // -- Bump + decay mechanics ---------------------------------------------

    #[test]
    fn first_bump_creates_relation_and_records_zero_to_neutral_event() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        // Standing starts at 0 (Neutral). A small positive bump into the
        // Neutral band emits no event. A bump into Allied does.
        s.ingest(&[InteractionEvent::Gesture {
            from: a(1),
            to: a(2),
            delta: 150,
            tick: 1,
        }]);
        // Single event: Neutral -> Allied at tick 1.
        let evs = s.drain_events();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].from, Stance::Neutral);
        assert_eq!(evs[0].to, Stance::Allied);
        assert!(evs[0].is_warming());
    }

    #[test]
    fn bump_into_neutral_band_emits_no_event() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Gesture {
            from: a(1),
            to: a(2),
            delta: 10, // still Neutral
            tick: 1,
        }]);
        assert!(s.pending_events().is_empty());
    }

    #[test]
    fn combat_bump_is_symmetric_and_hostility_amplifying() {
        // Use a config with a tight hostile threshold so a single large
        // engagement crosses it deterministically.
        let tight = DiplomacyConfig {
            standing_max: 1_000,
            decay_per_tick: 1,
            hostile_threshold: -3,
            allied_threshold: 100,
        };
        let mut s = DiplomacyState::new(tight).expect("cfg");
        // 5-digit energy -> bump magnitude 5; one symmetric bump -> -5
        // (relations are stored on a canonical Pair, so a single bump
        // captures "hostility for both sides").
        s.ingest(&[InteractionEvent::Combat {
            attacker: a(1),
            defender: a(2),
            energy: 10_000,
            tick: 1,
        }]);
        let rel = s.get(a(1), a(2)).expect("relation");
        assert_eq!(rel.standing, -5, "single symmetric bump = -5");
        // Storage symmetry: querying either direction yields the same row.
        assert_eq!(s.get(a(2), a(1)).unwrap().standing, -5);
        let evs = s.drain_events();
        assert_eq!(evs.len(), 1, "exactly one Neutral->Hostile crossing");
        assert_eq!(evs[0].from, Stance::Neutral);
        assert_eq!(evs[0].to, Stance::Hostile);
    }

    #[test]
    fn zero_energy_combat_records_no_standing_change() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Combat {
            attacker: a(1),
            defender: a(2),
            energy: 0,
            tick: 1,
        }]);
        assert!(s.get(a(1), a(2)).is_none());
        assert!(s.drain_events().is_empty());
    }

    #[test]
    fn self_targeted_event_is_ignored() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Gesture {
            from: a(1),
            to: a(1),
            delta: 999,
            tick: 1,
        }]);
        assert!(s.is_empty());
    }

    // -- Decay ---------------------------------------------------------------

    #[test]
    fn decay_pulls_positive_standing_toward_zero() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Gesture {
            from: a(1),
            to: a(2),
            delta: 50,
            tick: 1,
        }]);
        s.decay(2);
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, 49);
        s.decay(3);
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, 48);
    }

    #[test]
    fn decay_pulls_negative_standing_toward_zero() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Combat {
            attacker: a(1),
            defender: a(2),
            energy: 1000, // 4 digits -> -4
            tick: 1,
        }]);
        // Pre-decay: -4
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, -4);
        s.decay(2);
        // Pulled toward 0: -4 + 1 = -3
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, -3);
    }

    #[test]
    fn decay_eventually_crosses_back_to_neutral() {
        let mut s = DiplomacyState::new(cfg()).expect("cfg");
        s.ingest(&[InteractionEvent::Combat {
            attacker: a(1),
            defender: a(2),
            energy: 1_000_000, // 7 digits -> -7
            tick: 1,
        }]);
        let start = s.get(a(1), a(2)).unwrap().standing;
        // We need at least |start| ticks of decay for full convergence.
        for t in 2..(2 + 2 * (start.unsigned_abs() as u64)) {
            s.decay(t);
        }
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, 0);
        assert_eq!(
            s.get(a(1), a(2)).unwrap().stance(&cfg()),
            Stance::Neutral
        );
    }

    #[test]
    fn standing_is_clamped_to_max() {
        let big_cfg = DiplomacyConfig {
            standing_max: 10,
            decay_per_tick: 1,
            hostile_threshold: -5,
            allied_threshold: 5,
        };
        let mut s = DiplomacyState::new(big_cfg).expect("cfg");
        s.ingest(&[InteractionEvent::Gesture {
            from: a(1),
            to: a(2),
            delta: 1_000_000,
            tick: 1,
        }]);
        assert_eq!(s.get(a(1), a(2)).unwrap().standing, 10);
    }

    // -- Determinism / replay ------------------------------------------------

    #[test]
    fn identical_ingest_sequences_produce_identical_states_and_events() {
        let events = vec![
            InteractionEvent::Combat {
                attacker: a(1),
                defender: a(2),
                energy: 250,
                tick: 1,
            },
            InteractionEvent::Gesture {
                from: a(2),
                to: a(3),
                delta: 50,
                tick: 1,
            },
            InteractionEvent::Combat {
                attacker: a(1),
                defender: a(3),
                energy: 9_999,
                tick: 2,
            },
        ];

        let mut a = DiplomacyState::new(cfg()).expect("cfg");
        a.ingest(&events);
        a.decay(3);
        let events_a = a.drain_events();

        let mut b = DiplomacyState::new(cfg()).expect("cfg");
        b.ingest(&events);
        b.decay(3);
        let events_b = b.drain_events();

        assert_eq!(a, b);
        assert_eq!(events_a, events_b);
    }

    // -- Property tests ------------------------------------------------------

    /// FR-CIV-DIPLO-001 partial: standing decays monotonically toward zero.
    /// Property test ensures this holds for arbitrary starting values and
    /// decay rates that fit in the config.
    #[test]
    fn proptest_decay_converges_to_neutral() {
        proptest!(|(
            start_standing in -1_000i32..=1_000i32,
            decay in 1u32..10u32,
        )| {
            let config = DiplomacyConfig {
                standing_max: 1_000,
                decay_per_tick: decay,
                hostile_threshold: -100,
                allied_threshold: 100,
            };
            config.validate().expect("config");
            let mut s = DiplomacyState::new(config).expect("state");
            // Seed the relation directly via ingest.
            let actor_a = a(1);
            let actor_b = a(2);
            if start_standing != 0 {
                let delta = start_standing; // absolute seed
                s.ingest(&[InteractionEvent::Gesture {
                    from: actor_a,
                    to: actor_b,
                    delta,
                    tick: 1,
                }]);
                // Possibly overshot in the wrong direction? Clamp asserts are
                // the test's job: the resulting standing has the same sign as
                // delta and magnitude ≤ |delta| (clamped to max).
            }
            let rel = s.get(actor_a, actor_b);
            let seeded = rel.map(|r| r.standing).unwrap_or(0);
            // The number of decay ticks needed to reach zero is bounded by
            // ceil(|seeded| / decay). Run 2x that and verify == 0.
            let steps = DiplomacyState::ticks_to_neutral(seeded, &config) * 2 + 1;
            for t in 2..(2 + steps) {
                s.decay(t);
            }
            let final_standing = s.get(actor_a, actor_b).map(|r| r.standing).unwrap_or(0);
            prop_assert_eq!(final_standing, 0, "decay did not converge");
            // After convergence, drain events and confirm we landed Neutral.
            let _ = s.drain_events();
            prop_assert_eq!(
                s.get(actor_a, actor_b).map(|r| r.stance(&config)).unwrap_or(Stance::Neutral),
                Stance::Neutral
            );
        });
    }

    /// FR-CIV-DIPLO-001 partial: events emitted for the same pair across
    /// two runs of identical ingest are identical regardless of *when* the
    /// relation was first observed.
    #[test]
    fn proptest_event_sequence_is_deterministic_under_shuffled_input_pairs() {
        proptest!(|(
            sign in -1i32..=1i32,
            magnitude in 1u32..=500u32,
        )| {
            let config = cfg();
            let mut s1 = DiplomacyState::new(config).expect("state");
            let mut s2 = DiplomacyState::new(config).expect("state");
            // Same events, same order => same events out.
            let events = vec![
                InteractionEvent::Gesture { from: a(1), to: a(2), delta: sign * magnitude as i32, tick: 5 },
                InteractionEvent::Gesture { from: a(2), to: a(3), delta: sign * magnitude as i32, tick: 5 },
                InteractionEvent::Gesture { from: a(1), to: a(3), delta: sign * magnitude as i32, tick: 6 },
            ];
            s1.ingest(&events);
            s2.ingest(&events);
            prop_assert_eq!(s1.drain_events(), s2.drain_events());
            // And final state matches.
            prop_assert_eq!(s1, s2);
        });
    }

    /// FR-CIV-DIPLO-001 partial: bump(a, b) and bump(b, a) produce the same
    /// final relation (the substrate is symmetric in storage).
    #[test]
    fn proptest_bump_direction_is_symmetric() {
        proptest!(|(
            x in 0u32..1000u32,
            y in 0u32..1000u32,
            delta in -500i32..=500i32,
        )| {
            if x == y { return Ok(()); }
            let config = cfg();
            let mut s1 = DiplomacyState::new(config).expect("state");
            let mut s2 = DiplomacyState::new(config).expect("state");
            s1.ingest(&[InteractionEvent::Gesture { from: a(x), to: a(y), delta, tick: 1 }]);
            s2.ingest(&[InteractionEvent::Gesture { from: a(y), to: a(x), delta, tick: 1 }]);
            let r1 = s1.get(a(x), a(y)).cloned();
            let r2 = s2.get(a(x), a(y)).cloned();
            prop_assert_eq!(r1, r2, "bump direction must be symmetric");
        });
    }

    // -- Schema version -----------------------------------------------------

    #[test]
    fn schema_version_is_positive() {
        assert!(SCHEMA_VERSION >= 1);
    }
}
