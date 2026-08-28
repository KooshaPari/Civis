//! Era transition history and chronicle (FR-ERA).
//!
//! Records emergent age advances per faction when threshold evaluation
//! detects a strictly higher [`super::era::CivAge`].

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::era::CivAge;

/// Maximum chronicle lines retained in memory.
pub const ERA_CHRONICLE_MAX_LEN: usize = 200;

// ─── Existing Types (unchanged) ─────────────────────────────────────────

/// A single emergent era transition for one faction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraTransition {
    pub tick: u64,
    pub faction_id: u32,
    pub from: CivAge,
    pub to: CivAge,
}

/// Bounded chronicle of emergent era advances.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraHistory {
    transitions: VecDeque<EraTransition>,
    chronicle: VecDeque<String>,
}

impl EraHistory {
    /// Record an era advance when `to` is strictly after `from`.
    pub fn record_advance(&mut self, tick: u64, faction_id: u32, from: CivAge, to: CivAge) {
        if to <= from {
            return;
        }
        self.transitions.push_back(EraTransition {
            tick,
            faction_id,
            from,
            to,
        });
        while self.transitions.len() > ERA_CHRONICLE_MAX_LEN {
            self.transitions.pop_front();
        }

        let line = format!(
            "tick {tick}: faction {faction_id} entered the {} age (from {})",
            to.as_str(),
            from.as_str()
        );
        self.chronicle.push_back(line);
        while self.chronicle.len() > ERA_CHRONICLE_MAX_LEN {
            self.chronicle.pop_front();
        }
    }

    /// Recorded transitions (oldest first).
    #[must_use]
    pub fn transitions(&self) -> Vec<EraTransition> {
        self.transitions.iter().cloned().collect()
    }

    /// Chronicle lines for HUD / replay surfaces.
    #[must_use]
    pub fn chronicle(&self) -> Vec<String> {
        self.chronicle.iter().cloned().collect()
    }

    /// Number of era transitions recorded.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_only_forward_transitions_and_chronicle_lines() {
        let mut history = EraHistory::default();
        history.record_advance(3, 7, CivAge::Stone, CivAge::Bronze);
        history.record_advance(4, 7, CivAge::Bronze, CivAge::Bronze);

        assert_eq!(history.transition_count(), 1);
        assert_eq!(history.transitions()[0].faction_id, 7);
        assert_eq!(
            history.chronicle(),
            vec!["tick 3: faction 7 entered the Bronze age (from Stone)".to_string()]
        );
    }

    #[test]
    fn retains_only_the_bounded_tail() {
        let mut history = EraHistory::default();
        for tick in 0..=ERA_CHRONICLE_MAX_LEN {
            history.record_advance(tick as u64, 1, CivAge::Stone, CivAge::Bronze);
        }

        assert_eq!(history.transition_count(), ERA_CHRONICLE_MAX_LEN);
        assert_eq!(history.transitions()[0].tick, 1);
        assert!(history.chronicle()[0].starts_with("tick 1:"));
    }

    #[test]
    fn round_trips_through_serde() {
        let mut history = EraHistory::default();
        history.record_advance(9, 2, CivAge::Bronze, CivAge::Iron);
        let encoded = serde_json::to_vec(&history).expect("serialize history");
        let decoded: EraHistory = serde_json::from_slice(&encoded).expect("deserialize history");
        assert_eq!(decoded, history);
    }
}

/// Type of historical event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    War,
    Trade,
    Religion,
    Culture,
    Disaster,
    Political,
    Other,
}

/// A single historical event with tick, type, description, participants, consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub tick: u64,
    pub event_type: EventType,
    pub description: String,
    pub participants: Vec<u32>,
    pub consequences: Vec<String>,
}

impl Default for HistoricalEvent {
    fn default() -> Self {
        Self {
            tick: 0,
            event_type: EventType::Other,
            description: String::new(),
            participants: Vec::new(),
            consequences: Vec::new(),
        }
    }
}

/// Append-only log of historical events with summaries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryLog {
    pub events: Vec<HistoricalEvent>,
    pub summaries: HashMap<String, String>,
    pub max_events: usize,
}

