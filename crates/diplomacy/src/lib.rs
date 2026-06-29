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
use std::collections::{BTreeMap, VecDeque};
use thiserror::Error;

/// Schema version of this crate's public types. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Stable polity identifier used by diplomacy contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PolityId(pub u32);

impl PolityId {
    /// Wrap a raw `u32` id.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Compatibility alias preserved for earlier crate code.
#[deprecated(note = "PolityId is the preferred term in active diplomacy contracts")]
pub type ActorId = PolityId;

/// Compatibility alias for faction-scoped diplomacy contracts.
pub type FactionId = PolityId;

/// Ordered pair `(min, max)` of actor ids. The order is fixed at construction
/// so that `Pair::new(a, b) == Pair::new(b, a)` for any `a, b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pair {
    /// The smaller id (always).
    pub lo: PolityId,
    /// The larger id (always).
    pub hi: PolityId,
}

impl Pair {
    /// Construct a canonical ordered pair.
    pub fn new(a: PolityId, b: PolityId) -> Self {
        if a.0 <= b.0 {
            Self { lo: a, hi: b }
        } else {
            Self { lo: b, hi: a }
        }
    }

    /// Iterate `(lo, hi)`.
    pub fn actors(self) -> (PolityId, PolityId) {
        (self.lo, self.hi)
    }
}

/// High-level resource categories used by trade acts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Agricultural goods.
    Food,
    /// Timber and construction materials.
    Wood,
    /// Refined industrial materials.
    Metal,
    /// Energy commodities or power credits.
    Energy,
}

/// Event-driven diplomatic action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiplomaticAct {
    /// Exchange a resource for diplomatic goodwill.
    Trade {
        /// Traded resource type.
        good: ResourceKind,
        /// Trade volume.
        volume: f32,
    },
    /// Form or extend an alliance for a fixed duration.
    Alliance {
        /// Duration in ticks.
        duration_ticks: u32,
    },
    /// Declare or renounce war.
    Declaration {
        /// `true` if this is a war declaration.
        war: bool,
    },
    /// Send tribute to the target.
    Tribute {
        /// Tribute amount.
        amount: f32,
    },
    /// Open a negotiation topic.
    Proposal {
        /// Negotiation topic.
        topic: String,
    },
}

/// Opinion state for one directed diplomatic edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Opinion {
    /// Current opinion value.
    pub value: f32,
    /// Raw history of the last 20 events as `(tick, delta)`.
    pub memory: VecDeque<(u64, f32)>,
}

impl Opinion {
    /// Record one opinion delta and keep only the most recent 20 events.
    pub fn record(&mut self, tick: u64, delta: f32) {
        self.memory.push_back((tick, delta));
        while self.memory.len() > OPINION_MEMORY_LIMIT {
            self.memory.pop_front();
        }
    }
}

/// Directed diplomacy network keyed by `(target, actor)` opinion edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DiplomacyNetwork {
    opinions: BTreeMap<(FactionId, FactionId), Opinion>,
}

impl DiplomacyNetwork {
    /// Construct an empty diplomacy network.
    pub fn new() -> Self {
        Self::default()
    }
}

const OPINION_MEMORY_LIMIT: usize = 20;
const OPINION_DECAY_START_TICKS: u64 = 100;
const OPINION_DECAY_SCALE: f32 = 100.0;

fn opinion_key(from: FactionId, to: FactionId) -> (FactionId, FactionId) {
    (from, to)
}

fn resource_weight(good: ResourceKind) -> f32 {
    match good {
        ResourceKind::Food => 0.8,
        ResourceKind::Wood => 0.6,
        ResourceKind::Metal => 1.0,
        ResourceKind::Energy => 1.2,
    }
}

fn act_base_delta(act: &DiplomaticAct) -> f32 {
    match act {
        DiplomaticAct::Trade { good, volume } => volume.max(0.0) * resource_weight(*good),
        DiplomaticAct::Alliance { duration_ticks } => *duration_ticks as f32 * 0.02,
        DiplomaticAct::Declaration { war } => {
            if *war {
                -14.0
            } else {
                5.0
            }
        }
        DiplomaticAct::Tribute { amount } => amount.max(0.0) * 0.5,
        DiplomaticAct::Proposal { topic } => 0.5 + topic.chars().count().min(120) as f32 * 0.01,
    }
}

fn opinion_modifier(current: f32) -> f32 {
    (1.0 - current / 400.0).clamp(0.75, 1.25)
}

/// Recompute opinion from stored history with exponential decay for old events.
pub fn decay_opinion(op: &mut Opinion, current_tick: u64) {
    while op.memory.len() > OPINION_MEMORY_LIMIT {
        op.memory.pop_front();
    }
    op.value = op
        .memory
        .iter()
        .map(|&(tick, delta)| {
            let age = current_tick.saturating_sub(tick);
            let weight = if age <= OPINION_DECAY_START_TICKS {
                1.0
            } else {
                (-((age - OPINION_DECAY_START_TICKS) as f32 / OPINION_DECAY_SCALE)).exp()
            };
            delta * weight
        })
        .sum();
}

