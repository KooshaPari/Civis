#![cfg(all(feature = "bevy", feature = "egui"))]

//! World stats dashboard (`sim.snapshot` + `emergence.metrics` read-API polling).
//!
//! Toggle with `V`. Combines world snapshot counters (population / buildings /
//! prices / factions), and the latest emergence sample pulled by the existing
//! HUD poller.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(255, 180, 80);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldStatsDashboardOpen(pub bool);

impl Default for WorldStatsDashboardOpen {
    fn default() -> Self {
        Self(false)
    }
}

pub struct WorldStatsDashboardPlugin;

impl Plugin for WorldStatsDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldStatsDashboardOpen>()
            .add_systems(Update, toggle_world_stats_dashboard)
            .add_systems(EguiPrimaryContextPass, draw_world_stats_dashboard);
    }
}

fn toggle_world_stats_dashboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<WorldStatsDashboardOpen>,
) {
    if keys.just_pressed(KeyCode::KeyV) {
        open.0 = !open.0;
    }
}

fn draw_world_stats_dashboard(
    mut contexts: EguiContexts,
    open: Res<WorldStatsDashboardOpen>,
    hud: Option<Res<crate::HudState>>,
) {
    if !open.0 {
        return;
    }
    let Some(hud) = hud else {
        return;
    };
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let snapshot = &hud.snapshot;
    let world = &snapshot.world_stats;
    let emergence = snapshot.emergence.as_ref();

    egui::Window::new("World Stats")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .default_width(340.0)
        .resizable(true)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(14))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("World Stats")
                        .color(ACCENT)
                        .size(14.0)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("[V] hide").color(DIM).small().italics());
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);

            ui.label(egui::RichText::new("Read-API Snapshot").color(ACCENT));
            metric_row(
                ui,
                "Population",
                world.population.to_string().as_str(),
                ACCENT,
            );
            metric_row(
                ui,
                "Buildings",
                world.building_count.to_string().as_str(),
                DIM,
            );
            metric_row(
                ui,
                "Speed",
                format!("{}x", world.speed_multiplier).as_str(),
                GREEN,
            );
            if let Some(tick) = snapshot.tick {
                metric_row(ui, "Tick", tick.to_string().as_str(), DIM);
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Market Prices").color(ACCENT));
            if world.market_prices.is_empty() {
                ui.label(egui::RichText::new("No market pricing data yet").color(DIM).small());
            } else {
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for (good, cents) in world.market_prices.iter() {
                            metric_row(ui, good, cents.to_string().as_str(), DIM);
                        }
                    });
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Factions / Cohesion").color(ACCENT));
            if world.factions.is_empty() {
                ui.label(egui::RichText::new("No faction rows yet").color(DIM).small());
            } else {
                let mut faction_text = String::new();
                for faction in &world.factions {
                    faction_text.push_str(&format!(
                        "{}: {} — pop {} territory {} cohesion n/a\n",
                        faction.id, faction.name, faction.population, faction.territory_size,
                    ));
                }
                ui.label(egui::RichText::new(faction_text.trim_end()).small().color(DIM));
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Emergence / Discovery").color(ACCENT));
            match emergence {
                None => {
                    ui.label(
                        egui::RichText::new("Awaiting first emergence sample (tick 50+)")
                            .color(DIM)
                            .italics(),
                    );
                }
                Some(em) => {
                    metric_row(
                        ui,
                        "Entropy",
                        &format!("{:.3} norm ({:.3} bits)", em.entropy_norm, em.entropy_bits),
                        ACCENT,
                    );
                    metric_row(
                        ui,
                        "Structure count",
                        em.structure_count
                            .map(|n| n.to_string())
                            .as_deref()
                            .unwrap_or("—"),
                        DIM,
                    );
                    metric_row(
                        ui,
                        "Novelty",
                        &format!("{:.4} / {:.4}", em.novelty_rate, em.novelty_score),
                        GREEN,
                    );
                    metric_row(
                        ui,
                        "Coupling MI",
                        em.mi_material_faction_norm
                            .map(|value| format!("{value:.3}"))
                            .as_deref()
                            .unwrap_or("—"),
                        DIM,
                    );
                    metric_row(
                        ui,
                        "Criticality",
                        &format!("{:.3}", em.criticality_indicator),
                        criticality_color(em.criticality_indicator),
                    );
                }
            }
        });
}

fn metric_row(ui: &mut egui::Ui, label: &str, value: &str, value_color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(DIM).small());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(value).color(value_color).small().strong());
        });
    });
    ui.add_space(2.0);
}

fn criticality_color(value: f32) -> egui::Color32 {
    match value {
        value if value >= 1.2 => RED,
        value if value >= 0.8 => GOLD,
        value if value >= 0.4 => ORANGE,
        _ => GREEN,
    }
}
