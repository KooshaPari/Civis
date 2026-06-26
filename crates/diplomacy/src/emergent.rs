//! Emergent diplomacy (FR-CIV-DIPLOMACY).
//!
//! Where [`crate::DiplomacyState`] is the *substrate* (a pairwise scalar
//! standing graph bumped by interaction events and decayed each tick), this
//! module is the **emergent layer**: faction stance is *derived* from history
//! rather than scripted, treaties form and break from relation thresholds, and
//! betrayals propagate a reputation penalty across the whole faction set.
//!
//! Nothing here is hard-coded per-faction. Stance falls out of four observable
//! signals, each of which is itself produced by other sim systems:
//!
//! 1. **Shared enemies** — two factions that are both [`EmergentStance::Rival`]
//!    toward the same third faction drift toward each other ("the enemy of my
//!    enemy"). Sourced from the substrate standing graph.
//! 2. **Border friction** — territory overlap / proximity erodes relations
//!    (contested frontier). Sourced from [`Territory`] adjacency.
//! 3. **Trade interdependence** — mutual economic ties warm relations. Sourced
//!    from the economy crate's bilateral trade volume.
//! 4. **Belief / culture similarity** — close culture vectors warm relations,
//!    distant ones cool them. Sourced from agents/culture centroids.
//!
//! # Determinism
//!
//! Everything is integer or fixed-iteration float math over [`BTreeMap`]s and
//! sorted vectors. No `HashMap` iteration, no RNG, no wall-clock. Given the
//! same [`StanceInputs`] in the same order, two runs produce identical
//! relation scores, treaty ledgers, reputation maps, and emitted legend
//! events. The reputation/treaty thresholds are pure functions of the score.

use crate::{Pair, PolityId};
use civ_legends::ids::SourceCrate;
use civ_legends::model::{EventKind, RawSimEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Coarse emergent stance between two factions, derived from a continuous
/// relation score (see [`EmergentStance::from_score`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmergentStance {
    /// Score `<= rival_threshold`. Mutually hostile.
    Rival,
    /// Between the thresholds. No standing relationship.
    Neutral,
    /// Score `>= ally_threshold`. Cooperative; eligible for a treaty.
    Ally,
}

impl EmergentStance {
    /// Project a relation score in `[-1.0, 1.0]` to a coarse stance.
    pub fn from_score(score: f32, rival_threshold: f32, ally_threshold: f32) -> Self {
        if score <= rival_threshold {
            EmergentStance::Rival
        } else if score >= ally_threshold {
            EmergentStance::Ally
        } else {
            EmergentStance::Neutral
        }
    }
}

/// Per-faction territory footprint. Border friction emerges from overlap of
/// these cells; the diplomacy layer never reads the map directly.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Territory {
    /// Owner.
    pub owner: PolityId,
    /// Coarse occupied cell ids (e.g. region/sector keys). Sorted set keeps the
    /// overlap computation deterministic.
    pub cells: BTreeSet<u64>,
}

impl Territory {
    /// New empty footprint for `owner`.
    pub fn new(owner: PolityId) -> Self {
        Self {
            owner,
            cells: BTreeSet::new(),
        }
    }

    /// Builder: occupy `cells`.
    pub fn with_cells<I: IntoIterator<Item = u64>>(mut self, cells: I) -> Self {
        self.cells.extend(cells);
        self
    }

    /// Number of shared cells with `other` (the border-friction surface).
    pub fn overlap(&self, other: &Territory) -> usize {
        self.cells.intersection(&other.cells).count()
    }
}

/// The four history-derived signals that produce a pairwise relation score.
/// Each is independently sourced; the diplomacy layer only weighs and combines
/// them. All are already normalized to `[-1.0, 1.0]` (or `[0, 1]` where a
/// signal is one-directional) by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationDrivers {
    /// `[0,1]` strength of "shared enemy" pull (both rival a common third).
    pub shared_enemy: f32,
    /// `[0,1]` border friction (territory overlap, contested frontier).
    pub border_friction: f32,
    /// `[0,1]` trade interdependence (bilateral volume, normalized).
    pub trade: f32,
    /// `[-1,1]` culture/belief similarity (`+1` identical, `-1` opposed).
    pub culture_similarity: f32,
}

