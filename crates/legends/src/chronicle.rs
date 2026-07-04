//! Append-only deterministic chronicle of significant events (FR-CIV-PSYCHE-920).
//!
//! Per `docs/specs/requirements/FR-CIV-PSYCHE.md`:
//!
//! > The engine SHALL record significant emergent events into a queryable
//! > chronicle (births, deaths, migrations, conflicts, foundings,
//! > first-contacts).
//!
//! > Chronicle is append-only, deterministic, replay-stable; queryable per
//! > agent/place/polity → "legends mode".
//!
//! This module is the *factual* append-only record. It complements
//! [`crate::rumor::Chronicle`] (a single historian's witnessed-event subset)
//! by storing the **sim-wide** log of every significant event the engine
//! chooses to record — agent deaths, polity foundings, migrations, conflicts,
//! first-contacts — in the order they were recorded. Order is the append
//! order, never re-sorted, so a replay that produces the same input stream
//! always produces the same chronicle (FR-CIV-PSYCHE-920 "deterministic,
//! replay-stable").
//!
//! ## Design rules
//!
//! - **Append-only.** No public mutator removes or reorders entries. The
//!   only way to add an entry is [`Chronicle::record`].
//! - **Deterministic.** Ids are a monotonically increasing counter
//!   (`next_id`). Query results preserve insertion order, so the output
//!   is a pure function of the input stream — two runs with the same
//!   recorded events produce identical chronicles.
//! - **Pure-logic.** No Bevy, no IO, no global state. The struct owns its
//!   data and is `Send + Sync`.
//! - **Queryable per agent / place / polity.** [`Chronicle::by_agent`],
//!   [`Chronicle::by_place`] (region), and [`Chronicle::by_polity`]
//!   (cluster) return the matching subset in append order.
//!
//! This module does not depend on Bevy or any rendering layer; it is
//! consumed by the sim worker and the inspector's "legends mode" panel.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::ids::{ClusterId, Epoch, LegendEntityId, LegendEventId, RegionId};

/// A single significant event recorded into the chronicle
/// (FR-CIV-PSYCHE-920). Each entry captures the minimum a "legends mode"
/// UI needs to render a timeline row: when, where, what, and who.
///
/// The event kinds are the FR's enumerated examples (births, deaths,
/// migrations, conflicts, foundings, first-contacts) plus an `Other`
/// escape hatch so producers can extend the taxonomy without an engine
/// change. This mirrors the open-taxonomy pattern in
/// [`crate::model::EventKind`] but is local to the chronicle so the
/// chronicle can stand alone as a pure-logic module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChronicleEventKind {
    Birth,
    Death,
    Migration,
    Conflict,
    Founding,
    FirstContact,
    /// Escape hatch so producers can extend without an engine change.
    Other(String),
}

impl ChronicleEventKind {
    /// Stable human-readable label (the only thing the engine knows about
    /// a kind). `Other` carries its own label so the UI never has to
    /// special-case it.
    pub fn label(&self) -> &str {
        match self {
            ChronicleEventKind::Birth => "Birth",
            ChronicleEventKind::Death => "Death",
            ChronicleEventKind::Migration => "Migration",
            ChronicleEventKind::Conflict => "Conflict",
            ChronicleEventKind::Founding => "Founding",
            ChronicleEventKind::FirstContact => "FirstContact",
            ChronicleEventKind::Other(s) => s.as_str(),
        }
    }
}

/// One row of the chronicle (FR-CIV-PSYCHE-920). `participants` is the
/// agent set the event is "about"; a query for a single agent returns
/// every entry whose `participants` contains that id.
///
/// `place` and `polity` are optional: a sim-wide `Founding` may have no
/// specific region (only a cluster id), and a `Birth` always has a
/// participant agent but may not have a cluster yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChronicleEntry {
    /// Monotonic, append-order id (never reused).
    pub id: LegendEventId,
    /// Sim tick the producer recorded.
    pub tick: u64,
    /// Coarse epoch bucket (`tick / ticks_per_epoch`).
    pub epoch: Epoch,
    /// Spatial region the event occurred in, if known.
    pub place: Option<RegionId>,
    /// Polity cluster the event is attributed to, if known.
    pub polity: Option<ClusterId>,
    /// Kind of significant event (FR-CIV-PSYCHE-920 enumerated kinds).
    pub kind: ChronicleEventKind,
    /// All agents that participated in the event. Used by
    /// [`Chronicle::by_agent`].
    pub participants: SmallVec<[LegendEntityId; 4]>,
}

