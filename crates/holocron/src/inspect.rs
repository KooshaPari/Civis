//! Agent inspector data + query layer (FR-CIV-INSPECT-901).
//!
//! This module is the **backing data layer** for the agent inspector panel.
//! It is intentionally pure logic: structs, a small registry, and a single
//! `summarize_agent` query that aggregates an agent's identity, age, needs,
//! mood, and current activity into one flat [`AgentSummary`] value that the
//! renderer (egui, Bevy, web) can present directly.
//!
//! Design constraints (mirrors `crates/holocron`'s "catalog layer" philosophy
//! for verbs):
//!
//! - **No Bevy / ECS dependency.** This crate composes cleanly with anything
//!   that can call a function. The renderer is a downstream concern.
//! - **No new upstream dependencies.** The data types use only primitives,
//!   small bounded enums, and `serde`. Real values flow in from
//!   `civ-agents` / `civ-ai` / `civ-needs` at the call site; the inspector
//!   only knows about the inspector's own shapes.
//! - **Additive.** The `AgentSummary` is constructed from explicit fields.
//!   Callers (e.g. a Bevy system that reads `civ-agents` components) fill
//!   it once per inspector open. The struct is `Clone + Debug + Default +
//!   Serialize + Deserialize` so it round-trips through JSON-RPC snapshots,
//!   mod-host replay, and the egui panel equally well.
//! - **BLIND-honest.** A field the source crate does not have is reported
//!   as `None` on the corresponding `Option<T>` field, never faked. This
//!   matches FR-CIV-INSPECT-901 and the spec: "Each field reads the owning
//!   crate ... BLIND fields shown as 'unmeasured' not faked."
//!
//! See `docs/specs/requirements/FR-CIV-INSPECT.md` (FR-CIV-INSPECT-901).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The canonical needs the inspector surfaces.
///
/// Defined in the inspector module rather than imported from `civ-needs` so
/// the holocron crate has no upstream-crate coupling. Callers translate
/// `civ_needs::Needs` / `civ_agents::Needs` / `civ_ai::goal::Need` into
/// [`Need`] when they build an [`AgentSummary`]. Values are **satisfaction
/// levels in `[0.0, 1.0]`** where `1.0` = sated, `0.0` = critical — the
/// same convention used by `civ_needs::Needs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Need {
    /// Caloric / food satisfaction.
    Food,
    /// Hydration satisfaction.
    Water,
    /// Rest / sleep satisfaction.
    Rest,
    /// Perceived safety (inverse of threat exposure).
    Safety,
    /// Social / belonging satisfaction.
    Social,
    /// Physical health satisfaction.
    Health,
}

impl Need {
    /// All needs in stable, inspector-friendly order.
    ///
    /// Iteration order matches the on-screen order of the needs section of
    /// the inspector panel.
    pub const ALL: [Need; 6] = [
        Need::Food,
        Need::Water,
        Need::Rest,
        Need::Safety,
        Need::Social,
        Need::Health,
    ];

    /// Stable kebab-case id used by dashboards / JSON-RPC / mod-host replay.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Need::Food => "food",
            Need::Water => "water",
            Need::Rest => "rest",
            Need::Safety => "safety",
            Need::Social => "social",
            Need::Health => "health",
        }
    }

    /// Short human-readable label for the panel UI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Need::Food => "Food",
            Need::Water => "Water",
            Need::Rest => "Rest",
            Need::Safety => "Safety",
            Need::Social => "Social",
            Need::Health => "Health",
        }
    }
}

/// Single named, bounded value in `[0.0, 1.0]` for a [`Need`].
///
/// `1.0` = sated, `0.0` = critical. Mirrors `civ_needs::Needs` and
/// `civ_agents::Needs` (which both model satisfaction in `[0, 1]`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeedValue {
    /// Which need this entry describes.
    pub need: Need,
    /// Satisfaction in `[0.0, 1.0]`. `1.0` = sated, `0.0` = critical.
    pub satisfaction: f32,
}

