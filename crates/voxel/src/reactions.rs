//! Pairwise material reactions for the voxel CA.

use serde::{Deserialize, Serialize};

use crate::MaterialId;

/// Result of a binary reaction rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionResult {
    /// Material written into the first reacting cell.
    pub left: MaterialId,
    /// Material written into the second reacting cell.
    pub right: MaterialId,
}

/// Binary reaction rule keyed by unordered material ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionRule {
    /// First reactant.
    pub a: MaterialId,
    /// Second reactant.
    pub b: MaterialId,
    /// Output pair.
    pub result: ReactionResult,
}

impl ReactionRule {
    /// Returns `true` when the rule matches the unordered pair.
    #[must_use]
    pub fn matches(self, left: MaterialId, right: MaterialId) -> bool {
        (self.a == left && self.b == right) || (self.a == right && self.b == left)
    }
}

/// Standard voxel reaction table.
pub const REACTIONS: &[ReactionRule] = &[
    ReactionRule {
        a: crate::material::LAVA,
        b: crate::material::WATER,
        result: ReactionResult {
            left: crate::material::STONE,
            right: crate::material::STEAM,
        },
    },
    ReactionRule {
        a: crate::material::FIRE,
        b: crate::material::OIL,
        result: ReactionResult {
            left: crate::material::FIRE,
            right: crate::material::FIRE,
        },
    },
    ReactionRule {
        a: crate::material::ACID,
        b: crate::material::STONE,
        result: ReactionResult {
            left: crate::material::AIR,
            right: crate::material::AIR,
        },
    },
    ReactionRule {
        a: crate::material::ACID,
        b: crate::material::WOOD,
        result: ReactionResult {
            left: crate::material::AIR,
            right: crate::material::AIR,
        },
    },
    ReactionRule {
        a: crate::material::WATER,
        b: crate::material::ICE,
        result: ReactionResult {
            left: crate::material::ICE,
            right: crate::material::ICE,
        },
    },
    ReactionRule {
        a: crate::material::GUNPOWDER,
        b: crate::material::FIRE,
        result: ReactionResult {
            left: crate::material::FIRE,
            right: crate::material::FIRE,
        },
    },
];

/// Looks up the first matching rule for an unordered pair.
#[must_use]
pub fn reaction_for(left: MaterialId, right: MaterialId) -> Option<ReactionRule> {
    REACTIONS
        .iter()
        .copied()
        .find(|rule| rule.matches(left, right))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_requested_rules() {
        let lava_water =
            reaction_for(crate::material::LAVA, crate::material::WATER).expect("lava + water");
        assert_eq!(lava_water.result.left, crate::material::STONE);
        assert_eq!(lava_water.result.right, crate::material::STEAM);

        let acid_wood =
            reaction_for(crate::material::WOOD, crate::material::ACID).expect("acid + wood");
        assert_eq!(acid_wood.result.left, crate::material::AIR);
        assert_eq!(acid_wood.result.right, crate::material::AIR);
    }

    #[test]
    fn table_contains_requested_interactions() {
        assert!(reaction_for(crate::material::FIRE, crate::material::OIL).is_some());
        assert!(reaction_for(crate::material::WATER, crate::material::ICE).is_some());
        assert!(reaction_for(crate::material::GUNPOWDER, crate::material::FIRE).is_some());
    }
}
