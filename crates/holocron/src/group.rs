//! Verb categorization for the Holocron UI.
//!
//! Groups partition the verb catalog into the four panes the player sees in
//! the HolocronPanel and filters the CommandKOverlay results.
//!
//! The grouping is **semantic, not syntactic** — it reflects the player's
//! mental model of godgame verbs (what they accomplish), not the underlying
//! engine subsystem they touch.

use serde::{Deserialize, Serialize};

/// Coarse categorization of godgame verbs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum VerbGroup {
    /// Population, culture, faith, law, language — the slow civic substrate.
    Civic,
    /// Production, trade, markets, prices, storage, money.
    Economic,
    /// Miracles, blessings, curses, disasters, divine interventions.
    Divine,
    /// Inspection tools, time controls, save/load, debug overlays.
    Debug,
    /// Catch-all for verbs that don't fit the four canonical groups yet.
    /// Kept small — when a verb lands here it should be promoted to a real
    /// group or given a new one.
    Misc,
}

impl VerbGroup {
    /// Short label for HUD use (≤ 8 chars).
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Civic => "Civic",
            Self::Economic => "Econ",
            Self::Divine => "Divine",
            Self::Debug => "Debug",
            Self::Misc => "Misc",
        }
    }

    /// Long label for the HolocronPanel header.
    pub fn long_label(self) -> &'static str {
        match self {
            Self::Civic => "Civic & Society",
            Self::Economic => "Economy & Trade",
            Self::Divine => "Divine & Miracles",
            Self::Debug => "Inspection & Debug",
            Self::Misc => "Miscellaneous",
        }
    }

    /// Sort key for stable ordering in panels.
    pub fn sort_key(self) -> u8 {
        match self {
            Self::Civic => 0,
            Self::Economic => 1,
            Self::Divine => 2,
            Self::Debug => 3,
            Self::Misc => 4,
        }
    }

    /// Iterate over all groups in canonical display order.
    pub fn all() -> [Self; 5] {
        [Self::Civic, Self::Economic, Self::Divine, Self::Debug, Self::Misc]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_match_group() {
        for g in VerbGroup::all() {
            assert!(!g.short_label().is_empty());
            assert!(!g.long_label().is_empty());
        }
    }

    #[test]
    fn sort_key_unique() {
        let keys: Vec<u8> = VerbGroup::all().iter().map(|g| g.sort_key()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len(), "sort_key must be unique per group");
    }
}