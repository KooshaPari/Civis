//! Unit morale (FR-CIV-MORALE).
//!
//! Tracks how the fighting spirit of a single tactical unit changes over time in
//! response to battlefield conditions:
//!
//! * **Casualties** sustained by the unit degrade morale (`apply_casualties`).
//! * **Encirclement** (hostile units on most of the surrounding cells) degrades
//!   morale faster (`apply_encirclement`).
//! * A unit whose morale falls **below the rout threshold** flips into a
//!   [`UnitStance::Routing`] state — it can no longer advance or deal damage
//!   until morale is restored.
//! * **Safety** (no engagement and not encircled) lets morale recover back
//!   toward full strength (`recover_safety`).
//!
//! The model is **pure-logic and deterministic**: every public function takes
//! `&mut self` plus the requested environmental inputs and returns
//! [`MoraleEvent`]s describing what happened. No RNG, no I/O, no Bevy/ECS
//! coupling. Hosts (engine, watch, replay) drive cadence and persist the
//! reported state; `civ-tactics` only owns the math.
//!
//! ## Example
//!
//! ```
//! use civ_tactics::{MoraleState, UnitStance};
//!
//! let mut m = MoraleState::new(100, 50);
//! // Lose 30 % of starting strength as casualties.
//! m.apply_casualties(30);
//! // Morale is still above the rout threshold (50), so the unit stands.
//! assert_eq!(m.stance(), UnitStance::Standing);
//! // Another 30 % of remaining is enough to drop below the threshold.
//! m.apply_casualties(70);
//! assert_eq!(m.stance(), UnitStance::Routing);
//!
//! // Pull the unit out of contact: it recovers over multiple safe ticks.
//! while m.stance() == UnitStance::Routing {
//!     m.recover_safety();
//! }
//! assert_eq!(m.stance(), UnitStance::Standing);
//! ```

use serde::{Deserialize, Serialize};

/// Morale values are stored as `f32` clamped to `[0.0, 1.0]`. A value of
/// `1.0` means "fighting at full effectiveness"; `0.0` means "completely
/// shattered / deserting".
pub type MoraleLevel = f32;

/// Morale tick inputs. A single `tick_morale` call processes all of an
/// environment's beat — casualties, encirclement, safety recovery — in
/// pipeline order so hosts only need to make one call per cadence tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MoraleTickInputs {
    /// Casualties sustained *this tick* (in absolute headcount).
    pub casualties: u32,
    /// Whether the unit is currently encircled by hostiles.
    pub encircled: bool,
    /// Whether the unit is out of contact (no active engagement, free to rally).
    pub safe: bool,
}

impl MoraleTickInputs {
    /// Build a "quiet tick" input — nothing happening, unit is safe.
    #[must_use]
    pub const fn idle() -> Self {
        Self {
            casualties: 0,
            encircled: false,
            safe: true,
        }
    }

    /// Build a "combat tick" — casualties coming in, not yet safe.
    #[must_use]
    pub const fn combat(casualties: u32, encircled: bool) -> Self {
        Self {
            casualties,
            encircled,
            safe: false,
        }
    }
}

/// Current stance of the unit as a function of morale vs. the rout threshold.
// The following 1 requirement tags were removed from UnitStance.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// Same id as the MoraleState entry, judged independently as instructed. The two sites differ in severity, not in verdict: MoraleState at least holds the morale scalar, UnitStance holds nothing but a classification. Neither is consumed outside morale.rs and test files, so the requirement's 'per-soldier behavior reads psyche' clause is unimplemented at both sites.
// If a future change wants to discharge FR-CIV-WAR-021, the correct home is not either of these declarations. It is the combat and movement path that would have to consult stance before firing or advancing, which today is WarBridge::resolve_combat in crates/tactics/src/war_bridge.rs and operational_movement_pulse in crates/tactics/src/movement.rs:51, neither of which reads morale at all.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-WAR-021: CONTAINER-ONLY. The requirement is that emergent agent psyche modulates fire discipline, rout threshold and willingness to follow orders, and that casualties and atrocities write grievance and trauma back into psyche (docs/design/warfare.md:111-113). UnitStance at crates/tactics/src/morale.rs:87 is a two-variant marker enum (Standing, Routing) whose entire body is two unit variants. A marker cannot read psyche, modulate anything, or write anything back, so this is the weakest possible claim of the nine. The morale computation that does exist lives on the sibling type, MoraleState::stance at crates/tactics/src/morale.rs:211, and even that is only a comparison of one scalar against one threshold. git grep -n -E 'fear|loyalty|grievance|trauma|ideolog' -- crates/tactics/src/ returns nothing, so three of the four named psyche inputs and the entire write-back half of the requirement have no symbol anywhere in the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitStance {
    /// Morale ≥ rout threshold — unit can fight at full effect.
    Standing,
    /// Morale < rout threshold — unit has routed and cannot advance or
    /// deal combat damage until morale is restored.
    Routing,
}