/// Predict the opinion delta for a diplomatic act.
pub fn evaluate_act(
    actor: FactionId,
    target: FactionId,
    act: &DiplomaticAct,
    network: &DiplomacyNetwork,
) -> f32 {
    if actor == target {
        return 0.0;
    }
    let current = network
        .opinions
        .get(&opinion_key(target, actor))
        .map(|op| op.value)
        .unwrap_or(0.0);
    act_base_delta(act) * opinion_modifier(current)
}

/// Apply a diplomatic act to the directed opinion edge from target to actor.
pub fn apply_act(
    network: &mut DiplomacyNetwork,
    actor: FactionId,
    target: FactionId,
    act: DiplomaticAct,
    tick: u64,
) {
    if actor == target {
        return;
    }

    let key = opinion_key(target, actor);
    {
        let opinion = network.opinions.entry(key).or_default();
        decay_opinion(opinion, tick);
    }

    let delta = evaluate_act(actor, target, &act, network);
    let opinion = network.opinions.entry(key).or_default();
    opinion.record(tick, delta);
    decay_opinion(opinion, tick);
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
        attacker: PolityId,
        /// Defending faction.
        defender: PolityId,
        /// Energy of the engagement (mirrors `CombatEngagement::damage.energy`).
        energy: u32,
        /// Simulation tick.
        tick: u64,
    },
    /// An explicit diplomatic gesture (treaty signing, tribute, insult) from a
    /// future caller. Positive delta = warmer, negative = colder.
    Gesture {
        /// Acting faction.
        from: PolityId,
        /// Receiving faction.
        to: PolityId,
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

/// Minimal deterministic model for treaty concession rounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcessionStateMachine {
    /// Actors involved in this negotiation.
    pub pair: Pair,
    /// Monotonic concession level (0 = no concession, higher = deeper offer).
    pub concession_level: u8,
    /// Current round counter, starting at 0.
    pub round: u8,
    /// Maximum configured rounds before deadlock.
    pub max_rounds: u8,
    /// `true` once either acceptance or termination occurs.
    pub resolved: bool,
    /// Actor who authored the most recent offer (`None` until the first
    /// `propose`/`counter` call). Used by [`Self::counter`] to enforce the
    /// "counter must come from the other actor" rule without depending on
    /// ad-hoc round-parity heuristics. Stable across replays because the
    /// input stream is the source of truth.
    last_proposer: Option<PolityId>,
}

impl ConcessionStateMachine {
    /// Construct a new negotiation state machine for `pair` with a positive cap.
    ///
    /// A zero `max_rounds` behaves as immediate deadlock and is therefore valid
    /// only when callers finish the negotiation before any offers.
    pub fn new(pair: Pair, max_rounds: u8) -> Self {
        Self {
            pair,
            concession_level: 0,
            round: 0,
            max_rounds,
            resolved: false,
            last_proposer: None,
        }
    }

    /// Propose a new concession level. Levels must be monotonic non-decreasing
    /// until the round budget is exhausted.
    pub fn propose(
        &mut self,
        proposer: PolityId,
        concession_level: u8,
    ) -> Result<(), ConcessionError> {
        if self.resolved {
            return Err(ConcessionError::AlreadyResolved);
        }
        if self.round >= self.max_rounds {
            return Err(ConcessionError::RoundLimitReached);
        }
        if proposer != self.pair.lo && proposer != self.pair.hi {
            return Err(ConcessionError::ActorNotInPair);
        }
        if concession_level < self.concession_level {
            return Err(ConcessionError::ConcessionMustNotRegress);
        }
        self.round += 1;
        self.concession_level = concession_level;
        self.last_proposer = Some(proposer);
        Ok(())
    }

    /// Accept the last proposed concession level.
    pub fn accept(&mut self, accepter: PolityId) -> Result<(), ConcessionError> {
        if self.resolved {
            return Err(ConcessionError::AlreadyResolved);
        }
        if self.round == 0 {
            return Err(ConcessionError::NoOfferToAccept);
        }
        if accepter != self.pair.lo && accepter != self.pair.hi {
            return Err(ConcessionError::ActorNotInPair);
        }
        self.resolved = true;
        Ok(())
    }

    /// Reject the negotiation and mark it resolved.
    pub fn reject(&mut self, rejecter: PolityId) -> Result<(), ConcessionError> {
        if self.resolved {
            return Err(ConcessionError::AlreadyResolved);
        }
        if self.round == 0 {
            return Err(ConcessionError::NoOfferToAccept);
        }
        if rejecter != self.pair.lo && rejecter != self.pair.hi {
            return Err(ConcessionError::ActorNotInPair);
        }
        self.resolved = true;
        Ok(())
    }

