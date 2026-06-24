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
    pub fn ingest(&mut self, raw: RawSimEvent) {
        let epoch = self.graph.config.epoch_of(raw.tick);
        if epoch.0 > self.last_maintained_epoch.0 {
            self.run_maintenance(epoch);
        }
        self.graph.ingest(raw);
    }

    /// Drain a batch of events (e.g. one bus poll) into the graph.
    pub fn drain<I: IntoIterator<Item = RawSimEvent>>(&mut self, events: I) {
        for raw in events {
            self.ingest(raw);
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