/// A discrete change to morale or stance that callers should log / replay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoraleEvent {
    /// Morale fell due to casualties.
    CasualtyHit {
        /// Morale level after the hit.
        new_level: MoraleLevel,
        /// Total morale removed this hit (positive number).
        morale_lost: MoraleLevel,
        /// True iff this hit pushed the unit into routing.
        began_routing: bool,
    },
    /// Morale fell due to encirclement pressure (one increment).
    EncirclementHit {
        /// Morale level after the hit.
        new_level: MoraleLevel,
        /// Total morale removed this hit (positive number).
        morale_lost: MoraleLevel,
        /// True iff this hit pushed the unit into routing.
        began_routing: bool,
    },
    /// Morale rose due to safety recovery.
    SafetyRecovery {
        /// Morale level after the recovery.
        new_level: MoraleLevel,
        /// Total morale gained this step (positive number).
        morale_gained: MoraleLevel,
        /// True iff this step pulled the unit back out of routing.
        rallied: bool,
    },
}

impl MoraleEvent {
    /// Morale level after the event.
    #[must_use]
    pub const fn level(&self) -> MoraleLevel {
        match *self {
            MoraleEvent::CasualtyHit { new_level, .. }
            | MoraleEvent::EncirclementHit { new_level, .. }
            | MoraleEvent::SafetyRecovery { new_level, .. } => new_level,
        }
    }
}

