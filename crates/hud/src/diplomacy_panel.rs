//! Diplomacy panel — inter-polity relations for the HUD dashboard.
//!
//! Provides structured data types for the relations matrix, treaty
//! status, and aggregate threat level. All fields are serialisable
//! to JSON for wire transport to any client.
//!
//! Design contract:
//! 1. **Pure data, no engine.**
//! 2. **Additive only.**
//! 3. **Serialisation-safe.**

use serde::{Deserialize, Serialize};

/// Diplomatic stance between two polities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StanceKind {
    /// Active alliance or defensive pact.
    Allied,
    /// Non-aggression or ceasefire in effect.
    Neutral,
    /// Active hostility or at war.
    Hostile,
}

/// One entry in the pairwise relations matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationEntry {
    /// First polity id.
    pub polity_a: u32,
    /// Second polity id.
    pub polity_b: u32,
    /// Current stance classification.
    pub stance: StanceKind,
    /// Signed standing scalar (positive = warm, negative = cold).
    pub standing: i32,
}

/// Treaty summary for the panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreatySummary {
    /// Treaty identifier.
    pub id: u64,
    /// Parties involved.
    pub parties: (u32, u32),
    /// Treaty type label (e.g. "trade", "alliance").
    pub treaty_type: String,
    /// Current status label (e.g. "active", "proposed").
    pub status: String,
}

/// Aggregate threat level for the civilisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatLevel {
    /// No hostile relations detected.
    Peaceful,
    /// Some tensions but no open hostility.
    Tense,
    /// Active hostilities with at least one polity.
    AtWar,
}

/// Aggregated diplomacy panel snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyPanel {
    /// Pairwise relations matrix.
    pub relations: Vec<RelationEntry>,
    /// Active and proposed treaties.
    pub treaties: Vec<TreatySummary>,
    /// Aggregate threat level derived from the relations.
    pub threat_level: ThreatLevel,
}

impl DiplomacyPanel {
    /// Construct an empty panel with no relations.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            relations: Vec::new(),
            treaties: Vec::new(),
            threat_level: ThreatLevel::Peaceful,
        }
    }

    /// Number of tracked pairwise relations.
    #[must_use]
    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    /// Number of active treaties (status == "active").
    #[must_use]
    pub fn active_treaty_count(&self) -> usize {
        self.treaties
            .iter()
            .filter(|t| t.status == "active")
            .count()
    }

    /// Number of hostile relations.
    #[must_use]
    pub fn hostile_count(&self) -> usize {
        self.relations
            .iter()
            .filter(|r| r.stance == StanceKind::Hostile)
            .count()
    }

    /// Compute aggregate threat level from relation stances.
    #[must_use]
    pub fn compute_threat_level(&self) -> ThreatLevel {
        if self.hostile_count() > 0 {
            ThreatLevel::AtWar
        } else if self.relation_count() > 0 {
            ThreatLevel::Tense
        } else {
            ThreatLevel::Peaceful
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_panel() -> DiplomacyPanel {
        DiplomacyPanel {
            relations: vec![
                RelationEntry {
                    polity_a: 1,
                    polity_b: 2,
                    stance: StanceKind::Allied,
                    standing: 150,
                },
                RelationEntry {
                    polity_a: 1,
                    polity_b: 3,
                    stance: StanceKind::Hostile,
                    standing: -200,
                },
                RelationEntry {
                    polity_a: 2,
                    polity_b: 3,
                    stance: StanceKind::Neutral,
                    standing: 10,
                },
            ],
            treaties: vec![
                TreatySummary {
                    id: 1,
                    parties: (1, 2),
                    treaty_type: "trade".into(),
                    status: "active".into(),
                },
                TreatySummary {
                    id: 2,
                    parties: (1, 3),
                    treaty_type: "alliance".into(),
                    status: "proposed".into(),
                },
            ],
            threat_level: ThreatLevel::AtWar,
        }
    }

    #[test]
    fn empty_panel_has_no_relations() {
        let p = DiplomacyPanel::empty();
        assert_eq!(p.relation_count(), 0);
        assert_eq!(p.threat_level, ThreatLevel::Peaceful);
    }

    #[test]
    fn hostile_count_matches_relation_stances() {
        let p = sample_panel();
        assert_eq!(p.hostile_count(), 1);
    }

    #[test]
    fn active_treaty_count_filters_correctly() {
        let p = sample_panel();
        assert_eq!(p.active_treaty_count(), 1);
    }

    #[test]
    fn compute_threat_level_at_war_when_hostile_present() {
        let p = sample_panel();
        assert_eq!(p.compute_threat_level(), ThreatLevel::AtWar);
    }

    #[test]
    fn compute_threat_level_tense_when_no_hostile() {
        let p = DiplomacyPanel {
            relations: vec![RelationEntry {
                polity_a: 1,
                polity_b: 2,
                stance: StanceKind::Neutral,
                standing: 10,
            }],
            treaties: Vec::new(),
            threat_level: ThreatLevel::Peaceful,
        };
        assert_eq!(p.compute_threat_level(), ThreatLevel::Tense);
    }

    #[test]
    fn panel_round_trips_via_serde_json() {
        let p = sample_panel();
        let json = serde_json::to_string(&p).expect("serialize");
        let back: DiplomacyPanel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p.relations.len(), back.relations.len());
        assert_eq!(p.treaties.len(), back.treaties.len());
        assert_eq!(p.threat_level, back.threat_level);
    }
}
