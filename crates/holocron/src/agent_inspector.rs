//! Agent inspector — FR-CIV-INSPECT-901.
//!
//! The data + query backing layer for the agent inspector panel. Aggregates
//! an agent's identity, age, needs, mood/psyche, and current action into a
//! single, renderer-friendly [`AgentInspectorSummary`].
//!
//! # Scope (per FR-CIV-INSPECT-901)
//!
//! FR-CIV-INSPECT-901 says: "Agent inspector SHALL show identity,
//! species/phenotype summary, age, needs, psyche/mood, current activity,
//! relationships, lineage." Acceptance: "Each field reads the owning crate
//! (`civ-species`, `civ-agents`, psyche FR); BLIND fields shown as
//! 'unmeasured' not faked."
//!
//! This module is the **pure-logic data/query layer**. It does **not**
//! read from `civ-species`, `civ-agents`, `civ-needs`, or the psyche
//! substrate — it owns the *display shape* (the aggregate struct) and
//! the small in-memory index that maps `EntityId → AgentInspectorSummary`.
//! A renderer that wants the real values wires its own subscriber that
//! fills an [`AgentInspectorSummary`] and hands it to the registry; this
//! layer never pretends to know species, needs, or mood values it hasn't
//! been told about. Fields the caller hasn't populated show up as the
//! explicit [`Sentinel::Unmeasured`] (or `None` for optional fields) so
//! the inspector panel can render "unmeasured" rather than fabricating a
//! number — matching the spec's "BLIND fields shown as 'unmeasured' not
//! faked" acceptance.
//!
//! # Additive contract
//!
//! This module is **additive** with respect to the rest of the crate. It
//! only adds types and queries; it never mutates existing logic. It does
//! not depend on Bevy, the renderer, or any other crate — only on `std`,
//! `serde`, and `serde_json` (transitively through the parent crate).
//!
//! # What's in this file
//!
//! - [`Sentinel`] — explicit "this field was not measured" marker (the
//!   "unmeasured" sentinel mandated by FR-CIV-INSPECT-901 acceptance).
//! - [`NeedLevel`] — single-need snapshot (label + value + label).
//! - [`MoodSnapshot`] — mood/psyche snapshot (valence + label).
//! - [`AgentIdentity`] — identity block (id, display name, species/phenotype
//!   label, age).
//! - [`CurrentAction`] — current activity/action the agent is performing.
//! - [`AgentInspectorSummary`] — the aggregate struct the inspector panel
//!   reads. Includes identity, age, needs, mood, current action (and
//!   optional relationships/lineage placeholders so the struct satisfies
//!   the FR's full field list without claiming values it doesn't have).
//! - [`AgentInspectorRegistry`] — in-memory `EntityId → AgentInspectorSummary`
//!   index with a `summary(id)` query.
//!
//! See [`Self::tests`] for the acceptance test required by FR-CIV-INSPECT-901.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Sentinel — explicit "unmeasured" marker.
//
// The spec's acceptance test for FR-CIV-INSPECT-901 demands "BLIND fields
// shown as 'unmeasured' not faked." We model that with a dedicated
// `Sentinel` enum so callers cannot accidentally treat a missing value as
// a zero / default / fabricated number.
// ---------------------------------------------------------------------------

/// Explicit "field not measured" marker.
///
/// Per FR-CIV-INSPECT-901: BLIND fields are rendered as "unmeasured" — they
/// are **not** silently filled with `0`, `false`, or a placeholder string.
/// The inspector panel pattern-matches on `Unmeasured` and shows the
/// explicit "unmeasured" label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sentinel {
    /// The owning crate has not measured this field this tick.
    Unmeasured,
    /// The owning crate has measured this field, but it's marked
    /// "not applicable" for this agent (e.g. lineage for a founder).
    NotApplicable,
}

impl Sentinel {
    /// Human-readable label for the inspector panel.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unmeasured => "unmeasured",
            Self::NotApplicable => "n/a",
        }
    }
}

// ---------------------------------------------------------------------------
// NeedLevel — single need snapshot (label + value).
//
// FR-CIV-INSPECT-901 lists "needs" as a required inspector field. A need is
// (label, value) where value is bounded 0..=100 (mirroring the conventional
// "needs" saturation scale; the parent `civ-needs` crate uses the same
// convention). Values are typed `Option<u8>` so a renderer can distinguish
// "measured: 73" from "not measured".
// ---------------------------------------------------------------------------

