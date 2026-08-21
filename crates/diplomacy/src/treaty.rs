//! FR-DIPL-001: Treaty logic
//!
//! Implements the lifecycle and effects of treaties between polities.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::PolityId;

/// The different types of treaties that can be formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TreatyType {
    /// A commitment to end hostilities.
    Peace,
    /// An agreement to trade resources.
    Trade,
    /// A formal military partnership.
    Alliance,
    /// A promise not to attack each other.
    NonAggression,
    /// A pact to defend each other against external threats.
    DefensivePact,
}

/// The current status of a treaty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TreatyStatus {
    /// Proposed but not yet accepted by all parties.
    Proposed,
    /// Active and in effect.
    Active,
    /// Broken by one of the parties.
    Broken,
    /// Expired due to the expiration tick being reached.
    Expired,
}

/// A specific term within a treaty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TreatyTerm {
    /// The key of the term (e.g., "resource_share", "territory_return").
    pub key: String,
    /// The value of the term (e.g., "0.1", "sector_5").
    pub value: String,
}

/// A treaty between two or more polities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Treaty {
    /// Unique identifier for the treaty.
    pub id: u64,
    /// The polities involved in the treaty.
    pub parties: (PolityId, PolityId),
    /// The type of treaty.
    pub treaty_type: TreatyType,
    /// The specific terms of the treaty.
    pub terms: Vec<TreatyTerm>,
    /// The tick at which the treaty expires (if any).
    pub expiration_tick: Option<u64>,
    /// The current status of the treaty.
    pub status: TreatyStatus,
}

/// Errors that can occur during treaty operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TreatyError {
    /// The specified treaty was not found.
    #[error("treaty not found: {0}")]
    NotFound(u64),
    /// The specified polity is not a party to the treaty.
    #[error("polity is not a party to treaty {0}")]
    NotParty(u64),
    /// The treaty is not in the expected state for the operation.
    #[error("treaty is in an invalid state: {0:?}")]
    InvalidState(TreatyStatus),
    /// The treaty has already expired.
    #[error("treaty has already expired")]
    AlreadyExpired,
}

/// Manages treaties between polities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyManager {
    /// The next available treaty ID.
    next_id: u64,
    /// All known treaties, indexed by ID.
    treaties: BTreeMap<u64, Treaty>,
}

impl TreatyManager {
    /// Create a new empty treaty manager.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            treaties: BTreeMap::new(),
        }
    }

    /// Propose a new treaty.
    pub fn propose_treaty(
        &mut self,
        proposer: PolityId,
        parties: (PolityId, PolityId),
        treaty_type: TreatyType,
        terms: Vec<TreatyTerm>,
        expiration_tick: Option<u64>,
    ) -> Result<u64, TreatyError> {
        // Validate proposer is one of the parties
        if proposer != parties.0 && proposer != parties.1 {
            return Err(TreatyError::NotParty(0)); // ID not assigned yet
        }

        let id = self.next_id;
        self.next_id += 1;

        let treaty = Treaty {
            id,
            parties,
            treaty_type,
            terms,
            expiration_tick,
            status: TreatyStatus::Proposed,
        };

        self.treaties.insert(id, treaty);
        Ok(id)
    }

    /// Accept a proposed treaty.
    pub fn accept_treaty(
        &mut self,
        accepter: PolityId,
        treaty_id: u64,
    ) -> Result<(), TreatyError> {
        let treaty = self
            .treaties
            .get_mut(&treaty_id)
            .ok_or(TreatyError::NotFound(treaty_id))?;

        if accepter != treaty.parties.0 && accepter != treaty.parties.1 {
            return Err(TreatyError::NotParty(treaty_id));
        }

        if treaty.status != TreatyStatus::Proposed {
            return Err(TreatyError::InvalidState(treaty.status));
        }

        treaty.status = TreatyStatus::Active;
        Ok(())
    }

    /// Break an active treaty.
    pub fn break_treaty(
        &mut self,
        breaker: PolityId,
        treaty_id: u64,
    ) -> Result<(), TreatyError> {
        let treaty = self
            .treaties
            .get_mut(&treaty_id)
            .ok_or(TreatyError::NotFound(treaty_id))?;

        if breaker != treaty.parties.0 && breaker != treaty.parties.1 {
            return Err(TreatyError::NotParty(treaty_id));
        }

        if treaty.status != TreatyStatus::Active {
            return Err(TreatyError::InvalidState(treaty.status));
        }

        treaty.status = TreatyStatus::Broken;
        Ok(())
    }

    /// Check for treaty effects and expirations. Returns a list of effects to be applied.
    pub fn check_treaty_effects(&mut self, tick: u64) -> Vec<TreatyEffect> {
        let mut effects = Vec::new();

        // Collect IDs of treaties that have expired
        let mut expired_ids = Vec::new();
        for (&id, treaty) in &self.treaties {
            if treaty.status == TreatyStatus::Active {
                if let Some(exp) = treaty.expiration_tick {
                    if tick >= exp {
                        expired_ids.push(id);
                    }
                }
            }
        }

        // Mark expired treaties
        for id in &expired_ids {
            if let Some(treaty) = self.treaties.get_mut(id) {
                treaty.status = TreatyStatus::Expired;
            }
        }

        // Generate effects for active treaties
        for treaty in self.treaties.values() {
            if treaty.status == TreatyStatus::Active {
                let effect = match treaty.treaty_type {
                    TreatyType::Peace | TreatyType::NonAggression => TreatyEffect {
                        standing_delta: 5,
                        resource_modifier: 0,
                    },
                    TreatyType::Trade => TreatyEffect {
                        standing_delta: 2,
                        resource_modifier: 10,
                    },
                    TreatyType::Alliance | TreatyType::DefensivePact => TreatyEffect {
                        standing_delta: 10,
                        resource_modifier: 5,
                    },
                };
                effects.push(effect);
            }
        }

        effects
    }

    /// Get a reference to a treaty by ID.
    pub fn get_treaty(&self, treaty_id: u64) -> Option<&Treaty> {
        self.treaties.get(&treaty_id)
    }
}