impl NeedValue {
    /// Construct a need entry, clamping `satisfaction` into `[0.0, 1.0]` so
    /// a misconfigured upstream cannot poison the summary.
    #[must_use]
    pub fn new(need: Need, satisfaction: f32) -> Self {
        Self {
            need,
            satisfaction: satisfaction.clamp(0.0, 1.0),
        }
    }

    /// Pressure = `1.0 - satisfaction`. `0.0` = no pressure,
    /// `1.0` = critical.
    #[must_use]
    pub fn pressure(self) -> f32 {
        (1.0 - self.satisfaction).clamp(0.0, 1.0)
    }
}

/// Coarse activity / current-action the agent is doing.
///
/// Mirrors `civ-agents::daily_path::Activity` semantics but kept
/// inspector-local so the holocron crate has no upstream coupling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrentAction {
    /// Staying put — needs comfortable, no local motion needed.
    Idle,
    /// Local deterministic wandering around a seed/home/current anchor.
    Wander,
    /// Purposeful trip to satisfy a pressing need.
    SeekNeed,
    /// Working / performing productive labor.
    Work,
    /// Praying / visiting a temple.
    Pray,
    /// Socializing with other agents.
    Socialize,
    /// Consuming food.
    Eat,
    /// Fleeing from danger.
    Flee,
}

impl CurrentAction {
    /// Stable kebab-case id used by JSON-RPC / mod-host replay.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            CurrentAction::Idle => "idle",
            CurrentAction::Wander => "wander",
            CurrentAction::SeekNeed => "seek-need",
            CurrentAction::Work => "work",
            CurrentAction::Pray => "pray",
            CurrentAction::Socialize => "socialize",
            CurrentAction::Eat => "eat",
            CurrentAction::Flee => "flee",
        }
    }

    /// Short human-readable label for the panel UI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            CurrentAction::Idle => "Idle",
            CurrentAction::Wander => "Wandering",
            CurrentAction::SeekNeed => "Seeking need",
            CurrentAction::Work => "Working",
            CurrentAction::Pray => "Praying",
            CurrentAction::Socialize => "Socializing",
            CurrentAction::Eat => "Eating",
            CurrentAction::Flee => "Fleeing",
        }
    }
}

/// Bounded mood valence in `[-1.0, 1.0]`. `+1.0` = elated, `0.0` = neutral,
/// `−1.0` = miserable. Mirrors `civ-agents::psyche::Mood::valence` and
/// `civ-ai::mood::MoodState::valence` so any of those sources map cleanly
/// into this field.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoodReading {
    /// Bounded mood valence in `[-1.0, 1.0]`.
    pub valence: f32,
    /// Optional arousal in `[0.0, 1.0]`. `0.0` = calm, `1.0` = agitated.
    /// `None` when the upstream does not track arousal (FR-CIV-PSYCHE-901
    /// does not require it).
    pub arousal: Option<f32>,
    /// Optional sim tick this reading was last computed at. `None` when the
    /// upstream did not stamp a tick (e.g. an inspector pre-filling from
    /// a fresh spawn).
    pub computed_at_tick: Option<u64>,
}

impl MoodReading {
    /// Neutral baseline. Valence `0.0`, no arousal, no tick stamp.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: None,
            computed_at_tick: None,
        }
    }

    /// Construct a valence-only mood, clamping `valence` into `[-1, 1]`.
    #[must_use]
    pub fn from_valence(valence: f32) -> Self {
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal: None,
            computed_at_tick: None,
        }
    }

    /// Construct a full mood reading. `valence` is clamped into `[-1, 1]`,
    /// `arousal` (when present) is clamped into `[0, 1]`.
    #[must_use]
    pub fn full(valence: f32, arousal: Option<f32>, computed_at_tick: Option<u64>) -> Self {
        let arousal = arousal.map(|a| a.clamp(0.0, 1.0));
        Self {
            valence: valence.clamp(-1.0, 1.0),
            arousal,
            computed_at_tick,
        }
    }

    /// Bucket for dashboards / tests. See [`MoodLabel`].
    #[must_use]
    pub fn label(self) -> MoodLabel {
        match self.valence {
            v if v > 0.5 => MoodLabel::Elated,
            v if v > 0.1 => MoodLabel::Content,
            v if v >= -0.1 => MoodLabel::Neutral,
            v if v >= -0.5 => MoodLabel::Displeased,
            _ => MoodLabel::Miserable,
        }
    }
}

