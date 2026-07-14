#![cfg(all(feature = "bevy", feature = "egui"))]

//! Save-slot browser panel for FR-CIV-SAVE-001 and FR-CIV-SAVE-002.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_engine::{
    delete_slot, list_slots, load_from_slot, save_bundle::SaveBundleError, save_to_slot,
    SaveSlotEntry,
};

use crate::sim_bridge::SimState;
use crate::ui_theme::{liquid_glass_frame, GLASS_FILL, KC_ACCENT, RADIUS_PANEL};

/// Panel visibility, cached slot rows, and transient save/load status.
#[derive(Resource, Debug)]
pub struct SaveLoadPanel {
    /// Whether the slot browser is currently visible.
    pub visible: bool,
    /// Name entered in the Save As field.
    pub draft_name: String,
    /// Last known slot rows.
    pub slots: Vec<SaveSlotEntry>,
    /// Last user-visible status line.
    pub last_status: String,
    refresh_requested: bool,
}

impl Default for SaveLoadPanel {
    fn default() -> Self {
        Self {
            visible: false,
            draft_name: "slot-1".to_string(),
            slots: Vec::new(),
            last_status: String::new(),
            refresh_requested: true,
        }
    }
}

/// Registers the save/load slot browser.
pub struct SaveLoadUiPlugin;

impl Plugin for SaveLoadUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadPanel>().add_systems(
            EguiPrimaryContextPass,
            (refresh_slots_when_needed, render_save_slot_browser).chain(),
        );
    }
}

/// Default saves folder relative to the process run directory.
#[must_use]
pub fn default_saves_dir() -> PathBuf {
    PathBuf::from("saves")
}

/// Slot rows prepared for UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSlotRow {
    /// Slot name without extension.
    pub name: String,
    /// Engine tick read from save metadata, if any.
    pub tick: Option<u64>,
}

/// Build sorted browser rows from a save-slot directory.
pub fn build_slot_rows(saves_dir: &Path) -> Result<Vec<SaveSlotRow>, SaveBundleError> {
    let mut rows: Vec<_> = list_slots(saves_dir)?
        .into_iter()
        .map(|entry| SaveSlotRow {
            name: entry.name,
            tick: (entry.tick > 0).then_some(entry.tick),
        })
        .collect();
    rows.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(rows)
}

fn refresh_slots_when_needed(mut panel: ResMut<SaveLoadPanel>) {
    if !panel.visible && !panel.refresh_requested {
        return;
    }
    if !panel.refresh_requested {
        return;
    }
    let saves_dir = default_saves_dir();
    match std::fs::create_dir_all(&saves_dir).and_then(|_| {
        list_slots(&saves_dir)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    }) {
        Ok(slots) => {
            panel.slots = slots;
            panel.refresh_requested = false;
        }
        Err(err) => {
            panel.last_status = format!("Could not read saves: {err}");
            panel.refresh_requested = false;
        }
    }
}

enum SaveSlotAction {
    SaveAs(String),
    Load(String),
    Delete(String),
    Refresh,
}

fn render_save_slot_browser(
    mut contexts: EguiContexts,
    mut panel: ResMut<SaveLoadPanel>,
    sim: Option<ResMut<SimState>>,
) {
    if !panel.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut open = panel.visible;
    let mut action = None;
    let has_local_sim = sim.is_some();

    egui::Window::new(egui::RichText::new("Save / Load").color(KC_ACCENT).strong())
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(480.0)
        .frame(
            liquid_glass_frame(egui::Margin::same(18), RADIUS_PANEL)
                .fill(GLASS_FILL)
                .inner_margin(egui::Margin::same(18)),
        )
        .show(ctx, |ui| {
            save_slot_browser_ui(ui, &mut panel, has_local_sim, &mut action);
        });

    panel.visible = open;

    if let Some(action) = action {
        apply_save_slot_action(action, &mut panel, sim);
    }
}