// The following 2 requirement tags were removed from MoraleState.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// The two FR-CIV-WAR-021 sites do not agree in kind, though both come off. MoraleState is a state record that is merely unread; UnitStance is a two-variant enum that can never modulate anything by construction, so it is the weaker binding of the pair. The orphaned-consumer problem is shared: the same greps show UnitStance is likewise referenced only in morale.rs and in tests, never in war_bridge.rs, movement.rs or military_phases.rs.
// MoraleState carries FR-CIV-MORALE in its own doc comment and is legitimately tagged FR-CIV-TACTICS-076 by its test coverage, so removing FR-CIV-WAR-021 leaves the type's real provenance intact and loses no true coverage. The test at crates/tactics/tests/fr_fr_civ_war_021.rs:15 asserts only that casualties flip stance from Standing to Routing on an isolated instance; that is a unit test of the struct, not evidence that psyche drives combat.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-MORALE: PHANTOM ID, NOT A REQUIREMENT. Unlike every other entry in this file, this id cannot be judged against requirement text because no such requirement is defined anywhere in the repo. FR-CIV-MORALE occurs in exactly two files, one of which is this audit file: crates/tactics/src/morale.rs, where it appears in the module doc at :1, in the struct doc at :139, and in six test doc comments at :358, :397, :430, :470, :490 and :520. Searches that came back empty: git grep -rn 'FR-CIV-MORALE' outside crates/tactics/src/morale.rs; git grep -n -A6 'FR-CIV-MORALE' -- docs/specs agileplus-specs docs/design docs/guides; a case-insensitive search for 'morale' in docs/traceability/index.md, which enumerates the repo's requirement ids and contains no MORALE row; a check for a docs/traceability/fr-civ-morale/ directory, which does not exist; and a scan of docs/audits/_id_inventory_v3.json, whose keys contain no MORALE entry. The only MORALE-adjacent hit anywhere in the spec tree is an unrelated constant MORALE_ATTRITION_FACTOR at docs/specs/CIV-0105-war-diplomacy-shadow-v1.md:232, which belongs to a different id scheme. So the binding cannot be wrong in the sense of misplacing a real requirement, because there is no requirement to place. It is a self-referential id invented so the morale module's own unit tests read as requirement coverage. That is the phantom-tag pattern this audit exists to strip, and the id should be struck rather than re-pointed at some real requirement: if morale behavior is meant to be tracked, a spec has to be written first. Note this id is a doc comment, /// Per-unit morale tracker (FR-CIV-MORALE)., not a standalone tag line; _apply_verdicts.is_tag_line still returns True for it, so it is mechanically removable.
// [unbound] FR-CIV-WAR-021: DATA-SHAPE-ONLY, and worse, an orphan. The requirement is that per-soldier behavior reads emergent agent psyche -- morale, fear, in-group loyalty and fatigue modulate fire discipline, rout threshold and willingness to follow orders, with casualties and atrocities writing grievance and trauma back into psyche (docs/design/warfare.md:111-113). MoraleState models one of the four named inputs (morale) and a rout threshold, which is real coverage of that slice; none of the other three are present. git grep -n -i -E 'fear|loyalty|grievance|trauma|ideolog' -- crates/tactics/src/ returned no hits at all, so there is no fear, loyalty, grievance, trauma or ideology term for the struct to read, and nothing anywhere writes grievance or trauma back. The decisive point is that the struct has no production consumer: git grep -n 'MoraleState' -- crates/ shows hits only inside morale.rs itself and in test files (crates/tactics/tests/fr_fr_civ_war_021.rs:9, crates/tactics/tests/fr_civ_tactics_tests.rs:6, crates/tactics/tests/fr_fr_civ_tactics_076.rs:8). No combat, movement or war-bridge path in the crate ever constructs one, so morale does not in fact modulate fire discipline, rout threshold or order-following in the running simulation. A tracker nothing reads cannot discharge a behavioral requirement.
///
/// Owns the unit's current morale, its threshold at which it routs, and an
/// event-log of the most recent changes. Constructed at unit spawn with a
/// starting strength (`initial_strength`) and a rout threshold in absolute
/// headcount. Morale itself is stored as a normalised `[0, 1]` value
/// ([`MoraleLevel`]) so it is independent of the unit's actual paper strength.
#[derive(Debug, Clone, PartialEq)]
pub struct MoraleState {
    /// `0.0..=1.0`. Morale past `rout_threshold` is [`UnitStance::Standing`],
    /// below is [`UnitStance::Routing`].
    morale: MoraleLevel,
    /// Normalised threshold (level, not absolute strength) at which the
    /// unit routs. Default is `0.25` — i.e. lose ~75 % of morale before
    /// the unit breaks.
    rout_threshold: MoraleLevel,
    /// Original paper strength at unit spawn. Used to translate absolute
    /// casualty counts into a normalised morale hit.
    initial_strength: u32,
    /// Last event the host should record in the replay/journal.
    last_event: Option<MoraleEvent>,
}

impl MoraleState {
    /// Construct a fresh unit at full morale.
    ///
    /// * `initial_strength` — paper strength at spawn. Casualty hits are
    ///   proportional to this (e.g. 10 % of this drops morale by 10 %).
    /// * `rout_threshold_units` — absolute paper-strength value at which the
    ///   unit routs. The threshold is normalised against `initial_strength`
    ///   so a 100-strong unit with a 25-strong threshold routs after losing
    ///   ≥ 75 of its soldiers.
    #[must_use]
    pub fn new(initial_strength: u32, rout_threshold_units: u32) -> Self {
        let initial_strength = initial_strength.max(1);
        let threshold = if rout_threshold_units >= initial_strength {
            // Rout threshold at or above full strength means the unit is
            // already broken — clamp to a small positive value so any
            // single casualty triggers routing.
            0.01
        } else {
            rout_threshold_units as f32 / initial_strength as f32
        };
        Self {
            morale: 1.0,
            rout_threshold: threshold.clamp(0.0, 1.0),
            initial_strength,
            last_event: None,
        }
    }