impl HistoryLog {
    /// Create a new HistoryLog with a capacity limit.
    #[must_use]
    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            summaries: HashMap::new(),
            max_events,
        }
    }

    /// Record a historical event, evicting oldest if over capacity.
    pub fn record_event(&mut self, event: HistoricalEvent) {
        self.events.push(event);
        while self.events.len() > self.max_events && self.max_events > 0 {
            self.events.remove(0);
        }
    }

    /// Summarize all events within a tick range into a single string.
    #[must_use]
    pub fn summarize_era(&self, era_name: &str, start_tick: u64, end_tick: u64) -> String {
        let count = self
            .events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .count();
        format!(
            "Era '{}' (ticks {}-{}): {} events recorded",
            era_name, start_tick, end_tick, count
        )
    }

    /// Query events by type within a tick range.
    #[must_use]
    pub fn query_events(
        &self,
        event_type: EventType,
        start_tick: u64,
        end_tick: u64,
    ) -> Vec<&HistoricalEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type && e.tick >= start_tick && e.tick <= end_tick)
            .collect()
    }

    /// Compute a normalized impact score for events in a tick range.
    /// Each event contributes based on its consequence count.
    #[must_use]
    pub fn compute_historical_impact(&self, start_tick: u64, end_tick: u64) -> f32 {
        let total_consequences: usize = self
            .events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .map(|e| e.consequences.len())
            .sum();
        (total_consequences as f32 / 100.0).min(1.0)
    }
}

/// Advance a history log by one tick (currently a no-op placeholder).
#[must_use]
pub fn tick_history(log: &HistoryLog, _tick: u64) -> HistoryLog {
    log.clone()
}

// ─── Timeline System ─────────────────────────────────────────────────────

/// A richly-described event on a timeline with actors and consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: u64,
    pub tick: u64,
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub actors: Vec<u32>,
    pub consequences: Vec<String>,
    pub importance: f32,
}

/// An ordered collection of timeline events with auto-incrementing IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Timeline {
    events: Vec<TimelineEvent>,
    next_id: u64,
}

impl Timeline {
    /// Create an empty timeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a [`TimelineEvent`], assigning it a unique ID and returning it.
    pub fn add_event(&mut self, mut event: TimelineEvent) -> u64 {
        let id = self.next_id;
        event.id = id;
        self.events.push(event);
        self.next_id = id + 1;
        id
    }

    /// Look up an event by its ID.
    #[must_use]
    pub fn get_event(&self, id: u64) -> Option<&TimelineEvent> {
        self.events.iter().find(|e| e.id == id)
    }

    /// Return all events whose tick falls in `[start_tick, end_tick]`.
    #[must_use]
    pub fn events_in_range(&self, start_tick: u64, end_tick: u64) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .collect()
    }

    /// Return the `n` most important events, sorted descending by importance.
    #[must_use]
    pub fn most_important(&self, n: usize) -> Vec<&TimelineEvent> {
        let mut sorted: Vec<&TimelineEvent> = self.events.iter().collect();
        sorted.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .expect("importance should be ordered")
        });
        sorted.into_iter().take(n).collect()
    }
}

// ─── Great Person Tracking ───────────────────────────────────────────────

/// A notable individual who leaves a mark on civilisation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreatPerson {
    pub name: String,
    pub birth_tick: u64,
    pub death_tick: Option<u64>,
    pub achievements: Vec<String>,
    pub faction_id: u32,
    pub impact_score: f32,
}

/// Initialise a new [`GreatPerson`] who is alive at `birth_tick`.
#[must_use]
pub fn create_great_person(name: &str, faction_id: u32, birth_tick: u64) -> GreatPerson {
    GreatPerson {
        name: name.to_string(),
        birth_tick,
        death_tick: None,
        achievements: Vec::new(),
        faction_id,
        impact_score: 0.0,
    }
}

/// Record an achievement for a great person, bumping their impact score.
pub fn add_achievement(person: &mut GreatPerson, achievement: &str, impact: f32) {
    person.achievements.push(achievement.to_string());
    person.impact_score += impact;
}

/// How many ticks the person has been alive (or dead) relative to `current_tick`.
#[must_use]
pub fn person_lifetime(person: &GreatPerson, current_tick: u64) -> u64 {
    let end = person.death_tick.unwrap_or(current_tick);
    end.saturating_sub(person.birth_tick)
}

