//! Context-aware verb ranking.
//!
//! Surfaces the top-N verbs that are most relevant given the current
//! sim state (disasters active, citizens starving, market crashed, ...).
//!
//! Phase 3 of the Holocron Keycap UI rollout. The closure API stays
//! but we add a typed `SimSnapshot` + `boost_for_snapshot` helper so
//! callers (bevy-ref, MCP) can plug in any sim-state source without
//! hand-rolling the boost table.

use crate::descriptor::VerbDescriptor;
use crate::group::VerbGroup;
use crate::registry::VerbRegistry;

/// Typed sim-state snapshot consumed by the ranker.
///
/// Holocron does not depend on `civ_engine`; the engine (or any other
/// caller) pushes a snapshot of the relevant scalars and Holocron
/// derives boosts. Keep this struct small + serde-friendly so it can
/// cross the MCP wire cheaply.
#[derive(Debug, Clone, Default)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct SimSnapshot {
    /// Current sim tick (monotonic).
    pub tick: u64,
    /// Number of active disasters (flood / storm / quake / plague / wildfire).
    pub active_disasters: u32,
    /// Dominant era — drives "founding-myth" and "age-of-iron" boosts.
    pub dominant_era: EraKind,
    /// Per-pair faction stance. Lower index = higher tension.
    pub faction_relations: Vec<(u32, u32, FactionStance)>,
    /// Total living population. Drove "nudge food" / "quell unrest" boosts.
    pub population: u32,
    /// 0.0 = balanced, 1.0 = total collapse. Drove trade-divert verbs.
    pub market_stress: f32,
    /// 0.0 = stable, 1.0 = chaotic drift. Drove culture/religion verbs.
    pub culture_drift: f32,
}

/// Era classification. Keep in sync with `crates/era` if it diverges.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EraKind {
    #[default]
    Founding,
    Expansion,
    Conflict,
    Stagnation,
    Renewal,
}

/// Stance between two factions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactionStance {
    Allied,
    Neutral,
    Tense,
    Hostile,
    AtWar,
}

impl FactionStance {
    /// Numeric tension score [0..1]. Higher = more verb boosts surface.
    pub fn tension(self) -> f32 {
        match self {
            FactionStance::Allied => 0.0,
            FactionStance::Neutral => 0.2,
            FactionStance::Tense => 0.5,
            FactionStance::Hostile => 0.8,
            FactionStance::AtWar => 1.0,
        }
    }
}

impl SimSnapshot {
    /// Average pairwise tension across all faction pairs in the snapshot.
    pub fn mean_faction_tension(&self) -> f32 {
        if self.faction_relations.is_empty() {
            return 0.0;
        }
        let sum: f32 = self
            .faction_relations
            .iter()
            .map(|(_, _, s)| s.tension())
            .sum();
        sum / self.faction_relations.len() as f32
    }
}

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

/// Default ranker: top-N by use_count alone (no sim-state boost).
pub fn rank_by_use(registry: &VerbRegistry, limit: usize) -> Vec<&VerbDescriptor> {
    rank_for_state(registry, |_| 0.0, limit)
}

/// Builds a sim-state score closure from a `SimSnapshot`.
///
/// The default boost table is intentionally narrow — it boosts verbs by
/// their group + keyword match against the snapshot's signals. Unknown
/// verbs get 0.0 boost (so they fall back to use-count ranking).
///
/// To extend the boost table, call this and add a custom rule layer:
/// `|id| boost_for_snapshot(snap, registry)(id) + my_extra_boost(id)`.
pub fn boost_for_snapshot<'a>(
    snap: &'a SimSnapshot,
    registry: &'a VerbRegistry,
) -> impl Fn(&str) -> f32 + 'a {
    move |id: &str| {
        let Some(desc) = registry.get(id) else {
            return 0.0;
        };

        // Per-signal boosts. Tunable constants — agents can override
        // by composing their own closure.
        let mut boost = 0.0_f32;

        // Disasters: surface any verb whose name/description mentions
        // a known disaster keyword OR whose group is Civic (governance
        // verbs dominate during disaster).
        if snap.active_disasters > 0 {
            let name = desc.name.to_ascii_lowercase();
            let desc_lower = desc.description.to_ascii_lowercase();
            let disaster_terms =
                ["flood", "storm", "quake", "plague", "wildfire", "meteor", "disaster"];
            if disaster_terms
                .iter()
                .any(|t| name.contains(t) || desc_lower.contains(t))
            {
                boost += 4.0 + snap.active_disasters as f32;
            }
            if matches!(desc.group, VerbGroup::Civic) {
                boost += 1.5;
            }
        }

        // Market stress: trade / economy verbs.
        if snap.market_stress > 0.3 {
            if matches!(
                desc.group,
                VerbGroup::Economic | VerbGroup::Debug | VerbGroup::Meta
            ) {
                boost += 3.0 * snap.market_stress;
            }
        }

        // Culture drift: culture + divine verbs.
        if snap.culture_drift > 0.3 {
            if matches!(desc.group, VerbGroup::Divine | VerbGroup::Civic) {
                boost += 2.5 * snap.culture_drift;
            }
        }

        // Faction tension: governance + civic.
        let mean_tension = snap.mean_faction_tension();
        if mean_tension > 0.4 {
            if matches!(desc.group, VerbGroup::Civic | VerbGroup::Divine) {
                boost += 3.0 * mean_tension;
            }
        }

        // Era-specific nudges.
        match snap.dominant_era {
            EraKind::Founding => {
                if matches!(desc.group, VerbGroup::Civic) {
                    boost += 2.0;
                }
            }
            EraKind::Conflict => {
                if matches!(desc.group, VerbGroup::Civic | VerbGroup::Divine) {
                    boost += 2.5;
                }
            }
            EraKind::Stagnation => {
                if matches!(desc.group, VerbGroup::Economic | VerbGroup::Divine) {
                    boost += 1.5;
                }
            }
            EraKind::Renewal => {
                if matches!(desc.group, VerbGroup::Divine | VerbGroup::Civic) {
                    boost += 1.0;
                }
            }
            EraKind::Expansion => {}
        }

        // Low-population: triage verbs (debug / meta) get a small boost
        // so operators can inspect sim state when the world is failing.
        if snap.population < 100 && matches!(desc.group, VerbGroup::Debug | VerbGroup::Meta) {
            boost += 1.0;
        }

        boost
    }
}

