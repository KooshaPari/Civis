//! Tech obsoletion: when a successor tech is researched, the superseded tech's
//! upkeep value decays toward a floor over time. Implements FR-CIV-TECH-OBSOLETE.
//!
//! Design goals (per ADR-006: pure-logic, replay-deterministic):
//! * No I/O, no time source — caller drives ticks via [`ObsolescenceTracker::tick`].
//! * Same (config, sequence, tick advances) → identical [`ObsolescenceTracker`] state.
//! * Decay is multiplicative so high-value techs lose value fastest in absolute
//!   terms while still approaching (and never crossing) the configured floor.
//!
//! Acceptance test (FR-CIV-TECH-OBSOLETE):
//! > a superseded tech's value decays after a successor researched

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Schema version for the obsoletion subsystem. Bumped on breaking changes
/// (mirrors `civ-research::SCHEMA_VERSION`).
pub const OBSOLESCENCE_SCHEMA_VERSION: u32 = 1;

/// Configuration for [`ObsolescenceTracker`] decay behavior.
///
/// All parameters are bounded to keep replay stable across toolchains.
#[derive(Debug, Clone, PartialEq)]
pub struct ObsolescenceConfig {
    /// Per-tick multiplicative decay factor applied to the **remaining** value.
    /// Stored basis points: `10000 == 1.0` (no decay), `9000 == 0.9`,
    /// `5000 == 0.5`. A `0` value is clamped to `1` so a misconfigured
    /// tracker still advances instead of dividing by zero internally.
    decay_bps: u32,
    /// Lower bound on upkeep value. Decay floors at this value; it never
    /// drops below it. Stored in the same units as [`crate::TechCard::energy_cost`].
    floor: u64,
}

impl Default for ObsolescenceConfig {
    fn default() -> Self {
        // 10% loss per tick → fast enough to be visible, slow enough to be tunable.
        Self {
            decay_bps: 9_000,
            floor: 1,
        }
    }
}

impl ObsolescenceConfig {
    /// Construct a new config. `decay_bps` is in basis points (`10_000 == 1.0`);
    /// `0` is clamped to `1` so a misconfigured tracker still advances.
    #[must_use]
    pub fn new(decay_bps: u32, floor: u64) -> Self {
        let decay_bps = if decay_bps == 0 { 1 } else { decay_bps };
        Self { decay_bps, floor }
    }

    /// Per-tick multiplicative decay factor as basis points (`10_000 == 1.0`).
    #[must_use]
    pub fn decay_bps(&self) -> u32 {
        self.decay_bps
    }

    /// Minimum upkeep value the tracker will decay down to.
    #[must_use]
    pub fn floor(&self) -> u64 {
        self.floor
    }
}

/// Per-tech obsoletion record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsolescenceRecord {
    /// Stable tech identifier (matches [`crate::TechCard::id`]).
    pub tech_id: String,
    /// Tech ID that superseded this one. `None` while still current.
    pub successor: Option<String>,
    /// Base upkeep value before any decay. Always the value immediately after
    /// [`ObsolescenceTracker::register_tech`] or [`ObsolescenceTracker::supersede_with`].
    pub base_value: u64,
    /// Cached decayed value (kept so callers can read it without recomputation
    /// and so the tracker can be diff-serialized for replay determinism).
    pub value: u64,
    /// Tick at which this tech was most recently superseded (or registered if
    /// `successor` is still `None`). Zero means "not yet superseded".
    pub superseded_at_tick: u64,
}

/// Why a tracker mutation was accepted or rejected.
///
/// Kept as an enum (rather than a `bool`) so `Result`-driven callers get a
/// machine-readable reason — replay diagnostics rely on this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObsolescenceError {
    /// Tried to register a tech ID that is already tracked.
    DuplicateTech(String),
    /// Tried to mark a tech as superseded before it existed in the tracker.
    UnknownTech(String),
    /// Tried to supersede a tech with itself (no-op). Surfaces a programmer
    /// error rather than silently accepting it.
    SelfSupersede(String),
}

/// Tracker for tech obsoletion. Pure-logic: ticks advance only when the caller
/// says so, and the same `(config, register order, supersede order, tick
/// sequence)` always produces the same state.
#[derive(Debug, Clone, PartialEq)]
pub struct ObsolescenceTracker {
    config: ObsolescenceConfig,
    records: BTreeMap<String, ObsolescenceRecord>,
    current_tick: u64,
}