impl RelationDrivers {
    /// Combine the four drivers into a single relation score in `[-1.0, 1.0]`.
    ///
    /// Weights are deliberately simple and fixed so the mapping is auditable:
    /// shared enemies and trade *warm*, border friction *cools*, culture
    /// pushes either way. The weighted sum is clamped to the unit range.
    pub fn score(&self) -> f32 {
        let raw = 0.35 * self.shared_enemy + 0.35 * self.trade
            - 0.40 * self.border_friction
            + 0.30 * self.culture_similarity;
        raw.clamp(-1.0, 1.0)
    }
}

/// All inputs needed to recompute emergent stance for one tick. The caller
/// (engine) assembles these from the substrate standing graph, territory,
/// economy, and culture systems.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StanceInputs {
    /// Per-pair drivers. Only pairs present here are (re)scored this tick.
    pub drivers: BTreeMap<Pair, RelationDrivers>,
    /// Current simulation tick (stamped onto emitted events).
    pub tick: u64,
}

impl StanceInputs {
    /// New empty input set for `tick`.
    pub fn new(tick: u64) -> Self {
        Self {
            drivers: BTreeMap::new(),
            tick,
        }
    }

    /// Insert/replace the drivers for `(a, b)`.
    pub fn set(&mut self, a: PolityId, b: PolityId, drivers: RelationDrivers) {
        self.drivers.insert(Pair::new(a, b), drivers);
    }
}

/// Treaty type. Both are emergent: an alliance forms when standing is high and
/// a non-aggression pact is the lighter-weight cooperative bond. The kind
/// recorded in the ledger lets downstream systems weight obligations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TreatyKind {
    /// Mutual-defense alliance (formed at the highest relation band).
    Alliance,
}

/// What happened to a pair's treaty status on a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatyOutcome {
    /// A new treaty formed (relation crossed up through the ally threshold).
    Formed(TreatyKind),
    /// An existing treaty broke because the relation collapsed (betrayal).
    Broken(TreatyKind),
}

/// One audit-trail entry for a treaty transition.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TreatyLedgerEntry {
    /// Tick the transition was observed.
    pub tick: u64,
    /// The pair involved.
    pub pair: Pair,
    /// Formation or break.
    pub outcome: TreatyOutcome,
    /// Relation score at the moment of transition.
    pub score: f32,
}

/// Reputation: a faction's trustworthiness as observed by *everyone else*. A
/// betrayal (breaking a treaty) lowers the betrayer's reputation globally, so
/// third parties become warier of forming their own treaties with it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reputation {
    /// Per-faction trust score in `[-1.0, 1.0]`; starts at 0 (unknown).
    scores: BTreeMap<PolityId, f32>,
}

impl Default for Reputation {
    fn default() -> Self {
        Self::new()
    }
}

impl Reputation {
    /// Empty reputation table (every faction implicitly at 0).
    pub fn new() -> Self {
        Self {
            scores: BTreeMap::new(),
        }
    }

    /// Current reputation for `who` (0 if never observed).
    pub fn get(&self, who: PolityId) -> f32 {
        self.scores.get(&who).copied().unwrap_or(0.0)
    }

    /// Apply a clamped delta to `who`'s reputation.
    fn adjust(&mut self, who: PolityId, delta: f32) {
        let e = self.scores.entry(who).or_insert(0.0);
        *e = (*e + delta).clamp(-1.0, 1.0);
    }
}

/// Tunables for the emergent layer. Defaults are chosen so the standard
/// `[-1,1]` score band gives a sensible Rival/Neutral/Ally split.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EmergentConfig {
    /// Score at/below which a pair is [`EmergentStance::Rival`].
    pub rival_threshold: f32,
    /// Score at/above which a pair is [`EmergentStance::Ally`] and forms a treaty.
    pub ally_threshold: f32,
    /// Score at/below which an existing treaty breaks (a betrayal). Strictly
    /// below `ally_threshold` to create hysteresis (a treaty does not flap on a
    /// one-tick dip just under the formation line).
    pub break_threshold: f32,
    /// Reputation penalty applied to a betrayer per betrayal.
    pub betrayal_penalty: f32,
}

impl Default for EmergentConfig {
    fn default() -> Self {
        Self {
            rival_threshold: -0.5,
            ally_threshold: 0.7,
            break_threshold: -0.5,
            betrayal_penalty: 0.5,
        }
    }
}

/// The emergent-diplomacy engine. Owns the derived relation scores, the active
/// treaty set, the reputation table, and the per-tick ledger of transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergentDiplomacy {
    config: EmergentConfig,
    /// Latest derived relation score per pair.
    scores: BTreeMap<Pair, f32>,
    /// Pairs that currently hold a treaty.
    treaties: BTreeMap<Pair, TreatyKind>,
    /// Global trust table.
    reputation: Reputation,
    /// Transition ledger for the most recent [`Self::update`] call.
    ledger: Vec<TreatyLedgerEntry>,
}

