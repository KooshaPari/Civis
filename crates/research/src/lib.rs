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

    /// FR-CIV-RESEARCH-000 — schema present.
    #[test]
    fn schema_version_present() {
        assert_eq!(SCHEMA_VERSION, 0);
    }

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
}
