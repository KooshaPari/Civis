//! Tests for FR-CIV-QOL-100
//!
//! Epic: FR-CIV-QOL
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-QOL-100: Tutorial / Onboarding — teach by guided play.
//! Engine-side: verify TutorialMilestone and TutorialProgress types exist.

#[cfg(test)]
mod fr_fr_civ_qol_100 {
    /// TutorialMilestone type exists for tracking onboarding progress.
    #[test]
    fn tutorial_milestone_exists() {
        let milestone = civ_engine::TutorialMilestone::FirstFaction;
        let _ = milestone;
    }

    /// TutorialProgress type exists for persistence and has a default.
    #[test]
    fn tutorial_progress_exists() {
        let progress = civ_engine::TutorialProgress::default();
        // Default progress starts at FirstFaction.
        assert!(matches!(progress.current, civ_engine::TutorialMilestone::FirstFaction));
    }
}