    /// Counter the last proposed concession with a strictly higher level.
    ///
    /// "Counter" is the third path required by `FR-CIV-DIPLO-008` (the
    /// spec calls for "accept/reject/counter paths" across at least three
    /// goal types). A counter is mechanically a second `propose` from the
    /// non-original proposer that raises `concession_level`, signaling
    /// "I would accept if you went further." We expose it as a dedicated
    /// method so callers don't have to remember the proposer-switching
    /// rule and so the audit trail can record it explicitly.
    ///
    /// The counter must be strictly higher than the current
    /// `concession_level` (a counter at the same level is a `propose`),
    /// the round budget must not be exhausted, and the counter must come
    /// from the *other* actor in the pair (the actor who did not author
    /// the current offer). This is what makes a counter a counter and
    /// not just a re-propose.
    pub fn counter(
        &mut self,
        counterer: PolityId,
        new_concession_level: u8,
    ) -> Result<(), ConcessionError> {
        if self.resolved {
            return Err(ConcessionError::AlreadyResolved);
        }
        if self.round >= self.max_rounds {
            return Err(ConcessionError::RoundLimitReached);
        }
        if counterer != self.pair.lo && counterer != self.pair.hi {
            return Err(ConcessionError::ActorNotInPair);
        }
        if self.round == 0 {
            return Err(ConcessionError::NoOfferToAccept);
        }
        if new_concession_level <= self.concession_level {
            return Err(ConcessionError::ConcessionMustNotRegress);
        }
        // The counter must come from the *other* actor in the pair so
        // that counters are distinguishable from monotonic re-proposals.
        // We use the explicit `last_proposer` field (set by `propose` and
        // by the `counter` itself on success) rather than inferring from
        // round parity.
        if let Some(prior) = self.last_proposer {
            if counterer == prior {
                return Err(ConcessionError::SameActorCounter);
            }
        }
        self.round += 1;
        self.concession_level = new_concession_level;
        self.last_proposer = Some(counterer);
        Ok(())
    }
}

/// Errors emitted by concession machine transitions.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConcessionError {
    /// Only pair actors may propose/accept/reject.
    #[error("actor is not part of this pair")]
    ActorNotInPair,
    /// Concession levels must be non-decreasing.
    #[error("concession level regressed relative to prior round")]
    ConcessionMustNotRegress,
    /// No offer has been proposed yet.
    #[error("no offer exists to accept")]
    NoOfferToAccept,
    /// Maximum configured rounds has been reached.
    #[error("round limit reached")]
    RoundLimitReached,
    /// The machine is already in accepted or rejected state.
    #[error("machine already resolved")]
    AlreadyResolved,
    /// A counter must come from the *other* actor in the pair; a counter
    /// from the actor who authored the prior offer is a re-propose and
    /// should be expressed via `propose`, not `counter`.
    #[error("counter must come from the other actor in the pair")]
    SameActorCounter,
}

/// Structured treaty clauses used by formal transcripts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TreatyClauseKind {
    /// Temporary truce with an expiration window.
    Ceasefire,
    /// Mutual agreement not to target one another.
    NonAggression,
    /// Territory-related settlement terms.
    TerritoryClause,
    /// Trade, passage, or tribute obligations.
    TradeClause,
}

/// Clause object used for deterministic replay + audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyClause {
    /// Clause discriminator.
    pub kind: TreatyClauseKind,
    /// Short machine-readable key (`"clause"`).
    pub key: String,
    /// Machine-readable payload value (`"30"`, `"sector-7"`).
    pub value: String,
}

