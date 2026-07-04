//! Myth formation from recurring chronicle events (FR-CIV-MYTH-FORM).
//!
//! Repeated similar chronicle events crystallize into a named myth/archetype.
//! This module is a **pure-logic observer** over the existing
//! [`crate::chronicle::Chronicle`]: it scans the append-only log, counts
//! occurrences of each [`crate::chronicle::ChronicleEventKind`], and once
//! any kind crosses a configurable threshold it emits a named
//! [`Myth`]/[`Archetype`] record. The myth is a derived summary — the
//! chronicle itself stays append-only and untouched (emergence charter:
//! the engine never authors outcomes, it only labels them).
//!
//! ## Design rules
//!
//! - **Pure-logic.** No Bevy, no IO, no global state. The struct owns its
//!   data and is `Send + Sync`.
//! - **Deterministic + replay-stable.** The output is a pure function of
//!   the input stream and the configured [`MythFormationConfig`]. Two
//!   runs with the same input + config produce the same myths in the
//!   same order.
//! - **Append-only.** New myths are added; existing myths are never
//!   mutated or removed (a re-scan that finds more evidence updates
//!   the bookkeeping counts on a separate `evidence` field, but the
//!   `id` and `name` of a crystallized myth stay stable).
//! - **Additive.** This module does not modify any existing data
//!   structure; it only reads from [`crate::chronicle::Chronicle`] and
//!   produces a fresh [`MythIndex`].
//!
//! ## Myth naming
//!
//! A myth's `name` is the camel-case concatenation of the underlying
//! [`crate::chronicle::ChronicleEventKind`] label(s) plus a stable
//! ordinal so two crystallizations of different recurring patterns do
//! not collide (e.g. `MigrationI`, `MigrationII`, `ConflictI`). The
//! `archetype` is the kind label itself (e.g. `Migration`,
//! `Conflict`), giving downstream narration code a stable key to group
//! myths by.
//!
//! ## Acceptance
//!
//! > recurring event-type past threshold forms a named myth
//!
//! See the `fr_civ_myth_form_recurring_kind_past_threshold_forms_myth`
//! test below for the canonical input/output contract.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::chronicle::{Chronicle, ChronicleEntry, ChronicleEventKind};

/// Configuration for myth formation (FR-CIV-MYTH-FORM).
///
/// The default threshold matches the spec acceptance test
/// ("recurring event-type past threshold forms a named myth"). Lowering
/// the threshold makes myths crystallize faster; raising it makes them
/// rarer and more significant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MythFormationConfig {
    /// Minimum number of chronicle entries of the SAME
    /// [`ChronicleEventKind`] required to crystallize a myth. Must be
    /// `>= 1`; the default of `3` matches the spec.
    pub threshold: u32,
    /// Whether to coalesce `Other(label)` kinds by their label string
    /// (true, default) or by structural equality (false). Coalescing
    /// by label is the narratively meaningful choice — a producer can
    /// emit `Other("Plague")` many times and still form a single
    /// "Plague" myth.
    pub coalesce_other_by_label: bool,
}

impl Default for MythFormationConfig {
    fn default() -> Self {
        Self {
            threshold: 3,
            coalesce_other_by_label: true,
        }
    }
}

/// Stable id of a crystallized myth/archetype. Monotonically increasing
/// in the order myths crystallized (i.e. the order they were first
/// emitted by [`MythIndex::scan`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MythId(pub u64);

/// A named myth/archetype crystallized from a recurring chronicle
/// event-type (FR-CIV-MYTH-FORM).
///
/// `name` is the human-readable identifier (e.g. `"MigrationI"`,
/// `"PlagueI"`). `archetype` is the kind label (e.g. `"Migration"`,
/// `"Plague"`) so downstream code can group myths by recurring
/// pattern. `evidence` is the sorted list of [`LegendEventId`](crate::ids::LegendEventId)s
/// from the chronicle that fed the myth; on a re-scan the bookkeeping
/// `evidence` field can grow, but the `id` and `name` are stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Myth {
    /// Monotonic crystallization id, stable across re-scans.
    pub id: MythId,
    /// Stable human-readable name (e.g. `MigrationI`).
    pub name: String,
    /// The recurring event-type label this myth crystallizes around
    /// (e.g. `Migration`, `Plague`, `Conflict`).
    pub archetype: String,
    /// [`ChronicleEntry::id`]s that contributed evidence, in append
    /// order. Always non-empty — a myth requires at least `threshold`
    /// entries to exist.
    pub evidence: Vec<crate::ids::LegendEventId>,
    /// First epoch at which the myth crystallized (the epoch of the
    /// entry that crossed the threshold).
    pub first_epoch: crate::ids::Epoch,
    /// Last epoch at which new evidence was observed. Updated on
    /// re-scan; for a one-shot scan equals `first_epoch`.
    pub last_epoch: crate::ids::Epoch,
}

