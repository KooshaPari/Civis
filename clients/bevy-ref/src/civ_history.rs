#![cfg(all(feature = "bevy", feature = "egui"))]
//! Civilization statistics history panel (FR-CIV-CLIENT-013).
//! Y key toggles. Samples every 10 ticks. ASCII sparklines (8 levels).

use std::collections::VecDeque;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::menus::in_game;

const HISTORY_CAP: usize = 200;
const SAMPLE_EVERY: u64 = 10;

#[derive(Resource)]
pub struct CivHistory {
    pub population:   VecDeque<u32>,
    pub entropy:      VecDeque<f32>,
    pub faction_count: VecDeque<u8>,
    pub power_law:    VecDeque<f32>,
    last_sampled_tick: u64,
}

impl Default for CivHistory {
    fn default() -> Self {
        Self {
            population:    VecDeque::with_capacity(HISTORY_CAP),
            entropy:       VecDeque::with_capacity(HISTORY_CAP),
            faction_count: VecDeque::with_capacity(HISTORY_CAP),
            power_law:     VecDeque::with_capacity(HISTORY_CAP),
            last_sampled_tick: 0,
        }
    }
}

impl CivHistory {
    fn push<T: Copy>(buf: &mut VecDeque<T>, v: T) {
        if buf.len() >= HISTORY_CAP { buf.pop_front(); }
        buf.push_back(v);
    }
}

#[derive(Resource, Default)]
pub struct CivHistoryPanelOpen(pub bool);

pub struct CivHistoryPlugin;
impl Plugin for CivHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CivHistory>()
           .init_resource::<CivHistoryPanelOpen>()
           .add_systems(Update, (toggle_history_panel, sample_history, draw_history_panel)
               .chain().run_if(in_game));
    }
}

fn toggle_history_panel(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<CivHistoryPanelOpen>) {
    if keys.just_pressed(KeyCode::KeyY) { open.0 = !open.0; }
}

fn sample_history(hud: Res<crate::HudState>, mut hist: ResMut<CivHistory>) {
    let tick = hud.snapshot.tick.unwrap_or(0);
    if tick == 0 || tick.saturating_sub(hist.last_sampled_tick) < SAMPLE_EVERY { return; }
    hist.last_sampled_tick = tick;
    CivHistory::push(&mut hist.population, hud.snapshot.civilian_count as u32);
    CivHistory::push(&mut hist.faction_count, hud.snapshot.faction_count.min(255) as u8);
    let (ent, pl) = hud.snapshot.emergence.as_ref()
        .map(|e| (e.entropy_norm, e.power_law_alpha))
        .unwrap_or((0.0, 0.0));
    CivHistory::push(&mut hist.entropy, ent);
    CivHistory::push(&mut hist.power_law, pl);
}

fn sparkline(buf: &VecDeque<impl Into<f64> + Copy>) -> String {
    const BARS: &[char] = &[' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
    if buf.is_empty() { return "—".to_string(); }
    let vals: Vec<f64> = buf.iter().map(|&v| v.into()).collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-9);
    vals.iter().map(|&v| {
        let idx = (((v - mn) / range) * 8.0).round() as usize;
        BARS[idx.min(8)]
    }).collect()
}

fn stat_row<T: Into<f64> + Copy + std::fmt::Display>(
    ui: &mut egui::Ui, label: &str, buf: &VecDeque<T>
) {
    let current = buf.back().copied();
    let mn = buf.iter().cloned().map(|v| v.into()).fold(f64::INFINITY, f64::min);
    let mx = buf.iter().cloned().map(|v| v.into()).fold(f64::NEG_INFINITY, f64::max);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{label:<14}")).monospace()
            .color(egui::Color32::from_rgb(126, 186, 181)).size(11.0));
        let spark = sparkline(buf);
        ui.label(egui::RichText::new(&spark).monospace()
            .color(egui::Color32::from_rgb(160, 200, 180)).size(11.0));
        if let Some(cur) = current {
            ui.label(egui::RichText::new(format!("  {cur}  [{mn:.1}–{mx:.1}]")).monospace()
                .color(egui::Color32::from_rgb(180, 185, 190)).size(10.0));
        }
    });
}

fn draw_history_panel(
    open: Res<CivHistoryPanelOpen>,
    hist: Res<CivHistory>,
    mut contexts: EguiContexts,
) {
    if !open.0 { return; }
    let ctx = contexts.ctx_mut();
    let screen = ctx.screen_rect();
    egui::Window::new("Civilization History")
        .fixed_pos(egui::pos2(screen.center().x - 260.0, 60.0))
        .fixed_size([520.0, 200.0])
        .collapsible(false)
        .frame(egui::Frame::window(ctx.style().as_ref())
            .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(126, 186, 181))))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(format!("{} samples, every {} ticks  [Y] close",
                hist.population.len(), SAMPLE_EVERY))
                .color(egui::Color32::from_rgb(100, 110, 120)).size(10.0));
            ui.separator();
            stat_row(ui, "Population", &hist.population);
            stat_row(ui, "Factions", &hist.faction_count);
            stat_row(ui, "Entropy", &hist.entropy);
            stat_row(ui, "Power-law α", &hist.power_law);
        });
}