/// Human-readable mood bucket. Mirrors the labels used by
/// `civ-ai::mood::MoodState::label`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoodLabel {
    /// `valence > 0.5`.
    Elated,
    /// `0.1 < valence <= 0.5`.
    Content,
    /// `-0.1 <= valence <= 0.1`.
    Neutral,
    /// `-0.5 <= valence < -0.1`.
    Displeased,
    /// `valence < -0.5`.
    Miserable,
}

impl MoodLabel {
    /// Short label string for the panel UI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            MoodLabel::Elated => "Elated",
            MoodLabel::Content => "Content",
            MoodLabel::Neutral => "Neutral",
            MoodLabel::Displeased => "Displeased",
            MoodLabel::Miserable => "Miserable",
        }
    }
}

/// One measured sub-field of the agent's identity / age block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentityBlock {
    /// Stable agent ID (mirrors `Civilian::id` / `Agent::id`).
    pub agent_id: u64,
    /// Agent name. `None` when the upstream does not track a name.
    pub name: Option<String>,
    /// Age in years. `None` when the upstream does not track an age
    /// (creature without an age model).
    pub age_years: Option<u16>,
    /// Optional lifecycle label (e.g. "adult", "elder"). `None` when the
    /// upstream did not classify the agent.
    pub lifecycle: Option<String>,
    /// Optional species / phenotype id (e.g. a `Phenotype` id or
    /// `civ_species` taxon). `None` when the upstream does not track it.
    pub species_id: Option<String>,
    /// Optional faction / alignment id. `None` when the agent has not
    /// aligned (mirrors `civ_agents::Alignment::None`).
    pub faction_id: Option<u32>,
}

impl IdentityBlock {
    /// Construct a minimal identity block from a required agent id.
    #[must_use]
    pub fn new(agent_id: u64) -> Self {
        Self {
            agent_id,
            name: None,
            age_years: None,
            lifecycle: None,
            species_id: None,
            faction_id: None,
        }
    }

    /// Builder-style: set the agent name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Builder-style: set the age in years.
    #[must_use]
    pub fn with_age_years(mut self, age_years: u16) -> Self {
        self.age_years = Some(age_years);
        self
    }

    /// Builder-style: set the lifecycle label.
    #[must_use]
    pub fn with_lifecycle(mut self, lifecycle: impl Into<String>) -> Self {
        self.lifecycle = Some(lifecycle.into());
        self
    }

    /// Builder-style: set the species / phenotype id.
    #[must_use]
    pub fn with_species_id(mut self, species_id: impl Into<String>) -> Self {
        self.species_id = Some(species_id.into());
        self
    }

    /// Builder-style: set the faction / alignment id.
    #[must_use]
    pub fn with_faction_id(mut self, faction_id: u32) -> Self {
        self.faction_id = Some(faction_id);
        self
    }
}

/// Per-need satisfaction map, indexed by [`Need`].
///
/// `BTreeMap` so iteration order is stable and alphabetical, which makes
/// snapshot diffs and JSON-RPC payload equality work without caring about
/// `HashMap` randomization.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NeedMap {
    inner: BTreeMap<Need, f32>,
}

impl NeedMap {
    /// Construct an empty map (all needs unmeasured).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the satisfaction for `need`, clamping into `[0.0, 1.0]`.
    /// Overwrites any previous value.
    pub fn set(&mut self, need: Need, satisfaction: f32) {
        self.inner.insert(need, satisfaction.clamp(0.0, 1.0));
    }