/// Compute posthumous impact that decays exponentially after death.
///
/// Returns `0.0` if the person has not yet died.  The decay constant is
/// `e^(-lambda * ticks_since_death)` with `lambda = 0.01`.
#[must_use]
pub fn posthumous_impact(person: &GreatPerson, ticks_since_death: u64) -> f32 {
    if person.death_tick.is_none() {
        return 0.0;
    }
    // Exponential decay: e^(-0.01 * t)
    let lambda = 0.01_f32;
    let exponent = -lambda * (ticks_since_death as f32);
    let decay = exponent.exp();
    person.impact_score * decay
}

// ─── Era Transitions ─────────────────────────────────────────────────────

/// Requirement thresholds for advancing *into* a given era.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraDefinition {
    pub name: String,
    pub required_population: u32,
    pub required_tech: u32,
    pub required_food_surplus: i64,
}

/// Standard era progression from Stone through Industrial.
///
/// Evaluated in order; the first era whose requirements are all met is returned.
#[must_use]
pub fn era_definitions() -> Vec<EraDefinition> {
    vec![
        EraDefinition {
            name: "Stone".to_string(),
            required_population: 0,
            required_tech: 0,
            required_food_surplus: i64::MIN,
        },
        EraDefinition {
            name: "Bronze".to_string(),
            required_population: 300,
            required_tech: 1,
            required_food_surplus: 0,
        },
        EraDefinition {
            name: "Iron".to_string(),
            required_population: 800,
            required_tech: 3,
            required_food_surplus: 0,
        },
        EraDefinition {
            name: "Classical".to_string(),
            required_population: 2_000,
            required_tech: 5,
            required_food_surplus: 1,
        },
        EraDefinition {
            name: "Medieval".to_string(),
            required_population: 5_000,
            required_tech: 8,
            required_food_surplus: 1,
        },
        EraDefinition {
            name: "Industrial".to_string(),
            required_population: 8_000,
            required_tech: 10,
            required_food_surplus: 1,
        },
    ]
}

/// Map an era name string back to a [`CivAge`] variant.
fn age_from_name(name: &str) -> Option<CivAge> {
    match name {
        "Stone" => Some(CivAge::Stone),
        "Bronze" => Some(CivAge::Bronze),
        "Iron" => Some(CivAge::Iron),
        "Classical" => Some(CivAge::Classical),
        "Medieval" => Some(CivAge::Medieval),
        "Industrial" => Some(CivAge::Industrial),
        _ => None,
    }
}

/// Evaluate which [`CivAge`] a faction qualifies for based on the given state.
///
/// Returns the highest era whose thresholds are all satisfied.
#[must_use]
pub fn evaluate_era_transition(
    _faction_id: u32,
    population: u32,
    tech_level: u32,
    food_surplus: i64,
) -> Option<CivAge> {
    let mut result = None;
    for def in era_definitions() {
        if population >= def.required_population
            && tech_level >= def.required_tech
            && food_surplus >= def.required_food_surplus
        {
            result = age_from_name(&def.name);
        }
    }
    result
}

/// Fractional progress through the full era progression, from `0.0` (Stone) to `1.0` (Industrial).
#[must_use]
pub fn era_progress_percentage(current: CivAge) -> f32 {
    let total = 5_u32; // 6 eras, 5 transitions
    let idx = match current {
        CivAge::Stone => 0,
        CivAge::Bronze => 1,
        CivAge::Iron => 2,
        CivAge::Classical => 3,
        CivAge::Medieval => 4,
        CivAge::Industrial => 5,
    };
    idx as f32 / total as f32
}

// ─── Historical What-If Branching ────────────────────────────────────────

/// A decision point in an alternate timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPoint {
    pub tick: u64,
    pub event_description: String,
    pub alternatives: Vec<String>,
    pub chosen: usize,
}

/// An alternate timeline that diverges from the main timeline at a given tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateTimeline {
    pub events: Vec<TimelineEvent>,
    pub branch_points: Vec<BranchPoint>,
    pub divergence_tick: u64,
}

impl AlternateTimeline {
    /// Create a new alternate timeline seeded with a copy of the base events.
    #[must_use]
    pub fn new(base_events: Vec<TimelineEvent>, divergence_tick: u64) -> Self {
        Self {
            events: base_events,
            branch_points: Vec::new(),
            divergence_tick,
        }
    }

    /// Record a branch point in this alternate timeline.
    pub fn add_branch(&mut self, branch: BranchPoint) {
        self.branch_points.push(branch);
    }