impl Default for EmergentDiplomacy {
    fn default() -> Self {
        Self::new(EmergentConfig::default())
    }
}

impl EmergentDiplomacy {
    /// New engine with `config`.
    pub fn new(config: EmergentConfig) -> Self {
        Self {
            config,
            scores: BTreeMap::new(),
            treaties: BTreeMap::new(),
            reputation: Reputation::new(),
            ledger: Vec::new(),
        }
    }

    /// Current relation score for `(a, b)`, if scored.
    pub fn score(&self, a: PolityId, b: PolityId) -> Option<f32> {
        self.scores.get(&Pair::new(a, b)).copied()
    }

    /// Current emergent stance for `(a, b)`.
    pub fn stance(&self, a: PolityId, b: PolityId) -> EmergentStance {
        let score = self.score(a, b).unwrap_or(0.0);
        EmergentStance::from_score(
            score,
            self.config.rival_threshold,
            self.config.ally_threshold,
        )
    }

    /// True if `(a, b)` currently hold a treaty.
    pub fn has_treaty(&self, a: PolityId, b: PolityId) -> bool {
        self.treaties.contains_key(&Pair::new(a, b))
    }

    /// Reputation of `who` as observed globally.
    pub fn reputation(&self, who: PolityId) -> f32 {
        self.reputation.get(who)
    }

    /// The treaty transitions recorded by the most recent [`Self::update`].
    pub fn ledger(&self) -> &[TreatyLedgerEntry] {
        &self.ledger
    }

    /// Derive the shared-enemy driver strength from the substrate: for the pair
    /// `(a, b)`, how many third factions does *each* of them have a hostile
    /// (negative) standing toward in common, relative to the number of common
    /// counterparties. Pure function over the supplied rival sets.
    ///
    /// `rivals_of` maps each faction to the set of factions it is hostile
    /// toward (caller derives this from [`crate::DiplomacyState`]). Returns a
    /// `[0,1]` strength: 1.0 = every common counterparty is a shared enemy.
    pub fn shared_enemy_strength(
        a: PolityId,
        b: PolityId,
        rivals_of: &BTreeMap<PolityId, BTreeSet<PolityId>>,
    ) -> f32 {
        let empty = BTreeSet::new();
        let ra = rivals_of.get(&a).unwrap_or(&empty);
        let rb = rivals_of.get(&b).unwrap_or(&empty);
        let shared = ra.intersection(rb).filter(|&&t| t != a && t != b).count();
        if shared == 0 {
            return 0.0;
        }
        // Saturating curve: one shared enemy already gives a strong pull; more
        // approach 1.0. Deterministic integer→float.
        let s = shared as f32;
        (s / (s + 1.0)).min(1.0)
    }

    /// Border-friction strength `[0,1]` from territory overlap. The more cells
    /// two factions contest, the higher the friction (saturating).
    pub fn border_friction_strength(a: &Territory, b: &Territory) -> f32 {
        let overlap = a.overlap(b) as f32;
        if overlap == 0.0 {
            return 0.0;
        }
        (overlap / (overlap + 2.0)).min(1.0)
    }

    /// Recompute stance for every pair in `inputs`, transition treaties across
    /// thresholds, propagate reputation on betrayal, and emit the corresponding
    /// [`RawSimEvent`]s for the legends bus. Clears and repopulates the ledger.
    ///
    /// Returns the emitted legend events (`Treaty` on formation, `Betrayal` on
    /// break) in deterministic pair order.
    pub fn update(&mut self, inputs: &StanceInputs) -> Vec<RawSimEvent> {
        self.ledger.clear();
        let mut events = Vec::new();
        // BTreeMap iteration is sorted by Pair → deterministic order.
        for (&pair, drivers) in &inputs.drivers {
            let score = drivers.score();
            self.scores.insert(pair, score);
            let had_treaty = self.treaties.contains_key(&pair);

            if !had_treaty && score >= self.config.ally_threshold {
                // Treaty forms.
                self.treaties.insert(pair, TreatyKind::Alliance);
                self.ledger.push(TreatyLedgerEntry {
                    tick: inputs.tick,
                    pair,
                    outcome: TreatyOutcome::Formed(TreatyKind::Alliance),
                    score,
                });
                events.push(treaty_event(inputs.tick, pair, EventKind::Treaty));
            } else if had_treaty && score <= self.config.break_threshold {
                // Treaty breaks — this is a betrayal. Both members observed it,
                // and reputation propagates globally below.
                self.treaties.remove(&pair);
                self.ledger.push(TreatyLedgerEntry {
                    tick: inputs.tick,
                    pair,
                    outcome: TreatyOutcome::Broken(TreatyKind::Alliance),
                    score,
                });
                events.push(treaty_event(inputs.tick, pair, EventKind::Betrayal));
                // Reputation penalty: a broken alliance damages *both* members'
                // standing globally (a collapsed alliance signals untrust to
                // every third party). Determinism: applied to lo then hi.
                self.reputation
                    .adjust(pair.lo, -self.config.betrayal_penalty);
                self.reputation
                    .adjust(pair.hi, -self.config.betrayal_penalty);
            }
        }
        events
    }

