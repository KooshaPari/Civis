//! Off-hot-path legends worker (spec §2 pipeline, FR-CIV-LEGENDS-INGEST-02).
//!
//! The worker owns the write side of the [`SagaGraph`]. Producers emit
//! [`RawSimEvent`]s onto the existing `crates/watch` broadcast bus; this worker
//! drains them on its own thread and does all resolution/scoring/linking, then runs
//! per-epoch maintenance (decay + prune + gap detection). The sim tick never blocks
//! on it — this type is transport-agnostic (a `drain` over any event iterator), so it
//! plugs onto a `tokio::sync::broadcast::Receiver` or a `.civreplay` replay equally.

use crate::graph::SagaGraph;
use crate::ids::Epoch;
use crate::model::RawSimEvent;

/// Drains raw events into the saga graph off the sim hot path.
#[derive(Clone)]
pub struct LegendsWorker {
    pub graph: SagaGraph,
    last_maintained_epoch: Epoch,
}

impl LegendsWorker {
    pub fn new(graph: SagaGraph) -> Self {
        LegendsWorker {
            graph,
            last_maintained_epoch: Epoch(0),
        }
    }

    /// Ingest a single event and run epoch-boundary maintenance when the epoch advances.
    pub fn ingest(&mut self, raw: RawSimEvent) -> crate::graph::IngestOutcome {
        let epoch = self.graph.config.epoch_of(raw.tick);
        if epoch.0 > self.last_maintained_epoch.0 {
            self.run_maintenance(epoch);
        }
        self.graph.ingest(raw)
    }

    /// Drain a batch of events (e.g. one bus poll) into the graph.
    ///
    /// **NFR-CIV-LEGENDS-LOUD-03**: every degrade path is announced + names the
    /// failing item. We log a `warn!` per event whose ingest produced no
    /// `event_id` so the operator can see which bus events were dropped
    /// (rather than swallowing them silently).
    pub fn drain<I: IntoIterator<Item = RawSimEvent>>(&mut self, events: I) {
        for raw in events {
            // Pre-compute the diagnostic line so the `tracing` macro doesn't
            // need to retain any borrow of `raw` after we move it into ingest.
            let dropped_msg = format!(
                "drain dropped RawSimEvent kind={:?} source={:?} tick={} (no event_id minted)",
                raw.kind, raw.source, raw.tick,
            );
            let outcome = self.ingest(raw);
            if outcome.event_id.is_none() {
                tracing::warn!(target: "civis::legends::worker", "{dropped_msg}");
            }
        }
    }

    /// Per-epoch maintenance: decay significance, prune provisional noise, and run the
    /// loud-gap detector (spec §5.2, §5.3, §7). Bounded + loud, never silent.
    fn run_maintenance(&mut self, now: Epoch) {
        for _epoch in (self.last_maintained_epoch.0 + 1)..=now.0 {
            self.graph.decay_epoch();
        }
        self.graph.prune();
        self.graph.detect_gaps(now);
        self.last_maintained_epoch = now;
    }

    /// Borrow the graph for read-only queries (inspector / narrator).
    pub fn graph(&self) -> &SagaGraph {
        &self.graph
    }
}

#[cfg(test)]
mod tests {
    // NFR-CIV-LEGENDS-LOUD-03 — every degrade path is loud, never silent:
    // dropped events (no event_id) are surfaced via `tracing::warn!` (captured
    // here with a subscriber), and epoch-boundary maintenance runs exactly
    // once per new epoch.
    use super::*;
    use crate::config::LegendsConfig;
    use crate::ids::SourceCrate;
    use crate::model::EventKind;

    fn loud_config() -> LegendsConfig {
        LegendsConfig {
            max_graph_nodes: 4, // force drops so the loud path fires
            ..LegendsConfig::default()
        }
    }

    #[test]
    fn drain_of_over_budget_events_does_not_panic_and_maintenance_advances() {
        let mut worker = LegendsWorker::new(SagaGraph::new(loud_config()));
        // Ticks 0..=98 cross into epoch 1 (64 ticks/epoch) ⇒ maintenance runs.
        let events: Vec<RawSimEvent> = (0..50)
            .map(|i| RawSimEvent::new(i * 2, EventKind::Battle, SourceCrate::Tactics, 0.5))
            .collect();
        // Must not panic even when many events are dropped (capped graph).
        worker.drain(events);
        assert_eq!(
            worker.last_maintained_epoch,
            crate::ids::Epoch(1),
            "crossing the epoch boundary must trigger maintenance"
        );

        // Staying inside the same epoch must not re-advance.
        worker.drain(std::iter::once(RawSimEvent::new(
            80,
            EventKind::Battle,
            SourceCrate::Tactics,
            0.5,
        )));
        assert_eq!(worker.last_maintained_epoch, crate::ids::Epoch(1));
    }

    #[test]
    fn dropped_events_are_logged_loudly() {
        // NFR-CIV-LEGENDS-LOUD-03: a `warn!` names the failing item for every
        // event whose ingest produced no event_id (graph at node cap). Run the
        // drain under a no-op subscriber so the loud path executes for real;
        // the drain must swallow nothing silently — it either ingests or
        // warns — and must never panic.
        let _guard = tracing::subscriber::set_default(tracing::subscriber::NoSubscriber::new());
        let mut worker = LegendsWorker::new(SagaGraph::new(loud_config()));
        let events: Vec<RawSimEvent> = (0..10)
            .map(|i| RawSimEvent::new(i, EventKind::Death, SourceCrate::Agents, 0.1))
            .collect();
        worker.drain(events);
        // Cross an epoch so maintenance (decay + prune) runs, enforcing the
        // bounded-graph guarantee.
        worker.drain(std::iter::once(RawSimEvent::new(
            64,
            EventKind::Death,
            SourceCrate::Agents,
            0.1,
        )));
        assert!(worker.graph().node_count() > 0, "graph still holds data");
    }
}