/// Bookkeeping for one recurring event-type observed while scanning
/// the chronicle. Internal to [`MythIndex`]; exposed only via the
/// public [`MythIndex::counts`] accessor for tests + inspector views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct KindBucket {
    archetype: String,
    evidence: Vec<crate::ids::LegendEventId>,
    first_epoch: crate::ids::Epoch,
    last_epoch: crate::ids::Epoch,
    /// Number of times this bucket has crystallized into distinct
    /// myths. Each crystallization requires `threshold` new entries —
    /// so a stream of 7 `Migration` entries with threshold 3 produces
    /// exactly 2 myths (`MigrationI`, `MigrationII`), leaving 1 entry
    /// of "uncommitted" slack in the running tally.
    crystallized: u32,
    /// Number of entries accumulated since the last crystallization.
    /// Once this reaches `threshold` a new myth is emitted and the
    /// counter resets to 0.
    pending: u32,
}

/// Append-only deterministic index of crystallized myths
/// (FR-CIV-MYTH-FORM).
///
/// `myths` is in crystallization order (insertion order); `next_id` is
/// the counter the next crystallization will hand out. `buckets` is
/// the per-kind running tally — public read-only via
/// [`MythIndex::counts`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MythIndex {
    /// The configured threshold + coalescing policy.
    pub config: MythFormationConfig,
    /// Crystallized myths, in crystallization order. Append-only.
    pub myths: Vec<Myth>,
    /// Monotonic counter for the next crystallization. Equal to
    /// `myths.last().map(|m| m.id.0 + 1).unwrap_or(0)` between scans.
    pub next_id: u64,
    /// Internal per-kind running tally. Kept on the struct so a re-scan
    /// can continue crystallizing myths incrementally instead of
    /// starting from scratch.
    buckets: BTreeMap<String, KindBucket>,
}

impl MythIndex {
    /// Build an empty myth index with the given configuration.
    pub fn new(config: MythFormationConfig) -> Self {
        Self {
            config,
            myths: Vec::new(),
            next_id: 0,
            buckets: BTreeMap::new(),
        }
    }

    /// Build an empty myth index with [`MythFormationConfig::default`].
    pub fn with_default_config() -> Self {
        Self::new(MythFormationConfig::default())
    }

    /// Number of crystallized myths.
    pub fn len(&self) -> usize {
        self.myths.len()
    }

    /// True iff no myths have crystallized.
    pub fn is_empty(&self) -> bool {
        self.myths.is_empty()
    }

    /// All crystallized myths in crystallization order.
    pub fn all(&self) -> &[Myth] {
        &self.myths
    }

    /// Look up a myth by its stable [`MythId`]. Returns `None` if the
    /// id has not been crystallized yet.
    pub fn get(&self, id: MythId) -> Option<&Myth> {
        self.myths.iter().find(|m| m.id == id)
    }

    /// All myths sharing a given archetype label, in crystallization
    /// order. An empty slice is returned if no myth of that archetype
    /// has crystallized.
    pub fn by_archetype(&self, archetype: &str) -> Vec<&Myth> {
        self.myths
            .iter()
            .filter(|m| m.archetype == archetype)
            .collect()
    }

    /// Read-only view of the per-archetype running tally
    /// (`archetype -> (evidence_count, crystallized_count,
    /// pending_count)`). Useful for inspector dashboards that want to
    /// show "almost-myth" previews.
    pub fn counts(&self) -> BTreeMap<String, (usize, u32, u32)> {
        self.buckets
            .iter()
            .map(|(k, b)| (k.clone(), (b.evidence.len(), b.crystallized, b.pending)))
            .collect()
    }