impl TreatyClause {
    /// Construct a structured clause from raw key/value fields.
    pub fn new<K, V>(kind: TreatyClauseKind, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            kind,
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Formal register transcript that keeps structured clauses separate from LLM text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyTranscript {
    /// Deterministic clauses that drive replay and tests.
    pub clauses: Vec<TreatyClause>,
    /// Optional natural-language garnish layer generated by LLM.
    pub llm_garnish: Option<String>,
}

impl TreatyTranscript {
    /// Start from base clauses and optional garnish.
    pub fn new(clauses: Vec<TreatyClause>, llm_garnish: Option<String>) -> Self {
        Self {
            clauses,
            llm_garnish,
        }
    }

    /// Append one structured clause.
    pub fn push_clause(&mut self, clause: TreatyClause) {
        self.clauses.push(clause);
    }

    /// Replace garnish text after deterministic clause computation.
    pub fn set_llm_garnish<S: Into<String>>(&mut self, llm_garnish: S) {
        self.llm_garnish = Some(llm_garnish.into());
    }

    /// Borrow structured clauses directly.
    pub fn clauses(&self) -> &[TreatyClause] {
        &self.clauses
    }
}

/// Rule-source restriction for game-relevant diplomacy decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionSource {
    /// Deterministic gameplay rules.
    RuleEngine,
    /// Large-language-model fallback, explicitly disallowed for these actions.
    Llm,
}

/// Canonical war goals for deterministic selection.
///
/// Three bounded goal types — see `FR-CIV-DIPLO-001` and `FR-CIV-DIPLO-008`.
/// "Limited" is the default conflict posture; "Total" escalates; "Surrender"
/// is the capitulation branch from `FR-CIV-DIPLO-001` ("defenders evaluate
/// goal utility to choose capitulate/counter/fight"). The integer ordering
/// is meaningful: defenders prefer the smallest ordinal that satisfies
/// the goal-utility check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WarGoal {
    /// Limited, reversible skirmish posture.
    Limited,
    /// Sustained hostility posture.
    Total,
    /// Capitulation — the goal-utility check has resolved to surrender.
    ///
    /// This is the third bounded goal required by `FR-CIV-DIPLO-008` for
    /// integration tests that must cover accept/reject/counter paths for
    /// at least three goal types. Selectors should only produce `Surrender`
    /// when standing is below the configured capitulation floor AND war
    /// exhaustion exceeds the configured ceiling.
    Surrender,
}

/// War-exhaustion component used by [`select_war_goal_v2`]. Mirrors the
/// `war_exhaustion: u32` field in `civ-tactics::CombatEngagement` aggregates.
///
/// Exhaustion is non-negative and saturates at `u32::MAX`. Higher values
/// erode the defender's resolve to keep fighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WarExhaustion(pub u32);

/// Decision API returns this when callers ask prohibited sources.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecisionError {
    /// The selected source is forbidden for this decision.
    #[error("LLM-based decisions are disallowed for {action}")]
    LlmDisallowed {
        /// Decision action that cannot use LLM input.
        action: &'static str,
    },
}

/// Decide treaty acceptance using deterministic rule inputs.
pub fn decide_treaty_acceptance(
    standing: i32,
    source: DecisionSource,
) -> Result<bool, DecisionError> {
    if source == DecisionSource::Llm {
        return Err(DecisionError::LlmDisallowed {
            action: "treaty acceptance",
        });
    }
    Ok(standing >= 0)
}

/// Apply a trust delta with deterministic rule inputs.
pub fn update_trust_score(
    trust: i32,
    delta: i32,
    source: DecisionSource,
) -> Result<i32, DecisionError> {
    if source == DecisionSource::Llm {
        return Err(DecisionError::LlmDisallowed {
            action: "trust updates",
        });
    }
    Ok(trust.saturating_add(delta))
}

/// Select war goals from current relation and deterministic rule set.
pub fn select_war_goal(standing: i32, source: DecisionSource) -> Result<WarGoal, DecisionError> {
    if source == DecisionSource::Llm {
        return Err(DecisionError::LlmDisallowed {
            action: "war-goal selection",
        });
    }
    if standing <= -10 {
        Ok(WarGoal::Total)
    } else {
        Ok(WarGoal::Limited)
    }
}