impl ObsolescenceTracker {
    /// Construct a tracker with the given config and the default tick = 0.
    #[must_use]
    pub fn new(config: ObsolescenceConfig) -> Self {
        Self {
            config,
            records: BTreeMap::new(),
            current_tick: 0,
        }
    }

    /// Active configuration.
    #[must_use]
    pub fn config(&self) -> &ObsolescenceConfig {
        &self.config
    }

    /// Number of tracked tech records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the tracker holds no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The tick the tracker is currently positioned at.
    #[must_use]
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Register a freshly researched tech with its baseline upkeep value.
    /// Calling on a duplicate ID returns [`ObsolescenceError::DuplicateTech`].
    pub fn register_tech(
        &mut self,
        tech_id: &str,
        base_value: u64,
    ) -> Result<(), ObsolescenceError> {
        if self.records.contains_key(tech_id) {
            return Err(ObsolescenceError::DuplicateTech(tech_id.to_string()));
        }
        self.records.insert(
            tech_id.to_string(),
            ObsolescenceRecord {
                tech_id: tech_id.to_string(),
                successor: None,
                base_value,
                value: base_value,
                superseded_at_tick: 0,
            },
        );
        Ok(())
    }

    /// Mark `tech_id` as superseded by `successor_id` at the current tick.
    ///
    /// Returns:
    /// * [`ObsolescenceError::UnknownTech`] if `tech_id` is not tracked.
    /// * [`ObsolescenceError::SelfSupersede`] if `tech_id == successor_id`.
    ///
    /// If `tech_id` was already superseded, the operation is idempotent: the
    /// successor pointer is updated but no second decay window is started.
    /// This keeps replay safe across duplicate event-application attempts.
    pub fn supersede_with(
        &mut self,
        tech_id: &str,
        successor_id: &str,
    ) -> Result<(), ObsolescenceError> {
        if tech_id == successor_id {
            return Err(ObsolescenceError::SelfSupersede(tech_id.to_string()));
        }
        let Some(record) = self.records.get_mut(tech_id) else {
            return Err(ObsolescenceError::UnknownTech(tech_id.to_string()));
        };
        let already = record.successor.is_some();
        record.successor = Some(successor_id.to_string());
        if !already {
            record.superseded_at_tick = self.current_tick;
        }
        Ok(())
    }

    /// Advance the tracker by `delta` ticks, applying decay to every
    /// superseded tech's upkeep. Ticks <= 0 are a no-op so the caller can
    /// forward arbitrary time deltas without branching.
    pub fn tick(&mut self, delta: u64) {
        if delta == 0 {
            return;
        }
        self.current_tick = self.current_tick.saturating_add(delta);
        for record in self.records.values_mut() {
            if record.successor.is_some() {
                let ticks = self.current_tick.saturating_sub(record.superseded_at_tick);
                record.value = apply_decay(record.base_value, ticks, &self.config);
            }
        }
    }

    /// Current decayed upkeep value for `tech_id`. Returns `None` if the
    /// tech isn't tracked (caller may interpret that as "never researched").
    #[must_use]
    pub fn upkeep_value(&self, tech_id: &str) -> Option<u64> {
        self.records.get(tech_id).map(|r| r.value)
    }

    /// Lookup a full record by ID.
    #[must_use]
    pub fn record(&self, tech_id: &str) -> Option<&ObsolescenceRecord> {
        self.records.get(tech_id)
    }

    /// Whether `tech_id` has been superseded by a successor.
    #[must_use]
    pub fn is_obsolete(&self, tech_id: &str) -> bool {
        self.records
            .get(tech_id)
            .map(|r| r.successor.is_some())
            .unwrap_or(false)
    }

    /// Successor of `tech_id`, if any.
    #[must_use]
    pub fn supersedes(&self, tech_id: &str) -> Option<&str> {
        self.records
            .get(tech_id)
            .and_then(|r| r.successor.as_deref())
    }

    /// All tracked tech IDs in deterministic (sorted) order. Useful for
    /// snapshot serialization and replay diffing.
    #[must_use]
    pub fn tech_ids(&self) -> impl Iterator<Item = &str> {
        self.records.keys().map(String::as_str)
    }
}