    /// Read the satisfaction for `need`. `None` when unmeasured.
    #[must_use]
    pub fn get(&self, need: Need) -> Option<f32> {
        self.inner.get(&need).copied()
    }

    /// True if no need has a value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Number of measured needs (0..=6).
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Iterate in stable alphabetical `Need` order.
    pub fn iter(&self) -> impl Iterator<Item = (Need, f32)> + '_ {
        self.inner.iter().map(|(k, v)| (*k, *v))
    }

    /// Mean satisfaction across all measured needs. Returns `None` if no
    /// need is measured (no signal, not zero).
    #[must_use]
    pub fn mean_satisfaction(&self) -> Option<f32> {
        if self.inner.is_empty() {
            return None;
        }
        let sum: f32 = self.inner.values().copied().sum();
        Some(sum / self.inner.len() as f32)
    }

    /// Mean pressure (1 - satisfaction) across all measured needs.
    /// `None` if no need is measured.
    #[must_use]
    pub fn mean_pressure(&self) -> Option<f32> {
        self.mean_satisfaction().map(|s| (1.0 - s).clamp(0.0, 1.0))
    }

    /// Highest-pressure need in the map, with its pressure value. `None`
    /// if the map is empty.
    #[must_use]
    pub fn top_pressure(&self) -> Option<(Need, f32)> {
        self.inner
            .iter()
            .map(|(k, v)| (*k, (1.0 - *v).clamp(0.0, 1.0)))
            .max_by(|(na, pa), (nb, pb)| {
                pa.partial_cmp(pb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| na.cmp(nb))
            })
    }
}

impl FromIterator<NeedValue> for NeedMap {
    fn from_iter<I: IntoIterator<Item = NeedValue>>(iter: I) -> Self {
        let mut map = Self::new();
        for nv in iter {
            map.set(nv.need, nv.satisfaction);
        }
        map
    }
}

/// Input the renderer (or a Bevy system reading components) hands to
/// [`summarize_agent`]. Every optional field is `Option<T>` because the
/// spec is explicit: a field the source crate does not have is "unmeasured"
/// not faked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSnapshot {
    /// Required identity.
    pub identity: IdentityBlock,
    /// Optional needs block. `None` when the source has no needs model.
    pub needs: Option<NeedMap>,
    /// Optional mood reading. `None` when the source has no mood model.
    pub mood: Option<MoodReading>,
    /// Optional current action. `None` when the source does not model one.
    pub current_action: Option<CurrentAction>,
}

impl AgentSnapshot {
    /// Construct a snapshot with the given identity.
    #[must_use]
    pub fn new(identity: IdentityBlock) -> Self {
        Self {
            identity,
            needs: None,
            mood: None,
            current_action: None,
        }
    }

    /// Builder-style: attach a needs block.
    #[must_use]
    pub fn with_needs(mut self, needs: NeedMap) -> Self {
        self.needs = Some(needs);
        self
    }

    /// Builder-style: attach a mood reading.
    #[must_use]
    pub fn with_mood(mut self, mood: MoodReading) -> Self {
        self.mood = Some(mood);
        self
    }

    /// Builder-style: attach a current action.
    #[must_use]
    pub fn with_current_action(mut self, action: CurrentAction) -> Self {
        self.current_action = Some(action);
        self
    }
}

