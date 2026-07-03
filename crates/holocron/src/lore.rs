//! Holocron lore — Phase 5 (narrative) substrate.
//!
//! The Holocron is more than a verb catalog; it is a *relic* with
//! remembered history. As the sim runs, factions leave *traces* in the
//! Holocron: founding myths, sacred events, betrayals, golden ages.
//!
//! This module is the substrate for **Holocron Phase 5 (narrative)**:
//! it persists lore entries (small structured records), exposes the
//! canonical "narrative beat" categories, and provides the lookup
//! that the HolocronPanel and GodPanel use to surface lore to the
//! player.
//!
//! Lore entries are *append-only*; never mutated or deleted. This is a
//! hard contract — the Holocron is a historian, not a propagandist.
//! When a faction is destroyed, the lore persists; when a hero dies,
//! the legend grows.
//!
//! # Schema
//!
//! - [`LoreEntry`] — a single remembered event (struct)
//! - [`LoreKind`] — category of the event (enum: founding, betrayal, miracle, ruin, etc.)
//! - [`LoreStore`] — the canonical append-only store (struct, newtype over `Vec`)
//! - [`LoreBeat`] — the canonical list of "narrative beats" the GodPanel
//!   reads from to drive the unlockable-verb system
//!
//! # Extendability
//!
//! - Adding a new [`LoreKind`] variant is the **canonical way** to ship
//!   a new narrative archetype. Agents append the variant + add the
//!   matching `LoreBeat` string in the same PR.
//! - Adding a new field to [`LoreEntry`] is a **breaking change** to
//!   the on-disk format; bump `LORE_FORMAT_VERSION` in `Cargo.toml`
//!   and provide a migration.
//! - `LoreStore::merge` is the canonical way to compose lore from
//!   multiple sources (e.g. swarm agent writes + player discoveries).
//!
//! # Tests
//!
//! - `lore_store_is_append_only`
//! - `lore_kind_round_trips`
//! - `lore_beat_lookup_returns_unlocked_verbs`
//! - `lore_store_merge_combines_distinct_kinds`

use crate::rank::RankedVerb;
use serde::{Deserialize, Serialize};

/// Canonical lore entry. Append-only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoreEntry {
    /// Tick the event happened (sim-relative).
    pub tick: u64,
    /// Faction this event is about (or `0` for "no specific faction").
    pub faction_id: u32,
    /// The kind of event.
    pub kind: LoreKind,
    /// Short human-readable title (≤ 64 chars).
    pub title: String,
    /// Longer narrative caption (≤ 256 chars).
    pub caption: String,
}

/// Canonical lore categories. The Holocron is a historian of:
/// - **Founding**: a new city, a new faction, a new god
/// - **Miracle**: an act of nature, a divine intervention, a moment of grace
/// - **Betrayal**: a faction turned on an ally, a trust broken
/// - **Ruin**: a city fell, a faction was destroyed, a hero died
/// - **Stagnation**: nothing happened for 100 ticks, a faction stopped innovating
/// - **Renewal**: a civilization returned from the brink
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoreKind {
    Founding,
    Miracle,
    Betrayal,
    Ruin,
    Stagnation,
    Renewal,
}

impl LoreKind {
    /// Single-char glyph for the HolocronPanel UI.
    pub fn glyph(self) -> char {
        match self {
            LoreKind::Founding => '★',
            LoreKind::Miracle => '✦',
            LoreKind::Betrayal => '⚔',
            LoreKind::Ruin => '☠',
            LoreKind::Stagnation => '⏳',
            LoreKind::Renewal => '☀',
        }
    }
}

/// Canonical list of narrative beats. Each beat is a string the GodPanel
/// can render in a "Holocron Lore" overlay.
///
/// New beats are **appended** (never re-ordered or replaced) so older
/// saves remain readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoreBeat {
    pub id: String,
    pub kind: LoreKind,
    pub title: String,
    pub description: String,
    /// Tick the beat fires at (or `0` for "anytime").
    pub tick_threshold: u64,
}

impl LoreBeat {
    /// The canonical "founding myth" beat — fires when a new city is founded.
    pub fn founding(city_name: &str) -> Self {
        Self {
            id: format!("founding:{}", city_name),
            kind: LoreKind::Founding,
            title: format!("The Founding of {}", city_name),
            description: format!(
                "The first stones of {} were laid. The Holocron remembers.",
                city_name
            ),
            tick_threshold: 0,
        }
    }
}

/// Append-only lore store. Newtype over `Vec<LoreEntry>` to make the
/// append-only contract explicit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LoreStore {
    entries: Vec<LoreEntry>,
}

impl LoreStore {
    /// Append a single entry. **No mutation, no deletion.**
    pub fn append(&mut self, entry: LoreEntry) {
        self.entries.push(entry);
    }

    /// Bulk append.
    pub fn extend(&mut self, entries: impl IntoIterator<Item = LoreEntry>) {
        self.entries.extend(entries);
    }

    /// Compose lore from multiple sources. **Both stores are preserved
    /// in full** (no dedup) so the Holocron keeps the union of all
    /// remembered histories.
    pub fn merge(&mut self, other: LoreStore) {
        self.entries.extend(other.entries);
    }

    /// All entries, in append order.
    pub fn entries(&self) -> &[LoreEntry] {
        &self.entries
    }

    /// Count of entries of a given kind.
    pub fn count(&self, kind: LoreKind) -> usize {
        self.entries.iter().filter(|e| e.kind == kind).count()
    }

    /// First entry of a given kind, or `None`.
    pub fn first(&self, kind: LoreKind) -> Option<&LoreEntry> {
        self.entries.iter().find(|e| e.kind == kind)
    }