    /// Return all events that occur up to (but not including) the specified branch point.
    ///
    /// This lets a caller "explore" what the timeline looked like before a
    /// particular decision was made.
    #[must_use]
    pub fn explore_alternative(&self, branch_idx: usize, _alt_idx: usize) -> Vec<&TimelineEvent> {
        let branch = self
            .branch_points
            .get(branch_idx)
            .expect("branch_idx out of range");
        self.events
            .iter()
            .filter(|e| e.tick < branch.tick)
            .collect()
    }
}

// ─── Chronicle Generation ────────────────────────────────────────────────

/// Generate a prose narrative summarising a list of timeline events.
#[must_use]
pub fn generate_chronicle(events: &[TimelineEvent], era_name: &str) -> String {
    if events.is_empty() {
        return format!("The {era_name} era passed without notable events.");
    }
    let mut out = format!("Chronicle of the {era_name} era:\n");
    for (i, ev) in events.iter().enumerate() {
        let actors_str = if ev.actors.is_empty() {
            "Unknown actors".to_string()
        } else {
            ev.actors
                .iter()
                .map(|a| format!("faction {a}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!(
            "{num}. [tick {tick}] {title} – {desc} (participants: {actors})\n",
            num = i + 1,
            tick = ev.tick,
            title = ev.title,
            desc = ev.description,
            actors = actors_str,
        ));
        if !ev.consequences.is_empty() {
            out.push_str(&format!(
                "   Consequences: {}\n",
                ev.consequences.join("; ")
            ));
        }
    }
    out
}

/// One-line summary of a [`GreatPerson`].
#[must_use]
pub fn summarize_person(person: &GreatPerson) -> String {
    let status = match person.death_tick {
        Some(d) => format!("died tick {d}"),
        None => "alive".to_string(),
    };
    format!(
        "{name} (faction {fid}, born tick {bt}, {status}, impact: {impact:.1}, achievements: {ach})",
        name = person.name,
        fid = person.faction_id,
        bt = person.birth_tick,
        status = status,
        impact = person.impact_score,
        ach = person.achievements.len(),
    )
}

/// Produce a human-readable summary of an era transition between two [`CivAge`]s.
#[must_use]
pub fn era_transition_summary(from: CivAge, to: CivAge, faction_id: u32) -> String {
    format!(
        "Faction {faction_id} progressed from the {from} age to the {to} age.",
        faction_id = faction_id,
        from = from.as_str(),
        to = to.as_str(),
    )
}

// ─── Original Extended Tests ─────────────────────────────────────────────

#[cfg(test)]
mod history_extended_tests {
    use super::*;

    #[test]
    fn history_log_with_capacity() {
        let log = HistoryLog::with_capacity(5);
        assert_eq!(log.max_events, 5);
        assert!(log.events.is_empty());
    }

    #[test]
    fn history_log_record_event() {
        let mut log = HistoryLog::with_capacity(10);
        let event = HistoricalEvent {
            tick: 1,
            event_type: EventType::War,
            description: "Battle of X".into(),
            participants: vec![1, 2],
            consequences: vec!["territory change".into()],
        };
        log.record_event(event);
        assert_eq!(log.events.len(), 1);
    }

    #[test]
    fn history_log_evicts_oldest() {
        let mut log = HistoryLog::with_capacity(3);
        for i in 0..5 {
            log.record_event(HistoricalEvent {
                tick: i,
                ..HistoricalEvent::default()
            });
        }
        assert_eq!(log.events.len(), 3);
        assert_eq!(log.events[0].tick, 2);
    }

    #[test]
    fn summarize_era_count() {
        let mut log = HistoryLog::with_capacity(100);
        for i in 0..5 {
            log.record_event(HistoricalEvent {
                tick: i,
                ..HistoricalEvent::default()
            });
        }
        let summary = log.summarize_era("Test Era", 0, 4);
        assert!(summary.contains("5 events"));
    }

    #[test]
    fn query_events_by_type() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 1,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 2,
            event_type: EventType::Trade,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 3,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        let wars = log.query_events(EventType::War, 0, 10);
        assert_eq!(wars.len(), 2);
    }

    #[test]
    fn query_events_tick_range() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 5,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 15,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        let wars = log.query_events(EventType::War, 0, 10);
        assert_eq!(wars.len(), 1);
    }

    #[test]
    fn compute_historical_impact_zero() {
        let log = HistoryLog::with_capacity(100);
        assert!((log.compute_historical_impact(0, 100) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_historical_impact_with_events() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 1,
            consequences: vec!["a".into(), "b".into(), "c".into()],
            ..HistoricalEvent::default()
        });
        let impact = log.compute_historical_impact(0, 10);
        assert!(impact > 0.0);
    }

    #[test]
    fn tick_history_clones() {
        let log = HistoryLog::with_capacity(10);
        let ticked = tick_history(&log, 100);
        assert_eq!(ticked.max_events, log.max_events);
    }

    #[test]
    fn historical_event_default() {
        let e = HistoricalEvent::default();
        assert_eq!(e.event_type, EventType::Other);
        assert!(e.description.is_empty());
    }
}