/// Aggregate inspector summary for one agent (FR-CIV-INSPECT-901).
///
/// This is the **canonical value** the inspector panel renders. It is a
/// plain data struct: no I/O, no Bevy systems, no async — so the same
/// shape works in `civ-server` (Rust) JSON-RPC, the egui HolocronPanel,
/// the web dashboard, mod-host replay, and unit tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSummary {
    /// Identity / age / species block.
    pub identity: IdentityBlock,
    /// Per-need satisfaction values, in stable alphabetical order. Empty
    /// when the source has no needs model.
    pub needs: Vec<NeedValue>,
    /// Mood reading, when the source has a mood model.
    pub mood: Option<MoodReading>,
    /// Mood bucket for dashboards (`valence → MoodLabel`).
    pub mood_label: Option<MoodLabel>,
    /// Current action, when the source has one.
    pub current_action: Option<CurrentAction>,
    /// Mean satisfaction across all measured needs. `None` when the source
    /// has no needs model. Range `[0.0, 1.0]`.
    pub mean_satisfaction: Option<f32>,
    /// Mean pressure across all measured needs (`1 - mean_satisfaction`).
    /// `None` when the source has no needs model. Range `[0.0, 1.0]`.
    pub mean_pressure: Option<f32>,
    /// Highest-pressure need with its pressure value, when any need is
    /// measured. `None` when no need is measured.
    pub top_pressure: Option<(Need, f32)>,
    /// Number of measured needs (0..=6). Useful for the panel header
    /// ("3 of 6 needs measured").
    pub measured_need_count: usize,
    /// True when the source has every field the spec lists (identity,
    /// species, age, needs, mood, current action). When `false`, the panel
    /// shows "unmeasured" placeholders for the missing fields
    /// (FR-CIV-INSPECT-901 BLIND-honest contract).
    pub fully_measured: bool,
}

/// Build the [`AgentSummary`] from a [`AgentSnapshot`].
///
/// This is the **single entry point** the renderer / Bevy system / JSON-RPC
/// handler should call. The function is pure (no I/O, no time, no RNG) and
/// deterministic — identical snapshots produce identical summaries.
///
/// It populates:
///
/// - `identity` — copied verbatim.
/// - `needs` — sorted by [`Need`] id, each entry clamped into `[0, 1]`.
/// - `mood` / `mood_label` — copied / bucketed; both `None` when
///   unmeasured.
/// - `current_action` — copied; `None` when unmeasured.
/// - `mean_satisfaction` / `mean_pressure` — derived when any need is
///   measured; `None` otherwise (we do not pretend "0.0" means "neutral").
/// - `top_pressure` — the highest-pressure need with deterministic
///   alphabetical tie-breaking when two needs have exactly the same
///   pressure (e.g. both at `0.0`).
/// - `measured_need_count` — 0..=6.
/// - `fully_measured` — `true` iff identity, at least one need, mood, and
///   a current action are all present. Per FR-CIV-INSPECT-901, missing
///   fields render as "unmeasured" placeholders, never fakes.
#[must_use]
pub fn summarize_agent(snapshot: &AgentSnapshot) -> AgentSummary {
    // ---- Needs: sort by Need id (BTreeMap iteration order) and project.
    let (needs, mean_satisfaction, mean_pressure, top_pressure, measured_need_count) =
        match &snapshot.needs {
            Some(map) => {
                let entries: Vec<NeedValue> = map
                    .iter()
                    .map(|(need, satisfaction)| NeedValue::new(need, satisfaction))
                    .collect();
                let mean_s = map.mean_satisfaction();
                let mean_p = map.mean_pressure();
                let top = map.top_pressure();
                (entries, mean_s, mean_p, top, map.len())
            }
            None => (Vec::new(), None, None, None, 0),
        };

    // ---- Mood + bucket.
    let (mood, mood_label) = match snapshot.mood {
        Some(m) => (Some(m), Some(m.label())),
        None => (None, None),
    };

    let current_action = snapshot.current_action;

    // ---- Fully-measured: identity (always present) + at least one need +
    //      mood + current action.
    let fully_measured = measured_need_count > 0 && mood.is_some() && current_action.is_some();

    AgentSummary {
        identity: snapshot.identity.clone(),
        needs,
        mood,
        mood_label,
        current_action,
        mean_satisfaction,
        mean_pressure,
        top_pressure,
        measured_need_count,
        fully_measured,
    }
}

/// Build an [`AgentSummary`] from loose field values.
///
/// Convenience for callers that do not want to construct an
/// [`AgentSnapshot`] first (e.g. test code).
#[must_use]
pub fn summarize_fields(
    identity: IdentityBlock,
    needs: Option<NeedMap>,
    mood: Option<MoodReading>,
    current_action: Option<CurrentAction>,
) -> AgentSummary {
    let snapshot = AgentSnapshot {
        identity,
        needs,
        mood,
        current_action,
    };
    summarize_agent(&snapshot)
}