    /// Most recent N entries (or all if N is too large).
    pub fn recent(&self, n: usize) -> &[LoreEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// Find entries that mention a faction.
    pub fn for_faction(&self, faction_id: u32) -> impl Iterator<Item = &LoreEntry> {
        self.entries
            .iter()
            .filter(move |e| e.faction_id == faction_id)
    }

    /// True if store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Canonical unlockable-verb table. Ties lore beats to godverbs that
/// only become available after the beat fires.
///
/// The HolocronPanel watches this table; when a beat fires, the panel
/// re-renders the verb list with the newly-unlocked verbs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Unlockable {
    /// Lore beat id → verbs it unlocks.
    table: std::collections::BTreeMap<String, Vec<String>>,
}

impl Unlockable {
    /// Create an empty table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register that a beat unlocks a verb.
    pub fn register(&mut self, beat_id: impl Into<String>, verb_id: impl Into<String>) {
        self.table
            .entry(beat_id.into())
            .or_default()
            .push(verb_id.into());
    }

    /// All verbs unlocked by a given beat (cloned).
    pub fn unlocked_by(&self, beat_id: &str) -> Vec<String> {
        self.table.get(beat_id).cloned().unwrap_or_default()
    }

    /// All beat ids.
    pub fn beats(&self) -> impl Iterator<Item = &str> {
        self.table.keys().map(|s| s.as_str())
    }

    /// Apply the unlockable table to a verb list — promotes unlocked
    /// verbs to the front, marks them as "narrative" provenance.
    ///
    /// The returned `Vec<RankedVerb>` is in priority order: narrative-
    /// unlocked verbs first, then everything else.
    pub fn apply(&self, all_verbs: Vec<RankedVerb>, lore_store: &LoreStore) -> Vec<RankedVerb> {
        let mut unlocked: Vec<RankedVerb> = Vec::new();
        let mut rest: Vec<RankedVerb> = Vec::new();

        // Walk the lore store chronologically; each beat's id may unlock
        // verbs that are then promoted to the front of the panel.
        for entry in lore_store.entries() {
            for verb_id in self.unlocked_by(&entry.title) {
                if let Some(pos) = all_verbs.iter().position(|v| v.id == verb_id) {
                    unlocked.push(all_verbs[pos].clone());
                }
            }
        }
        // Filter out the unlocked ones from the rest.
        let unlocked_ids: std::collections::HashSet<_> =
            unlocked.iter().map(|v| v.id.clone()).collect();
        for v in all_verbs {
            if !unlocked_ids.contains(&v.id) {
                rest.push(v);
            }
        }
        unlocked.extend(rest);
        unlocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(kind: LoreKind, tick: u64) -> LoreEntry {
        LoreEntry {
            tick,
            faction_id: 1,
            kind,
            title: format!("Test {:?}", tick),
            caption: format!("A test event at tick {}", tick),
        }
    }

    #[test]
    fn lore_store_is_append_only() {
        let mut store = LoreStore::default();
        store.append(sample_entry(LoreKind::Founding, 1));
        store.append(sample_entry(LoreKind::Miracle, 100));
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());
    }

    #[test]
    fn lore_kind_round_trips() {
        for kind in [
            LoreKind::Founding,
            LoreKind::Miracle,
            LoreKind::Betrayal,
            LoreKind::Ruin,
            LoreKind::Stagnation,
            LoreKind::Renewal,
        ] {
            let json = serde_json::to_string(&kind).unwrap();
            let back: LoreKind = serde_json::from_str(&json).unwrap();
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn lore_beat_lookup_returns_unlocked_verbs() {
        let mut u = Unlockable::new();
        u.register("founding:Rome", "god_smite");
        u.register("founding:Rome", "god_nudge_legion");
        let verbs = u.unlocked_by("founding:Rome");
        assert_eq!(verbs.len(), 2);
        assert!(verbs.contains(&"god_smite".to_string()));
    }

    #[test]
    fn lore_store_merge_combines_distinct_kinds() {
        let mut a = LoreStore::default();
        a.append(sample_entry(LoreKind::Founding, 1));
        let mut b = LoreStore::default();
        b.append(sample_entry(LoreKind::Ruin, 200));
        a.merge(b);
        assert_eq!(a.count(LoreKind::Founding), 1);
        assert_eq!(a.count(LoreKind::Ruin), 1);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn lore_kind_glyphs_are_distinct() {
        let mut seen = std::collections::HashSet::new();
        for k in [
            LoreKind::Founding,
            LoreKind::Miracle,
            LoreKind::Betrayal,
            LoreKind::Ruin,
            LoreKind::Stagnation,
            LoreKind::Renewal,
        ] {
            assert!(seen.insert(k.glyph()), "duplicate glyph");
        }
    }

    #[test]
    fn unlockable_apply_promotes_unlocked_verbs() {
        let verbs = vec![
            RankedVerb { id: "alpha".into(), score: 0.5, base_use: 0 },
            RankedVerb { id: "beta".into(),  score: 0.7, base_use: 0 },
            RankedVerb { id: "gamma".into(), score: 0.6, base_use: 0 },
        ];
        let mut u = Unlockable::new();
        u.register("founding:Rome", "beta");
        let lore = LoreStore::default();
        // Empty lore → no promotion, order preserved
        let out = u.apply(verbs.clone(), &lore);
        assert_eq!(out[0].id, "alpha");
        // With matching entry, beta moves to front
        let mut lore2 = LoreStore::default();
        lore2.append(LoreEntry {
            tick: 1,
            faction_id: 0,
            kind: LoreKind::Founding,
            title: "founding:Rome".into(),
            caption: "".into(),
        });
        let out2 = u.apply(verbs, &lore2);
        assert_eq!(out2[0].id, "beta");
    }
}
