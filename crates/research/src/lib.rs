//! civ-research — R&D proposal validator + replay-safe cache.
//!
//! Per ADR-006, every LLM-proposed tech card must declare
//! `{inputs, energy_cost, byproducts, dependencies}` and is validated against
//! the versioned [`civ_laws::LawDb`] before becoming canon. This crate ships
//! the typed validator + a hash-keyed cache stub; the actual LLM client +
//! WebSocket integration land in a follow-up PR.
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-RESEARCH-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use civ_laws::LawDb;
use serde::{Deserialize, Serialize};

#[cfg(feature = "firepass-kimi")]
pub mod firepass;

/// Cached, offline-safe LLM-garnish hook for deterministic flavor-text/name generation.
/// Implements FR-CIV-LLM: zero network calls, all results from seeded cache/lookup.
pub mod garnish;

/// Tech prerequisite graph and unlock gating for research progression.
pub mod tech_prereq;

/// Schema version for `civ-research`. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 0;

/// A proposed tech card. Hand-authored cards or LLM-generated cards both
/// take this shape so the validator is one entry point.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TechCard {
    /// Stable identifier.
    pub id: String,
    /// Era at which this tech becomes available (must be ≥ `era_min` of all
    /// referenced laws).
    pub era: u16,
    /// Input resource IDs consumed by this tech.
    pub inputs: Vec<String>,
    /// Energy cost per unit application (integer; tunable scale defined by
    /// the simulation).
    pub energy_cost: u64,
    /// Byproducts / waste outputs.
    pub byproducts: Vec<String>,
    /// Law IDs that must exist in the DB for this tech to be valid.
    pub dependencies: Vec<String>,
}

/// Outcome of validating a tech card against a law DB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    /// The card is canon and may be added to the live tech tree.
    Accept,
    /// The card was rejected; the reason explains why.
    Reject(RejectReason),
}

/// Error returned by an LLM client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmError {
    /// The network or service is unavailable.
    NetworkUnavailable,
    /// The client was rate limited.
    RateLimited,
    /// The model returned a response that could not be interpreted as a tech card.
    InvalidResponse(String),
}

/// Async client for proposing tech cards from a prompt and snapshot hash.
#[allow(async_fn_in_trait)]
pub trait LlmClient: Send + Sync {
    /// Propose a tech card from the given prompt and snapshot hash.
    async fn propose_tech_card(
        &self,
        prompt: &str,
        snapshot_hash: &[u8],
    ) -> Result<TechCard, LlmError>;
}

/// Why a card was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectReason {
    /// One of the declared dependency law IDs is not in the DB.
    UnknownDependency(String),
    /// One of the dependency laws is not unlocked at the card's era.
    DependencyEraGated {
        /// The dependency law ID.
        law: String,
        /// The card's declared era.
        card_era: u16,
        /// The law's `era_min`.
        law_era_min: u16,
    },
    /// The card declared no inputs, outputs, or byproducts — equivalent to
    /// `FictionalExtensionUnderspecified` for tech cards.
    NoEffects,
}

/// Per-save progression mode (ADR-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplayMode {
    /// Historical tech tree only; replay refuses any `LlmEvent`.
    Canonical,
    /// Canonical backbone plus LLM side-tech; replay requires cache hits.
    Hybrid,
    /// LLM may propose alt-physics/biology; replay requires cache hits.
    Free,
}

/// Hash-keyed LLM output recorded in the event log (ADR-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmEvent {
    /// RNG seed supplied to the model call.
    pub seed: u64,
    /// Blake3 of prompt template + variables.
    pub prompt_hash: [u8; 32],
    /// Provider model identifier.
    pub model_id: String,
    /// Provider model version.
    pub model_version: String,
    /// Blake3 of the snapshot region the call observed.
    pub input_snapshot_hash: [u8; 32],
    /// Blake3 of serialized output.
    pub output_hash: [u8; 32],
    /// Validated tech card emitted by the call.
    pub output: TechCard,
    /// Simulation tick when the event was recorded.
    pub tick: u64,
}

impl LlmEvent {
    /// Composite cache key: `(prompt_hash, input_snapshot_hash, model_id, model_version)`.
    #[must_use]
    pub fn cache_key(&self) -> Vec<u8> {
        let mut key = Vec::with_capacity(64 + self.model_id.len() + self.model_version.len());
        key.extend_from_slice(&self.prompt_hash);
        key.extend_from_slice(&self.input_snapshot_hash);
        key.extend_from_slice(self.model_id.as_bytes());
        key.extend_from_slice(self.model_version.as_bytes());
        key
    }
}