    /// Scan the chronicle, crystallize any newly-eligible myths, and
    /// return the freshly-formed myths from this scan in crystallization
    /// order.
    ///
    /// The function is **idempotent over the same chronicle contents**
    /// — a second scan that finds no new entries returns an empty Vec
    /// and does not double-crystallize (the bucket running tally is
    /// preserved across scans, so each scan only emits "new" myths).
    /// To rebuild from scratch, construct a fresh [`MythIndex`] and
    /// call [`MythIndex::scan_chronicle`] once.
    ///
    /// The acceptance contract for FR-CIV-MYTH-FORM: a recurring
    /// event-type past the configured threshold MUST form a named
    /// myth.
    pub fn scan_chronicle(&mut self, chronicle: &Chronicle) -> Vec<Myth> {
        let mut newly_crystallized = Vec::new();
        for entry in chronicle.all() {
            if let Some(myth) = self.observe(entry) {
                self.myths.push(myth.clone());
                newly_crystallized.push(myth);
            }
        }
        newly_crystallized
    }

    /// Observe a single chronicle entry; returns `Some(Myth)` iff this
    /// observation crystallized a new myth. Splitting this from
    /// [`MythIndex::scan_chronicle`] lets callers feed entries from a
    /// live event stream without re-scanning the whole chronicle.
    pub fn observe(&mut self, entry: &ChronicleEntry) -> Option<Myth> {
        // Threshold 0 would crystallize a myth per single event, which
        // is meaningless. The contract is "past threshold", so any
        // value < 1 is clamped to 1 — equal to one myth per event,
        // which is the degenerate upper bound. Producers that want
        // "every event is a myth" should pass threshold=1 explicitly.
        let threshold = self.config.threshold.max(1);

        let archetype = self.archetype_key(entry);
        let bucket = self
            .buckets
            .entry(archetype.clone())
            .or_insert_with(|| KindBucket {
                archetype: archetype.clone(),
                evidence: Vec::new(),
                first_epoch: entry.epoch,
                last_epoch: entry.epoch,
                crystallized: 0,
                pending: 0,
            });

        if bucket.evidence.contains(&entry.id) {
            return None;
        }

        bucket.evidence.push(entry.id);
        bucket.last_epoch = entry.epoch;
        bucket.pending = bucket.pending.saturating_add(1);

        if bucket.pending < threshold {
            return None;
        }

        // Threshold crossed: crystallize a new myth, reset the
        // pending counter. The crystallized counter advances so the
        // next myth of this archetype is `MigrationII`, `MigrationIII`,
        // etc.
        bucket.crystallized = bucket.crystallized.saturating_add(1);
        bucket.pending = 0;

        let myth = Myth {
            id: MythId(self.next_id),
            name: format!("{}{}", archetype, roman::to_roman(bucket.crystallized)),
            archetype: archetype.clone(),
            // Snapshot the most recent `threshold` entries as evidence
            // for this crystallization. We grab them by index from the
            // back so older crystallizations' evidence is not double-
            // counted on a future scan.
            evidence: bucket
                .evidence
                .iter()
                .rev()
                .take(threshold as usize)
                .copied()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
            first_epoch: entry.epoch,
            last_epoch: entry.epoch,
        };
        self.next_id = self.next_id.checked_add(1).expect("MythId overflow");
        Some(myth)
    }