fn save_slot_browser_ui(
    ui: &mut egui::Ui,
    panel: &mut SaveLoadPanel,
    has_local_sim: bool,
    action: &mut Option<SaveSlotAction>,
) {
    ui.label(egui::RichText::new("Local saves").color(KC_ACCENT).strong());
    ui.label(egui::RichText::new("saves/").monospace().small());
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .max_height(260.0)
        .show(ui, |ui| {
            if panel.slots.is_empty() {
                ui.label(egui::RichText::new("No save slots yet.").italics());
            }
            for slot in &panel.slots {
                save_slot_row(ui, slot, has_local_sim, action);
            }
        });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.add(
            egui::TextEdit::singleline(&mut panel.draft_name)
                .desired_width(220.0)
                .hint_text("slot name"),
        );
        if ui
            .add_enabled(has_local_sim, egui::Button::new("Save As"))
            .clicked()
        {
            *action = Some(SaveSlotAction::SaveAs(panel.draft_name.clone()));
        }
        if ui.button("Refresh").clicked() {
            *action = Some(SaveSlotAction::Refresh);
        }
    });

    if !has_local_sim {
        ui.label(
            egui::RichText::new("Local simulation state is not available in this mode.").small(),
        );
    }
    if !panel.last_status.is_empty() {
        ui.add_space(6.0);
        ui.label(egui::RichText::new(&panel.last_status).small());
    }
}

fn save_slot_row(
    ui: &mut egui::Ui,
    slot: &SaveSlotEntry,
    has_local_sim: bool,
    action: &mut Option<SaveSlotAction>,
) {
    liquid_glass_frame(egui::Margin::symmetric(10, 8), 8)
        .fill(GLASS_FILL.gamma_multiply(1.08))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&slot.name).strong());
                    let tick = if slot.tick > 0 {
                        format!("tick {}", slot.tick)
                    } else {
                        "tick unavailable".to_string()
                    };
                    ui.label(egui::RichText::new(tick).monospace().small());
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Delete").clicked() {
                        *action = Some(SaveSlotAction::Delete(slot.name.clone()));
                    }
                    if ui
                        .add_enabled(has_local_sim, egui::Button::new("Load"))
                        .clicked()
                    {
                        *action = Some(SaveSlotAction::Load(slot.name.clone()));
                    }
                });
            });
        });
    ui.add_space(6.0);
}

fn apply_save_slot_action(
    action: SaveSlotAction,
    panel: &mut SaveLoadPanel,
    sim: Option<ResMut<SimState>>,
) {
    let saves_dir = default_saves_dir();
    if let Err(err) = std::fs::create_dir_all(&saves_dir) {
        panel.last_status = format!("Could not create saves/: {err}");
        return;
    }

    match action {
        SaveSlotAction::SaveAs(name) => {
            let Some(sim) = sim else {
                panel.last_status = "No local simulation to save.".to_string();
                return;
            };
            match save_to_slot(&saves_dir, &name, &sim.0) {
                Ok(()) => {
                    panel.last_status = format!("Saved {name}");
                    panel.refresh_requested = true;
                }
                Err(err) => panel.last_status = format!("Save failed: {err}"),
            }
        }
        SaveSlotAction::Load(name) => {
            let Some(mut sim) = sim else {
                panel.last_status = "No local simulation to load into.".to_string();
                return;
            };
            match load_from_slot(&saves_dir, &name) {
                Ok(loaded) => {
                    sim.0 = loaded;
                    panel.last_status = format!("Loaded {name}");
                    panel.visible = false;
                    panel.refresh_requested = true;
                }
                Err(err) => panel.last_status = format!("Load failed: {err}"),
            }
        }
        SaveSlotAction::Delete(name) => match delete_slot(&saves_dir, &name) {
            Ok(true) => {
                panel.last_status = format!("Deleted {name}");
                panel.refresh_requested = true;
            }
            Ok(false) => panel.last_status = format!("Slot not found: {name}"),
            Err(err) => panel.last_status = format!("Delete failed: {err}"),
        },
        SaveSlotAction::Refresh => {
            panel.refresh_requested = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_slot_rows_sorts_fake_slots_by_name() {
        let dir = std::env::temp_dir().join(format!("civis-save-ui-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp saves dir");
        std::fs::write(dir.join("zeta.civsave.zst"), b"not a real save").expect("zeta");
        std::fs::write(dir.join("alpha.civsave.zst"), b"not a real save").expect("alpha");

        let rows = build_slot_rows(&dir).expect("rows");
        let names: Vec<_> = rows.into_iter().map(|row| row.name).collect();

        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(names, vec!["alpha".to_string(), "zeta".to_string()]);
    }
}
