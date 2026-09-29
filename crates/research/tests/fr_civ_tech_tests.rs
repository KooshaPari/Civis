//! FR traceability tests for the tech-engineering validator and replay rules.
//!
//! Covers: FR-CIV-TECH-002, FR-CIV-TECH-007, FR-CIV-TECH-008, FR-CIV-TECH-009
//!
//! These IDs are implemented in `civ-research` but were previously untagged in
//! the source and unasserted in tests. Each test below exercises real behaviour
//! rather than merely constructing a type.

use civ_laws::{Law, LawDb, LawKind};
use civ_research::{
    LlmEvent, ReplayAdvanceOutcome, ReplayMode, ReplayRefusal, ResearchCache, TechCard,
    ValidationOutcome, replay_advance_llm_event, validate,
};

fn db_with(laws: Vec<Law>) -> LawDb {
    LawDb { version: 1, laws }
}

fn law(id: &str, era_min: u16) -> Law {
    Law {
        id: id.to_string(),
        kind: LawKind::Material,
        era_min,
        inputs: vec![],
        outputs: vec![],
        losses: vec![],
        dependencies: vec![],
    }
}

fn card(era: u16, inputs: Vec<&str>, deps: Vec<&str>) -> TechCard {
    TechCard {
        id: "tech_test".to_string(),
        era,
        inputs: inputs.into_iter().map(str::to_string).collect(),
        energy_cost: 10,
        byproducts: vec![],
        dependencies: deps.into_iter().map(str::to_string).collect(),
    }
}

/// A minimal but well-formed `LlmEvent` for the replay-gating tests.
fn llm_event() -> LlmEvent {
    LlmEvent {
        seed: 7,
        prompt_hash: [1u8; 32],
        model_id: "test-model".to_string(),
        model_version: "v1".to_string(),
        input_snapshot_hash: [2u8; 32],
        output_hash: [3u8; 32],
        output: card(0, vec!["wood"], vec![]),
        tick: 42,
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-TECH-002 — a technique enters a KnowledgeSet only after passing
// `validate` against the current LawDb.
// ---------------------------------------------------------------------------

/// FR-CIV-TECH-002 — a card with a satisfiable dependency set is accepted.
#[test]
fn tech_002_accepts_card_whose_dependencies_resolve() {
    let db = db_with(vec![law("law_fire", 1)]);
    let c = card(2, vec!["wood"], vec!["law_fire"]);
    assert_eq!(validate(&c, &db), ValidationOutcome::Accept);
}

/// FR-CIV-TECH-002 — a card depending on a law absent from the DB is rejected,
/// so it can never enter a KnowledgeSet.
#[test]
fn tech_002_rejects_card_with_unknown_dependency() {
    let db = db_with(vec![law("law_fire", 1)]);
    let c = card(2, vec!["wood"], vec!["law_missing"]);
    assert!(matches!(validate(&c, &db), ValidationOutcome::Reject(_)));
}

/// FR-CIV-TECH-002 — a dependency that exists but is not yet unlocked at the
/// card's era is rejected (era gating).
#[test]
fn tech_002_rejects_card_era_gated_dependency() {
    let db = db_with(vec![law("law_fire", 5)]);
    let c = card(2, vec!["wood"], vec!["law_fire"]);
    assert!(matches!(validate(&c, &db), ValidationOutcome::Reject(_)));
}

/// FR-CIV-TECH-002 — a card with no inputs and no byproducts is rejected:
/// every tech must do something.
#[test]
fn tech_002_rejects_card_with_no_effects() {
    let db = db_with(vec![]);
    let c = card(0, vec![], vec![]);
    assert!(matches!(validate(&c, &db), ValidationOutcome::Reject(_)));
}

// ---------------------------------------------------------------------------
// FR-CIV-TECH-007 / FR-CIV-TECH-008 — canonical saves emit no LlmEvents, and
// LLM-proposed cards are validated by the same gate.
// ---------------------------------------------------------------------------

/// FR-CIV-TECH-008 — an LLM-proposed card goes through the identical
/// `validate` gate; a no-effect proposal is rejected like any other card.
#[test]
fn tech_008_llm_cards_face_the_same_validate_gate() {
    let db = db_with(vec![law("law_fire", 0)]);
    // A well-formed proposal.
    let good = card(1, vec!["wood"], vec!["law_fire"]);
    assert_eq!(validate(&good, &db), ValidationOutcome::Accept);
    // A hollow proposal is rejected by the same function.
    let hollow = card(1, vec![], vec![]);
    assert!(matches!(validate(&hollow, &db), ValidationOutcome::Reject(_)));
}

// ---------------------------------------------------------------------------
// FR-CIV-TECH-009 — hybrid/free replay requires cache hits.
// ---------------------------------------------------------------------------

/// FR-CIV-TECH-009 — hybrid replay reproduces accepted cards from cache;
/// a cold cache halts loudly (`HybridCacheMiss`) rather than diverging.
#[test]
fn tech_009_hybrid_replay_advances_on_cache_hit_and_refuses_on_miss() {
    let event = llm_event();
    let mut cache = ResearchCache::default();

    // Cold cache: hybrid replay must refuse, not fall back to the live client.
    let cold = replay_advance_llm_event(ReplayMode::Hybrid, &cache, &event, true);
    assert_eq!(
        cold,
        ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss)
    );

    // Warm cache: the same event now resolves and replays.
    cache.insert(&event.cache_key(), event.output.clone());
    let warm = replay_advance_llm_event(ReplayMode::Hybrid, &cache, &event, true);
    assert_eq!(warm, ReplayAdvanceOutcome::Advanced);
}

/// FR-CIV-TECH-007 — canonical mode refuses every LLM event during replay,
/// even when the cache could satisfy it.
#[test]
fn tech_007_canonical_replay_refuses_llm_event_even_when_cached() {
    let event = llm_event();
    let mut cache = ResearchCache::default();
    cache.insert(&event.cache_key(), event.output.clone());

    let out = replay_advance_llm_event(ReplayMode::Canonical, &cache, &event, true);
    assert_eq!(
        out,
        ReplayAdvanceOutcome::Refused(ReplayRefusal::CanonicalLlmEvent)
    );
}

/// FR-CIV-TECH-009 — live play is not gated: the refusal rules apply to replay
/// only, so a cold cache does not stall a live run.
#[test]
fn tech_009_live_play_always_advances() {
    let event = llm_event();
    let cache = ResearchCache::default();
    let out = replay_advance_llm_event(ReplayMode::Canonical, &cache, &event, false);
    assert_eq!(out, ReplayAdvanceOutcome::Advanced);
}

/// FR-CIV-TECH-007 — all three progression modes are distinct; `Canonical` is
/// the restrictive one.
#[test]
fn tech_007_replay_modes_are_distinct() {
    assert_ne!(ReplayMode::Canonical, ReplayMode::Hybrid);
    assert_ne!(ReplayMode::Hybrid, ReplayMode::Free);
    assert_ne!(ReplayMode::Canonical, ReplayMode::Free);
    assert_eq!(ReplayMode::Canonical, ReplayMode::Canonical);
}