    /// Resolve a chronicle event kind to its archetype key — the
    /// stable string that groups recurring events together for the
    /// purpose of myth formation.
    fn archetype_key(&self, entry: &ChronicleEntry) -> String {
        match &entry.kind {
            // Closed taxonomy: each variant is its own archetype.
            ChronicleEventKind::Birth => "Birth".to_string(),
            ChronicleEventKind::Death => "Death".to_string(),
            ChronicleEventKind::Migration => "Migration".to_string(),
            ChronicleEventKind::Conflict => "Conflict".to_string(),
            ChronicleEventKind::Founding => "Founding".to_string(),
            ChronicleEventKind::FirstContact => "FirstContact".to_string(),
            ChronicleEventKind::Other(label) => {
                if self.config.coalesce_other_by_label {
                    label.clone()
                } else {
                    // Structural equality disabled: every emitted `Other`
                    // event remains isolated and cannot accumulate with
                    // same-label events into a named myth.
                    format!("Other({label})#{}", entry.id.0)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! FR-CIV-MYTH-FORM acceptance test.
    //!
    //! Acceptance criteria (from the task):
    //! 1. A recurring event-type past the threshold forms a named myth.
    //! 2. The myth is named (stable `name` + `archetype`).
    //! 3. Multiple distinct archetypes crystallize independently.
    //! 4. Below threshold, NO myth forms.
    //! 5. Replay-stable: the same chronicle + config yields the same
    //!    myths in the same order.
    use super::*;
    use crate::chronicle::Chronicle;
    use crate::ids::{ClusterId, Epoch, LegendEntityId, LegendEventId, RegionId};
    use smallvec::SmallVec;

    fn region(n: u64) -> RegionId {
        RegionId(n)
    }
    fn cluster(n: u64) -> ClusterId {
        ClusterId(n)
    }

    fn parts(ids: &[u64]) -> SmallVec<[LegendEntityId; 4]> {
        SmallVec::from_iter(ids.iter().copied().map(LegendEntityId))
    }

    /// Convenience: append `n` events of `kind` to a chronicle with
    /// monotonically increasing tick + epoch.
    fn push_n(c: &mut Chronicle, n: u64, kind: ChronicleEventKind, start_tick: u64) {
        for i in 0..n {
            c.record(
                start_tick + i,
                Epoch((start_tick + i) / 4),
                kind.clone(),
                Some(region(1)),
                Some(cluster(1)),
                parts(&[1]),
            );
        }
    }

    /// FR-CIV-MYTH-FORM acceptance: a recurring event-type past the
    /// threshold forms a named myth.
    #[test]
    fn fr_civ_myth_form_recurring_kind_past_threshold_forms_myth() {
        let mut chronicle = Chronicle::new();
        // 4 migrations, threshold 3 -> exactly one `MigrationI` myth.
        push_n(&mut chronicle, 4, ChronicleEventKind::Migration, 0);

        let mut idx = MythIndex::with_default_config();
        let new = idx.scan_chronicle(&chronicle);

        // The acceptance contract: a named myth was formed.
        assert_eq!(
            new.len(),
            1,
            "4 migrations past threshold 3 must form exactly one myth"
        );
        let myth = &new[0];
        assert_eq!(
            myth.archetype, "Migration",
            "myth archetype must be the kind label"
        );
        assert_eq!(
            myth.name, "MigrationI",
            "first Migration myth must be named `MigrationI`"
        );
        assert_eq!(
            myth.evidence.len(),
            3,
            "evidence must hold exactly the threshold entries"
        );
        assert!(idx.is_empty() == false);
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.all()[0].name, "MigrationI");
    }

    /// Below threshold: no myth forms. Above threshold (the next
    /// `threshold` entries of the same archetype): a SECOND myth
    /// forms. The crystallized counter advances deterministically.
    #[test]
    fn fr_civ_myth_form_below_threshold_no_myth_above_threshold_repeated() {
        let mut chronicle = Chronicle::new();
        // 2 migrations is below the default threshold of 3 -> no myth.
        push_n(&mut chronicle, 2, ChronicleEventKind::Migration, 0);

        let mut idx = MythIndex::with_default_config();
        assert!(idx.scan_chronicle(&chronicle).is_empty());
        assert_eq!(idx.len(), 0);
        assert!(idx.all().is_empty());
        assert!(idx.by_archetype("Migration").is_empty());

        // Add 4 more migrations: 2 + 4 = 6 total. Threshold 3 -> 2 myths
        // (6 / 3 == 2). The 2 leftover "pending" entries do not
        // crystallize a 3rd myth.
        push_n(&mut chronicle, 4, ChronicleEventKind::Migration, 2);
        let new = idx.scan_chronicle(&chronicle);

        assert_eq!(
            new.len(),
            2,
            "6 migrations past threshold 3 must form exactly two myths (got {:?})",
            new.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
        assert_eq!(new[0].name, "MigrationI");
        assert_eq!(new[1].name, "MigrationII");
        assert_eq!(new[0].id, MythId(0));
        assert_eq!(new[1].id, MythId(1));

        // One pending migration left over — `counts()` reflects this.
        let counts = idx.counts();
        let migration = counts
            .get("Migration")
            .expect("Migration bucket must exist");
        assert_eq!(migration.0, 6, "all 6 entries recorded as evidence");
        assert_eq!(migration.1, 2, "2 crystallized myths");
        assert_eq!(
            migration.2, 0,
            "0 pending after the second crystallization consumed 3 entries"
        );
    }

    /// Distinct archetypes crystallize independently and produce
    /// distinct named myths.
    #[test]
    fn fr_civ_myth_form_distinct_archetypes_crystallize_independently() {
        let mut chronicle = Chronicle::new();
        // 3 migrations + 3 conflicts interleaved -> 1 MigrationI + 1
        // ConflictI, in the order each archetype first crosses
        // threshold.
        let interleaved = [
            ChronicleEventKind::Migration,
            ChronicleEventKind::Conflict,
            ChronicleEventKind::Migration,
            ChronicleEventKind::Conflict,
            ChronicleEventKind::Migration,
            ChronicleEventKind::Conflict,
        ];
        for (i, k) in interleaved.iter().enumerate() {
            chronicle.record(
                i as u64,
                Epoch(0),
                k.clone(),
                Some(region(1)),
                Some(cluster(1)),
                parts(&[1]),
            );
        }

        let mut idx = MythIndex::with_default_config();
        let new = idx.scan_chronicle(&chronicle);

        assert_eq!(
            new.len(),
            2,
            "two archetypes crossed threshold -> two myths"
        );
        let names: Vec<&str> = new.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"MigrationI"));
        assert!(names.contains(&"ConflictI"));

        // by_archetype returns each in isolation.
        assert_eq!(idx.by_archetype("Migration").len(), 1);
        assert_eq!(idx.by_archetype("Conflict").len(), 1);
        assert_eq!(idx.by_archetype("Founding").len(), 0);

        // Lookup by stable id works.
        assert!(idx.get(MythId(0)).is_some());
        assert!(idx.get(MythId(99)).is_none());
    }

    /// `Other(label)` events coalesce by label (default) — repeated
    /// `Other("Plague")` crystallizes a single "PlagueI" myth.
    #[test]
    fn fr_civ_myth_form_other_label_coalesces_by_default() {
        let mut chronicle = Chronicle::new();
        for i in 0..5 {
            chronicle.record(
                i,
                Epoch(0),
                ChronicleEventKind::Other("Plague".to_string()),
                Some(region(1)),
                Some(cluster(1)),
                parts(&[1]),
            );
        }

        let mut idx = MythIndex::with_default_config();
        let new = idx.scan_chronicle(&chronicle);

        // 5 plagues, threshold 3 -> 1 myth (3 evidence), 2 pending.
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].archetype, "Plague");
        assert_eq!(new[0].name, "PlagueI");
        assert_eq!(new[0].evidence.len(), 3);

        // With coalescing disabled, every distinct `Other` string is
        // a separate archetype. A single `Other("Plague")` bucket
        // alone can never cross threshold 3 — so NO myth forms.
        let mut chronicle2 = Chronicle::new();
        for i in 0..5 {
            chronicle2.record(
                i,
                Epoch(0),
                ChronicleEventKind::Other("Plague".to_string()),
                Some(region(1)),
                Some(cluster(1)),
                parts(&[1]),
            );
        }
        let mut idx2 = MythIndex::new(MythFormationConfig {
            threshold: 3,
            coalesce_other_by_label: false,
        });
        assert!(idx2.scan_chronicle(&chronicle2).is_empty());
    }

