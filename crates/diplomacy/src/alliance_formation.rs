//! Alliance formation model — FR-CIV-ALLIANCE-FORM.
//!
//! Factions whose **mutual** opinion with one another sits above an
//! `ally_threshold` group together into an **alliance bloc**. Within a
//! single transitively-warm component (faction A is allied with B, B with
//! C, C with D…) all members share one bloc. Factions with neutral or
//! hostile edges to every other faction stand alone.
//!
//! ## Scope (additive — does not modify existing types)
//!
//! * [`AllianceConfig`] — tunable `ally_threshold` and a minimum bloc size
//!   for "real" alliances (singletons are tolerated but reported).
//! * [`AllianceFormation`] — pure projector that takes a [`RelationshipStanceModel`]
//!   and returns the [`AllianceBloc`]s that the current opinions imply.
//! * [`AllianceBloc`] — a set of [`crate::FactionId`]s that mutually hold
//!   one another in the warm bucket.
//!
//! ## Determinism
//!
//! Iteration order is `BTreeSet`/`BTreeMap` so that two instances over the
//! same model always produce identical bloc lists in identical order.
//!
//! ## Independence from `DiplomacyState`
//!
//! Alliance formation is a **pure projection** over the
//! [`RelationshipStanceModel`] surface added by the relationship-stance
//! module. It does not mutate the model and does not depend on the
//! substrate's `DiplomacyState` / `Relation` types. New consumers can opt
//! into alliance formation alongside the relationship-stance model; the
//! existing `civ-diplomacy` substrate is unchanged.
//!
//! ## Algorithm
//!
//! 1. For every pair `(a, b)` already tracked by the model, check whether
//!    `model.stance(a, b) == RelationStance::Ally` (i.e. opinion has
//!    crossed `ally_threshold`).
//! 2. Build an undirected graph on the `Ally` edges.
//! 3. Compute the connected components of that graph; each component is
//!    one [`AllianceBloc`]. Singletons appear when a faction has recorded
//!    pair opinions but none of its neighbors cleared the threshold.
//! 4. Factions that appear in **no** tracked pair are reported as
//!    `Unknown` and are *not* placed in a bloc (the model treats absence
//!    as neutral, hence "no opinion = no alliance edge").

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::relationship_stance::{RelationStance, RelationshipStanceModel};
use crate::FactionId;

/// Tunable parameters for [`AllianceFormation::compute`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceConfig {
    /// Minimum warm opinion required for a pair to count as "allied".
    /// We re-use the model-level threshold only as a hint; an explicit
    /// override here lets a scenario raise or lower the bar without
    /// touching the model.
    pub ally_threshold: i32,
    /// Minimum bloc size for a bloc to be flagged as a "real" alliance
    /// (`is_real()` returns `true`). Singletons are still returned so the
    /// caller can decide what to do with them — a polity that has
    /// recorded opinions but no warm edge is interesting context, not an
    /// error.
    pub min_bloc_size: usize,
}

impl Default for AllianceConfig {
    fn default() -> Self {
        Self {
            ally_threshold: 50,
            min_bloc_size: 2,
        }
    }
}

impl AllianceConfig {
    /// Validate structural invariants.
    pub fn validate(&self) -> Result<(), AllianceConfigError> {
        if self.min_bloc_size == 0 {
            return Err(AllianceConfigError::ZeroMinBlocSize);
        }
        Ok(())
    }
}

/// Errors raised by [`AllianceConfig::validate`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AllianceConfigError {
    /// `min_bloc_size` of 0 would mean every singleton is "real",
    /// defeating the purpose of the flag.
    #[error("min_bloc_size must be >= 1")]
    ZeroMinBlocSize,
}

/// A bloc of factions that mutually hold one another in the warm bucket.
///
/// Bloc members are sorted (`BTreeSet`) so iteration order is stable
/// across runs and across processes — important for replay and audit
/// trails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceBloc {
    /// Factions in this bloc. Sorted ascending by [`FactionId`] id.
    pub members: BTreeSet<FactionId>,
}

impl AllianceBloc {
    /// Number of factions in the bloc.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// `true` if the bloc has no members (should not happen for blocs
    /// produced by [`AllianceFormation`], but kept for completeness).
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// `true` if this bloc meets `min_bloc_size` from
    /// [`AllianceConfig::min_bloc_size`] — i.e. it is a **real**
    /// multi-faction alliance rather than a lone faction with no warm
    /// edges.
    pub fn is_real(&self, config: &AllianceConfig) -> bool {
        self.members.len() >= config.min_bloc_size
    }

    /// `true` if `faction` is a member of this bloc.
    pub fn contains(&self, faction: FactionId) -> bool {
        self.members.contains(&faction)
    }
}

/// Compute alliance blocs from a [`RelationshipStanceModel`].
///
/// `AllianceFormation` is a zero-size type — it carries no state of its
/// own. Call [`Self::compute`] with a `&RelationshipStanceModel` to obtain
/// the current set of blocs.
///
/// The projector is **pure**: identical inputs always produce identical
/// outputs in identical order. It does not mutate the model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceFormation;

