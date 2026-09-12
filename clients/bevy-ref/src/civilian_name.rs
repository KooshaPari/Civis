//! Pure formatter for `CivilianStateEntry` display names.
//!
//! Lives outside `game_ui` so the live-streamed Bevy presentation layer can
//! label agent markers with `Text2d` even when the optional `egui` feature is
//! off (the `bevy` standalone window keeps the floating name labels alive
//! without pulling in the egui HUD crate). The egui `game_ui` module re-exports
//! this exact function so existing call sites keep their public API.

use civ_protocol_3d::CivilianStateEntry;

/// Display name for a civilian wire entry (genome summary or stable id).
#[must_use]
pub fn civilian_display_name(entry: &CivilianStateEntry) -> String {
    if entry.genome_summary.summary.is_empty() {
        format!("Civilian #{}", entry.id)
    } else {
        entry.genome_summary.summary.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civilian_display_name_falls_back_to_id() {
        use civ_protocol_3d::GenomeSummary3d;

        let entry = CivilianStateEntry {
            id: 7,
            faction_id: 0,
            needs: Default::default(),
            profession: String::new(),
            genome_summary: GenomeSummary3d::default(),
            species: String::new(),
            health: 1.0,
        };
        assert_eq!(civilian_display_name(&entry), "Civilian #7");
    }

    #[test]
    fn civilian_display_name_prefers_genome_summary() {
        use civ_protocol_3d::GenomeSummary3d;

        let entry = CivilianStateEntry {
            id: 12,
            faction_id: 1,
            needs: Default::default(),
            profession: "smith".to_string(),
            genome_summary: GenomeSummary3d {
                summary: "Ada".to_string(),
                ..Default::default()
            },
            species: "human".to_string(),
            health: 0.9,
        };
        assert_eq!(civilian_display_name(&entry), "Ada");
    }
}