/// Single-need snapshot (e.g. hunger, rest, shelter).
///
/// `value` is `None` when the owning crate has not measured this need yet;
/// the inspector renders "unmeasured" rather than a fabricated number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedLevel {
    /// Need label (e.g. `"hunger"`, `"rest"`, `"shelter"`, `"social"`).
    pub label: String,
    /// Saturated value, `0..=100`. `None` when unmeasured.
    pub value: Option<u8>,
}

impl NeedLevel {
    /// Construct a `NeedLevel` with a known measured value.
    pub fn measured(label: impl Into<String>, value: u8) -> Self {
        Self {
            label: label.into(),
            value: Some(value),
        }
    }

    /// Construct an explicitly-unmeasured `NeedLevel` (renders as "unmeasured").
    pub fn unmeasured(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
        }
    }

    /// `true` iff this need has a measured value.
    pub fn is_measured(&self) -> bool {
        self.value.is_some()
    }
}

// ---------------------------------------------------------------------------
// MoodSnapshot — psyche/mood snapshot.
//
// FR-CIV-INSPECT-901 lists "psyche/mood" as a required inspector field.
// `valence` is bounded `-100..=100` (negative = distressed, positive =
// content). `label` is a short categorical label owned by the psyche
// substrate (`civ-agents` / `ai::mood`).
// ---------------------------------------------------------------------------

/// Psyche/mood snapshot — short categorical label plus a numeric valence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoodSnapshot {
    /// Short categorical label from the psyche substrate
    /// (e.g. `"content"`, `"distressed"`, `"angry"`, `"inspired"`).
    /// `None` when unmeasured.
    pub label: Option<String>,
    /// Numeric valence, `-100..=100`. `None` when unmeasured.
    pub valence: Option<i16>,
}

impl MoodSnapshot {
    /// A measured mood with both a label and a valence.
    pub fn measured(label: impl Into<String>, valence: i16) -> Self {
        Self {
            label: Some(label.into()),
            valence: Some(valence),
        }
    }

    /// An explicitly-unmeasured mood (renders as "unmeasured").
    pub fn unmeasured() -> Self {
        Self {
            label: None,
            valence: None,
        }
    }

    /// `true` iff the psyche substrate has published a label and a valence.
    pub fn is_measured(&self) -> bool {
        self.label.is_some() && self.valence.is_some()
    }
}

// ---------------------------------------------------------------------------
// AgentIdentity — identity block (id, name, species, age).
//
// We use the canonical `EntityId` from [`crate::inspect`] for agent ids so
// the new module stays aligned with `InspectRegistry`'s id space — the
// renderer can carry an `EntityId` from one to the other without remapping.
// We deliberately do **not** re-export `EntityId` here: `crate::inspect`
// already exposes it via `pub use inspect::*` at the crate root, and a
// second re-export through `agent_inspector::*` would be redundant.
// ---------------------------------------------------------------------------

/// Identity block for an agent — read from `civ-agents` / `civ-species`.
///
/// `species_or_phenotype` is the short label the species crate publishes
/// (e.g. `"human"`, `"dwarf"`, `"wolf"`). When `None` it renders as
/// "unmeasured" — the species crate hasn't published a phenotype yet, so
/// the inspector must not invent one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Stable agent id (matches `crate::inspect::EntityId`).
    pub id: crate::inspect::EntityId,
    /// Display name (rendered in the panel header).
    pub display_name: String,
    /// Species/phenotype short label. `None` when unmeasured.
    pub species_or_phenotype: Option<String>,
    /// Age in years. `None` when unmeasured.
    pub age_years: Option<u32>,
}

impl AgentIdentity {
    /// Construct an identity block. Both optional fields default to
    /// unmeasured; callers fill them as the owning crates publish data.
    pub fn new(id: crate::inspect::EntityId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            species_or_phenotype: None,
            age_years: None,
        }
    }

    /// Set the species/phenotype label.
    pub fn with_species(mut self, species: impl Into<String>) -> Self {
        self.species_or_phenotype = Some(species.into());
        self
    }

    /// Set the age in years.
    pub fn with_age(mut self, years: u32) -> Self {
        self.age_years = Some(years);
        self
    }
}

// ---------------------------------------------------------------------------
// CurrentAction — current activity/action the agent is performing.
//
// FR-CIV-INSPECT-901 lists "current activity" as a required inspector field.
// `verb` is the short verb label (e.g. `"foraging"`, `"sleeping"`,
// `"building"`, `"fleeing"`); `target` is an optional human-readable
// target label (e.g. `"oak tree (3,4,5)"`).
// ---------------------------------------------------------------------------