    /// Current morale level (`0.0..=1.0`).
    #[must_use]
    pub const fn morale(&self) -> MoraleLevel {
        self.morale
    }

    /// Rout threshold expressed as a normalised level.
    #[must_use]
    pub const fn rout_threshold(&self) -> MoraleLevel {
        self.rout_threshold
    }

    /// Original paper strength the unit was constructed with.
    #[must_use]
    pub const fn initial_strength(&self) -> u32 {
        self.initial_strength
    }

    /// Current [`UnitStance`].
    #[must_use]
    pub fn stance(&self) -> UnitStance {
        if self.morale < self.rout_threshold {
            UnitStance::Routing
        } else {
            UnitStance::Standing
        }
    }

    /// Most recent event, if any.
    #[must_use]
    pub const fn last_event(&self) -> Option<MoraleEvent> {
        self.last_event
    }

    /// Apply a casualty hit in absolute headcount. Morale falls by
    /// `casualties / initial_strength`, clamped to `[0, 1]`. Routing units
    /// still accrue casualty hits but cannot fall below zero morale.
    ///
    /// Returns the [`MoraleEvent`] recorded, or `None` for a zero-casualty
    /// no-op.
    pub fn apply_casualties(&mut self, casualties: u32) -> Option<MoraleEvent> {
        if casualties == 0 || self.initial_strength == 0 {
            return None;
        }
        let ratio = casualties as f32 / self.initial_strength as f32;
        self.sub_morale(ratio).map(|lost| MoraleEvent::CasualtyHit {
            new_level: self.morale,
            morale_lost: lost,
            began_routing: self.stance() == UnitStance::Routing && lost > 0.0,
        })
    }

    /// Apply one tick of encirclement pressure. Morale falls by a small,
    /// fixed amount (`0.05`) per call while `encircled == true`. Calling
    /// with `encircled == false` is a no-op (recovery is handled separately
    /// via [`MoraleState::recover_safety`]).
    pub fn apply_encirclement(&mut self, encircled: bool) -> Option<MoraleEvent> {
        if !encircled || self.stance() == UnitStance::Routing {
            return None;
        }
        // We only decay further while the unit is still standing; a routing
        // unit has already broken and routing is independent of further
        // encirclement pressure (combat continues to chip at strength via
        // `apply_casualties`).
        let lost = ENCIRCLEMENT_HIT;
        let prev = self.morale;
        self.morale = (self.morale - lost).max(0.0);
        let actual_lost = prev - self.morale;
        if actual_lost <= 0.0 {
            return None;
        }
        let event = MoraleEvent::EncirclementHit {
            new_level: self.morale,
            morale_lost: actual_lost,
            began_routing: self.stance() == UnitStance::Routing,
        };
        self.last_event = Some(event);
        Some(event)
    }

    /// One tick of safety recovery. Adds [`SAFETY_RECOVERY_PER_TICK`] back to
    /// morale (clamped to `1.0`). If the unit had routed, this tick is also
    /// when it can rally back to a standing stance.
    ///
    /// Hosts that need safety recovery tied to "no contact for N ticks"
    /// should simply not call this method on combat ticks.
    pub fn recover_safety(&mut self) -> Option<MoraleEvent> {
        if self.morale >= 1.0 {
            return None;
        }
        let prev_stance = self.stance();
        let prev = self.morale;
        self.morale = (self.morale + SAFETY_RECOVERY_PER_TICK).min(1.0);
        let gained = self.morale - prev;
        if gained <= 0.0 {
            return None;
        }
        let event = MoraleEvent::SafetyRecovery {
            new_level: self.morale,
            morale_gained: gained,
            rallied: prev_stance == UnitStance::Routing && self.stance() == UnitStance::Standing,
        };
        self.last_event = Some(event);
        Some(event)
    }