// ─── New Extended Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod new_history_tests {
    use super::*;

    // Helper to build a minimal TimelineEvent.
    fn make_event(tick: u64, title: &str, importance: f32) -> TimelineEvent {
        TimelineEvent {
            id: 0, // overwritten by Timeline::add_event
            tick,
            event_type: EventType::Other,
            title: title.to_string(),
            description: format!("Description of {title}"),
            actors: Vec::new(),
            consequences: Vec::new(),
            importance,
        }
    }

    // ── Timeline ─────────────────────────────────────────────────────

    #[test]
    fn timeline_add_and_retrieve_event() {
        let mut tl = Timeline::new();
        let ev = TimelineEvent {
            id: 0,
            tick: 10,
            event_type: EventType::War,
            title: "Great War".into(),
            description: "A world-spanning conflict".into(),
            actors: vec![1, 2],
            consequences: vec!["peace treaty".into()],
            importance: 0.9,
        };
        let id = tl.add_event(ev);
        assert_eq!(id, 0);
        let retrieved = tl.get_event(0).expect("event 0 should exist");
        assert_eq!(retrieved.title, "Great War");
        assert_eq!(retrieved.id, 0);
    }

    #[test]
    fn timeline_add_assigns_sequential_ids() {
        let mut tl = Timeline::new();
        let id0 = tl.add_event(make_event(1, "First", 0.5));
        let id1 = tl.add_event(make_event(2, "Second", 0.6));
        let id2 = tl.add_event(make_event(3, "Third", 0.7));
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn timeline_events_in_range() {
        let mut tl = Timeline::new();
        for tick in [5, 10, 15, 20] {
            tl.add_event(make_event(tick, &format!("Event at {tick}"), 0.5));
        }
        let in_range = tl.events_in_range(8, 16);
        assert_eq!(in_range.len(), 2);
        assert_eq!(in_range[0].tick, 10);
        assert_eq!(in_range[1].tick, 15);
    }

    #[test]
    fn timeline_most_important_ordering() {
        let mut tl = Timeline::new();
        for imp in [0.3, 0.9, 0.5, 0.95, 0.1] {
            tl.add_event(make_event(0, "event", imp));
        }
        let top2 = tl.most_important(2);
        assert_eq!(top2.len(), 2);
        assert!(top2[0].importance >= top2[1].importance);
        assert!((top2[0].importance - 0.95).abs() < f32::EPSILON);
        assert!((top2[1].importance - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn timeline_get_nonexistent_event() {
        let tl = Timeline::new();
        assert!(tl.get_event(42).is_none());
    }

    #[test]
    fn timeline_empty_range_returns_nothing() {
        let mut tl = Timeline::new();
        tl.add_event(make_event(100, "Far future", 0.5));
        let result = tl.events_in_range(0, 50);
        assert!(result.is_empty());
    }

    #[test]
    fn timeline_most_important_clamped_to_available() {
        let mut tl = Timeline::new();
        tl.add_event(make_event(1, "Only event", 0.4));
        let top10 = tl.most_important(10);
        assert_eq!(top10.len(), 1);
    }

    // ── Great Person ─────────────────────────────────────────────────

    #[test]
    fn create_great_person_defaults() {
        let gp = create_great_person("Socrates", 3, 100);
        assert_eq!(gp.name, "Socrates");
        assert_eq!(gp.faction_id, 3);
        assert_eq!(gp.birth_tick, 100);
        assert!(gp.death_tick.is_none());
        assert!(gp.achievements.is_empty());
        assert!((gp.impact_score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn add_achievement_accumulates() {
        let mut gp = create_great_person("Ada", 1, 10);
        add_achievement(&mut gp, "Invented programming", 0.8);
        add_achievement(&mut gp, "Advanced mathematics", 0.5);
        assert_eq!(gp.achievements.len(), 2);
        assert_eq!(gp.achievements[0], "Invented programming");
        assert!((gp.impact_score - 1.3).abs() < f32::EPSILON);
    }

    #[test]
    fn person_lifetime_while_alive() {
        let gp = create_great_person("Gandhi", 0, 50);
        assert_eq!(person_lifetime(&gp, 200), 150);
    }

    #[test]
    fn person_lifetime_after_death() {
        let mut gp = create_great_person("Gandhi", 0, 50);
        gp.death_tick = Some(120);
        // current_tick is ignored once death_tick is set
        assert_eq!(person_lifetime(&gp, 200), 70);
    }

    #[test]
    fn person_lifetime_at_birth() {
        let gp = create_great_person("Baby", 0, 100);
        assert_eq!(person_lifetime(&gp, 100), 0);
    }

    #[test]
    fn posthumous_impact_zero_while_alive() {
        let gp = create_great_person("Living", 0, 10);
        assert!((posthumous_impact(&gp, 0) - 0.0).abs() < f32::EPSILON);
        assert!((posthumous_impact(&gp, 999) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn posthumous_impact_decays() {
        let mut gp = create_great_person("Dead", 0, 0);
        gp.impact_score = 10.0;
        gp.death_tick = Some(10);
        let immediate = posthumous_impact(&gp, 0);
        let later = posthumous_impact(&gp, 200);
        assert!((immediate - 10.0).abs() < f32::EPSILON);
        assert!(later < immediate);
        assert!(later > 0.0);
    }

    // ── Era Transitions ──────────────────────────────────────────────

    #[test]
    fn era_definitions_count_and_names() {
        let defs = era_definitions();
        assert_eq!(defs.len(), 6);
        assert_eq!(defs[0].name, "Stone");
        assert_eq!(defs[1].name, "Bronze");
        assert_eq!(defs[2].name, "Iron");
        assert_eq!(defs[3].name, "Classical");
        assert_eq!(defs[4].name, "Medieval");
        assert_eq!(defs[5].name, "Industrial");
    }

    #[test]
    fn evaluate_era_transition_stone() {
        assert_eq!(evaluate_era_transition(0, 0, 0, 0), Some(CivAge::Stone));
    }

    #[test]
    fn evaluate_era_transition_bronze() {
        assert_eq!(evaluate_era_transition(0, 300, 1, 0), Some(CivAge::Bronze));
    }

    #[test]
    fn evaluate_era_transition_industrial() {
        assert_eq!(
            evaluate_era_transition(0, 8_000, 10, 100),
            Some(CivAge::Industrial)
        );
    }

    #[test]
    fn era_progress_percentage_boundaries() {
        assert!((era_progress_percentage(CivAge::Stone) - 0.0).abs() < f32::EPSILON);
        assert!((era_progress_percentage(CivAge::Industrial) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn era_progress_percentage_monotonic() {
        let eras = [
            CivAge::Stone,
            CivAge::Bronze,
            CivAge::Iron,
            CivAge::Classical,
            CivAge::Medieval,
            CivAge::Industrial,
        ];
        for window in eras.windows(2) {
            assert!(
                era_progress_percentage(window[0]) < era_progress_percentage(window[1]),
                "{:?} should be less than {:?}",
                window[0],
                window[1],
            );
        }
    }

    // ── What-If Branching ────────────────────────────────────────────

    #[test]
    fn alternate_timeline_creation() {
        let events = vec![make_event(10, "War", 0.8)];
        let alt = AlternateTimeline::new(events.clone(), 10);
        assert_eq!(alt.divergence_tick, 10);
        assert_eq!(alt.events.len(), 1);
        assert_eq!(alt.events[0].title, "War");
        assert!(alt.branch_points.is_empty());
    }

    #[test]
    fn alternate_timeline_add_branch_and_explore() {
        let events = vec![make_event(5, "Election", 0.4), make_event(15, "War", 0.9)];
        let mut alt = AlternateTimeline::new(events, 10);
        alt.add_branch(BranchPoint {
            tick: 10,
            event_description: "Peace vs War".into(),
            alternatives: vec!["Peace".into(), "War".into()],
            chosen: 1,
        });
        let explored = alt.explore_alternative(0, 0);
        assert_eq!(explored.len(), 1);
        assert_eq!(explored[0].tick, 5);
        assert_eq!(explored[0].title, "Election");
    }

    #[test]
    fn alternate_timeline_multiple_branches() {
        let events = vec![
            make_event(5, "A", 0.1),
            make_event(15, "B", 0.2),
            make_event(25, "C", 0.3),
        ];
        let mut alt = AlternateTimeline::new(events, 10);
        alt.add_branch(BranchPoint {
            tick: 10,
            event_description: "First fork".into(),
            alternatives: vec!["Left".into(), "Right".into()],
            chosen: 0,
        });
        alt.add_branch(BranchPoint {
            tick: 20,
            event_description: "Second fork".into(),
            alternatives: vec!["Up".into(), "Down".into()],
            chosen: 1,
        });
        // Explore first branch: events before tick 10
        let before_first = alt.explore_alternative(0, 0);
        assert_eq!(before_first.len(), 1);
        // Explore second branch: events before tick 20
        let before_second = alt.explore_alternative(1, 0);
        assert_eq!(before_second.len(), 2);
    }

    // ── Chronicle Generation ─────────────────────────────────────────

    #[test]
    fn generate_chronicle_empty_era() {
        let result = generate_chronicle(&[], "Classical");
        assert_eq!(result, "The Classical era passed without notable events.");
    }

    #[test]
    fn generate_chronicle_with_events() {
        let events = vec![TimelineEvent {
            id: 0,
            tick: 42,
            event_type: EventType::Religion,
            title: "Great Awakening".into(),
            description: "A spiritual movement".into(),
            actors: vec![1, 3],
            consequences: vec!["temples built".into()],
            importance: 0.7,
        }];
        let result = generate_chronicle(&events, "Medieval");
        assert!(result.contains("Great Awakening"));
        assert!(result.contains("Medieval"));
        assert!(result.contains("faction 1"));
        assert!(result.contains("temples built"));
    }

    #[test]
    fn generate_chronicle_no_actors() {
        let events = vec![TimelineEvent {
            id: 0,
            tick: 1,
            event_type: EventType::Disaster,
            title: "Plague".into(),
            description: "A devastating plague".into(),
            actors: Vec::new(),
            consequences: vec!["population decline".into()],
            importance: 0.8,
        }];
        let result = generate_chronicle(&events, "Iron");
        assert!(result.contains("Unknown actors"));
    }

    #[test]
    fn summarize_person_alive() {
        let gp = create_great_person("Archimedes", 2, 200);
        let summary = summarize_person(&gp);
        assert!(summary.contains("Archimedes"));
        assert!(summary.contains("alive"));
        assert!(summary.contains("faction 2"));
    }

    #[test]
    fn summarize_person_dead() {
        let mut gp = create_great_person("Plato", 1, 50);
        gp.death_tick = Some(100);
        let summary = summarize_person(&gp);
        assert!(summary.contains("Plato"));
        assert!(summary.contains("died tick 100"));
    }

    #[test]
    fn era_transition_summary_text() {
        let s = era_transition_summary(CivAge::Bronze, CivAge::Iron, 5);
        assert!(s.contains("Faction 5"));
        assert!(s.contains("Bronze"));
        assert!(s.contains("Iron"));
    }

    // ── Edge Cases ───────────────────────────────────────────────────

    #[test]
    fn timeline_serde_round_trip() {
        let mut tl = Timeline::new();
        tl.add_event(make_event(10, "Event A", 0.5));
        tl.add_event(make_event(20, "Event B", 0.8));
        let encoded = serde_json::to_string(&tl).expect("serialize timeline");
        let decoded: Timeline = serde_json::from_str(&encoded).expect("deserialize timeline");
        assert_eq!(decoded.events.len(), 2);
        assert_eq!(decoded.next_id, 2);
    }

    #[test]
    fn great_person_serde_round_trip() {
        let mut gp = create_great_person("Napoleon", 1, 500);
        add_achievement(&mut gp, "Conquered Europe", 0.9);
        gp.death_tick = Some(600);
        let encoded = serde_json::to_string(&gp).expect("serialize great person");
        let decoded: GreatPerson =
            serde_json::from_str(&encoded).expect("deserialize great person");
        assert_eq!(decoded.name, "Napoleon");
        assert_eq!(decoded.death_tick, Some(600));
        assert_eq!(decoded.achievements.len(), 1);
    }
}
