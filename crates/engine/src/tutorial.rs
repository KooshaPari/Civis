//! Tutorial/onboarding milestone progression (FR-CIV-TUTORIAL).
//!
//! This is intentionally additive: it observes live sim state after the
//! existing phase pipeline has run and advances a small state machine that
//! clients can read from snapshots.

use serde::{Deserialize, Serialize};

use crate::engine::Simulation;

/// Ordered tutorial milestones surfaced to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
// The following 1 requirement tags were removed from TutorialMilestone.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// A retarget onto advance_from_sim would be correct today; the tag currently claims coverage from a bare enum.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-NOTIFY-920: The requirement in docs/specs/requirements/FR-CIV-NOTIFY.md is tutorial milestones as a behavior: milestones SHALL advance as the simulation reaches them. `TutorialMilestone` is a five-variant ordered enum naming the stages. The advancement does exist, but it is `advance_from_sim` further down the same file, which is therefore the real implementing symbol; the tag sits on the enum that only enumerates them, and that enum implements nothing.
pub enum TutorialMilestone {
    FirstFaction,
    FirstTech,
    FirstWar,
    FirstReligion,
    Complete,
}

/// Engine-side tutorial progression state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TutorialProgress {
    pub current: TutorialMilestone,
    pub faction_exists: bool,
    pub tech_unlocked: bool,
    pub war_declared: bool,
    pub religion_emerged: bool,
}

impl Default for TutorialProgress {
    fn default() -> Self {
        Self {
            current: TutorialMilestone::FirstFaction,
            faction_exists: false,
            tech_unlocked: false,
            war_declared: false,
            religion_emerged: false,
        }
    }
}

impl TutorialProgress {
    #[must_use]
    pub fn completed(self) -> bool {
        matches!(self.current, TutorialMilestone::Complete)
    }

    pub fn advance_from_sim(&mut self, sim: &Simulation) {
        let faction_exists = !sim.state.factions.is_empty();
        let tech_unlocked = sim
            .era_progression
            .faction_tech
            .values()
            .any(|tech| tech.tech_level > 0);
        let war_declared = sim
            .diplomacy_events()
            .iter()
            .any(|event| matches!(event.kind, crate::engine::DiplomacyKind::Conflict));
        let religion_emerged = !sim.religious_profiles.is_empty();

        if faction_exists {
            self.faction_exists = true;
            self.current = self.current.max(TutorialMilestone::FirstFaction);
        }
        if tech_unlocked {
            self.tech_unlocked = true;
            self.current = self.current.max(TutorialMilestone::FirstTech);
        }
        if war_declared {
            self.war_declared = true;
            self.current = self.current.max(TutorialMilestone::FirstWar);
        }
        if religion_emerged {
            self.religion_emerged = true;
            self.current = self.current.max(TutorialMilestone::FirstReligion);
        }
        if self.faction_exists && self.tech_unlocked && self.war_declared && self.religion_emerged {
            self.current = TutorialMilestone::Complete;
        }
    }
}