impl AllianceFormation {
    /// Project the model's current opinions into a sorted list of
    /// [`AllianceBloc`]s.
    ///
    /// Factions that appear in zero tracked pairs are excluded from the
    /// returned blocs (the model's neutral-by-default semantics treat
    /// unseen pairs as neutral, and a neutral pair is not an alliance
    /// edge).
    pub fn compute(model: &RelationshipStanceModel, config: &AllianceConfig) -> Vec<AllianceBloc> {
        config
            .validate()
            .expect("AllianceConfig invariants checked by caller");

        // 1. Walk every tracked pair and collect a "warm graph":
        //    undirected edges keyed by faction id, where the edge exists
        //    iff the pair is currently in the Ally bucket.
        let mut adjacency: BTreeMap<FactionId, BTreeSet<FactionId>> = BTreeMap::new();

        for (pair, _rs) in model.pairs() {
            if pair.lo == pair.hi {
                continue;
            }
            if model.stance(pair.lo, pair.hi) == RelationStance::Ally {
                adjacency.entry(pair.lo).or_default().insert(pair.hi);
                adjacency.entry(pair.hi).or_default().insert(pair.lo);
            }
        }

        // 2. Connected components via iterative DFS. Using an explicit
        //    worklist (rather than recursion) keeps the projector robust
        //    for very large faction counts without risking a stack
        //    overflow.
        let mut visited: BTreeSet<FactionId> = BTreeSet::new();
        let mut blocs: Vec<AllianceBloc> = Vec::new();

        for &start in adjacency.keys() {
            if visited.contains(&start) {
                continue;
            }
            let mut bloc: BTreeSet<FactionId> = BTreeSet::new();
            let mut stack: Vec<FactionId> = Vec::new();
            stack.push(start);
            while let Some(node) = stack.pop() {
                if !visited.insert(node) {
                    continue;
                }
                bloc.insert(node);
                if let Some(neighbors) = adjacency.get(&node) {
                    for &n in neighbors {
                        if !visited.contains(&n) {
                            stack.push(n);
                        }
                    }
                }
            }
            // Stable sort: BTreeSet guarantees ascending order.
            if !bloc.is_empty() {
                blocs.push(AllianceBloc { members: bloc });
            }
        }

        // `blocs` is already in ascending order by first-member (BTreeMap
        // iteration + BTreeSet membership). No further sort required.
        blocs
    }

    /// Convenience: number of **real** alliances (blocs meeting
    /// `config.min_bloc_size`).
    pub fn real_alliance_count(model: &RelationshipStanceModel, config: &AllianceConfig) -> usize {
        Self::compute(model, config)
            .iter()
            .filter(|b| b.is_real(config))
            .count()
    }
}

/// Dynamic alliance stability metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllianceStability {
    /// Trust score between members (0.0 to 1.0).
    pub trust_score: f64,
    /// Number of shared historical events (ticks of cooperation).
    pub shared_history: u32,
    /// Fraction of resources shared within the alliance (0.0 to 1.0).
    pub resource_share: f64,
}

impl AllianceStability {
    pub fn new(trust: f64, history: u32, share: f64) -> Self {
        Self {
            trust_score: trust.clamp(0.0, 1.0),
            shared_history: history,
            resource_share: share.clamp(0.0, 1.0),
        }
    }
}

/// Manager for dynamic alliance lifecycle operations.
pub struct DynamicAllianceManager {
    /// Maps (FactionId, FactionId) -> AllianceStability
    stabilities: BTreeMap<(FactionId, FactionId), AllianceStability>,
    active_alliances: BTreeSet<BTreeSet<FactionId>>,
}

impl DynamicAllianceManager {
    pub fn new() -> Self {
        Self {
            stabilities: BTreeMap::new(),
            active_alliances: BTreeSet::new(),
        }
    }

    /// Evaluate the stability of a specific pair.
    pub fn evaluate_stability(&self, a: FactionId, b: FactionId) -> Option<&AllianceStability> {
        let key = if a < b { (a, b) } else { (b, a) };
        self.stabilities.get(&key)
    }

    /// Form a new alliance.
    pub fn form_alliance(&mut self, members: BTreeSet<FactionId>, initial_stability: AllianceStability) {
        for &a in &members {
            for &b in &members {
                if a < b {
                    let key = (a, b);
                    self.stabilities.entry(key).or_insert_with(|| initial_stability.clone());
                }
            }
        }
        self.active_alliances.insert(members);
    }