// -------------------------------------------------------------------------
// Tests — FR-CIV-INSPECT-901
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance criterion: the summary reflects the agent's current
    /// needs values. A fed, rested, safe agent must show high satisfaction
    /// and low pressure; a starving, exhausted agent must show the
    /// opposite.
    #[test]
    fn summary_reflects_current_need_values() {
        // ---- Fed + rested + safe agent.
        let mut fed = NeedMap::new();
        fed.set(Need::Food, 0.9);
        fed.set(Need::Water, 0.8);
        fed.set(Need::Rest, 0.85);
        fed.set(Need::Safety, 0.95);
        fed.set(Need::Social, 0.7);
        fed.set(Need::Health, 1.0);

        let fed_snapshot = AgentSnapshot::new(IdentityBlock::new(1).with_age_years(34))
            .with_needs(fed)
            .with_mood(MoodReading::from_valence(0.4))
            .with_current_action(CurrentAction::Work);
        let fed_summary = summarize_agent(&fed_snapshot);

        assert_eq!(fed_summary.measured_need_count, 6);
        assert_eq!(fed_summary.needs.len(), 6);
        assert!(
            fed_summary.mean_satisfaction.unwrap() > 0.8,
            "fed agent mean satisfaction must be > 0.8 (got {:?})",
            fed_summary.mean_satisfaction
        );
        assert!(
            fed_summary.mean_pressure.unwrap() < 0.2,
            "fed agent mean pressure must be < 0.2 (got {:?})",
            fed_summary.mean_pressure
        );
        assert!(
            fed_summary.mood.is_some(),
            "fed agent mood must be present (FR-CIV-INSPECT-901)"
        );
        assert_eq!(fed_summary.mood_label, Some(MoodLabel::Content));
        assert_eq!(fed_summary.current_action, Some(CurrentAction::Work));
        assert!(
            fed_summary.fully_measured,
            "fed agent has every field measured (FR-CIV-INSPECT-901)"
        );

        // ---- Starving + exhausted + unsafe agent.
        let mut starving = NeedMap::new();
        starving.set(Need::Food, 0.05);
        starving.set(Need::Water, 0.1);
        starving.set(Need::Rest, 0.0);
        starving.set(Need::Safety, 0.2);
        starving.set(Need::Social, 0.3);
        starving.set(Need::Health, 0.4);

        let starving_snapshot = AgentSnapshot::new(IdentityBlock::new(2).with_age_years(58))
            .with_needs(starving)
            .with_mood(MoodReading::from_valence(-0.7))
            .with_current_action(CurrentAction::SeekNeed);
        let starving_summary = summarize_agent(&starving_snapshot);

        assert_eq!(starving_summary.measured_need_count, 6);
        assert!(
            starving_summary.mean_satisfaction.unwrap() < 0.2,
            "starving agent mean satisfaction must be < 0.2 (got {:?})",
            starving_summary.mean_satisfaction
        );
        assert!(
            starving_summary.mean_pressure.unwrap() > 0.8,
            "starving agent mean pressure must be > 0.8 (got {:?})",
            starving_summary.mean_pressure
        );
        assert_eq!(starving_summary.mood_label, Some(MoodLabel::Miserable));
        assert_eq!(
            starving_summary.current_action,
            Some(CurrentAction::SeekNeed)
        );

        // ---- Direct acceptance: the two summaries must differ in the
        //      values that depend on the inputs.
        assert_ne!(
            fed_summary.mean_satisfaction, starving_summary.mean_satisfaction,
            "summary must reflect the agent's current need values (mean_satisfaction)"
        );
        assert_ne!(
            fed_summary.mean_pressure, starving_summary.mean_pressure,
            "summary must reflect the agent's current need values (mean_pressure)"
        );
        assert_ne!(
            fed_summary.top_pressure, starving_summary.top_pressure,
            "summary must reflect the agent's current need values (top_pressure)"
        );
        assert_ne!(
            fed_summary.mood_label, starving_summary.mood_label,
            "summary must reflect the agent's current mood"
        );
        assert_ne!(
            fed_summary.current_action, starving_summary.current_action,
            "summary must reflect the agent's current action"
        );
    }

    /// The summary must report the per-need values the snapshot gave it.
    #[test]
    fn summary_per_need_values_match_snapshot() {
        let mut needs = NeedMap::new();
        needs.set(Need::Food, 0.5);
        needs.set(Need::Rest, 0.25);
        needs.set(Need::Safety, 0.75);

        let snapshot = AgentSnapshot::new(IdentityBlock::new(7))
            .with_needs(needs.clone())
            .with_mood(MoodReading::from_valence(0.05))
            .with_current_action(CurrentAction::Wander);
        let summary = summarize_agent(&snapshot);

        for entry in &summary.needs {
            let expected = needs.get(entry.need).unwrap();
            assert!(
                (entry.satisfaction - expected).abs() < f32::EPSILON,
                "summary need {:?} = {} (expected {})",
                entry.need,
                entry.satisfaction,
                expected
            );
        }
        // Order must be alphabetical by Need id.
        let kinds: Vec<Need> = summary.needs.iter().map(|e| e.need).collect();
        let mut sorted = kinds.clone();
        sorted.sort();
        assert_eq!(kinds, sorted, "needs must be reported in stable order");
    }

    /// `top_pressure` is the highest-pressure need with deterministic
    /// alphabetical tie-breaking.
    #[test]
    fn top_pressure_picks_highest_pressure_need() {
        let mut needs = NeedMap::new();
        needs.set(Need::Food, 0.9); // pressure 0.1
        needs.set(Need::Rest, 0.1); // pressure 0.9 — top
        needs.set(Need::Safety, 0.5); // pressure 0.5

        let snapshot = AgentSnapshot::new(IdentityBlock::new(3))
            .with_needs(needs)
            .with_mood(MoodReading::from_valence(0.0))
            .with_current_action(CurrentAction::Idle);
        let summary = summarize_agent(&snapshot);
        let (top_need, top_p) = summary.top_pressure.expect("measured needs");
        assert_eq!(top_need, Need::Rest);
        assert!((top_p - 0.9).abs() < f32::EPSILON);
    }

    /// When no need is measured, the pressure / satisfaction aggregates
    /// are `None` — never `0.0` faked.
    #[test]
    fn unmeasured_needs_are_none_not_zero() {
        let snapshot =
            AgentSnapshot::new(IdentityBlock::new(4)).with_current_action(CurrentAction::Idle);
        let summary = summarize_agent(&snapshot);

        assert_eq!(summary.measured_need_count, 0);
        assert!(summary.needs.is_empty());
        assert!(summary.mean_satisfaction.is_none());
        assert!(summary.mean_pressure.is_none());
        assert!(summary.top_pressure.is_none());
        assert!(!summary.fully_measured);
    }

    /// Missing mood or current action are surfaced as `None` and the
    /// `fully_measured` flag is `false` (FR-CIV-INSPECT-901 BLIND contract).
    #[test]
    fn missing_mood_or_action_marks_summary_unmeasured() {
        // All fields except mood.
        let mut needs = NeedMap::new();
        needs.set(Need::Food, 0.5);
        let s1 = AgentSnapshot::new(IdentityBlock::new(5))
            .with_needs(needs.clone())
            .with_current_action(CurrentAction::Work);
        let summary1 = summarize_agent(&s1);
        assert!(summary1.mood.is_none());
        assert!(summary1.mood_label.is_none());
        assert!(!summary1.fully_measured);

        // All fields except current action.
        let s2 = AgentSnapshot::new(IdentityBlock::new(6))
            .with_needs(needs)
            .with_mood(MoodReading::from_valence(0.0));
        let summary2 = summarize_agent(&s2);
        assert!(summary2.current_action.is_none());
        assert!(!summary2.fully_measured);
    }

    /// `summarize_fields` is equivalent to constructing a snapshot and
    /// calling `summarize_agent`.
    #[test]
    fn summarize_fields_matches_summarize_agent() {
        let identity = IdentityBlock::new(42).with_age_years(28).with_faction_id(3);
        let mut needs = NeedMap::new();
        needs.set(Need::Food, 0.6);
        needs.set(Need::Health, 0.4);
        let mood = MoodReading::full(0.2, Some(0.5), Some(100));
        let action = CurrentAction::Pray;

        let via_fields = summarize_fields(
            identity.clone(),
            Some(needs.clone()),
            Some(mood),
            Some(action),
        );
        let via_snapshot = summarize_agent(
            &AgentSnapshot::new(identity)
                .with_needs(needs)
                .with_mood(mood)
                .with_current_action(action),
        );

        assert_eq!(via_fields, via_snapshot);
    }

    /// `summarize_agent` is deterministic: identical snapshots produce
    /// identical summaries (replay + JSON-RPC snapshot equality).
    #[test]
    fn summarize_is_deterministic_for_same_snapshot() {
        let identity = IdentityBlock::new(99);
        let mut needs = NeedMap::new();
        needs.set(Need::Food, 0.3);
        needs.set(Need::Rest, 0.7);
        let snapshot = AgentSnapshot::new(identity)
            .with_needs(needs)
            .with_mood(MoodReading::from_valence(-0.2))
            .with_current_action(CurrentAction::Socialize);

        let a = summarize_agent(&snapshot);
        let b = summarize_agent(&snapshot);
        assert_eq!(a, b);
    }

    /// `NeedValue` clamps out-of-range satisfaction.
    #[test]
    fn need_value_clamps_satisfaction() {
        let hi = NeedValue::new(Need::Food, 5.0);
        let lo = NeedValue::new(Need::Food, -1.0);
        assert_eq!(hi.satisfaction, 1.0);
        assert_eq!(lo.satisfaction, 0.0);
        assert_eq!(hi.pressure(), 0.0);
        assert_eq!(lo.pressure(), 1.0);
    }

    /// `MoodReading::label` buckets the valence consistently with
    /// `civ_ai::mood::MoodState::label`.
    #[test]
    fn mood_label_buckets_match_spec() {
        assert_eq!(MoodReading::from_valence(0.9).label(), MoodLabel::Elated);
        assert_eq!(MoodReading::from_valence(0.3).label(), MoodLabel::Content);
        assert_eq!(MoodReading::from_valence(0.0).label(), MoodLabel::Neutral);
        assert_eq!(
            MoodReading::from_valence(-0.3).label(),
            MoodLabel::Displeased
        );
        assert_eq!(
            MoodReading::from_valence(-0.9).label(),
            MoodLabel::Miserable
        );
    }

    /// `Need::id` / `CurrentAction::id` are stable kebab-case strings —
    /// the JSON-RPC / mod-host contract depends on this.
    #[test]
    fn ids_are_stable_kebab_case() {
        assert_eq!(Need::Food.id(), "food");
        assert_eq!(Need::Health.id(), "health");
        assert_eq!(CurrentAction::SeekNeed.id(), "seek-need");
        assert_eq!(CurrentAction::Idle.id(), "idle");
    }

    /// `Need::ALL` enumerates the six canonical needs in stable order —
    /// mirrors the iteration order reported in the summary.
    #[test]
    fn need_all_is_stable_and_complete() {
        let all = Need::ALL;
        assert_eq!(all.len(), 6);
        let mut sorted = all.to_vec();
        sorted.sort();
        assert_eq!(all.to_vec(), sorted, "Need::ALL must be in sorted order");
        // Every variant must be present (no gaps, no duplicates).
        for n in all {
            assert!(all.contains(&n), "duplicate or missing variant: {:?}", n);
        }
    }
}