/// Append-only deterministic chronicle of significant events
/// (FR-CIV-PSYCHE-920).
///
/// `entries` is the full log in append order; `next_id` is the counter
/// the next [`Chronicle::record`] will hand out. The struct is the only
/// mutating surface — there is no global state, no IO, no Bevy
/// dependency.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chronicle {
    /// Append-order log. Never re-sorted; queries preserve this order
    /// (the acceptance contract: "appending events preserves order").
    pub entries: Vec<ChronicleEntry>,
    /// Monotonic counter for the next [`Chronicle::record`] call.
    /// `entries.last().map(|e| e.id.0 + 1).unwrap_or(0)` is always
    /// equal to `next_id` between calls; the field is materialized so
    /// the constructor never has to scan.
    pub next_id: u64,
}

impl Chronicle {
    /// Build an empty chronicle. The next [`Chronicle::record`] will
    /// assign `id = 0`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a chronicle pre-loaded with the given entries, assigning ids
    /// in the order they appear. Used by replay/loading paths; the
    /// acceptance contract ("appending events preserves order") means
    /// loading a stream then appending more produces the same id
    /// sequence as a single stream of the same total length.
    ///
    /// Panics if `entries` is non-empty and any id is non-monotonic
    /// (strictly increasing), since that would mean the caller broke the
    /// append-only invariant. An empty `entries` slice is always valid.
    pub fn from_entries(entries: Vec<ChronicleEntry>) -> Self {
        let mut next_id: u64 = 0;
        for (i, e) in entries.iter().enumerate() {
            if i > 0 && e.id.0 <= entries[i - 1].id.0 {
                panic!(
                    "Chronicle::from_entries: ids must be strictly increasing (entry {} id={} <= prev id={})",
                    i,
                    e.id.0,
                    entries[i - 1].id.0
                );
            }
            next_id = e.id.0 + 1;
        }
        Self { entries, next_id }
    }

    /// Append a new significant event to the chronicle.
    ///
    /// Returns the assigned [`LegendEventId`], which is the previous
    /// `next_id` value. The entry is pushed onto `entries` and the
    /// counter advances — no other mutation is performed.
    ///
    /// The function never panics on input shape; an empty
    /// `participants` slice is permitted (some sim-wide events, e.g. a
    /// polity founding with no specific agent subject, legitimately
    /// have no participants).
    pub fn record(
        &mut self,
        tick: u64,
        epoch: Epoch,
        kind: ChronicleEventKind,
        place: Option<RegionId>,
        polity: Option<ClusterId>,
        participants: SmallVec<[LegendEntityId; 4]>,
    ) -> LegendEventId {
        let id = LegendEventId(self.next_id);
        let entry = ChronicleEntry {
            id,
            tick,
            epoch,
            place,
            polity,
            kind,
            participants,
        };
        self.entries.push(entry);
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("Chronicle::record: id overflow");
        id
    }

    /// All entries, in append order.
    pub fn all(&self) -> &[ChronicleEntry] {
        &self.entries
    }

    /// Number of entries currently in the chronicle.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True iff no entries have been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries whose `participants` contains `agent`, in append order.
    ///
    /// The acceptance contract for FR-CIV-PSYCHE-920: a query by agent
    /// MUST return only that agent's events. The filter is exact-match
    /// on the `LegendEntityId` — no fuzzy / no partial.
    pub fn by_agent(&self, agent: LegendEntityId) -> Vec<&ChronicleEntry> {
        self.entries
            .iter()
            .filter(|e| e.participants.iter().any(|p| *p == agent))
            .collect()
    }

    /// Entries whose `place` equals `region`, in append order.
    /// Entries with `place == None` are NEVER returned (the agent is
    /// "about" a place, and a no-place event is not a place event).
    pub fn by_place(&self, region: RegionId) -> Vec<&ChronicleEntry> {
        self.entries
            .iter()
            .filter(|e| e.place == Some(region))
            .collect()
    }

    /// Entries whose `polity` equals `cluster`, in append order. Same
    /// `Option` semantics as [`Chronicle::by_place`].
    pub fn by_polity(&self, cluster: ClusterId) -> Vec<&ChronicleEntry> {
        self.entries
            .iter()
            .filter(|e| e.polity == Some(cluster))
            .collect()
    }

    /// Filter by event kind, in append order. Useful for the
    /// "foundings in epoch 12" type inspector query.
    pub fn by_kind(&self, kind: &ChronicleEventKind) -> Vec<&ChronicleEntry> {
        self.entries.iter().filter(|e| &e.kind == kind).collect()
    }
}

#[cfg(test)]
mod tests {
    //! FR-CIV-PSYCHE-920 acceptance test.
    //!
    //! Acceptance criteria (from the task):
    //! 1. Appending events preserves order.
    //! 2. Query by agent returns only that agent's events.
    //! 3. Deterministic — same input stream ⇒ same chronicle.
    use super::*;
    use smallvec::smallvec;

