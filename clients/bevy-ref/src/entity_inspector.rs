//! P1.3.1 — click-to-select entity inspector panel for the Bevy reference client.
//!
//! Reads [`LiveSelection`] from [`crate::live_pick`] (live attach raycast pick) and
//! optional standalone inspect data, then draws a persistent right-side egui panel
//! with entity type, position, faction, and health (for civilians).

#![cfg(all(feature = "bevy", feature = "egui"))]

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::game_ui::{
    inspector_details_for_live_entity, inspector_details_from_civilian, parse_health_fraction,
    SelectedEntityDetails,
};
use crate::live_pick::LiveSelection;
use crate::live_stream::{
    LiveAgentTag, LiveBuildingTag, LiveGraphParcelTag, LiveStreamScene,
};
use crate::{AttachMode, LiveEntityKind, SelectedLiveEntity};

/// Plugin: syncs pick selection into inspector rows and draws the right-side panel.
pub struct EntityInspectorPlugin;

impl Plugin for EntityInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedEntityDetails>()
            .add_systems(
                Update,
                (
                    sync_inspector_from_live_selection,
                    sync_inspector_from_inspected,
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                draw_entity_inspector_panel,
            );
    }
}

/// Populate inspector rows from the live viewport pick (`LiveSelection`).
fn sync_inspector_from_live_selection(
    selection: Res<LiveSelection>,
    scene: Res<LiveStreamScene>,
    agents: Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: Query<(&LiveGraphParcelTag, &GlobalTransform)>,
    mut details: ResMut<SelectedEntityDetails>,
) {
    if !selection.is_changed() && !scene.is_changed() {
        return;
    }

    let Some(selected) = selection.0 else {
        if selection.is_changed() {
            *details = SelectedEntityDetails::default();
        }
        return;
    };

    let position = live_entity_world_position(selected, &agents, &buildings, &graph_parcels);

    *details = if selected.kind == LiveEntityKind::Agent {
        scene
            .civilian_entries
            .get(&selected.id)
            .map(|entry| {
                let mut rows = inspector_details_from_civilian(entry);
                if let Some(pos) = position {
                    rows.position = crate::game_ui::format_world_position(pos);
                }
                rows
            })
            .unwrap_or_else(|| inspector_details_for_live_entity(selected, position))
    } else {
        inspector_details_for_live_entity(selected, position)
    };
}

/// Standalone sandbox: mirror [`crate::inspect::InspectedDetails`] into the panel.
fn sync_inspector_from_inspected(
    attach: Res<AttachMode>,
    inspected: Option<Res<crate::inspect::InspectedDetails>>,
    mut details: ResMut<SelectedEntityDetails>,
) {
    if *attach != AttachMode::Standalone {
        return;
    }
    let Some(inspected) = inspected else {
        return;
    };
    if !inspected.is_changed() {
        return;
    }
    *details = inspected.0.clone();
}

fn live_entity_world_position(
    selected: SelectedLiveEntity,
    agents: &Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: &Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: &Query<(&LiveGraphParcelTag, &GlobalTransform)>,
) -> Option<Vec3> {
    match selected.kind {
        LiveEntityKind::Agent => agents
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
        LiveEntityKind::Building => buildings
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
        LiveEntityKind::GraphParcel => graph_parcels
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
        LiveEntityKind::VoxelChunk => Some(crate::chunk_world_centre(
            civ_voxel::ChunkId(selected.id),
            crate::VOXEL_CHUNK_EDGE,
        )
        .into()),
    }
}

/// Persistent right-side inspector panel (shown while a selection is active).
fn draw_entity_inspector_panel(
    mut contexts: EguiContexts,
    selection: Res<LiveSelection>,
    details: Res<SelectedEntityDetails>,
) {
    let has_live = selection.0.is_some();
    let has_details = !details.name.is_empty() || !details.entity_type.is_empty();
    if !has_live && !has_details {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::right("civis_entity_inspector")
        .resizable(true)
        .default_width(268.0)
        .frame(inspector_panel_frame(egui::Margin::same(14)))
        .show(ctx, |ui| {
            inspector_panel_body(ui, &details);
        });
}

fn inspector_panel_frame(margin: egui::Margin) -> egui::Frame {
    use crate::ui_theme::{PANEL_FILL, RADIUS};
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .inner_margin(margin)
        .corner_radius(egui::CornerRadius::same(RADIUS))
}

fn inspector_panel_body(ui: &mut egui::Ui, details: &SelectedEntityDetails) {
    use crate::ui_theme::{ACCENT, DIM, TEXT};

    ui.heading(egui::RichText::new("\u{25a4} Inspector").color(ACCENT));
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);

    inspector_row(ui, "Type", &details.entity_type);
    inspector_row(ui, "Name", &details.name);
    inspector_row(ui, "Faction", &details.faction);

    ui.add_space(2.0);
    ui.label(egui::RichText::new("Health").color(DIM).small());
    if let Some(frac) = parse_health_fraction(&details.health) {
        let color = if frac > 0.66 {
            egui::Color32::from_rgb(120, 220, 130)
        } else if frac > 0.33 {
            egui::Color32::from_rgb(240, 200, 90)
        } else {
            egui::Color32::from_rgb(230, 90, 90)
        };
        ui.add(
            egui::ProgressBar::new(frac)
                .fill(color)
                .text(details.health.clone()),
        );
    } else if details.health.is_empty() || details.health == "—" {
        ui.label(egui::RichText::new("—").color(TEXT));
    } else {
        ui.label(egui::RichText::new(&details.health).strong());
    }
    ui.add_space(2.0);

    inspector_row(ui, "Profession", &details.profession);
    inspector_row(ui, "Species", &details.species);
    inspector_row(ui, "Needs", &details.needs);
    inspector_row(ui, "Position", &details.position);
}

fn inspector_row(ui: &mut egui::Ui, name: &str, value: &str) {
    use crate::ui_theme::{DIM, TEXT};
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).color(TEXT).strong());
        });
    });
}