    /// Explicitly record a unilateral betrayal by `betrayer` against `victim`
    /// (e.g. a surprise attack while a treaty was active), independent of the
    /// score-threshold path. Breaks the treaty if present, penalizes only the
    /// betrayer's reputation, appends to the ledger, and returns the emitted
    /// `Betrayal` legend event.
    pub fn record_betrayal(
        &mut self,
        betrayer: PolityId,
        victim: PolityId,
        tick: u64,
    ) -> RawSimEvent {
        let pair = Pair::new(betrayer, victim);
        let score = self.scores.get(&pair).copied().unwrap_or(0.0);
        self.treaties.remove(&pair);
        self.ledger.push(TreatyLedgerEntry {
            tick,
            pair,
            outcome: TreatyOutcome::Broken(TreatyKind::Alliance),
            score,
        });
        // Only the betrayer is penalized for a unilateral betrayal.
        self.reputation
            .adjust(betrayer, -self.config.betrayal_penalty);
        treaty_event(tick, pair, EventKind::Betrayal)
    }
}

/// Build a legend [`RawSimEvent`] for a treaty transition. Diplomacy runs
/// inside the engine, so events are sourced as [`SourceCrate::Engine`]; the
/// pair's two members ride along as participants so the saga graph can resolve
/// them to polity-cluster entities.
fn treaty_event(tick: u64, pair: Pair, kind: EventKind) -> RawSimEvent {
    use civ_legends::ids::SimRuntimeId;
    use civ_legends::model::Role;
    // Higher raw magnitude for betrayals: they reshape the political map.
    let magnitude = if kind == EventKind::Betrayal { 0.85 } else { 0.7 };
    RawSimEvent::new(tick, kind, SourceCrate::Engine, magnitude)
        .with_participant(SourceCrate::Engine, SimRuntimeId(u64::from(pair.lo.0)), Role::Leader)
        .with_participant(SourceCrate::Engine, SimRuntimeId(u64::from(pair.hi.0)), Role::Leader)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    // -- Stance projection ---------------------------------------------------

    /// FR-CIV-DIPLOMACY: score band partitions into Rival/Neutral/Ally.
    #[test]
    fn stance_from_score_partitions_band() {
        assert_eq!(EmergentStance::from_score(-0.9, -0.5, 0.7), EmergentStance::Rival);
        assert_eq!(EmergentStance::from_score(0.0, -0.5, 0.7), EmergentStance::Neutral);
        assert_eq!(EmergentStance::from_score(0.8, -0.5, 0.7), EmergentStance::Ally);
    }

    // -- A. Emergent stance --------------------------------------------------

    /// FR-CIV-DIPLOMACY: allies emerge from a shared enemy.
    ///
    /// Two factions (1, 2) both hold a hostile standing toward faction 3. The
    /// shared-enemy driver alone is enough to warm them past the ally line.
    #[test]
    fn allies_emerge_from_shared_enemies() {
        let mut rivals: BTreeMap<PolityId, BTreeSet<PolityId>> = BTreeMap::new();
        rivals.entry(p(1)).or_default().insert(p(3));
        rivals.entry(p(2)).or_default().insert(p(3));

        let strength = EmergentDiplomacy::shared_enemy_strength(p(1), p(2), &rivals);
        assert!(strength > 0.0, "shared enemy must produce positive pull");

        let drivers = RelationDrivers {
            shared_enemy: strength,
            border_friction: 0.0,
            trade: 0.9,
            culture_similarity: 0.5,
        };
        let mut inputs = StanceInputs::new(1);
        inputs.set(p(1), p(2), drivers);

        let mut dip = EmergentDiplomacy::default();
        dip.update(&inputs);
        assert_eq!(dip.stance(p(1), p(2)), EmergentStance::Ally);
    }

    /// FR-CIV-DIPLOMACY: no shared enemy ⇒ no shared-enemy pull.
    #[test]
    fn no_shared_enemy_is_zero_strength() {
        let mut rivals: BTreeMap<PolityId, BTreeSet<PolityId>> = BTreeMap::new();
        rivals.entry(p(1)).or_default().insert(p(3));
        rivals.entry(p(2)).or_default().insert(p(4));
        assert_eq!(EmergentDiplomacy::shared_enemy_strength(p(1), p(2), &rivals), 0.0);
    }

    /// FR-CIV-DIPLOMACY: allies emerge from trade interdependence.
    #[test]
    fn allies_emerge_from_trade() {
        let drivers = RelationDrivers {
            shared_enemy: 0.0,
            border_friction: 0.0,
            trade: 1.0,
            culture_similarity: 1.0,
        };
        assert!(drivers.score() >= 0.7);
        let mut inputs = StanceInputs::new(5);
        inputs.set(p(10), p(11), drivers);
        let mut dip = EmergentDiplomacy::default();
        dip.update(&inputs);
        assert_eq!(dip.stance(p(10), p(11)), EmergentStance::Ally);
    }

    /// FR-CIV-DIPLOMACY: border friction cools relations toward Rival.
    #[test]
    fn border_friction_drives_rivalry() {
        let a = Territory::new(p(1)).with_cells([1, 2, 3, 4, 5, 6]);
        let b = Territory::new(p(2)).with_cells([3, 4, 5, 6, 7, 8]);
        let friction = EmergentDiplomacy::border_friction_strength(&a, &b);
        assert!(friction > 0.5, "heavy overlap = high friction");
        let drivers = RelationDrivers {
            shared_enemy: 0.0,
            border_friction: friction,
            trade: 0.0,
            culture_similarity: -1.0,
        };
        assert!(drivers.score() <= -0.5);
        assert_eq!(
            EmergentStance::from_score(drivers.score(), -0.5, 0.7),
            EmergentStance::Rival
        );
    }

    // -- B. Treaties ---------------------------------------------------------

    /// FR-CIV-DIPLOMACY: a treaty forms at the ally threshold and emits a
    /// `Treaty` legend event.
    #[test]
    fn treaty_forms_at_threshold_and_emits_legend_event() {
        let drivers = RelationDrivers {
            shared_enemy: 1.0,
            border_friction: 0.0,
            trade: 1.0,
            culture_similarity: 1.0,
        };
        let mut inputs = StanceInputs::new(7);
        inputs.set(p(1), p(2), drivers);
        let mut dip = EmergentDiplomacy::default();
        let events = dip.update(&inputs);
        assert!(dip.has_treaty(p(1), p(2)));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Treaty);
        // Ledger records the formation.
        assert_eq!(dip.ledger().len(), 1);
        assert!(matches!(
            dip.ledger()[0].outcome,
            TreatyOutcome::Formed(TreatyKind::Alliance)
        ));
    }

    /// FR-CIV-DIPLOMACY: a treaty breaks when the relation collapses, emitting
    /// a `Betrayal` legend event.
    #[test]
    fn treaty_breaks_at_low_threshold_and_emits_betrayal() {
        let warm = RelationDrivers {
            shared_enemy: 1.0,
            border_friction: 0.0,
            trade: 1.0,
            culture_similarity: 1.0,
        };
        let cold = RelationDrivers {
            shared_enemy: 0.0,
            border_friction: 1.0,
            trade: 0.0,
            culture_similarity: -1.0,
        };
        let mut dip = EmergentDiplomacy::default();
        // Form.
        let mut t1 = StanceInputs::new(1);
        t1.set(p(1), p(2), warm);
        dip.update(&t1);
        assert!(dip.has_treaty(p(1), p(2)));
        // Collapse.
        let mut t2 = StanceInputs::new(2);
        t2.set(p(1), p(2), cold);
        let events = dip.update(&t2);
        assert!(!dip.has_treaty(p(1), p(2)));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, EventKind::Betrayal);
    }

    /// FR-CIV-DIPLOMACY: hysteresis — a one-tick dip just below the formation
    /// line does not break a treaty (break threshold is strictly lower).
    #[test]
    fn treaty_survives_dip_above_break_threshold() {
        let warm = RelationDrivers {
            shared_enemy: 1.0,
            border_friction: 0.0,
            trade: 1.0,
            culture_similarity: 1.0,
        };
        let mut dip = EmergentDiplomacy::default();
        let mut t1 = StanceInputs::new(1);
        t1.set(p(1), p(2), warm);
        dip.update(&t1);
        // Neutral-ish score (above break threshold): treaty persists.
        let mild = RelationDrivers {
            shared_enemy: 0.0,
            border_friction: 0.0,
            trade: 0.3,
            culture_similarity: 0.0,
        };
        assert!(mild.score() > -0.5 && mild.score() < 0.7);
        let mut t2 = StanceInputs::new(2);
        t2.set(p(1), p(2), mild);
        let events = dip.update(&t2);
        assert!(dip.has_treaty(p(1), p(2)));
        assert!(events.is_empty());
    }

    // -- C. Reputation -------------------------------------------------------

    /// FR-CIV-DIPLOMACY: a betrayal lowers the betrayer's reputation as
    /// observed by third parties.
    #[test]
    fn betrayal_lowers_reputation_with_third_parties() {
        let mut dip = EmergentDiplomacy::default();
        assert_eq!(dip.reputation(p(1)), 0.0);
        let event = dip.record_betrayal(p(1), p(2), 9);
        assert_eq!(event.kind, EventKind::Betrayal);
        // Betrayer penalized; victim untouched by a unilateral betrayal.
        assert!(dip.reputation(p(1)) < 0.0);
        assert_eq!(dip.reputation(p(2)), 0.0);
        // Third party (3) observes the lowered reputation of 1.
        assert!(dip.reputation(p(1)) <= -0.5 + f32::EPSILON);
    }

    /// FR-CIV-DIPLOMACY: a collapsed alliance penalizes both members globally.
    #[test]
    fn collapsed_alliance_penalizes_both_members() {
        let warm = RelationDrivers {
            shared_enemy: 1.0,
            border_friction: 0.0,
            trade: 1.0,
            culture_similarity: 1.0,
        };
        let cold = RelationDrivers {
            shared_enemy: 0.0,
            border_friction: 1.0,
            trade: 0.0,
            culture_similarity: -1.0,
        };
        let mut dip = EmergentDiplomacy::default();
        let mut t1 = StanceInputs::new(1);
        t1.set(p(1), p(2), warm);
        dip.update(&t1);
        let mut t2 = StanceInputs::new(2);
        t2.set(p(1), p(2), cold);
        dip.update(&t2);
        assert!(dip.reputation(p(1)) < 0.0);
        assert!(dip.reputation(p(2)) < 0.0);
    }

    /// Reputation is clamped to `[-1, 1]` under repeated betrayals.
    #[test]
    fn reputation_is_clamped() {
        let mut dip = EmergentDiplomacy::default();
        for t in 0..10 {
            dip.record_betrayal(p(1), p(2), t);
        }
        assert!(dip.reputation(p(1)) >= -1.0);
    }

    // -- D. Determinism ------------------------------------------------------

    /// FR-CIV-DIPLOMACY: same inputs ⇒ identical state, ledger, and events.
    #[test]
    fn determinism_same_inputs_same_outcome() {
        let build = || {
            let mut dip = EmergentDiplomacy::default();
            let mut all_events = Vec::new();
            for tick in 1..=4u64 {
                let mut inputs = StanceInputs::new(tick);
                inputs.set(
                    p(1),
                    p(2),
                    RelationDrivers {
                        shared_enemy: 0.5,
                        border_friction: 0.1 * tick as f32,
                        trade: 1.0,
                        culture_similarity: 1.0,
                    },
                );
                inputs.set(
                    p(2),
                    p(3),
                    RelationDrivers {
                        shared_enemy: 0.0,
                        border_friction: 0.9,
                        trade: 0.0,
                        culture_similarity: -1.0,
                    },
                );
                all_events.extend(dip.update(&inputs));
            }
            (dip, all_events)
        };
        let (dip_a, events_a) = build();
        let (dip_b, events_b) = build();
        assert_eq!(dip_a, dip_b);
        assert_eq!(events_a, events_b);
    }

    /// Driver score is a pure, deterministic function of its inputs.
    #[test]
    fn driver_score_is_pure() {
        let d = RelationDrivers {
            shared_enemy: 0.3,
            border_friction: 0.2,
            trade: 0.6,
            culture_similarity: 0.1,
        };
        assert_eq!(d.score(), d.score());
    }
}
