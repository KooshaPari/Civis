// TODO(cleanup-surgeon): stub module — `EraHistory`/`EraTransition` types were
// removed by an earlier lane. `era.rs` still imports them. Restore the
// original or rewrite callers.

use serde::{Deserialize, Serialize};

/// Stub `EraHistory` for callers that read era transition timelines.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraHistory {
    /// Tick the era was first entered.
    pub era_entered_tick: u64,
}

impl EraHistory {
    /// Record an era advance for `faction_id` at `tick`. Stub: no-op until
    /// the original timeline is restored.
    pub fn record_advance(
        &mut self,
        _tick: u64,
        _faction_id: u32,
        _previous: crate::era::CivAge,
        _next: crate::era::CivAge,
    ) {
    }
}

/// Stub `EraTransition` for callers that emit era transitions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EraTransition {
    /// Tick the transition was emitted.
    pub tick: u64,
    /// Faction id entering the new era.
    pub faction_id: u32,
}