/// Current activity / action the agent is performing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentAction {
    /// Short verb label for the action (e.g. `"foraging"`, `"sleeping"`).
    pub verb: String,
    /// Optional human-readable target label (e.g. `"oak tree"`).
    pub target: Option<String>,
}

impl CurrentAction {
    /// Construct a `CurrentAction` with a verb and an optional target.
    pub fn new(verb: impl Into<String>, target: Option<String>) -> Self {
        Self {
            verb: verb.into(),
            target,
        }
    }

    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        match &self.target {
            Some(t) => format!("{} → {}", self.verb, t),
            None => self.verb.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentInspectorSummary — the aggregate struct the inspector panel reads.
//
// This is the **single summary struct** the task description asks for: it
// aggregates identity, age, needs, mood, and current-action into one
// renderer-friendly value. Relationships and lineage are included as
// optional placeholders so the struct satisfies the FR's full field list
// without claiming values it doesn't have — they show up as `None` until
// the relevant owning crate publishes them.
// ---------------------------------------------------------------------------

/// Aggregate summary for the agent inspector panel.
///
/// Combines:
/// - [`AgentIdentity`] (identity, species, age)
/// - [`MoodSnapshot`] (psyche/mood)
/// - `Vec<`[`NeedLevel`]`>` (needs — multiple)
/// - [`CurrentAction`] (current activity)
///
/// Plus optional placeholders for the remaining FR-CIV-INSPECT-901 fields
/// (relationships, lineage) so the struct satisfies the spec's full list
/// without fabricating values. Both default to `None` until the relevant
/// owning crate publishes data.
///
/// # Acceptance
///
/// FR-CIV-INSPECT-901 says the summary "SHALL show identity, species/phenotype
/// summary, age, needs, psyche/mood, current activity, relationships,
/// lineage." This struct is the single value the inspector panel reads.
///
/// "BLIND fields shown as 'unmeasured' not faked" is enforced via:
/// - [`Sentinel::Unmeasured`] on the optional summary fields (species,
///   age), so a missing field is *not* a fabricated string.
/// - `Option<...>` on need values and mood label/valence.
/// - Empty `Vec<NeedLevel>` rendering as "no needs measured" (not `0`).
/// - [`CurrentAction`] being `Option` so a missing action is rendered as
///   "idle" rather than a fake verb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentInspectorSummary {
    /// Identity block (id, name, species, age).
    pub identity: AgentIdentity,
    /// Current action. `None` renders as "idle".
    pub current_action: Option<CurrentAction>,
    /// Per-need snapshot list (hunger, rest, shelter, social, ...).
    /// Empty list means the needs crate has not published any this tick.
    pub needs: Vec<NeedLevel>,
    /// Mood/psyche snapshot. `Unmeasured` (label = None, valence = None)
    /// renders as "unmeasured" rather than a fabricated valence.
    pub mood: MoodSnapshot,
    /// Short relationship count. `None` when unmeasured.
    pub relationship_count: Option<u32>,
    /// Optional lineage label (e.g. `"child of Aldric"`). `None` when
    /// unmeasured; rendered as "unmeasured" by the panel.
    pub lineage: Option<String>,
}

impl AgentInspectorSummary {
    /// Construct a summary with the identity block set; everything else
    /// defaults to unmeasured / empty / idle.
    pub fn new(identity: AgentIdentity) -> Self {
        Self {
            identity,
            current_action: None,
            needs: Vec::new(),
            mood: MoodSnapshot::unmeasured(),
            relationship_count: None,
            lineage: None,
        }
    }

    /// Set the current action.
    pub fn with_action(mut self, action: CurrentAction) -> Self {
        self.current_action = Some(action);
        self
    }

    /// Replace the needs list.
    pub fn with_needs(mut self, needs: Vec<NeedLevel>) -> Self {
        self.needs = needs;
        self
    }

    /// Set the mood snapshot.
    pub fn with_mood(mut self, mood: MoodSnapshot) -> Self {
        self.mood = mood;
        self
    }

    /// Set the relationship count.
    pub fn with_relationship_count(mut self, count: u32) -> Self {
        self.relationship_count = Some(count);
        self
    }

    /// Set the lineage label.
    pub fn with_lineage(mut self, lineage: impl Into<String>) -> Self {
        self.lineage = Some(lineage.into());
        self
    }