    /// Replay-stable: two independent indexes fed the same chronicle
    /// produce identical myth sequences.
    #[test]
    fn fr_civ_myth_form_replay_stable_under_same_input_stream() {
        let mut chronicle = Chronicle::new();
        // A mixed stream designed to crystallize 1 of each kind.
        let kinds = [
            ChronicleEventKind::Migration,
            ChronicleEventKind::Birth,
            ChronicleEventKind::Migration,
            ChronicleEventKind::Founding,
            ChronicleEventKind::Migration,
            ChronicleEventKind::Birth,
            ChronicleEventKind::Founding,
            ChronicleEventKind::Birth,
            ChronicleEventKind::Founding,
        ];
        for (i, k) in kinds.iter().enumerate() {
            chronicle.record(
                i as u64,
                Epoch((i as u64) / 4),
                k.clone(),
                Some(region(1)),
                Some(cluster(1)),
                parts(&[1]),
            );
        }

        let mut a = MythIndex::with_default_config();
        let mut b = MythIndex::with_default_config();
        let new_a = a.scan_chronicle(&chronicle);
        let new_b = b.scan_chronicle(&chronicle);

        assert_eq!(new_a.len(), new_b.len());
        for (ma, mb) in new_a.iter().zip(new_b.iter()) {
            assert_eq!(ma.id, mb.id);
            assert_eq!(ma.name, mb.name);
            assert_eq!(ma.archetype, mb.archetype);
            assert_eq!(ma.evidence, mb.evidence);
        }
        assert_eq!(a.all(), b.all());
        assert_eq!(a.next_id, b.next_id);

        // Serialized form must also be identical (save/load round trip
        // contract).
        let json_a = serde_json::to_string(&a).expect("serialize a");
        let json_b = serde_json::to_string(&b).expect("serialize b");
        assert_eq!(json_a, json_b);
    }

