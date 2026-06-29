//! Tech prerequisite graph and unlock gating.
//!
//! This module is pure data logic: callers provide researched tech ids,
//! prerequisite edges, and optional point thresholds. The graph answers whether
//! a tech can unlock without touching engine state or persistence.

use std::collections::{BTreeMap, BTreeSet};

/// Stable tech identifier.
pub type TechId = String;

/// A single tech entry in the prerequisite graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechPrereqNode {
    /// Tech id.
    pub id: TechId,
    /// Tech ids that must already be researched.
    pub prerequisites: BTreeSet<TechId>,
    /// Research points required to unlock this tech.
    pub required_points: u32,
}

impl TechPrereqNode {
    /// Create a node from an id, prerequisite list, and point threshold.
    #[must_use]
    pub fn new<I, S>(id: impl Into<TechId>, prerequisites: I, required_points: u32) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<TechId>,
    {
        Self {
            id: id.into(),
            prerequisites: prerequisites.into_iter().map(Into::into).collect(),
            required_points,
        }
    }
}

/// Errors returned when building or mutating a prerequisite graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechPrereqError {
    /// A prerequisite references a tech id that is not registered.
    UnknownTech(TechId),
    /// Adding the dependency would create a cycle.
    Cycle { tech: TechId, prereq: TechId },
}

/// Directed prerequisite graph: `tech -> prerequisites`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TechPrereqGraph {
    nodes: BTreeMap<TechId, TechPrereqNode>,
}

impl TechPrereqGraph {
    /// Build an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tech id if absent.
    pub fn register(&mut self, tech: impl Into<TechId>) {
        let tech = tech.into();
        self.nodes
            .entry(tech.clone())
            .or_insert_with(|| TechPrereqNode::new(tech, std::iter::empty::<TechId>(), 0));
    }

    /// Insert or replace a tech node.
    pub fn insert(&mut self, node: TechPrereqNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add `prereq` as a prerequisite for `tech`.
    pub fn require(
        &mut self,
        tech: impl Into<TechId>,
        prereq: impl Into<TechId>,
    ) -> Result<(), TechPrereqError> {
        let tech = tech.into();
        let prereq = prereq.into();
        if !self.nodes.contains_key(&tech) {
            return Err(TechPrereqError::UnknownTech(tech));
        }
        if !self.nodes.contains_key(&prereq) {
            return Err(TechPrereqError::UnknownTech(prereq));
        }
        if tech == prereq || self.depends_on(&prereq, &tech) {
            return Err(TechPrereqError::Cycle { tech, prereq });
        }
        self.nodes
            .get_mut(&tech)
            .expect("tech existence checked")
            .prerequisites
            .insert(prereq);
        Ok(())
    }

    /// Return prerequisites for `tech`, if it is registered.
    #[must_use]
    pub fn prerequisites(&self, tech: &str) -> Option<&BTreeSet<TechId>> {
        self.nodes.get(tech).map(|node| &node.prerequisites)
    }

    /// True when every prerequisite for `tech` is in `researched`.
    #[must_use]
    pub fn can_unlock<'a>(
        &self,
        tech: &str,
        researched: impl IntoIterator<Item = &'a str>,
    ) -> bool {
        let Some(node) = self.nodes.get(tech) else {
            return false;
        };
        let researched: BTreeSet<&str> = researched.into_iter().collect();
        node.prerequisites
            .iter()
            .all(|p| researched.contains(p.as_str()))
    }

    /// Returns `true` when the tech exists, all prerequisites are researched,
    /// and the accrued research points are sufficient.
    #[must_use]
    pub fn is_unlocked(
        &self,
        tech_id: &str,
        researched: &BTreeSet<TechId>,
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
        researched: &BTreeSet<TechId>,
        points: u32,
    ) -> bool {
        !self.is_unlocked(tech_id, researched, points)
    }

    fn depends_on(&self, tech: &str, needle: &str) -> bool {
        let Some(node) = self.nodes.get(tech) else {
            return false;
        };
        node.prerequisites
            .iter()
            .any(|p| p == needle || self.depends_on(p, needle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_unlock_when_all_prereqs_are_researched() {
        let mut graph = TechPrereqGraph::new();
        graph.register("fire");
        graph.register("kiln");
        graph.require("kiln", "fire").unwrap();

        assert!(graph.can_unlock("kiln", ["fire"]));
        assert!(!graph.can_unlock("kiln", []));
    }

    #[test]
    fn rejects_cycles() {
        let mut graph = TechPrereqGraph::new();
        graph.register("a");
        graph.register("b");
        graph.require("b", "a").unwrap();

        assert_eq!(
            graph.require("a", "b"),
            Err(TechPrereqError::Cycle {
                tech: "a".into(),
                prereq: "b".into()
            })
        );
    }

    /// FR-CIV-TECH-PREREQ - tech stays locked until prereqs are met, then unlocks.
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