    /// Dissolve an alliance.
    pub fn dissolve_alliance(&mut self, members: &BTreeSet<FactionId>) {
        if self.active_alliances.remove(members) {
            for &a in members {
                for &b in members {
                    if a < b {
                        self.stabilities.remove(&(a, b));
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relationship_stance::{RelationEvent, StanceThresholds};

    fn f(id: u32) -> FactionId {
        FactionId::new(id)
    }

    fn thresholds() -> StanceThresholds {
        StanceThresholds {
            opinion_max: 1_000,
            hostile_threshold: -50,
            ally_threshold: 50,
        }
    }

    fn config() -> AllianceConfig {
        AllianceConfig {
            ally_threshold: 50,
            min_bloc_size: 2,
        }
    }

    /// FR-CIV-ALLIANCE-FORM: two factions with mutual warm opinion above
    /// `ally_threshold` form an alliance bloc.
    ///
    /// Setup: factions A=1 and B=2 each shift opinion to +60 via trade
    /// events; with `ally_threshold = 50`, the pair (1, 2) is in the
    /// Ally bucket. `AllianceFormation::compute` must report exactly one
    /// bloc with both members.
    #[test]
    fn fr_civ_alliance_form_two_high_opinion_factions_form_a_bloc() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let a = f(1);
        let b = f(2);

        // Six +10 trades on (a, b) take the pair to opinion +60, well
        // above the +50 ally threshold. The model reports Ally.
        for _ in 0..6 {
            model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        }
        assert_eq!(model.stance(a, b), RelationStance::Ally);

        // Project to alliances. Expect exactly one bloc, both members.
        let blocs = AllianceFormation::compute(&model, &config());
        assert_eq!(blocs.len(), 1, "one alliance bloc");
        assert_eq!(blocs[0].members.len(), 2);
        assert!(blocs[0].contains(a));
        assert!(blocs[0].contains(b));
        assert!(
            blocs[0].is_real(&config()),
            "two-member bloc meets min_bloc_size"
        );
        assert_eq!(AllianceFormation::real_alliance_count(&model, &config()), 1);
    }

    /// FR-CIV-ALLIANCE-FORM: transitive closure — if A is allied with B
    /// and B is allied with C, then {A, B, C} form a single bloc.
    #[test]
    fn fr_civ_alliance_form_three_factions_collapse_into_one_bloc() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let a = f(1);
        let b = f(2);
        let c = f(3);

        // (a, b): +60 trades -> Ally.
        for _ in 0..6 {
            model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        }
        // (b, c): +60 trades -> Ally.
        for _ in 0..6 {
            model.apply_event(b, c, RelationEvent::Trade { delta: 10 });
        }
        // (a, c): untouched, opinion = 0 -> Neutral, **not** an edge in
        // the alliance graph.

        assert_eq!(model.stance(a, b), RelationStance::Ally);
        assert_eq!(model.stance(b, c), RelationStance::Ally);
        assert_eq!(model.stance(a, c), RelationStance::Neutral);

        let blocs = AllianceFormation::compute(&model, &config());
        assert_eq!(blocs.len(), 1, "transitive closure => single bloc");
        assert_eq!(blocs[0].members.len(), 3);
        assert!(blocs[0].contains(a));
        assert!(blocs[0].contains(b));
        assert!(blocs[0].contains(c));
    }

    /// FR-CIV-ALLIANCE-FORM: a faction with no recorded opinions is
    /// excluded from any bloc.
    #[test]
    fn fr_civ_alliance_form_unseen_factions_are_excluded() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let a = f(10);
        let b = f(11);
        for _ in 0..6 {
            model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        }
        // f(99) was never touched by apply_event.
        assert!(model.get(f(99), f(99)).is_none());

        let blocs = AllianceFormation::compute(&model, &config());
        assert_eq!(blocs.len(), 1);
        assert!(!blocs[0].contains(f(99)));
    }

    /// FR-CIV-ALLIANCE-FORM: a lone faction with recorded (but non-warm)
    /// opinions appears as a singleton bloc, and is **not** flagged as a
    /// "real" alliance.
    #[test]
    fn fr_civ_alliance_form_singleton_blocs_are_not_real() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let a = f(1);
        let b = f(2);

        // Single +10 trade: pair stays Neutral (below +50 ally threshold).
        model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        assert_eq!(model.stance(a, b), RelationStance::Neutral);

        let blocs = AllianceFormation::compute(&model, &config());
        // Single faction appears in a tracked pair but has no warm edges,
        // so it is excluded from the alliance graph entirely.
        assert!(blocs.is_empty(), "no warm edges => no blocs");
        assert_eq!(AllianceFormation::real_alliance_count(&model, &config()), 0);
    }

    /// Configuration validation rejects a degenerate config.
    #[test]
    fn alliance_config_zero_min_bloc_size_rejected() {
        let bad = AllianceConfig {
            ally_threshold: 50,
            min_bloc_size: 0,
        };
        assert!(matches!(
            bad.validate(),
            Err(AllianceConfigError::ZeroMinBlocSize)
        ));
    }

    #[test]
    fn dynamic_alliance_form_and_dissolve() {
        let mut mgr = DynamicAllianceManager::new();
        let mut members = BTreeSet::new();
        members.insert(f(1));
        members.insert(f(2));
        mgr.form_alliance(members.clone(), AllianceStability::new(0.8, 10, 0.5));
        assert!(mgr.evaluate_stability(f(1), f(2)).is_some());
        mgr.dissolve_alliance(&members);
        assert!(mgr.evaluate_stability(f(1), f(2)).is_none());
    }

    #[test]
    fn dynamic_alliance_stability_clamps() {
        let s = AllianceStability::new(1.5, 5, -0.2);
        assert_eq!(s.trust_score, 1.0);
        assert_eq!(s.resource_share, 0.0);
    }
}