/// Convenience: rank top-N verbs for a snapshot.
pub fn rank_for_snapshot(
    registry: &VerbRegistry,
    snap: &SimSnapshot,
    limit: usize,
) -> Vec<&VerbDescriptor> {
    rank_for_state(registry, boost_for_snapshot(snap, registry), limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::VerbDescriptor;
    use crate::group::VerbGroup;
    use crate::provenance::Provenance;
    use crate::registry::VerbRegistry;

    fn make(id: &str, uses: u64, group: VerbGroup) -> VerbDescriptor {
        VerbDescriptor::builder(id, id, group)
            .description("test verb")
            .provenance(Provenance::Mcp)
            .use_count(uses)
            .build()
    }

    #[test]
    fn rank_by_use_returns_highest_first() {
        let mut reg = VerbRegistry::new();
        reg.register(make("a", 10, VerbGroup::Civic)).unwrap();
        reg.register(make("b", 100, VerbGroup::Civic)).unwrap();
        reg.register(make("c", 50, VerbGroup::Civic)).unwrap();

        let ranked = rank_by_use(&reg, 3);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].id, "b");
        assert_eq!(ranked[1].id, "c");
        assert_eq!(ranked[2].id, "a");
    }

    #[test]
    fn rank_for_state_boosts_relevant_verbs() {
        let mut reg = VerbRegistry::new();
        reg.register(make("rarely_used", 1, VerbGroup::Civic)).unwrap();
        reg.register(make("often_used", 100, VerbGroup::Civic)).unwrap();

        let ranked = rank_for_state(
            &reg,
            |id| if id == "rarely_used" { 1000.0 } else { 0.0 },
            2,
        );
        assert_eq!(ranked[0].id, "rarely_used");
        assert_eq!(ranked[1].id, "often_used");
    }

    #[test]
    fn rank_respects_limit() {
        let mut reg = VerbRegistry::new();
        for ch in b'a'..=b'e' {
            let id = (ch as char).to_string();
            reg.register(make(&id, 1, VerbGroup::Civic)).unwrap();
        }
        let ranked = rank_by_use(&reg, 2);
        assert_eq!(ranked.len(), 2);
    }

    fn snap_with(
        active_disasters: u32,
        market_stress: f32,
        culture_drift: f32,
        mean_tension: f32,
    ) -> SimSnapshot {
        let mut s = SimSnapshot::default();
        s.active_disasters = active_disasters;
        s.market_stress = market_stress;
        s.culture_drift = culture_drift;
        s.population = 1000;
        if mean_tension > 0.0 {
            s.faction_relations.push((0, 1, FactionStance::AtWar));
        }
        s
    }

    #[test]
    fn boost_disasters_surfaces_flood_verb() {
        let mut reg = VerbRegistry::new();
        reg.register(make("calm", 1, VerbGroup::Civic)).unwrap();
        reg.register(make("banish_flood", 1, VerbGroup::Civic)).unwrap();
        let snap = snap_with(2, 0.0, 0.0, 0.0);
        let ranked = rank_for_snapshot(&reg, &snap, 2);
        assert_eq!(ranked[0].id, "banish_flood");
    }

    #[test]
    fn boost_market_stress_surfaces_economic_verbs() {
        let mut reg = VerbRegistry::new();
        reg.register(make("calm_civic", 1, VerbGroup::Civic)).unwrap();
        reg.register(make("market_dump", 1, VerbGroup::Economic)).unwrap();
        let snap = snap_with(0, 0.9, 0.0, 0.0);
        let ranked = rank_for_snapshot(&reg, &snap, 2);
        assert_eq!(ranked[0].id, "market_dump");
    }

    #[test]
    fn boost_no_signals_falls_back_to_use_count() {
        let mut reg = VerbRegistry::new();
        reg.register(make("rare_civic", 1, VerbGroup::Civic)).unwrap();
        reg.register(make("common_civic", 100, VerbGroup::Civic)).unwrap();
        let snap = SimSnapshot::default();
        let ranked = rank_for_snapshot(&reg, &snap, 2);
        assert_eq!(ranked[0].id, "common_civic");
        assert_eq!(ranked[1].id, "rare_civic");
    }

    #[test]
    fn boost_faction_tension_surfaces_civic() {
        let mut reg = VerbRegistry::new();
        reg.register(make("calm_divine", 1, VerbGroup::Divine)).unwrap();
        reg.register(make("mediate_war", 1, VerbGroup::Civic)).unwrap();
        let snap = snap_with(0, 0.0, 0.0, 1.0);
        let ranked = rank_for_snapshot(&reg, &snap, 2);
        assert_eq!(ranked[0].id, "mediate_war");
    }
}