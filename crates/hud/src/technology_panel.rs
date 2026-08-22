//! Technology panel — research progress for the HUD dashboard.
//!
//! Provides structured data types for active research projects,
//! completion status, tech tree coverage, and overall research
//! output. All fields are serialisable to JSON for wire transport.
//!
//! Design contract:
//! 1. **Pure data, no engine.**
//! 2. **Additive only.**
//! 3. **Serialisation-safe.**

use serde::{Deserialize, Serialize};

/// Status of a single research project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Currently receiving research points.
    Active,
    /// Completed and awaiting integration.
    Completed,
    /// Blocked by a missing dependency.
    Blocked,
}

/// One research project in the panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchProject {
    /// Stable project identifier.
    pub id: String,
    /// Human-readable project name.
    pub name: String,
    /// Current status.
    pub status: ProjectStatus,
    /// Research points invested so far.
    pub points_invested: u64,
    /// Total points required for completion.
    pub points_required: u64,
    /// Focus weight (higher = more points allocated per tick).
    pub weight: u64,
}

impl ResearchProject {
    /// Completion fraction (0.0-1.0). Clamped at 1.0.
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.points_required == 0 {
            return 0.0;
        }
        (self.points_invested as f32 / self.points_required as f32).min(1.0)
    }
}

/// Tech tree coverage statistic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechTreeCoverage {
    /// Total techs in the tree.
    pub total: u32,
    /// Techs unlocked / researched.
    pub unlocked: u32,
    /// Current era index.
    pub current_era: u16,
}

impl TechTreeCoverage {
    /// Fraction of the tree unlocked (0.0-1.0).
    #[must_use]
    pub fn unlocked_fraction(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.unlocked as f32 / self.total as f32
    }
}

/// Aggregated technology panel snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnologyPanel {
    /// Active and recent research projects.
    pub projects: Vec<ResearchProject>,
    /// Tech tree coverage summary.
    pub tree: TechTreeCoverage,
    /// Total research points spent this tick.
    pub points_this_tick: u64,
}

impl TechnologyPanel {
    /// Construct an empty panel.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            projects: Vec::new(),
            tree: TechTreeCoverage { total: 0, unlocked: 0, current_era: 0 },
            points_this_tick: 0,
        }
    }

    /// Number of currently active projects.
    #[must_use]
    pub fn active_project_count(&self) -> usize {
        self.projects
            .iter()
            .filter(|p| p.status == ProjectStatus::Active)
            .count()
    }

    /// Number of completed projects.
    #[must_use]
    pub fn completed_project_count(&self) -> usize {
        self.projects
            .iter()
            .filter(|p| p.status == ProjectStatus::Completed)
            .count()
    }

    /// Number of blocked projects.
    #[must_use]
    pub fn blocked_project_count(&self) -> usize {
        self.projects
            .iter()
            .filter(|p| p.status == ProjectStatus::Blocked)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel() -> TechnologyPanel {
        TechnologyPanel {
            projects: vec![
                ResearchProject {
                    id: "rail_track".into(),
                    name: "Rail Track".into(),
                    status: ProjectStatus::Active,
                    points_invested: 75,
                    points_required: 100,
                    weight: 3,
                },
                ResearchProject {
                    id: "steel_forge".into(),
                    name: "Steel Forge".into(),
                    status: ProjectStatus::Completed,
                    points_invested: 50,
                    points_required: 50,
                    weight: 1,
                },
                ResearchProject {
                    id: "void_drive".into(),
                    name: "Void Drive".into(),
                    status: ProjectStatus::Blocked,
                    points_invested: 0,
                    points_required: 200,
                    weight: 0,
                },
            ],
            tree: TechTreeCoverage { total: 40, unlocked: 12, current_era: 3 },
            points_this_tick: 15,
        }
    }

    #[test]
    fn empty_panel_has_no_projects() {
        let p = TechnologyPanel::empty();
        assert_eq!(p.active_project_count(), 0);
        assert_eq!(p.completed_project_count(), 0);
        assert_eq!(p.blocked_project_count(), 0);
    }

    #[test]
    fn progress_computes_correctly() {
        let proj = &sample_panel().projects[0];
        assert!((proj.progress() - 0.75).abs() < 1e-6);

        let completed = &sample_panel().projects[1];
        assert!((completed.progress() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn active_completed_blocked_counts() {
        let p = sample_panel();
        assert_eq!(p.active_project_count(), 1);
        assert_eq!(p.completed_project_count(), 1);
        assert_eq!(p.blocked_project_count(), 1);
    }

    #[test]
    fn tree_unlocked_fraction() {
        let tree = &sample_panel().tree;
        assert!((tree.unlocked_fraction() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn tree_unlocked_fraction_empty_tree() {
        let tree = TechTreeCoverage { total: 0, unlocked: 0, current_era: 0 };
        assert!((tree.unlocked_fraction() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn panel_round_trips_via_serde_json() {
        let p = sample_panel();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: TechnologyPanel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p.projects.len(), back.projects.len());
        assert_eq!(p.tree.current_era, back.tree.current_era);
        assert_eq!(p.points_this_tick, back.points_this_tick);
    }
}
