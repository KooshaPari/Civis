//! Context-aware verb ranking.
//!
//! Surfaces the top-N verbs that are most relevant given the current
//! sim state (disasters active, citizens starving, market crashed, ...).
//!
//! Phase 4 of the Holocron Keycap UI rollout. Currently a no-op
//! `rank_for_state` that returns deterministic risk/id order plus caller
//! supplied sim-state boosts.

use crate::descriptor::VerbDescriptor;
use crate::registry::VerbRegistry;

/// Simulation context for ranking — currently a placeholder.
/// In later phases, this will capture world state (disasters, resources, etc.)
/// to inform ranking decisions.
pub struct SimContext;

/// Ranked verb result — wraps a descriptor with its ranking score.
pub struct RankedVerb {
    pub descriptor: VerbDescriptor,
    pub rank_score: f32,
}

/// Ranks verbs for a given sim-state snapshot.
///
/// `sim_state_score` is a closure that, for a given verb id, returns
/// a non-negative boost reflecting how relevant the verb is *right now*.
/// A score of 0.0 means "no boost, fall back to base ranking".
///
/// The base ranking is the inverse risk sort key; the boost is added on top.
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
            let base = 1.0 / (1.0 + f32::from(d.risk.sort_key()));
            let boost = sim_state_score(id).max(0.0);
            (id, base + boost)
        })
        .collect();

    // Stable sort: by score desc, then by id asc for determinism.
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(b.0))
    });

    scored
        .into_iter()
        .take(limit)
        .filter_map(|(id, _)| registry.get(id))
        .collect()
}

/// Default ranker: low-risk verbs first (no sim-state boost).
pub fn rank_by_risk(registry: &VerbRegistry, limit: usize) -> Vec<&VerbDescriptor> {
    rank_for_state(registry, |_| 0.0, limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::VerbDescriptor;
    use crate::group::VerbGroup;
    use crate::registry::VerbRegistry;

    fn make(id: &'static str, risk: crate::risk::RiskTier) -> VerbDescriptor {
        VerbDescriptor::builder(id, id, VerbGroup::Civic)
            .summary("test")
            .risk(risk)
            .provenance(crate::provenance::Provenance::Mcp)
            .build()
    }

    #[test]
    fn rank_by_risk_returns_lowest_risk_first() {
        let mut reg = VerbRegistry::empty();
        reg.register(make("a", crate::risk::RiskTier::Critical));
        reg.register(make("b", crate::risk::RiskTier::ReadOnly));
        reg.register(make("c", crate::risk::RiskTier::Minor));

        let ranked = rank_by_risk(&reg, 3);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].id, "b");
        assert_eq!(ranked[1].id, "c");
        assert_eq!(ranked[2].id, "a");
    }

    #[test]
    fn rank_for_state_boosts_relevant_verbs() {
        let mut reg = VerbRegistry::empty();
        reg.register(make("rarely_used", crate::risk::RiskTier::Critical));
        reg.register(make("often_used", crate::risk::RiskTier::ReadOnly));

        // Boost the rarely-used one heavily: it should jump to the top.
        let ranked = rank_for_state(&reg, |id| if id == "rarely_used" { 1000.0 } else { 0.0 }, 2);
        assert_eq!(ranked[0].id, "rarely_used");
        assert_eq!(ranked[1].id, "often_used");
    }

    #[test]
    fn rank_respects_limit() {
        let mut reg = VerbRegistry::empty();
        for id in ["a", "b", "c", "d", "e"] {
            reg.register(make(id, crate::risk::RiskTier::Minor));
        }
        let ranked = rank_by_risk(&reg, 2);
        assert_eq!(ranked.len(), 2);
    }
}