/// Why replay refused to advance on an LLM event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayRefusal {
    /// Canonical mode encountered an `LlmEvent` in the log.
    CanonicalLlmEvent,
    /// Hybrid/Free replay could not resolve the event from cache.
    HybridCacheMiss,
}

/// Outcome of attempting to apply an `LlmEvent` during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayAdvanceOutcome {
    /// Cache hit (Hybrid/Free) or live run — event may be applied.
    Advanced,
    /// Replay must halt until the log/cache is repaired.
    Refused(ReplayRefusal),
}

/// Apply replay rules from ADR-006 for a single `LlmEvent`.
///
/// During live play (`is_replay == false`) all modes advance. During replay,
/// Canonical refuses every LLM event; Hybrid/Free require a cache hit.
#[must_use]
pub fn replay_advance_llm_event(
    mode: ReplayMode,
    cache: &ResearchCache,
    event: &LlmEvent,
    is_replay: bool,
) -> ReplayAdvanceOutcome {
    if !is_replay {
        return ReplayAdvanceOutcome::Advanced;
    }

    match mode {
        ReplayMode::Canonical => ReplayAdvanceOutcome::Refused(ReplayRefusal::CanonicalLlmEvent),
        ReplayMode::Hybrid | ReplayMode::Free => {
            if cache.get(&event.cache_key()).is_some() {
                ReplayAdvanceOutcome::Advanced
            } else {
                ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss)
            }
        }
    }
}

/// Outcome of a research cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResearchOutcome {
    /// The proposed card was accepted and inserted into the cache.
    Accepted(TechCard),
    /// The proposed card was rejected by the validator.
    Rejected(RejectReason),
    /// The cache already had a result for this snapshot hash.
    CacheHit(TechCard),
    /// The client failed before producing a usable card.
    ClientError(LlmError),
}

/// Validate `card` against `db`. Pure function; no I/O.
#[must_use]
pub fn validate(card: &TechCard, db: &LawDb) -> ValidationOutcome {
    // 1) No-effect cards are rejected — every tech must do *something*.
    if card.inputs.is_empty() && card.byproducts.is_empty() {
        return ValidationOutcome::Reject(RejectReason::NoEffects);
    }
    // 2) Every declared dependency must exist.
    for dep in &card.dependencies {
        let Some(law) = db.get(dep) else {
            return ValidationOutcome::Reject(RejectReason::UnknownDependency(dep.clone()));
        };
        // 3) And be unlocked at or before the card's era.
        if law.era_min > card.era {
            return ValidationOutcome::Reject(RejectReason::DependencyEraGated {
                law: law.id.clone(),
                card_era: card.era,
                law_era_min: law.era_min,
            });
        }
    }
    ValidationOutcome::Accept
}

/// Run the research pipeline for a prompt/snapshot pair.
pub async fn run_research_cycle<C: LlmClient>(
    client: &C,
    cache: &mut ResearchCache,
    db: &LawDb,
    prompt: &str,
    snapshot_hash: &[u8],
) -> ResearchOutcome {
    if let Some(card) = cache.get(snapshot_hash) {
        return ResearchOutcome::CacheHit(card.clone());
    }

    let card = match client.propose_tech_card(prompt, snapshot_hash).await {
        Ok(card) => card,
        Err(err) => return ResearchOutcome::ClientError(err),
    };

    match validate(&card, db) {
        ValidationOutcome::Accept => {
            cache.insert(snapshot_hash, card.clone());
            ResearchOutcome::Accepted(card)
        }
        ValidationOutcome::Reject(reason) => ResearchOutcome::Rejected(reason),
    }
}

/// Hash of `(prompt_hash, input_snapshot_hash)` keying the LLM cache.
pub type CacheKey = [u8; 64];

/// Replay-safe cache stub. Real implementation uses blake3; this version stores
/// keys by serialised bytes so the API is settled while the hashing dep gets
/// pinned across the Phenotype-org toolchain.
#[derive(Debug, Default, Clone)]
pub struct ResearchCache {
    /// Cached outputs keyed by `CacheKey`.
    entries: BTreeMap<Vec<u8>, TechCard>,
}

impl ResearchCache {
    /// Insert a cached card under `key`.
    pub fn insert(&mut self, key: &[u8], card: TechCard) {
        self.entries.insert(key.to_vec(), card);
    }

    /// Look up a cached card.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&TechCard> {
        self.entries.get(key)
    }

    /// Number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the cache empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Deterministic client for tests.
#[derive(Debug, Default, Clone)]
pub struct DummyLlmClient;