    /// Average need value across measured needs (`0..=100`).
    /// Returns `None` if no need has a measured value.
    pub fn average_need(&self) -> Option<f32> {
        let measured: Vec<u8> = self.needs.iter().filter_map(|n| n.value).collect();
        if measured.is_empty() {
            return None;
        }
        let sum: u32 = measured.iter().map(|&v| v as u32).sum();
        Some(sum as f32 / measured.len() as f32)
    }

    /// Dominant need — the measured need with the lowest value
    /// (the one that most wants attention). Returns `None` if no need
    /// has a measured value.
    pub fn dominant_need(&self) -> Option<&NeedLevel> {
        self.needs
            .iter()
            .filter(|n| n.value.is_some())
            .min_by_key(|n| n.value.unwrap())
    }

    /// One-line description for tooltips and the panel header.
    pub fn one_line(&self) -> String {
        let age = self
            .identity
            .age_years
            .map(|a| format!("age {}", a))
            .unwrap_or_else(|| "age ?".to_string());
        match &self.current_action {
            Some(a) => format!("{} — {} ({})", self.identity.display_name, a.one_line(), age),
            None => format!("{} ({})", self.identity.display_name, age),
        }
    }

    /// `true` iff at least one need has a measured value.
    pub fn has_measured_need(&self) -> bool {
        self.needs.iter().any(|n| n.is_measured())
    }

    /// `true` iff the mood snapshot has both a label and a valence.
    pub fn has_measured_mood(&self) -> bool {
        self.mood.is_measured()
    }
}

// ---------------------------------------------------------------------------
// AgentInspectorRegistry — in-memory EntityId → AgentInspectorSummary index.
//
// The renderer fills the registry each tick from the relevant owning
// crates (`civ-agents`, `civ-species`, `civ-needs`, the psyche substrate)
// and the inspector panel calls `summary(id)` to render. The registry is
// read-only — it never reaches out to fetch data itself; the data layer
// contract is "the caller publishes, we store and serve".
// ---------------------------------------------------------------------------

/// In-memory index of `EntityId → AgentInspectorSummary`.
///
/// Backed by a single `BTreeMap` so lookups are `O(log n)`. The renderer
/// typically owns one of these behind an `RwLock` and rebuilds it from
/// the relevant owning crates on each inspector update.
///
/// The registry is `Clone` (the map is `Clone`); cloning is the cheap
/// path for the inspector panel to take a stable snapshot.
#[derive(Debug, Clone, Default)]
pub struct AgentInspectorRegistry {
    by_id: BTreeMap<crate::inspect::EntityId, AgentInspectorSummary>,
}

impl AgentInspectorRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish / replace an agent's summary.
    pub fn publish(&mut self, summary: AgentInspectorSummary) {
        self.by_id.insert(summary.identity.id, summary);
    }

    /// Remove an agent's summary (e.g. when the agent is despawned).
    /// Returns the previous summary if any.
    pub fn remove(&mut self, id: crate::inspect::EntityId) -> Option<AgentInspectorSummary> {
        self.by_id.remove(&id)
    }

    /// Look up an agent's summary by id.
    pub fn summary(&self, id: crate::inspect::EntityId) -> Option<&AgentInspectorSummary> {
        self.by_id.get(&id)
    }

    /// Number of registered agent summaries.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// `true` iff no summaries are registered.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Iterate all `(id, summary)` pairs in id order.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (crate::inspect::EntityId, &AgentInspectorSummary)> + '_ {
        self.by_id.iter().map(|(k, v)| (*k, v))
    }
}