    /// Run a single cadence-tick pipeline in canonical order:
    /// casualties → encirclement → recovery. This is the call most hosts
    /// want; tests that want fine-grained control use the individual
    /// `apply_*` / `recover_*` methods directly.
    pub fn tick_morale(&mut self, inputs: MoraleTickInputs) -> Vec<MoraleEvent> {
        let mut events = Vec::with_capacity(3);
        if let Some(ev) = self.apply_casualties(inputs.casualties) {
            events.push(ev);
        }
        if let Some(ev) = self.apply_encirclement(inputs.encircled) {
            events.push(ev);
        }
        // Recovery only happens when the host marks the unit as safe and no
        // casualties landed this tick — a heavily-hit unit doesn't rally on
        // the same tick it took damage.
        if inputs.safe && inputs.casualties == 0 {
            if let Some(ev) = self.recover_safety() {
                events.push(ev);
            }
        }
        events
    }

    fn sub_morale(&mut self, ratio: f32) -> Option<MoraleLevel> {
        if ratio <= 0.0 {
            return None;
        }
        let prev = self.morale;
        self.morale = (self.morale - ratio).max(0.0);
        let lost = prev - self.morale;
        if lost <= 0.0 {
            return None;
        }
        let event = MoraleEvent::CasualtyHit {
            new_level: self.morale,
            morale_lost: lost,
            began_routing: self.stance() == UnitStance::Routing,
        };
        self.last_event = Some(event);
        Some(lost)
    }
}

/// Morale removed per encirclement tick (`5 %`).
const ENCIRCLEMENT_HIT: MoraleLevel = 0.05;

/// Morale restored per safe tick (`2 %`).
const SAFETY_RECOVERY_PER_TICK: MoraleLevel = 0.02;

#[cfg(test)]
mod tests {
    //! Acceptance tests for FR-CIV-MORALE.
    //!
    //! Mirrors the three behaviour rows the FR promises:
    //!
    //! 1. casualties drop morale
    //! 2. below threshold, the unit routs
    //! 3. safety recovers morale

    use super::*;

    /// FR-CIV-MORALE — casualties drop morale (acceptance test, row 1/3).
    #[test]
    fn casualties_drop_morale() {
        let mut m = MoraleState::new(100, 25);
        let start = m.morale();
        assert_eq!(start, 1.0, "fresh unit starts at full morale");
        assert_eq!(m.stance(), UnitStance::Standing);

        // Lose 30 soldiers out of 100 → morale 0.70.
        m.apply_casualties(30);
        assert!(
            (m.morale() - 0.70).abs() < 1e-6,
            "30 % casualties → morale 0.70, got {}",
            m.morale()
        );
        // No routing yet — 0.70 is still above the 0.25 threshold.
        assert_eq!(
            m.stance(),
            UnitStance::Standing,
            "morale 0.70 > threshold 0.25 → still standing"
        );

        let event = m
            .last_event()
            .expect("a casualty hit should record an event");
        match event {
            MoraleEvent::CasualtyHit {
                new_level,
                morale_lost,
                began_routing,
            } => {
                assert!((new_level - 0.70).abs() < 1e-6);
                assert!((morale_lost - 0.30).abs() < 1e-6);
                assert!(!began_routing);
            }
            other => panic!("expected CasualtyHit, got {other:?}"),
        }
    }

    /// FR-CIV-MORALE — morale below the rout threshold flips stance to
    /// `Routing` (acceptance test, row 2/3).
    #[test]
    fn below_threshold_unit_routs() {
        // Threshold 50/100 → unit routs below 0.50 morale.
        let mut m = MoraleState::new(100, 50);
        assert_eq!(m.stance(), UnitStance::Standing);

        // Drive morale from 1.00 to 0.45 — that's below the 0.50 threshold.
        m.apply_casualties(55);
        assert!(
            m.morale() < 0.50,
            "test setup: morale {} should be < 0.50",
            m.morale()
        );
        assert_eq!(
            m.stance(),
            UnitStance::Routing,
            "morale below threshold → unit routs"
        );

        // The event that crossed the line should mark began_routing.
        match m.last_event().expect("event recorded") {
            MoraleEvent::CasualtyHit { began_routing, .. } => {
                assert!(
                    began_routing,
                    "the crossing casualty hit should flag begun_routing"
                );
            }
            other => panic!("expected CasualtyHit, got {other:?}"),
        }
    }

