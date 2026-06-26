//! Context-aware verb ranking.
//!
//! Surfaces the top-N verbs that are most relevant given the current
//! sim state (disasters active, citizens starving, market crashed, ...).
//!
//! Phase 4 of the Holocron Keycap UI rollout. Currently a no-op
//! `rank_for_state` that returns the top-N by use-stats; richer
//! sim-state boosts land in a later PR.

use crate::descriptor::VerbDescriptor;
use crate::registry::VerbRegistry;

/// Ranks verbs for a given sim-state snapshot.
///
/// `sim_state_score` is a closure that, for a given verb id, returns
/// a non-negative boost reflecting how relevant the verb is *right now*.
/// A score of 0.0 means "no boost, fall back to base ranking".
///
/// The base ranking is `use_count` descending; the boost is added on top.
///
/// Returns up to `limit` descriptors, highest-ranked first.
pub fn rank_for_state<F>(
    registry: &VerbRegistry,
    sim_state_score: F,
    limit: usize,
) -> Vec<&VerbDescriptor>
where
    F: Fn(&str) -> f32,
{
    let mut scored: Vec<(&str, f32)> = registry
        .iter()
        .map(|(id, d)| {
            let base = (d.use_count as f32).ln_1p();
            let boost = sim_state_score(id).max(0.0);
            (id, base + boost)
        })
        .collect();

    // Stable sort: by score desc, then by id asc for determinism.
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(b.0)));

    scored
        .into_iter()
        .take(limit)
        .filter_map(|(id, _)| registry.get(id))
        .collect()
}

/// Default ranker: top-N by use_count alone (no sim-state boost).
pub fn rank_by_use(registry: &VerbRegistry, limit: usize) -> Vec<&VerbDescriptor> {
    rank_for_state(registry, |_| 0.0, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::VerbDescriptor;
    use crate::group::VerbGroup;
    use crate::provenance::Provenance;
    use crate::registry::VerbRegistry;

    fn make(id: &str, uses: u64) -> VerbDescriptor {
        VerbDescriptor::builder(id, id, VerbGroup::Civic)
            .description("test")
            .provenance(Provenance::Mcp)
            .use_count(uses)
            .build()
    }

    #[test]
    fn rank_by_use_returns_highest_first() {
        let mut reg = VerbRegistry::new();
        reg.register(make("a", 10)).unwrap();
        reg.register(make("b", 100)).unwrap();
        reg.register(make("c", 50)).unwrap();

        let ranked = rank_by_use(&reg, 3);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].id, "b");
        assert_eq!(ranked[1].id, "c");
        assert_eq!(ranked[2].id, "a");
    }

    #[test]
    fn rank_for_state_boosts_relevant_verbs() {
        let mut reg = VerbRegistry::new();
        reg.register(make("rarely_used", 1)).unwrap();
        reg.register(make("often_used", 100)).unwrap();

        // Boost the rarely-used one heavily: it should jump to the top.
        let ranked = rank_for_state(&reg, |id| if id == "rarely_used" { 1000.0 } else { 0.0 }, 2);
        assert_eq!(ranked[0].id, "rarely_used");
        assert_eq!(ranked[1].id, "often_used");
    }

    #[test]
    fn rank_respects_limit() {
        let mut reg = VerbRegistry::new();
        for ch in b'a'..=b'e' {
            let id = (ch as char).to_string();
            reg.register(make(&id, 1)).unwrap();
        }
        let ranked = rank_by_use(&reg, 2);
        assert_eq!(ranked.len(), 2);
    }
}