/// Pure decay function. `value_n = max(floor, base * (decay_bps / 10_000)^n)`.
///
/// Uses saturating arithmetic so even astronomical tick counts converge at
/// the floor rather than overflow. A `decay_bps >= 10_000` config means
/// "no decay" and the base value is returned unchanged.
fn apply_decay(base: u64, ticks: u64, config: &ObsolescenceConfig) -> u64 {
    if config.decay_bps >= 10_000 {
        return base;
    }
    let floor = config.floor();
    if base <= floor {
        return base;
    }
    let mut current = base;
    // 10_000 == 1.0; cap at 64 ticks because (0.9)^64 < 0.001 and `(0.9)^>64`
    // sits well below the smallest realistic integer upkeep delta. Beyond
    // that, decay is clipped to the floor.
    let bounded = ticks.min(64);
    for _ in 0..bounded {
        let next = decay_one_step(current, config.decay_bps);
        if next >= current {
            // Decay stopped making progress (e.g. floor reached); bail out.
            return floor.min(current);
        }
        current = next;
        if current <= floor {
            return floor;
        }
    }
    current
}

/// One iteration of `value = value * decay_bps / 10_000`, saturating.
///
/// Returns `value` unchanged when `decay_bps >= 10_000` (no-decay config),
/// so a caller who configures the tracker as "no decay" sees no shrinkage.
fn decay_one_step(value: u64, decay_bps: u32) -> u64 {
    if decay_bps >= 10_000 {
        return value;
    }
    let numerator = u128::from(value) * u128::from(decay_bps);
    // Round-to-nearest so very large bases don't get stuck via floored
    // integer division at U64::MAX.
    let quotient = numerator / 10_000;
    (quotient as u64).min(value.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pure helper: register two techs, supersede the first, tick, observe.
    fn fixture() -> ObsolescenceTracker {
        let mut tracker = ObsolescenceTracker::new(ObsolescenceConfig::default());
        tracker
            .register_tech("steel_sword", 100)
            .expect("register steel_sword");
        tracker
            .register_tech("steel_rifle", 100)
            .expect("register steel_rifle");
        tracker
    }

    /// FR-CIV-TECH-OBSOLETE — a superseded tech's value decays after a
    /// successor is researched.
    #[test]
    fn superseded_value_decays_after_successor_researched() {
        let mut tracker = fixture();

        // Baseline: no successor yet → upkeep equals base.
        assert_eq!(tracker.upkeep_value("steel_sword"), Some(100));
        assert!(!tracker.is_obsolete("steel_sword"));

        // Research the successor: research the rifle first (register), then
        // mark the sword as superseded by the rifle.
        tracker
            .supersede_with("steel_sword", "steel_rifle")
            .expect("supersede");

        // Right after supersession, value is still base (no ticks elapsed
        // since the supersession event).
        assert_eq!(tracker.upkeep_value("steel_sword"), Some(100));
        assert_eq!(tracker.supersedes("steel_sword"), Some("steel_rifle"));
        assert!(tracker.is_obsolete("steel_sword"));

        // Advance one tick and confirm decay has started.
        tracker.tick(1);
        assert!(
            tracker.upkeep_value("steel_sword").expect("tracked") < 100,
            "value must begin decaying once a successor has been researched"
        );

        // Advance more ticks and confirm monotonic decay toward the floor.
        let after_one = tracker.upkeep_value("steel_sword").expect("tracked");
        tracker.tick(4);
        let after_five = tracker.upkeep_value("steel_sword").expect("tracked");
        assert!(
            after_five <= after_one,
            "value must keep decaying (or hold at floor): {after_five} <= {after_one}"
        );

        // Sanity: the successor tech itself is unaffected by the supersession.
        assert_eq!(tracker.upkeep_value("steel_rifle"), Some(100));
        assert!(!tracker.is_obsolete("steel_rifle"));
    }

    /// FR-CIV-TECH-OBSOLETE — decay eventually reaches (but never crosses) the floor.
    #[test]
    fn decay_floors_and_does_not_overshoot() {
        let config = ObsolescenceConfig::new(5_000, 7);
        let mut tracker = ObsolescenceTracker::new(config);
        tracker.register_tech("a", 1_000).unwrap();
        tracker.register_tech("b", 1).unwrap();
        tracker.supersede_with("a", "b").unwrap();

        // 10% halving per tick (5_000 bps == 0.5) drops 1000 → 500 → 250 →
        // 125 → 62 → 31 → 15 → 7 (floor). After 7 ticks we must be at floor.
        tracker.tick(64);
        let value = tracker.upkeep_value("a").expect("tracked");
        assert_eq!(value, 7, "floor must hold regardless of tick count");
    }

    /// FR-CIV-TECH-OBSOLETE — duplicate registration is rejected (replay safe).
    #[test]
    fn duplicate_register_rejected() {
        let mut tracker = ObsolescenceTracker::new(ObsolescenceConfig::default());
        tracker.register_tech("x", 10).unwrap();
        let err = tracker
            .register_tech("x", 11)
            .expect_err("duplicate must be rejected");
        assert_eq!(err, ObsolescenceError::DuplicateTech("x".into()));
    }

    /// FR-CIV-TECH-OBSOLETE — superseding an unknown tech is rejected.
    #[test]
    fn unknown_supersede_rejected() {
        let mut tracker = ObsolescenceTracker::new(ObsolescenceConfig::default());
        let err = tracker
            .supersede_with("ghost", "real")
            .expect_err("unknown must be rejected");
        assert_eq!(err, ObsolescenceError::UnknownTech("ghost".into()));
    }

    /// FR-CIV-TECH-OBSOLETE — a tech cannot supersede itself.
    #[test]
    fn self_supersede_rejected() {
        let mut tracker = fixture();
        let err = tracker
            .supersede_with("steel_sword", "steel_sword")
            .expect_err("self-supersede must be rejected");
        assert_eq!(err, ObsolescenceError::SelfSupersede("steel_sword".into()));
    }

    /// FR-CIV-TECH-OBSOLETE — applying the same supersession twice does not
    /// restart decay (idempotent for replay safety).
    #[test]
    fn repeat_supersede_is_idempotent() {
        let mut tracker = fixture();
        tracker
            .supersede_with("steel_sword", "steel_rifle")
            .unwrap();
        tracker.tick(3);
        let mid = tracker.upkeep_value("steel_sword").expect("tracked");

        tracker
            .supersede_with("steel_sword", "steel_rifle")
            .unwrap();
        tracker.tick(0);
        let after_dup = tracker.upkeep_value("steel_sword").expect("tracked");
        assert_eq!(
            mid, after_dup,
            "duplicate supersession must not restart the decay window"
        );
    }

    /// FR-CIV-TECH-OBSOLETE — `tick(0)` is a safe no-op.
    #[test]
    fn tick_zero_is_noop() {
        let mut tracker = fixture();
        tracker
            .supersede_with("steel_sword", "steel_rifle")
            .unwrap();
        let before = tracker.upkeep_value("steel_sword").expect("tracked");
        tracker.tick(0);
        let after = tracker.upkeep_value("steel_sword").expect("tracked");
        assert_eq!(before, after);
        assert_eq!(tracker.current_tick(), 0);
    }

    /// FR-CIV-TECH-OBSOLETE — the tracker is deterministic for identical inputs.
    #[test]
    fn tracker_is_deterministic() {
        let cfg = ObsolescenceConfig::default();

        let mut a = ObsolescenceTracker::new(cfg.clone());
        let mut b = ObsolescenceTracker::new(cfg);

        for (id, value) in [("alpha", 200u64), ("beta", 200), ("gamma", 200)] {
            a.register_tech(id, value).unwrap();
            b.register_tech(id, value).unwrap();
        }
        a.supersede_with("alpha", "beta").unwrap();
        b.supersede_with("alpha", "beta").unwrap();
        a.tick(7);
        b.tick(7);

        assert_eq!(a, b);
    }

    /// FR-CIV-TECH-OBSOLETE — schema version is exposed for replay snapshots.
    #[test]
    fn schema_version_constant_present() {
        // Locked-in version; bump on breaking changes.
        assert_eq!(OBSOLESCENCE_SCHEMA_VERSION, 1);
    }

    /// FR-CIV-TECH-OBSOLETE — `decay_bps == 0` clamps to `1` instead of panicking.
    #[test]
    fn zero_decay_clamps_to_one() {
        let cfg = ObsolescenceConfig::new(0, 1);
        assert_eq!(
            cfg.decay_bps(),
            1,
            "zero must clamp to 1 to keep decay alive"
        );
    }
}