    fn agent(n: u64) -> LegendEntityId {
        LegendEntityId(n)
    }

    fn region(n: u64) -> RegionId {
        RegionId(n)
    }

    fn cluster(n: u64) -> ClusterId {
        ClusterId(n)
    }

    /// Convenience: build a small `participants` smallvec without
    /// importing the macro at every call site.
    fn parts(ids: &[u64]) -> SmallVec<[LegendEntityId; 4]> {
        SmallVec::from_iter(ids.iter().copied().map(LegendEntityId))
    }

    /// Acceptance: appending preserves order + query by agent returns
    /// only that agent's events.
    #[test]
    fn fr_civ_psyche_920_appending_preserves_order_and_query_by_agent_is_exact() {
        let mut c = Chronicle::new();

        // Record a stream of events for two agents interleaved.
        let alice = agent(1);
        let bob = agent(2);
        let r_a = region(10);
        let r_b = region(20);
        let p_x = cluster(100);

        c.record(
            0,
            Epoch(0),
            ChronicleEventKind::Birth,
            None,
            None,
            parts(&[1]),
        );
        c.record(
            5,
            Epoch(0),
            ChronicleEventKind::Migration,
            Some(r_a),
            None,
            parts(&[1]),
        );
        c.record(
            7,
            Epoch(0),
            ChronicleEventKind::Founding,
            Some(r_b),
            Some(p_x),
            parts(&[2]),
        );
        c.record(
            12,
            Epoch(1),
            ChronicleEventKind::Conflict,
            Some(r_a),
            None,
            parts(&[1, 2]),
        );
        c.record(
            20,
            Epoch(2),
            ChronicleEventKind::Death,
            Some(r_a),
            Some(p_x),
            parts(&[2]),
        );

        // 1) Appending preserves order: ids are monotonic and the
        //    tick/seq in `entries` matches the call order.
        let ids: Vec<u64> = c.all().iter().map(|e| e.id.0).collect();
        assert_eq!(
            ids,
            vec![0, 1, 2, 3, 4],
            "append-order ids must be monotonic"
        );
        for window in c.all().windows(2) {
            assert!(
                window[0].tick <= window[1].tick,
                "ticks must be non-decreasing in append order (got {} > {})",
                window[0].tick,
                window[1].tick
            );
        }

        // 2) Query by agent returns ONLY that agent's events, in order.
        let alice_events = c.by_agent(alice);
        let alice_kinds: Vec<&str> = alice_events.iter().map(|e| e.kind.label()).collect();
        assert_eq!(
            alice_kinds,
            vec!["Birth", "Migration", "Conflict"],
            "Alice must see exactly her own events"
        );
        // The conflict is shared — Bob must also see it.
        for e in &alice_events {
            assert!(
                e.participants.iter().any(|p| *p == alice),
                "by_agent must return only entries that include the agent"
            );
        }

        let bob_events = c.by_agent(bob);
        let bob_kinds: Vec<&str> = bob_events.iter().map(|e| e.kind.label()).collect();
        assert_eq!(
            bob_kinds,
            vec!["Founding", "Conflict", "Death"],
            "Bob must see exactly his own events"
        );

        // An unknown agent returns an empty (but still deterministic)
        // result.
        assert!(c.by_agent(agent(9999)).is_empty());

        // 2b) by_place / by_polity must be exact and order-preserving too.
        let in_a = c.by_place(r_a);
        let a_kinds: Vec<&str> = in_a.iter().map(|e| e.kind.label()).collect();
        assert_eq!(a_kinds, vec!["Migration", "Conflict", "Death"]);
        for e in &in_a {
            assert_eq!(e.place, Some(r_a));
        }
        let in_x = c.by_polity(p_x);
        let x_kinds: Vec<&str> = in_x.iter().map(|e| e.kind.label()).collect();
        assert_eq!(x_kinds, vec!["Founding", "Death"]);
        for e in &in_x {
            assert_eq!(e.polity, Some(p_x));
        }

        // 2c) The shared `Conflict` must appear in BOTH by_agent views,
        //     and the per-agent filters must not leak the other agent.
        assert_eq!(alice_events.len(), 3);
        assert_eq!(bob_events.len(), 3);
    }

