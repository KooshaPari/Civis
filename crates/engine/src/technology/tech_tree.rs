//! Tech tree with prerequisite enforcement (FR-CIV-TECH).
//!
//! Techs are keyed by [`TechId`] (a stable `u32` index) and arranged in a DAG.
//! [`TechTree::can_unlock`] and [`TechTree::try_unlock`] enforce that all
//! prerequisites are satisfied before a tech can be researched.

use crate::era::CivEra;

/// Stable identifier for a technology node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TechId(pub u32);

/// A single node in the tech DAG.
#[derive(Debug, Clone)]
pub struct TechNode {
    /// Stable identifier.
    pub id: TechId,
    /// Human-readable name.
    pub name: &'static str,
    /// Technologies that must already be researched before this one can be
    /// unlocked.
    pub prerequisites: Vec<TechId>,
    /// Earliest era in which this tech is available.
    pub era: CivEra,
}

/// The complete directed-acyclic graph of available technologies.
#[derive(Debug, Default, Clone)]
pub struct TechTree {
    /// All registered tech nodes.
    pub nodes: Vec<TechNode>,
}

impl TechTree {
    /// Returns `true` when all prerequisites for `tech_id` are present in
    /// `researched`.
    #[must_use]
    pub fn can_unlock(&self, tech_id: TechId, researched: &[TechId]) -> bool {
        let Some(node) = self.nodes.iter().find(|n| n.id == tech_id) else {
            return false;
        };
        node.prerequisites
            .iter()
            .all(|prereq| researched.contains(prereq))
    }

    /// Attempt to unlock `tech_id`, adding it to `researched` on success.
    ///
    /// # Errors
    /// Returns `Err` if the tech does not exist, is already researched, or has
    /// unmet prerequisites.
    pub fn try_unlock(
        &self,
        tech_id: TechId,
        researched: &mut Vec<TechId>,
    ) -> Result<(), String> {
        if self.nodes.iter().all(|n| n.id != tech_id) {
            return Err(format!("tech {:?} does not exist in the tree", tech_id));
        }
        if researched.contains(&tech_id) {
            return Err(format!("tech {:?} is already researched", tech_id));
        }
        if !self.can_unlock(tech_id, researched) {
            return Err(format!(
                "tech {:?} has unmet prerequisites",
                tech_id
            ));
        }
        researched.push(tech_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> TechTree {
        TechTree {
            nodes: vec![
                TechNode {
                    id: TechId(0),
                    name: "Agriculture",
                    prerequisites: vec![],
                    era: CivEra::Prehistoric,
                },
                TechNode {
                    id: TechId(1),
                    name: "Writing",
                    prerequisites: vec![TechId(0)],
                    era: CivEra::Ancient,
                },
                TechNode {
                    id: TechId(2),
                    name: "Philosophy",
                    prerequisites: vec![TechId(1)],
                    era: CivEra::Classical,
                },
            ],
        }
    }

    #[test]
    fn can_unlock_root_with_no_prereqs() {
        let tree = sample_tree();
        assert!(tree.can_unlock(TechId(0), &[]));
    }

    #[test]
    fn prereqs_enforced_can_unlock_false() {
        let tree = sample_tree();
        assert!(!tree.can_unlock(TechId(1), &[]));
    }

    #[test]
    fn prereqs_satisfied_can_unlock_true() {
        let tree = sample_tree();
        assert!(tree.can_unlock(TechId(1), &[TechId(0)]));
    }

    #[test]
    fn try_unlock_succeeds_when_prereqs_met() {
        let tree = sample_tree();
        let mut researched = vec![TechId(0)];
        assert!(tree.try_unlock(TechId(1), &mut researched).is_ok());
        assert!(researched.contains(&TechId(1)));
    }

    #[test]
    fn try_unlock_fails_missing_prereq() {
        let tree = sample_tree();
        let mut researched = vec![];
        let result = tree.try_unlock(TechId(1), &mut researched);
        assert!(result.is_err());
    }

    #[test]
    fn try_unlock_fails_already_researched() {
        let tree = sample_tree();
        let mut researched = vec![TechId(0)];
        tree.try_unlock(TechId(0), &mut researched).unwrap_err(); // already in vec
        // TechId(0) was already there before unlock attempt; confirm error
        let mut r2 = vec![];
        r2.push(TechId(0)); // pre-seed as if researched
        let err = tree.try_unlock(TechId(0), &mut r2);
        assert!(err.is_err());
    }

    #[test]
    fn unknown_tech_cannot_unlock() {
        let tree = sample_tree();
        assert!(!tree.can_unlock(TechId(99), &[]));
    }
}
