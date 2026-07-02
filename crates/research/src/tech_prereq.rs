//! Tech prerequisite graph and unlock gating.
//!
//! This module is pure data logic: callers provide researched tech ids and
//! prerequisite edges, and the graph answers whether a tech can unlock without
//! touching engine state or persistence.

use std::collections::{BTreeMap, BTreeSet};

/// Stable tech identifier.
pub type TechId = String;

/// Errors returned when building or mutating a prerequisite graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TechPrereqError {
    /// A prerequisite references a tech id that is not registered.
    UnknownTech(TechId),
    /// Adding the dependency would create a cycle.
    Cycle {
        /// Tech whose prerequisite edge was being added.
        tech: TechId,
        /// Prerequisite that would close the cycle.
        prereq: TechId,
    },
}

/// Directed prerequisite graph: `tech -> prerequisites`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TechPrereqGraph {
    prereqs: BTreeMap<TechId, BTreeSet<TechId>>,
}

impl TechPrereqGraph {
    /// Build an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tech id if absent.
    pub fn register(&mut self, tech: impl Into<TechId>) {
        self.prereqs.entry(tech.into()).or_default();
    }

    /// Add `prereq` as a prerequisite for `tech`.
    pub fn require(
        &mut self,
        tech: impl Into<TechId>,
        prereq: impl Into<TechId>,
    ) -> Result<(), TechPrereqError> {
        let tech = tech.into();
        let prereq = prereq.into();
        if !self.prereqs.contains_key(&tech) {
            return Err(TechPrereqError::UnknownTech(tech));
        }
        if !self.prereqs.contains_key(&prereq) {
            return Err(TechPrereqError::UnknownTech(prereq));
        }
        if tech == prereq || self.depends_on(&prereq, &tech) {
            return Err(TechPrereqError::Cycle { tech, prereq });
        }
        self.prereqs
            .get_mut(&tech)
            .expect("tech existence checked")
            .insert(prereq);
        Ok(())
    }

    /// Return prerequisites for `tech`, if it is registered.
    #[must_use]
    pub fn prerequisites(&self, tech: &str) -> Option<&BTreeSet<TechId>> {
        self.prereqs.get(tech)
    }

    /// True when every prerequisite for `tech` is in `researched`.
    #[must_use]
    pub fn can_unlock<'a>(
        &self,
        tech: &str,
        researched: impl IntoIterator<Item = &'a str>,
    ) -> bool {
        let Some(prereqs) = self.prereqs.get(tech) else {
            return false;
        };
        let researched: BTreeSet<&str> = researched.into_iter().collect();
        prereqs.iter().all(|p| researched.contains(p.as_str()))
    }

    fn depends_on(&self, tech: &str, needle: &str) -> bool {
        let Some(prereqs) = self.prereqs.get(tech) else {
            return false;
        };
        prereqs
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
}