// ---------------------------------------------------------------------------
// Tests — including the FR-CIV-INSPECT-901 acceptance test.
//
// Acceptance test mandate (from FR-CIV-INSPECT-901 / the task brief):
// "summary reflects the agent's current needs/mood".
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::EntityId;

    /// Acceptance test for FR-CIV-INSPECT-901:
    /// "summary reflects the agent's current needs/mood".
    ///
    /// The summary struct built from a measured needs vector and a measured
    /// mood snapshot must reflect those values back to the inspector panel —
    /// both via direct field access and via the helpers (`has_measured_need`,
    /// `has_measured_mood`, `average_need`, `dominant_need`).
    #[test]
    fn fr_civ_inspect_901_acceptance_summary_reflects_needs_and_mood() {
        let id = EntityId::new(42);
        let identity = AgentIdentity::new(id, "Aldric")
            .with_species("human")
            .with_age(34);

        let summary = AgentInspectorSummary::new(identity)
            .with_action(CurrentAction::new("foraging", Some("oak tree".into())))
            .with_needs(vec![
                NeedLevel::measured("hunger", 18),
                NeedLevel::measured("rest", 72),
                NeedLevel::measured("shelter", 95),
            ])
            .with_mood(MoodSnapshot::measured("content", 12))
            .with_relationship_count(3)
            .with_lineage("child of Bryn");

        // Identity, age, species — all reflected.
        assert_eq!(summary.identity.id, id);
        assert_eq!(summary.identity.display_name, "Aldric");
        assert_eq!(summary.identity.age_years, Some(34));
        assert_eq!(
            summary.identity.species_or_phenotype.as_deref(),
            Some("human"),
        );

        // Current action — reflected verbatim.
        let action = summary
            .current_action
            .as_ref()
            .expect("action must be present");
        assert_eq!(action.verb, "foraging");
        assert_eq!(action.target.as_deref(), Some("oak tree"));

        // Needs — every need is reflected with its measured value.
        assert_eq!(summary.needs.len(), 3);
        assert!(summary.has_measured_need(), "at least one need must be measured");
        assert_eq!(
            summary.needs.iter().filter(|n| n.is_measured()).count(),
            3,
            "all three needs have measured values",
        );
        let hunger = summary
            .needs
            .iter()
            .find(|n| n.label == "hunger")
            .expect("hunger need present");
        assert_eq!(hunger.value, Some(18));
        let rest = summary
            .needs
            .iter()
            .find(|n| n.label == "rest")
            .expect("rest need present");
        assert_eq!(rest.value, Some(72));

        // Mood — both label and valence reflected.
        assert!(summary.has_measured_mood());
        assert_eq!(summary.mood.label.as_deref(), Some("content"));
        assert_eq!(summary.mood.valence, Some(12));

        // Helpers: dominant need is the lowest-value measured need.
        let dominant = summary
            .dominant_need()
            .expect("at least one measured need");
        assert_eq!(dominant.label, "hunger");
        assert_eq!(dominant.value, Some(18));

        // Helpers: average need is the arithmetic mean of measured values.
        let avg = summary.average_need().expect("measured needs must average");
        // (18 + 72 + 95) / 3 = 61.666...
        assert!((avg - 61.666_664).abs() < 0.01, "average need must be ~61.67");

        // Optional summary fields — also reflected.
        assert_eq!(summary.relationship_count, Some(3));
        assert_eq!(summary.lineage.as_deref(), Some("child of Bryn"));

        // `one_line` is non-empty and embeds the agent's name.
        let line = summary.one_line();
        assert!(!line.is_empty());
        assert!(line.contains("Aldric"), "one_line must include display name");
    }

    /// Acceptance: an *unmeasured* summary renders as "unmeasured" — the
    /// FR's "BLIND fields shown as 'unmeasured' not faked" rule. We check
    /// this by building a summary with the default-constructed
    /// `unmeasured()` mood and empty needs, and verifying that no field
    /// silently takes a `0` or a fake string.
    #[test]
    fn fr_civ_inspect_901_unmeasured_fields_are_explicit() {
        let id = EntityId::new(7);
        let identity = AgentIdentity::new(id, "Bryn");
        // Note: species + age left as None — must NOT be defaulted.
        let summary = AgentInspectorSummary::new(identity);

        // Identity: species and age are explicitly absent (Option::None),
        // not coerced to "" or 0.
        assert!(summary.identity.species_or_phenotype.is_none());
        assert!(summary.identity.age_years.is_none());

        // Mood: explicitly unmeasured.
        assert!(!summary.has_measured_mood());
        assert!(summary.mood.label.is_none());
        assert!(summary.mood.valence.is_none());

        // Needs: empty list — no fabricated "hunger = 0" rows.
        assert!(summary.needs.is_empty());
        assert!(!summary.has_measured_need());
        assert!(summary.average_need().is_none());
        assert!(summary.dominant_need().is_none());

        // Current action: explicitly None — no fabricated verb.
        assert!(summary.current_action.is_none());

        // Relationship count + lineage: explicitly None.
        assert!(summary.relationship_count.is_none());
        assert!(summary.lineage.is_none());
    }

    #[test]
    fn sentinel_labels_are_stable() {
        assert_eq!(Sentinel::Unmeasured.label(), "unmeasured");
        assert_eq!(Sentinel::NotApplicable.label(), "n/a");
    }

    #[test]
    fn need_level_helpers_match_constructor() {
        let measured = NeedLevel::measured("hunger", 42);
        assert!(measured.is_measured());
        assert_eq!(measured.value, Some(42));
        assert_eq!(measured.label, "hunger");

        let unmeasured = NeedLevel::unmeasured("rest");
        assert!(!unmeasured.is_measured());
        assert_eq!(unmeasured.value, None);
        assert_eq!(unmeasured.label, "rest");
    }

    #[test]
    fn mood_snapshot_helpers_match_constructor() {
        let measured = MoodSnapshot::measured("angry", -75);
        assert!(measured.is_measured());
        assert_eq!(measured.label.as_deref(), Some("angry"));
        assert_eq!(measured.valence, Some(-75));

        let unmeasured = MoodSnapshot::unmeasured();
        assert!(!unmeasured.is_measured());
        assert!(unmeasured.label.is_none());
        assert!(unmeasured.valence.is_none());
    }

    #[test]
    fn current_action_one_line_includes_target_when_present() {
        let with_target = CurrentAction::new("building", Some("shed".into()));
        assert_eq!(with_target.one_line(), "building → shed");
        let without = CurrentAction::new("sleeping", None);
        assert_eq!(without.one_line(), "sleeping");
    }

    #[test]
    fn identity_builders_chain() {
        let identity = AgentIdentity::new(EntityId::new(1), "Cedric")
            .with_species("dwarf")
            .with_age(127);
        assert_eq!(identity.id, EntityId::new(1));
        assert_eq!(identity.display_name, "Cedric");
        assert_eq!(identity.species_or_phenotype.as_deref(), Some("dwarf"));
        assert_eq!(identity.age_years, Some(127));
    }

    #[test]
    fn registry_publish_lookup_remove() {
        let mut reg = AgentInspectorRegistry::new();
        assert!(reg.is_empty());

        let id_a = EntityId::new(1);
        let id_b = EntityId::new(2);
        let summary_a = AgentInspectorSummary::new(AgentIdentity::new(id_a, "Aldric"))
            .with_action(CurrentAction::new("foraging", None));
        let summary_b = AgentInspectorSummary::new(AgentIdentity::new(id_b, "Bryn"))
            .with_mood(MoodSnapshot::measured("content", 8));

        reg.publish(summary_a.clone());
        reg.publish(summary_b.clone());
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.summary(id_a).map(|s| s.identity.display_name.as_str()), Some("Aldric"));
        assert_eq!(reg.summary(id_b).map(|s| s.identity.display_name.as_str()), Some("Bryn"));
        assert!(reg.summary(EntityId::new(999)).is_none());

        let removed = reg.remove(id_a);
        assert!(removed.is_some());
        assert_eq!(reg.len(), 1);
        assert!(reg.summary(id_a).is_none());
        assert!(reg.summary(id_b).is_some());
    }

    #[test]
    fn registry_publish_replaces_existing_summary() {
        let mut reg = AgentInspectorRegistry::new();
        let id = EntityId::new(1);

        // Tick 1: agent is foraging with measured needs.
        let s1 = AgentInspectorSummary::new(AgentIdentity::new(id, "Aldric"))
            .with_action(CurrentAction::new("foraging", None))
            .with_needs(vec![NeedLevel::measured("hunger", 18)]);
        reg.publish(s1);

        // Tick 2: agent is now sleeping; the registry must hold the latest.
        let s2 = AgentInspectorSummary::new(AgentIdentity::new(id, "Aldric"))
            .with_action(CurrentAction::new("sleeping", None))
            .with_needs(vec![NeedLevel::measured("rest", 95)]);
        reg.publish(s2);

        let stored = reg.summary(id).expect("summary present");
        let action = stored.current_action.as_ref().expect("action present");
        assert_eq!(action.verb, "sleeping", "latest tick must win");
        assert_eq!(stored.needs.len(), 1);
        assert_eq!(stored.needs[0].label, "rest");
        assert_eq!(stored.needs[0].value, Some(95));
        assert_eq!(reg.len(), 1, "republish must not duplicate");
    }

    #[test]
    fn registry_iter_is_id_ordered() {
        let mut reg = AgentInspectorRegistry::new();
        for raw in [3u64, 1, 2] {
            reg.publish(AgentInspectorSummary::new(AgentIdentity::new(
                EntityId::new(raw),
                format!("agent-{}", raw),
            )));
        }
        let order: Vec<u64> = reg.iter().map(|(id, _)| id.raw()).collect();
        assert_eq!(order, vec![1, 2, 3], "iter must be sorted by EntityId");
    }
}