    /// Re-scanning the same chronicle does NOT double-crystallize:
    /// the bucket running tally persists across scans.
    #[test]
    fn fr_civ_myth_form_rescan_does_not_double_crystallize() {
        let mut chronicle = Chronicle::new();
        push_n(&mut chronicle, 4, ChronicleEventKind::Migration, 0);

        let mut idx = MythIndex::with_default_config();
        let first = idx.scan_chronicle(&chronicle);
        assert_eq!(first.len(), 1);
        assert_eq!(idx.len(), 1);

        // A second scan on the same chronicle MUST NOT emit more myths.
        let second = idx.scan_chronicle(&chronicle);
        assert!(
            second.is_empty(),
            "rescan must not double-crystallize (got {:?})",
            second.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
        assert_eq!(idx.len(), 1);
    }

    /// Live `observe` works entry-by-entry without re-scanning.
    #[test]
    fn fr_civ_myth_form_observe_entry_by_entry_matches_scan() {
        let mut chronicle = Chronicle::new();
        push_n(&mut chronicle, 5, ChronicleEventKind::Conflict, 0);

        // Scan-driven path.
        let mut scan_idx = MythIndex::with_default_config();
        let scan_myths = scan_idx.scan_chronicle(&chronicle);
        let scan_names: Vec<String> = scan_myths.iter().map(|m| m.name.clone()).collect();

        // Observe-driven path.
        let mut obs_idx = MythIndex::with_default_config();
        let mut obs_myths = Vec::new();
        for e in chronicle.all() {
            if let Some(m) = obs_idx.observe(e) {
                obs_myths.push(m);
            }
        }
        let obs_names: Vec<String> = obs_myths.iter().map(|m| m.name.clone()).collect();

        assert_eq!(scan_names, obs_names, "scan and observe must agree");
    }

    /// Empty chronicle -> empty index, no myths.
    #[test]
    fn fr_civ_myth_form_empty_chronicle_forms_no_myths() {
        let chronicle = Chronicle::new();
        let mut idx = MythIndex::with_default_config();
        let new = idx.scan_chronicle(&chronicle);
        assert!(new.is_empty());
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert!(idx.all().is_empty());
    }

    /// Sanity: a myth's `first_epoch` and `last_epoch` are set to the
    /// epoch of the entry that crossed the threshold.
    #[test]
    fn fr_civ_myth_form_epoch_fields_are_set_on_crystallization() {
        let mut chronicle = Chronicle::new();
        // Spread across epochs so the threshold-crossing epoch is
        // clearly the third one (epoch 4 / 4 = 1).
        chronicle.record(
            0,
            Epoch(0),
            ChronicleEventKind::Migration,
            Some(region(1)),
            Some(cluster(1)),
            parts(&[1]),
        );
        chronicle.record(
            2,
            Epoch(0),
            ChronicleEventKind::Migration,
            Some(region(1)),
            Some(cluster(1)),
            parts(&[1]),
        );
        chronicle.record(
            8,
            Epoch(2),
            ChronicleEventKind::Migration,
            Some(region(1)),
            Some(cluster(1)),
            parts(&[1]),
        );

        let mut idx = MythIndex::with_default_config();
        let new = idx.scan_chronicle(&chronicle);

        assert_eq!(new.len(), 1);
        let m = &new[0];
        assert_eq!(m.first_epoch, Epoch(2));
        assert_eq!(m.last_epoch, Epoch(2));
        // The evidence entries are the 3 contributing ids, in append
        // order (ids 0, 1, 2).
        assert_eq!(
            m.evidence,
            vec![LegendEventId(0), LegendEventId(1), LegendEventId(2)]
        );
    }
}

// ===========================================================================
// Roman-numeral helper (pure-logic, no external crate, deterministic).
// ===========================================================================
//
// Used to name successive crystallizations of the same archetype
// (`MigrationI`, `MigrationII`, ...). Implementation is the standard
// subtractive-roman; it covers 1..=3999 which is far above any
// realistic crystallization count.
mod roman {
    pub fn to_roman(mut n: u32) -> String {
        const SYMBOLS: &[(&str, u32)] = &[
            ("M", 1000),
            ("CM", 900),
            ("D", 500),
            ("CD", 400),
            ("C", 100),
            ("XC", 90),
            ("L", 50),
            ("XL", 40),
            ("X", 10),
            ("IX", 9),
            ("V", 5),
            ("IV", 4),
            ("I", 1),
        ];
        let mut out = String::new();
        for &(sym, val) in SYMBOLS {
            while n >= val {
                out.push_str(sym);
                n -= val;
            }
        }
        out
    }
}