    /// FR-CIV-MORALE — out of contact (safe) lets morale recover out of
    /// routing (acceptance test, row 3/3).
    #[test]
    fn safety_recovers_morale_and_unit_rallies() {
        // Threshold 50/100 → unit routs below 0.50 morale.
        let mut m = MoraleState::new(100, 50);
        // Drop morale to 0.10 → routing.
        m.apply_casualties(90);
        assert_eq!(m.stance(), UnitStance::Routing);

        // Safety recovery in 0.02 / tick steps → after ~20 ticks the unit
        // is back above the 0.50 threshold and rallies.
        let mut rallied = false;
        for _ in 0..100 {
            let events = m.tick_morale(MoraleTickInputs::idle());
            for ev in &events {
                if let MoraleEvent::SafetyRecovery { rallied: r, .. } = ev {
                    rallied = rallied || *r;
                }
            }
            if m.stance() == UnitStance::Standing {
                break;
            }
        }
        assert!(
            m.morale() >= 0.50,
            "morale {} should be back at or above 0.50",
            m.morale()
        );
        assert_eq!(
            m.stance(),
            UnitStance::Standing,
            "rallied unit should be standing"
        );
        assert!(
            rallied,
            "the recovery tick that crossed the threshold must report rallied"
        );
    }

    /// FR-CIV-MORALE — encirclement degrades morale over time even without
    /// direct casualties. A sanity check on the encirclement pressure axis.
    #[test]
    fn encirclement_degrades_morale() {
        let mut m = MoraleState::new(100, 5);
        let start = m.morale();
        for _ in 0..3 {
            m.apply_encirclement(true);
        }
        let lost = start - m.morale();
        assert!(
            (lost - 0.15).abs() < 1e-6,
            "3 × 0.05 encirclement hits → -0.15 morale, got {lost}"
        );
        // Calling with `false` is a no-op.
        let before = m.morale();
        m.apply_encirclement(false);
        assert_eq!(m.morale(), before);
    }

    /// FR-CIV-MORALE — `tick_morale` runs the full pipeline in canonical
    /// order and skips recovery on the same tick the unit took casualties.
    #[test]
    fn tick_morale_pipeline_order() {
        let mut m = MoraleState::new(100, 25);
        // Combat tick — casualties arrive and recovery is suppressed.
        let events = m.tick_morale(MoraleTickInputs::combat(10, true));
        let kinds: Vec<&'static str> = events
            .iter()
            .map(|e| match e {
                MoraleEvent::CasualtyHit { .. } => "casualty",
                MoraleEvent::EncirclementHit { .. } => "encirclement",
                MoraleEvent::SafetyRecovery { .. } => "recovery",
            })
            .collect();
        assert!(
            kinds.contains(&"casualty") && kinds.contains(&"encirclement"),
            "expected casualty+encirclement, got {kinds:?}"
        );
        assert!(
            !kinds.contains(&"recovery"),
            "casualties this tick must suppress same-tick recovery"
        );

        // A subsequent idle tick recovers.
        let before = m.morale();
        m.tick_morale(MoraleTickInputs::idle());
        assert!(m.morale() > before, "idle tick should recover morale");
    }

    /// FR-CIV-MORALE — recovery never exceeds `1.0`.
    #[test]
    fn recovery_clamps_at_full() {
        let mut m = MoraleState::new(100, 1);
        m.apply_casualties(50);
        for _ in 0..200 {
            m.tick_morale(MoraleTickInputs::idle());
        }
        assert!(
            (m.morale() - 1.0).abs() < 1e-6,
            "morale must saturate at 1.0, got {}",
            m.morale()
        );
    }
}