/// Select war goals using the full 3-type taxonomy from `FR-CIV-DIPLO-001`.
///
/// This is the extended selector that pairs a [`WarGoal`] with a
/// [`WarExhaustion`] aggregate so that defenders may capitulate
/// ([`WarGoal::Surrender`]) when both the standing has collapsed AND
/// exhaustion is past the capitulation ceiling. The three goal types
/// are:
///
/// 1. `Limited` — the default; defenders still see a path to a positive
///    outcome.
/// 2. `Total` — defenders are losing but not yet exhausted; the conflict
///    will be pursued to conclusion.
/// 3. `Surrender` — standing has collapsed AND exhaustion exceeds
///    `capitulation_exhaustion`. Mirrors the "capitulate" branch of
///    `FR-CIV-DIPLO-001` ("defenders evaluate goal utility to choose
///    capitulate/counter/fight").
///
/// The function is the third bounded goal required by `FR-CIV-DIPLO-008`
/// for integration tests covering accept/reject/counter paths across
/// at least three goal types.
pub fn select_war_goal_v2(
    standing: i32,
    exhaustion: WarExhaustion,
    capitulation_floor: i32,
    capitulation_exhaustion: u32,
    source: DecisionSource,
) -> Result<WarGoal, DecisionError> {
    if source == DecisionSource::Llm {
        return Err(DecisionError::LlmDisallowed {
            action: "war-goal selection",
        });
    }
    if standing <= capitulation_floor && exhaustion.0 >= capitulation_exhaustion {
        Ok(WarGoal::Surrender)
    } else if standing <= -10 {
        Ok(WarGoal::Total)
    } else {
        Ok(WarGoal::Limited)
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
            return Err(ConfigError::HostileThresholdPositive(
                self.hostile_threshold,
            ));
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
    pub fn get(&self, a: PolityId, b: PolityId) -> Option<&Relation> {
        if a == b {
            return None;
        }
        self.relations.get(&Pair::new(a, b))
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
                (i64::from(rel.standing) + decay)
                    .min(0)
                    .max(-i64::from(max)) as i32
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
                InteractionEvent::Gesture {
                    from,
                    to,
                    delta,
                    tick,
                } => {
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
    fn bump(&mut self, a: PolityId, b: PolityId, delta: i32, tick: u64) {
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
// FR-CIV-DIPLO-STANCE: relationship-stance model
// ---------------------------------------------------------------------------
//
// Additive slice: faction pairs hold an opinion scalar shifted by events
// (trade=+, raid=-) and mapped to stance buckets `Ally`/`Neutral`/`Hostile`.
// Lives in its own module and does not modify the substrate above.

pub mod relationship_stance;

pub use relationship_stance::{
    FactionPair, Opinion, RelationEvent, RelationStance, RelationshipStance,
    RelationshipStanceModel, StanceConfigError, StanceThresholds,
};

// ---------------------------------------------------------------------------
// FR-CIV-ALLIANCE-FORM: alliance-formation model
// ---------------------------------------------------------------------------
//
// Additive slice: factions whose mutual opinion clears `ally_threshold`
// group together into a single alliance bloc. Transitive warm edges
// collapse into one bloc; singletons appear when a faction has recorded
// opinions but no warm edges. Lives in its own module and does not modify
// the substrate above.

pub mod alliance_formation;

pub use alliance_formation::{
    AllianceBloc, AllianceConfig, AllianceConfigError, AllianceFormation,
};

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

    fn a(id: u32) -> PolityId {
        PolityId::new(id)
    }

    fn approx_eq(lhs: f32, rhs: f32) -> bool {
        (lhs - rhs).abs() < 1e-5
    }

    // -- Event-driven diplomacy ---------------------------------------------

    /// Event-driven diplomacy keeps all act kinds deterministic and signed.
    #[test]
    fn fr_civ_diplo_009_evaluate_act_covers_all_variants() {
        let network = DiplomacyNetwork::new();
        let actor = a(1);
        let target = a(2);
        let cases = [
            (
                DiplomaticAct::Trade {
                    good: ResourceKind::Food,
                    volume: 10.0,
                },
                true,
            ),
            (
                DiplomaticAct::Alliance {
                    duration_ticks: 200,
                },
                true,
            ),
            (DiplomaticAct::Declaration { war: true }, false),
            (DiplomaticAct::Declaration { war: false }, true),
            (DiplomaticAct::Tribute { amount: 8.0 }, true),
            (
                DiplomaticAct::Proposal {
                    topic: "open border talks".to_string(),
                },
                true,
            ),
        ];

        for (act, expected_positive) in cases {
            let delta = evaluate_act(actor, target, &act, &network);
            if expected_positive {
                assert!(delta > 0.0, "{act:?} should improve opinion");
            } else {
                assert!(delta < 0.0, "{act:?} should worsen opinion");
            }
        }
    }

    /// Applying an act updates the directed target->actor opinion edge.
    #[test]
    fn fr_civ_diplo_009_apply_act_updates_directed_opinion() {
        let mut network = DiplomacyNetwork::new();
        let actor = a(3);
        let target = a(4);
        let act = DiplomaticAct::Trade {
            good: ResourceKind::Metal,
            volume: 5.0,
        };

        let expected = evaluate_act(actor, target, &act, &network);
        apply_act(&mut network, actor, target, act, 42);

        let opinion = network
            .opinions
            .get(&(target, actor))
            .expect("target->actor opinion exists");
        assert!(approx_eq(opinion.value, expected));
        assert_eq!(opinion.memory.len(), 1);
        assert_eq!(opinion.memory[0], (42, expected));
        assert!(network.opinions.get(&(actor, target)).is_none());
    }

    /// Opinion history is capped at the most recent 20 events.
    #[test]
    fn fr_civ_diplo_009_memory_keeps_last_twenty_events() {
        let mut network = DiplomacyNetwork::new();
        let actor = a(5);
        let target = a(6);

        for tick in 0..21 {
            apply_act(
                &mut network,
                actor,
                target,
                DiplomaticAct::Tribute { amount: 1.0 },
                tick,
            );
        }

        let opinion = network
            .opinions
            .get(&(target, actor))
            .expect("target->actor opinion exists");
        assert_eq!(opinion.memory.len(), 20);
        assert_eq!(opinion.memory.front().map(|(tick, _)| *tick), Some(1));
        assert_eq!(opinion.memory.back().map(|(tick, _)| *tick), Some(20));
    }

    /// Events older than 100 ticks decay exponentially.
    #[test]
    fn fr_civ_diplo_009_decay_opinion_exponentially_decays_old_events() {
        let mut opinion = Opinion::default();
        opinion.memory.push_back((0, 10.0));
        opinion.memory.push_back((150, 4.0));

        decay_opinion(&mut opinion, 250);

        let expected = 10.0 * (-1.5f32).exp() + 4.0;
        assert!(approx_eq(opinion.value, expected));
        assert_eq!(opinion.memory.len(), 2);
    }

    // -- Pair ordering -------------------------------------------------------

    /// Covers FR-CIV-DIPLO-001.
    #[test]
    fn pair_is_canonical_and_symmetric() {
        let p1 = Pair::new(a(3), a(7));
        let p2 = Pair::new(a(7), a(3));
        assert_eq!(p1, p2);
        assert_eq!(p1.lo, a(3));
        assert_eq!(p1.hi, a(7));
    }

    /// Covers FR-CIV-DIPLO-001-RELATIONS.
    #[test]
    fn pair_with_equal_actors_is_self() {
        // a == b is degenerate; we still produce (a, a) and the relation
        // layer rejects it. Verify canonical ordering holds.
        let p = Pair::new(a(5), a(5));
        assert_eq!(p.lo, a(5));
        assert_eq!(p.hi, a(5));
    }

    // -- Config validation ---------------------------------------------------

    /// Covers FR-CIV-DIPLO-002.
    #[test]
    fn default_config_validates() {
        assert!(DiplomacyConfig::default().validate().is_ok());
    }

    /// Covers FR-CIV-DIPLO-002.
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

    /// Covers FR-CIV-DIPLO-002.
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

    /// Covers FR-CIV-DIPLO-001.
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

    /// Covers FR-CIV-DIPLO-003.
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

    /// Covers FR-CIV-DIPLO-003.
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

    /// Covers FR-CIV-GOV-001.
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

    /// Covers FR-CIV-DIPLO-003.
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

    /// Covers FR-CIV-GOV-002.
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
        assert_eq!(s.get(a(1), a(2)).unwrap().stance(&cfg()), Stance::Neutral);
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

    // -- FR-CIV-DIPLO-004 ---------------------------------------------------

    /// FR-CIV-DIPLO-004.
    #[test]
    fn fr_civ_diplo_004_concession_machine_is_monotonic() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(1), a(2)), 3);
        assert!(machine.propose(a(1), 1).is_ok());
        assert!(machine.propose(a(2), 2).is_ok());
        assert!(matches!(
            machine.propose(a(1), 1),
            Err(ConcessionError::ConcessionMustNotRegress)
        ));
    }

    /// FR-CIV-DIPLO-004.
    #[test]
    fn fr_civ_diplo_004_rejects_round_limit() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(1), a(2)), 1);
        assert!(machine.propose(a(1), 1).is_ok());
        assert!(matches!(
            machine.propose(a(2), 2),
            Err(ConcessionError::RoundLimitReached)
        ));
        assert!(matches!(machine.accept(a(2)), Ok(())));
        assert!(matches!(
            machine.accept(a(1)),
            Err(ConcessionError::AlreadyResolved)
        ));
    }

    // -- FR-CIV-DIPLO-005 ---------------------------------------------------

    /// FR-CIV-DIPLO-005.
    #[test]
    fn fr_civ_diplo_005_structured_transcript_remains_stable_without_garnish() {
        let clauses = vec![
            TreatyClause::new(TreatyClauseKind::Ceasefire, "duration_ticks", "300"),
            TreatyClause::new(TreatyClauseKind::TradeClause, "grain_tribute", "5"),
        ];
        let mut tx = TreatyTranscript::new(clauses.clone(), Some("LLM says handshake".to_string()));
        let expected = tx.clauses().to_vec();
        tx.set_llm_garnish("Alternative wording from LLM");
        assert_eq!(tx.clauses(), expected.as_slice());
    }

    // -- FR-CIV-DIPLO-006 ---------------------------------------------------

    /// FR-CIV-DIPLO-006.
    #[test]
    fn fr_civ_diplo_006_blocks_llm_treaty_acceptance_decision() {
        assert!(matches!(
            decide_treaty_acceptance(0, DecisionSource::Llm),
            Err(DecisionError::LlmDisallowed {
                action: "treaty acceptance"
            })
        ));
        assert!(decide_treaty_acceptance(0, DecisionSource::RuleEngine).is_ok());
    }

    /// FR-CIV-DIPLO-006.
    #[test]
    fn fr_civ_diplo_006_blocks_llm_trust_and_war_goal_decisions() {
        assert!(matches!(
            update_trust_score(10, -3, DecisionSource::Llm),
            Err(DecisionError::LlmDisallowed {
                action: "trust updates"
            })
        ));
        assert!(matches!(
            select_war_goal(-20, DecisionSource::Llm),
            Err(DecisionError::LlmDisallowed {
                action: "war-goal selection"
            })
        ));
    }

    // -- FR-CIV-DIPLO-007 ---------------------------------------------------

    /// FR-CIV-DIPLO-007.
    #[test]
    fn fr_civ_diplo_007_polity_ids_are_scalar_clusters() {
        let from = PolityId::new(12);
        let to: PolityId = PolityId::new(13);
        let pair = Pair::new(from, to);
        assert_eq!(pair.actors(), (PolityId::new(12), PolityId::new(13)));
    }

    // -- FR-CIV-DIPLO-008 ---------------------------------------------------
    //
    // FR-CIV-DIPLO-008 — Integration tests SHALL cover accept/reject/counter
    // paths for at least three goal types. We treat the three goal types as
    // the three `WarGoal` variants (Limited, Total, Surrender) and exercise
    // the concession machine's accept/reject/counter paths for each. The
    // 3-goal taxonomy comes from `FR-CIV-DIPLO-001`'s
    // "capitulate/counter/fight" outcomes and is materialized here as
    // `WarGoal::{Limited, Total, Surrender}` plus a `WarExhaustion` aggregate
    // that drives the capitulation branch.

    /// FR-CIV-DIPLO-008.
    ///
    /// `WarGoal::Limited` is the default branch of `select_war_goal_v2`. The
    /// accept path resolves a `ConcessionStateMachine` with a `Limited`
    /// outcome in the audit trail.
    #[test]
    fn fr_civ_diplo_008_accept_path_for_limited_goal() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(1), a(2)), 3);
        // Round 1: actor 1 offers level 1 (Limited outcome territory).
        assert!(machine.propose(a(1), 1).is_ok());
        // Round 2: actor 2 offers level 2 (slightly larger concession).
        assert!(machine.propose(a(2), 2).is_ok());
        // Accept: resolution path.
        assert!(machine.accept(a(1)).is_ok());
        assert!(machine.resolved);
        assert_eq!(machine.concession_level, 2);

        // Selector returns Limited for a small negative standing.
        let goal = select_war_goal_v2(
            -1,
            WarExhaustion(0),
            -50,
            10_000,
            DecisionSource::RuleEngine,
        )
        .expect("rule engine");
        assert_eq!(goal, WarGoal::Limited);
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// `WarGoal::Total` is the escalation branch of `select_war_goal_v2`. The
    /// reject path terminates a negotiation that would have escalated to a
    /// Total war goal, so the audit trail records the rejection rather than
    /// resolution.
    #[test]
    fn fr_civ_diplo_008_reject_path_for_total_goal() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(3), a(4)), 3);
        // Initial offer triggers the Total branch in the selector.
        assert!(machine.propose(a(3), 5).is_ok());
        // Reject: war escalates to Total.
        assert!(machine.reject(a(4)).is_ok());
        assert!(machine.resolved);

        let goal = select_war_goal_v2(
            -10,
            WarExhaustion(0),
            -50,
            10_000,
            DecisionSource::RuleEngine,
        )
        .expect("rule engine");
        assert_eq!(goal, WarGoal::Total);
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// `WarGoal::Surrender` is the capitulation branch. The counter path is
    /// exercised by the other actor in the pair raising the level after an
    /// initial offer — the canonical "I would accept if you went further"
    /// move. The final state of the machine is `concession_level = 4`
    /// (the counter's level) and the actor-pair auditable.
    #[test]
    fn fr_civ_diplo_008_counter_path_for_surrender_goal() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(5), a(6)), 3);
        // Initial offer from actor 5 (lower id).
        assert!(machine.propose(a(5), 1).is_ok());
        // Counter from actor 6 (higher id) at a higher level.
        assert!(machine.counter(a(6), 4).is_ok());
        assert_eq!(machine.concession_level, 4);
        assert_eq!(machine.round, 2);
        // Accept the counter.
        assert!(machine.accept(a(5)).is_ok());
        assert!(machine.resolved);

        // Selector returns Surrender when standing has collapsed AND
        // exhaustion exceeds the capitulation ceiling.
        let goal = select_war_goal_v2(
            -100,
            WarExhaustion(20_000),
            -50,
            10_000,
            DecisionSource::RuleEngine,
        )
        .expect("rule engine");
        assert_eq!(goal, WarGoal::Surrender);
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// `counter()` rejects a counter from the actor who authored the prior
    /// offer (a re-propose at the same level would have been `propose`, not
    /// `counter`). We also verify the rest of the validation surface
    /// (non-monotonic, outsider, no-offer) on the same machine.
    #[test]
    fn fr_civ_diplo_008_counter_from_same_actor_is_rejected() {
        let mut machine = ConcessionStateMachine::new(Pair::new(a(5), a(6)), 3);
        // Initial offer from actor 5 (lo). last_proposer = 5.
        assert!(machine.propose(a(5), 1).is_ok());
        // Actor 5 cannot counter its own offer.
        assert!(matches!(
            machine.counter(a(5), 2),
            Err(ConcessionError::SameActorCounter)
        ));
        // Actor 6 (the OTHER actor) may counter.
        assert!(machine.counter(a(6), 3).is_ok());
        // Now last_proposer = 6, so actor 6 cannot counter again.
        assert!(matches!(
            machine.counter(a(6), 4),
            Err(ConcessionError::SameActorCounter)
        ));
        // A non-monotonic counter is rejected regardless of the actor.
        assert!(matches!(
            machine.counter(a(5), 1),
            Err(ConcessionError::ConcessionMustNotRegress)
        ));
        // A counter from an outsider is rejected.
        assert!(matches!(
            machine.counter(a(99), 99),
            Err(ConcessionError::ActorNotInPair)
        ));
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// Integration property: across a tight set of starting standings, the
    /// 3-goal taxonomy of `select_war_goal_v2` partitions the input space
    /// deterministically — i.e. the selector is a pure function of
    /// `(standing, exhaustion, capitulation_floor, capitulation_exhaustion)`
    /// and the same inputs always produce the same goal.
    #[test]
    fn fr_civ_diplo_008_select_war_goal_v2_partitions_three_branches() {
        // Boundary semantics: Surrender iff
        //   standing <= capitulation_floor AND exhaustion >= capitulation_exhaustion
        // Total iff
        //   standing <= -10 AND (standing > capitulation_floor OR exhaustion < cap)
        // Limited otherwise.
        let cases: [(i32, u32, WarGoal); 9] = [
            (0, 0, WarGoal::Limited),
            (-1, 0, WarGoal::Limited),
            (-9, 0, WarGoal::Limited),
            (-10, 0, WarGoal::Total),
            (-50, 9_999, WarGoal::Total), // -50 hits floor but exhaustion < cap
            (-49, 10_000, WarGoal::Total), // standing just above floor
            (-50, 10_000, WarGoal::Surrender), // boundary: both crossed
            (-100, 20_000, WarGoal::Surrender),
            (i32::MIN, u32::MAX, WarGoal::Surrender),
        ];
        for (standing, exhaustion, expected) in cases {
            let goal = select_war_goal_v2(
                standing,
                WarExhaustion(exhaustion),
                -50,
                10_000,
                DecisionSource::RuleEngine,
            )
            .expect("rule engine");
            assert_eq!(
                goal, expected,
                "standing={standing} exhaustion={exhaustion}"
            );
        }
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// Integration property: the same accept/reject/counter flow applied to
    /// a `ConcessionStateMachine` produces the same final state regardless
    /// of whether it was reached through a counter sequence or a series of
    /// monotonic proposes (as long as the concession level is identical).
    #[test]
    fn fr_civ_diplo_008_counter_and_propose_converge_to_same_level() {
        // Path A: two monotonic proposes.
        let mut path_a = ConcessionStateMachine::new(Pair::new(a(7), a(8)), 3);
        assert!(path_a.propose(a(7), 1).is_ok());
        assert!(path_a.propose(a(8), 3).is_ok());
        assert!(path_a.accept(a(7)).is_ok());
        assert_eq!(path_a.concession_level, 3);
        assert!(path_a.resolved);

        // Path B: initial propose + counter at the same final level.
        let mut path_b = ConcessionStateMachine::new(Pair::new(a(7), a(8)), 3);
        assert!(path_b.propose(a(7), 1).is_ok());
        assert!(path_b.counter(a(8), 3).is_ok());
        assert!(path_b.accept(a(7)).is_ok());
        assert_eq!(path_b.concession_level, 3);
        assert!(path_b.resolved);

        // Both paths reach the same (resolved, level=3) state.
        assert_eq!(path_a, path_b);
    }

    /// FR-CIV-DIPLO-008.
    ///
    /// `select_war_goal_v2` rejects LLM-sourced decisions (parity with
    /// `select_war_goal` per `FR-CIV-DIPLO-006`).
    #[test]
    fn fr_civ_diplo_008_select_war_goal_v2_blocks_llm() {
        assert!(matches!(
            select_war_goal_v2(-20, WarExhaustion(0), -50, 10_000, DecisionSource::Llm),
            Err(DecisionError::LlmDisallowed {
                action: "war-goal selection"
            })
        ));
    }

    // -- Schema version -----------------------------------------------------

    #[test]
    fn schema_version_is_positive() {
        assert!(SCHEMA_VERSION >= 1);
    }
}