/// Represents the effect of a treaty on the game state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatyEffect {
    /// The change in standing between the parties per tick.
    pub standing_delta: i32,
    /// A multiplier or additive bonus to resource production (e.g., 10 = 10%).
    pub resource_modifier: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PolityId;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    #[test]
    fn test_treaty_lifecycle() {
        let mut manager = TreatyManager::new();

        // Propose a treaty
        let terms = vec![TreatyTerm {
            key: "grain_share".to_string(),
            value: "5".to_string(),
        }];

        let id = manager
            .propose_treaty(
                p(1),
                (p(1), p(2)),
                TreatyType::Trade,
                terms,
                Some(100),
            )
            .unwrap();

        assert_eq!(id, 1);
        let treaty = manager.get_treaty(id).unwrap();
        assert_eq!(treaty.status, TreatyStatus::Proposed);

        // Accept the treaty
        manager.accept_treaty(p(2), id).unwrap();
        let treaty = manager.get_treaty(id).unwrap();
        assert_eq!(treaty.status, TreatyStatus::Active);

        // Check effects
        let effects = manager.check_treaty_effects(50);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].resource_modifier, 10);

        // Break the treaty
        manager.break_treaty(p(1), id).unwrap();
        let treaty = manager.get_treaty(id).unwrap();
        assert_eq!(treaty.status, TreatyStatus::Broken);
    }

    #[test]
    fn test_treaty_expiration() {
        let mut manager = TreatyManager::new();
        let id = manager
            .propose_treaty(
                p(1),
                (p(1), p(2)),
                TreatyType::Peace,
                vec![],
                Some(10),
            )
            .unwrap();

        manager.accept_treaty(p(1), id).unwrap();

        // Not expired yet
        manager.check_treaty_effects(5);
        assert_eq!(manager.get_treaty(id).unwrap().status, TreatyStatus::Active);

        // Expire the treaty
        manager.check_treaty_effects(10);
        assert_eq!(manager.get_treaty(id).unwrap().status, TreatyStatus::Expired);

        // Should not produce effects after expiration
        let effects = manager.check_treaty_effects(11);
        assert!(effects.is_empty());
    }

    #[test]
    fn test_treaty_errors() {
        let mut manager = TreatyManager::new();
        let id = manager
            .propose_treaty(p(1), (p(1), p(2)), TreatyType::Alliance, vec![], None)
            .unwrap();

        // Non-party cannot accept
        assert!(matches!(
            manager.accept_treaty(p(3), id),
            Err(TreatyError::NotParty(_))
        ));

        // Cannot accept an already active treaty
        manager.accept_treaty(p(1), id).unwrap();
        assert!(matches!(
            manager.accept_treaty(p(2), id),
            Err(TreatyError::InvalidState(TreatyStatus::Active))
        ));

        // Cannot break a proposed treaty
        let id2 = manager
            .propose_treaty(p(1), (p(1), p(3)), TreatyType::Trade, vec![], None)
            .unwrap();
        assert!(matches!(
            manager.break_treaty(p(1), id2),
            Err(TreatyError::InvalidState(TreatyStatus::Proposed))
        ));
    }

    #[test]
    fn test_propose_with_non_party() {
        let mut manager = TreatyManager::new();
        // Polity 3 is not part of the party (1, 2)
        let result = manager.propose_treaty(p(3), (p(1), p(2)), TreatyType::Peace, vec![], None);
        assert!(matches!(result, Err(TreatyError::NotParty(_))));
    }

    #[test]
    fn test_break_expired_treaty() {
        let mut manager = TreatyManager::new();
        let id = manager
            .propose_treaty(p(1), (p(1), p(2)), TreatyType::Peace, vec![], Some(10))
            .unwrap();
        manager.accept_treaty(p(1), id).unwrap();
        
        // Expire the treaty
        manager.check_treaty_effects(10);
        
        // Attempting to break an expired treaty should fail
        assert!(matches!(
            manager.break_treaty(p(1), id),
            Err(TreatyError::InvalidState(TreatyStatus::Expired))
        ));
    }

    #[test]
    fn test_active_treaty_effects_per_tick() {
        let mut manager = TreatyManager::new();
        let id = manager
            .propose_treaty(
                p(1),
                (p(1), p(2)),
                TreatyType::Alliance,
                vec![],
                Some(100),
            )
            .unwrap();
        manager.accept_treaty(p(1), id).unwrap();

        // Check effects multiple times; they should be consistent as long as treaty is active
        let effects_1 = manager.check_treaty_effects(10);
        let effects_2 = manager.check_treaty_effects(50);
        
        assert_eq!(effects_1.len(), 1);
        assert_eq!(effects_2.len(), 1);
        assert_eq!(effects_1[0].standing_delta, 10); // Alliance effect
    }
}