    /// Acceptance: deterministic — two chronicles fed the same input
    /// stream produce identical entries.
    #[test]
    fn fr_civ_psyche_920_deterministic_under_same_input_stream() {
        let mut a = Chronicle::new();
        let mut b = Chronicle::new();

        let stream: Vec<(
            u64,
            ChronicleEventKind,
            Option<RegionId>,
            Option<ClusterId>,
            Vec<u64>,
        )> = vec![
            (0, ChronicleEventKind::Birth, None, None, vec![7]),
            (
                3,
                ChronicleEventKind::Migration,
                Some(region(1)),
                None,
                vec![7, 8],
            ),
            (
                4,
                ChronicleEventKind::Founding,
                Some(region(2)),
                Some(cluster(5)),
                vec![9],
            ),
            (
                10,
                ChronicleEventKind::Conflict,
                Some(region(1)),
                Some(cluster(5)),
                vec![7, 9],
            ),
            (15, ChronicleEventKind::FirstContact, None, None, vec![8, 9]),
            (
                22,
                ChronicleEventKind::Death,
                Some(region(2)),
                Some(cluster(5)),
                vec![8],
            ),
            (30, ChronicleEventKind::Birth, None, None, vec![10]),
        ];

        for (tick, kind, place, polity, agents) in &stream {
            let epoch = Epoch(tick / 4);
            a.record(*tick, epoch, kind.clone(), *place, *polity, parts(agents));
            b.record(*tick, epoch, kind.clone(), *place, *polity, parts(agents));
        }

        // Structural equality: every field, in order.
        assert_eq!(
            a.entries, b.entries,
            "same input stream must yield equal chronicles"
        );
        assert_eq!(a.next_id, b.next_id);

        // And every query result must agree.
        for query_agent in [7u64, 8, 9, 10, 9999] {
            assert_eq!(
                a.by_agent(agent(query_agent)),
                b.by_agent(agent(query_agent)),
                "by_agent({query_agent}) must agree"
            );
        }
        for q_region in [1u64, 2, 3] {
            assert_eq!(
                a.by_place(region(q_region)),
                b.by_place(region(q_region)),
                "by_place({q_region}) must agree"
            );
        }
        for q_cluster in [5u64, 99] {
            assert_eq!(
                a.by_polity(cluster(q_cluster)),
                b.by_polity(cluster(q_cluster)),
                "by_polity({q_cluster}) must agree"
            );
        }

        // The serialized form must also be identical — that's the
        // replay-stability acceptance (a save+load round trip on the
        // same run must reproduce the same chronicle bytes).
        let json_a = serde_json::to_string(&a).expect("serialize a");
        let json_b = serde_json::to_string(&b).expect("serialize b");
        assert_eq!(
            json_a, json_b,
            "serialized form must be identical for same input"
        );
    }

    /// Sanity: `from_entries` honors the monotonic-id contract and a
    /// re-loaded chronicle behaves identically to the live one.
    #[test]
    fn from_entries_round_trips_and_preserves_order() {
        let mut live = Chronicle::new();
        live.record(
            0,
            Epoch(0),
            ChronicleEventKind::Birth,
            None,
            None,
            parts(&[1]),
        );
        live.record(
            5,
            Epoch(1),
            ChronicleEventKind::Founding,
            Some(region(7)),
            Some(cluster(3)),
            parts(&[1, 2]),
        );

        let reloaded = Chronicle::from_entries(live.entries.clone());
        assert_eq!(live.entries, reloaded.entries);
        assert_eq!(live.next_id, reloaded.next_id);

        // Subsequent appends continue the id sequence — this is the
        // "appending preserves order" guarantee extended through a
        // load+append cycle.
        let mut reloaded = reloaded;
        let new_id = reloaded.record(
            9,
            Epoch(2),
            ChronicleEventKind::Death,
            Some(region(7)),
            Some(cluster(3)),
            parts(&[2]),
        );
        assert_eq!(new_id, LegendEventId(live.next_id));

        let mut live = live;
        let live_new_id = live.record(
            9,
            Epoch(2),
            ChronicleEventKind::Death,
            Some(region(7)),
            Some(cluster(3)),
            parts(&[2]),
        );
        assert_eq!(new_id, live_new_id);
    }

    /// Sanity: the empty chronicle is empty and queries return nothing.
    #[test]
    fn empty_chronicle_is_empty() {
        let c = Chronicle::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        assert!(c.by_agent(agent(0)).is_empty());
        assert!(c.by_place(region(0)).is_empty());
        assert!(c.by_polity(cluster(0)).is_empty());
        assert!(c.by_kind(&ChronicleEventKind::Birth).is_empty());
    }

    /// Sanity: `ChronicleEventKind::Other` round-trips its label so the
    /// escape hatch does not silently lose data.
    #[test]
    fn other_kind_carries_label() {
        let mut c = Chronicle::new();
        let id = c.record(
            0,
            Epoch(0),
            ChronicleEventKind::Other("MassMigration".to_string()),
            None,
            None,
            smallvec![agent(1)],
        );
        let e = &c.entries[0];
        assert_eq!(e.id, id);
        assert_eq!(e.kind.label(), "MassMigration");
        let only = c.by_kind(&ChronicleEventKind::Other("MassMigration".to_string()));
        assert_eq!(only.len(), 1);
    }
}