impl DummyLlmClient {
    fn hash_input(prompt: &str, snapshot_hash: &[u8]) -> u64 {
        let mut state: u64 = 0xcbf29ce484222325;
        for byte in prompt.as_bytes().iter().chain(snapshot_hash.iter()) {
            state ^= u64::from(*byte);
            state = state.wrapping_mul(0x100000001b3);
        }
        state
    }

    fn derived_card(prompt: &str, snapshot_hash: &[u8]) -> TechCard {
        let hash = Self::hash_input(prompt, snapshot_hash);
        let era = (hash as u16) % 10 + 1;
        let id = format!("tech_{hash:016x}");
        let energy_cost = hash.rotate_left(17) % 10_000 + 1;
        TechCard {
            id,
            era,
            inputs: vec![format!("input_{:08x}", hash as u32)],
            energy_cost,
            byproducts: vec![format!("byproduct_{:08x}", (hash >> 32) as u32)],
            dependencies: vec!["mass_conservation".into()],
        }
    }
}

impl LlmClient for DummyLlmClient {
    async fn propose_tech_card(
        &self,
        prompt: &str,
        snapshot_hash: &[u8],
    ) -> Result<TechCard, LlmError> {
        Ok(Self::derived_card(prompt, snapshot_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_laws::{Law, LawKind};

    fn sample_db() -> LawDb {
        LawDb {
            version: 0,
            laws: vec![
                Law {
                    id: "mass_conservation".into(),
                    kind: LawKind::Conservation,
                    era_min: 0,
                    inputs: vec![],
                    outputs: vec![],
                    losses: vec![],
                    dependencies: vec![],
                },
                Law {
                    id: "steel".into(),
                    kind: LawKind::Material,
                    era_min: 4,
                    inputs: vec!["iron_ore".into()],
                    outputs: vec!["steel_ingot".into()],
                    losses: vec![],
                    dependencies: vec!["mass_conservation".into()],
                },
            ],
        }
    }

    fn valid_research_db() -> LawDb {
        LawDb {
            version: 0,
            laws: vec![Law {
                id: "mass_conservation".into(),
                kind: LawKind::Conservation,
                era_min: 0,
                inputs: vec![],
                outputs: vec![],
                losses: vec![],
                dependencies: vec![],
            }],
        }
    }

    /// Covers FR-CIV-RESEARCH-000.
    /// FR-CIV-RESEARCH-000 — schema present.
    #[test]
    fn schema_version_present() {
        assert_eq!(SCHEMA_VERSION, 0);
    }

    /// FR-CIV-RESEARCH-001 — LLM cache hit returns byte-identical output.
    #[tokio::test]
    async fn llm_cache_hit() {
        let mut cache = ResearchCache::default();
        let db = valid_research_db();
        let key = b"snapshot-hash";
        let card = TechCard {
            id: "cached-tech".into(),
            era: 1,
            inputs: vec!["ore".into()],
            energy_cost: 1,
            byproducts: vec!["slag".into()],
            dependencies: vec!["mass_conservation".into()],
        };
        cache.insert(key, card.clone());
        let client = StaticClient {
            card: Ok(TechCard {
                id: "must-not-be-used".into(),
                era: 99,
                inputs: vec!["x".into()],
                energy_cost: 999,
                byproducts: vec!["y".into()],
                dependencies: vec!["mass_conservation".into()],
            }),
        };

        let outcome = run_research_cycle(&client, &mut cache, &db, "prompt", key).await;
        assert_eq!(outcome, ResearchOutcome::CacheHit(card.clone()));
        assert_eq!(cache.get(key), Some(&card));
    }

    /// Covers FR-CIV-RESEARCH-001.
    /// FR-CIV-RESEARCH-001 — a well-formed card with valid dependencies is
    /// accepted.
    #[test]
    fn accepts_well_formed_card() {
        let db = sample_db();
        let card = TechCard {
            id: "rail_track".into(),
            era: 5,
            inputs: vec!["steel_ingot".into()],
            energy_cost: 100,
            byproducts: vec!["slag".into()],
            dependencies: vec!["steel".into(), "mass_conservation".into()],
        };
        assert_eq!(validate(&card, &db), ValidationOutcome::Accept);
    }

    /// FR-CIV-RESEARCH-010 — unknown dependency rejected.
    #[test]
    fn rejects_unknown_dependency() {
        let db = sample_db();
        let card = TechCard {
            id: "void_drive".into(),
            era: 10,
            inputs: vec!["exotic".into()],
            energy_cost: 9999,
            byproducts: vec![],
            dependencies: vec!["impossibilium".into()],
        };
        assert!(matches!(
            validate(&card, &db),
            ValidationOutcome::Reject(RejectReason::UnknownDependency(_))
        ));
    }

    /// FR-CIV-RESEARCH-011 — era-gated dependency rejected.
    #[test]
    fn rejects_era_gated_dependency() {
        let db = sample_db();
        let card = TechCard {
            id: "prehistoric_railroad".into(),
            era: 1, // before steel's era_min=4
            inputs: vec!["wood".into()],
            energy_cost: 50,
            byproducts: vec![],
            dependencies: vec!["steel".into()],
        };
        assert!(matches!(
            validate(&card, &db),
            ValidationOutcome::Reject(RejectReason::DependencyEraGated { .. })
        ));
    }

    /// FR-CIV-RESEARCH-012 — no-effect card rejected.
    #[test]
    fn rejects_no_effect_card() {
        let db = sample_db();
        let card = TechCard {
            id: "vapourware".into(),
            era: 5,
            inputs: vec![],
            energy_cost: 0,
            byproducts: vec![],
            dependencies: vec![],
        };
        assert!(matches!(
            validate(&card, &db),
            ValidationOutcome::Reject(RejectReason::NoEffects)
        ));
    }

    /// FR-CIV-RESEARCH-020 — cache insert/get round-trips.
    #[test]
    fn cache_roundtrips() {
        let mut cache = ResearchCache::default();
        let card = TechCard {
            id: "x".into(),
            era: 0,
            inputs: vec!["a".into()],
            energy_cost: 1,
            byproducts: vec![],
            dependencies: vec![],
        };
        let key = b"some-key";
        cache.insert(key, card.clone());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(key), Some(&card));
    }

    struct StaticClient {
        card: Result<TechCard, LlmError>,
    }

    impl LlmClient for StaticClient {
        async fn propose_tech_card(
            &self,
            _prompt: &str,
            _snapshot_hash: &[u8],
        ) -> Result<TechCard, LlmError> {
            self.card.clone()
        }
    }

    /// FR-CIV-RESEARCH-030 — cache hit short-circuits client.
    #[tokio::test]
    async fn research_cycle_cache_hit_short_circuits_client() {
        let mut cache = ResearchCache::default();
        let db = valid_research_db();
        let key = b"snapshot";
        let card = TechCard {
            id: "cached".into(),
            era: 1,
            inputs: vec!["ore".into()],
            energy_cost: 1,
            byproducts: vec!["slag".into()],
            dependencies: vec!["mass_conservation".into()],
        };
        cache.insert(key, card.clone());
        let client = StaticClient {
            card: Ok(TechCard {
                id: "should_not_be_used".into(),
                era: 1,
                inputs: vec!["x".into()],
                energy_cost: 1,
                byproducts: vec!["y".into()],
                dependencies: vec!["mass_conservation".into()],
            }),
        };

        let outcome = run_research_cycle(&client, &mut cache, &db, "prompt", key).await;
        assert_eq!(outcome, ResearchOutcome::CacheHit(card));
    }

    /// FR-CIV-RESEARCH-031 — valid client output is accepted and cached.
    #[tokio::test]
    async fn research_cycle_accepts_valid_card() {
        let mut cache = ResearchCache::default();
        let db = valid_research_db();
        let key = b"snapshot-2";
        let card = TechCard {
            id: "new_tech".into(),
            era: 1,
            inputs: vec!["ore".into()],
            energy_cost: 10,
            byproducts: vec!["dust".into()],
            dependencies: vec!["mass_conservation".into()],
        };
        let client = StaticClient {
            card: Ok(card.clone()),
        };

        let outcome = run_research_cycle(&client, &mut cache, &db, "prompt", key).await;
        assert_eq!(outcome, ResearchOutcome::Accepted(card.clone()));
        assert_eq!(cache.get(key), Some(&card));
    }

    /// FR-CIV-RESEARCH-032 — client errors propagate.
    #[tokio::test]
    async fn research_cycle_propagates_client_error() {
        let mut cache = ResearchCache::default();
        let db = valid_research_db();
        let key = b"snapshot-3";
        let client = StaticClient {
            card: Err(LlmError::NetworkUnavailable),
        };

        let outcome = run_research_cycle(&client, &mut cache, &db, "prompt", key).await;
        assert_eq!(
            outcome,
            ResearchOutcome::ClientError(LlmError::NetworkUnavailable)
        );
        assert!(cache.is_empty());
    }

    fn sample_llm_event() -> LlmEvent {
        LlmEvent {
            seed: 1,
            prompt_hash: [0xAA; 32],
            model_id: "kimi-k2.6-turbo".into(),
            model_version: "2026-05-22".into(),
            input_snapshot_hash: [0xBB; 32],
            output_hash: [0xCC; 32],
            output: TechCard {
                id: "side_tech".into(),
                era: 2,
                inputs: vec!["ore".into()],
                energy_cost: 5,
                byproducts: vec!["slag".into()],
                dependencies: vec!["mass_conservation".into()],
            },
            tick: 42,
        }
    }

    /// Covers FR-CIV-RESEARCH-002.
    /// FR-CIV-RESEARCH-002 — Canonical replay refuses first `LlmEvent` (ADR-006).
    #[test]
    fn canonical_replay_refuses_llm() {
        let event = sample_llm_event();
        let cache = ResearchCache::default();

        let outcome = replay_advance_llm_event(ReplayMode::Canonical, &cache, &event, true);

        assert_eq!(
            outcome,
            ReplayAdvanceOutcome::Refused(ReplayRefusal::CanonicalLlmEvent),
            "canonical replay must halt on any LlmEvent until a deterministic fallback is supplied"
        );
    }

    /// Covers FR-CIV-RESEARCH-003.
    /// FR-CIV-RESEARCH-003 — Hybrid replay on cache miss refuses to advance (ADR-006).
    #[test]
    fn hybrid_cache_miss_refuses() {
        let event = sample_llm_event();
        let cache = ResearchCache::default();

        let outcome = replay_advance_llm_event(ReplayMode::Hybrid, &cache, &event, true);

        assert_eq!(
            outcome,
            ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss),
            "hybrid replay must not call a live LLM when the hash-keyed cache misses"
        );
    }

    /// FR-CIV-RESEARCH-033 — deterministic dummy client is stable.
    #[tokio::test]
    async fn dummy_client_is_deterministic() {
        let client = DummyLlmClient;
        let prompt = "build a rail line";
        let snapshot_hash = b"abc123";

        let first = client
            .propose_tech_card(prompt, snapshot_hash)
            .await
            .expect("card");
        let second = client
            .propose_tech_card(prompt, snapshot_hash)
            .await
            .expect("card");

        assert_eq!(first, second);
    }

    /// FR-CIV-RESEARCH-004 — `LlmEvent::cache_key` is deterministic and
    /// byte-composed of `(prompt_hash, input_snapshot_hash, model_id, model_version)`.
    #[test]
    fn cache_key_is_deterministic() {
        let event = sample_llm_event();
        let first = event.cache_key();
        let second = event.cache_key();
        assert_eq!(first, second);
    }

    /// FR-CIV-RESEARCH-004 — `LlmEvent::cache_key` byte structure matches spec.
    #[test]
    fn cache_key_composite_structure() {
        let event = LlmEvent {
            seed: 7,
            prompt_hash: [0x11; 32],
            model_id: "kimi-test".into(),
            model_version: "v1".into(),
            input_snapshot_hash: [0x22; 32],
            output_hash: [0x33; 32],
            output: TechCard {
                id: "tech_x".into(),
                era: 1,
                inputs: vec!["a".into()],
                energy_cost: 1,
                byproducts: vec!["b".into()],
                dependencies: vec!["mass_conservation".into()],
            },
            tick: 0,
        };
        let key = event.cache_key();
        assert_eq!(key.len(), 32 + 32 + 9 + 2); // prompt_hash + snapshot + model_id + version
        assert_eq!(&key[0..32], &[0x11; 32]);
        assert_eq!(&key[32..64], &[0x22; 32]);
        assert_eq!(&key[64..73], b"kimi-test");
        assert_eq!(&key[73..75], b"v1");
    }

    /// FR-CIV-RESEARCH-004 — `LlmEvent::cache_key` changes when any component changes.
    #[test]
    fn cache_key_changes_with_component() {
        let base = sample_llm_event();
        let base_key = base.cache_key();

        let mut changed_prompt = base.clone();
        changed_prompt.prompt_hash = [0xFF; 32];
        assert_ne!(base_key, changed_prompt.cache_key());

        let mut changed_snapshot = base.clone();
        changed_snapshot.input_snapshot_hash = [0xEE; 32];
        assert_ne!(base_key, changed_snapshot.cache_key());

        let mut changed_model = base.clone();
        changed_model.model_id = "other-model".into();
        assert_ne!(base_key, changed_model.cache_key());

        let mut changed_version = base.clone();
        changed_version.model_version = "2026-06-15".into();
        assert_ne!(base_key, changed_version.cache_key());
    }
}
