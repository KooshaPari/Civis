//! Tech prerequisite graph for research unlock gating.
//!
//! FR-CIV-TECH-PREREQ: a tech unlocks only when all prerequisite techs are
//! researched and the accumulated research points meet the tech's threshold.

use std::collections::{BTreeMap, BTreeSet};

/// A single tech entry in the prerequisite graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechPrereqNode {
    /// Tech id.
    pub id: String,
    /// Tech ids that must already be researched.
    pub prerequisites: BTreeSet<String>,
    /// Research points required to unlock this tech.
    pub required_points: u32,
}

impl TechPrereqNode {
    /// Create a node from an id, prerequisite list, and point threshold.
    #[must_use]
    pub fn new<I, S>(id: impl Into<String>, prerequisites: I, required_points: u32) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: id.into(),
            prerequisites: prerequisites.into_iter().map(Into::into).collect(),
            required_points,
        }
    }
}

/// Directed prerequisite graph for tech research.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TechPrereqGraph {
    nodes: BTreeMap<String, TechPrereqNode>,
}

impl TechPrereqGraph {
    /// Insert or replace a tech node.
    pub fn insert(&mut self, node: TechPrereqNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Returns `true` when the tech exists, all prerequisites are researched,
    /// and the accrued research points are sufficient.
    #[must_use]
    pub fn is_unlocked(
        &self,
        tech_id: &str,
        researched: &BTreeSet<String>,
        points: u32,
    ) -> bool {
        let Some(node) = self.nodes.get(tech_id) else {
            return false;
        };

        points >= node.required_points
            && node
                .prerequisites
                .iter()
                .all(|prereq| researched.contains(prereq))
    }

    /// Returns `true` when a tech is still locked under the current state.
    #[must_use]
    pub fn is_locked(
        &self,
        tech_id: &str,
        researched: &BTreeSet<String>,
        points: u32,
    ) -> bool {
        !self.is_unlocked(tech_id, researched, points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-TECH-PREREQ — tech stays locked until prereqs are met, then unlocks.
    #[test]
    fn tech_stays_locked_until_prereqs_met_then_unlocks() {
        let mut graph = TechPrereqGraph::default();
        graph.insert(TechPrereqNode::new("advanced_rail", ["basic_rail", "steel"], 100));

        let mut researched = BTreeSet::new();
        researched.insert("basic_rail".to_string());

        assert!(graph.is_locked("advanced_rail", &researched, 100));
        assert!(graph.is_locked("advanced_rail", &researched, 99));

        researched.insert("steel".to_string());
        assert!(graph.is_locked("advanced_rail", &researched, 99));

        assert!(graph.is_unlocked("advanced_rail", &researched, 100));
        assert!(!graph.is_locked("advanced_rail", &researched, 100));
    }
}
