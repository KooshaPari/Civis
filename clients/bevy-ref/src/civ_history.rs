#![cfg(all(feature = "bevy", feature = "egui"))]
//! Civilization statistics history panel (FR-CIV-CLIENT-013).
//! Covers: FR-CIV-CLIENT-013
//! Y key toggles. Samples every 10 ticks. ASCII sparklines (8 levels).

use crate::hud_state::HudState;
use crate::live_stream::ServerBridge;
use crate::menus::in_playing_state;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::collections::VecDeque;

const HISTORY_CAP: usize = 200;
const SAMPLE_EVERY: u64 = 10;

#[derive(Resource)]
pub struct CivHistory {
    pub population: VecDeque<u32>,
    pub entropy: VecDeque<f32>,
    pub faction_count: VecDeque<u8>,
    pub power_law: VecDeque<f32>,
    last_sampled_tick: u64,
}

impl Default for CivHistory {
    fn default() -> Self {
        Self {
            population: VecDeque::with_capacity(HISTORY_CAP),
            entropy: VecDeque::with_capacity(HISTORY_CAP),
            faction_count: VecDeque::with_capacity(HISTORY_CAP),
            power_law: VecDeque::with_capacity(HISTORY_CAP),
            last_sampled_tick: 0,
        }
    }
}

impl CivHistory {
    fn push<T: Copy>(buf: &mut VecDeque<T>, v: T) {
        if buf.len() >= HISTORY_CAP {
            buf.pop_front();
        }
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
            .add_systems(
                Update,
                (toggle_history_panel, sample_history, draw_history_panel)
                    .chain()
                    .run_if(in_playing_state),
            );
    }
}

fn toggle_history_panel(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<CivHistoryPanelOpen>) {
    if keys.just_pressed(KeyCode::KeyY) {
        open.0 = !open.0;
    }
}

fn sample_history(hud: Res<HudState>, mut hist: ResMut<CivHistory>) {
    let tick = hud.snapshot.tick.unwrap_or(0);
    if tick == 0 || tick.saturating_sub(hist.last_sampled_tick) < SAMPLE_EVERY {
        return;
    }
    hist.last_sampled_tick = tick;
    CivHistory::push(&mut hist.population, hud.snapshot.civilian_count as u32);
    CivHistory::push(
        &mut hist.faction_count,
        hud.snapshot.faction_count.min(255) as u8,
    );
    let (ent, pl) = hud
        .snapshot
        .emergence
        .as_ref()
        .map(|e| (e.entropy_norm, e.power_law_alpha))
        .unwrap_or((0.0, 0.0));
    CivHistory::push(&mut hist.entropy, ent);
    CivHistory::push(&mut hist.power_law, pl);
}

fn sparkline(buf: &VecDeque<impl Into<f64> + Copy>) -> String {
    const BARS: &[char] = &[
        ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    if buf.is_empty() {
        return "—".to_string();
    }
    let vals: Vec<f64> = buf.iter().map(|&v| v.into()).collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-9);
    vals.iter()
        .map(|&v| {
            let idx = (((v - mn) / range) * 8.0).round() as usize;
            BARS[idx.min(8)]
        })
        .collect()
}

fn stat_row<T: Into<f64> + Copy + std::fmt::Display>(
    ui: &mut egui::Ui,
    label: &str,
    buf: &VecDeque<T>,
) {
    let current = buf.back().copied();
    let mn = buf
        .iter()
        .cloned()
        .map(|v| v.into())
        .fold(f64::INFINITY, f64::min);
    let mx = buf
        .iter()
        .cloned()
        .map(|v| v.into())
        .fold(f64::NEG_INFINITY, f64::max);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{label:<14}"))
                .monospace()
                .color(egui::Color32::from_rgb(126, 186, 181))
                .size(11.0),
        );
        let spark = sparkline(buf);
        ui.label(
            egui::RichText::new(&spark)
                .monospace()
                .color(egui::Color32::from_rgb(160, 200, 180))
                .size(11.0),
        );
        if let Some(cur) = current {
            ui.label(
                egui::RichText::new(format!("  {cur}  [{mn:.1}–{mx:.1}]"))
                    .monospace()
                    .color(egui::Color32::from_rgb(180, 185, 190))
                    .size(10.0),
            );
        }
    });
}

fn draw_history_panel(
    open: Res<CivHistoryPanelOpen>,
    hist: Res<CivHistory>,
    mut contexts: EguiContexts,
    bridge: Option<Res<ServerBridge>>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen = ctx.content_rect();
    egui::Window::new("Civilization History")
        .fixed_pos(egui::pos2(screen.center().x - 260.0, 60.0))
        .fixed_size([520.0, 200.0])
        .collapsible(false)
        .frame(
            egui::Frame::window(ctx.style().as_ref())
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(126, 186, 181),
                )),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} samples, every {} ticks  [Y] close",
                    hist.population.len(),
                    SAMPLE_EVERY
                ))
                .color(egui::Color32::from_rgb(100, 110, 120))
                .size(10.0),
            );
            ui.separator();
            stat_row(ui, "Population", &hist.population);
            stat_row(ui, "Factions", &hist.faction_count);
            stat_row(ui, "Entropy", &hist.entropy);
            stat_row(ui, "Power-law \u{03b1}", &hist.power_law);
            ui.add_space(4.0);
            if let Some(ref bridge) = bridge {
                ui.horizontal(|ui| {
                    if ui.small_button("Fetch Legends").clicked() {
                        bridge.send_rpc("sim.legends", serde_json::json!({}));
                    }
                    if ui.small_button("Fetch Outcome").clicked() {
                        bridge.send_rpc("sim.outcome", serde_json::json!({}));
                    }
                });
            }
        });
}

#[cfg(test)]
mod tests {
    // FR-CIV-CLIENT-013 — civ history panel: the Y key toggles the panel,
    // stream state is sampled into ring buffers every 10 ticks (capped at
    // HISTORY_CAP), and samples render as ASCII sparklines.
    use super::*;
    use crate::{EmergenceHudData, LiveHudSnapshot};
    use bevy::ecs::system::SystemState;

    fn hud_with_tick(tick: u64) -> HudState {
        HudState {
            snapshot: LiveHudSnapshot {
                tick: Some(tick),
                civilian_count: 1000,
                faction_count: 4,
                emergence: Some(EmergenceHudData {
                    entropy_norm: 0.5,
                    power_law_alpha: 2.1,
                    novelty_rate: 0.01,
                    ..Default::default()
                }),
                ..Default::default()
            },
            text: Entity::PLACEHOLDER,
        }
    }

    // FR-CIV-CLIENT-013 — stream state lands in ring buffers every 10 ticks.
    #[test]
    fn fr_civ_client_013_samples_stream_state_every_ten_ticks() {
        let mut world = World::new();
        world.insert_resource(CivHistory::default());
        let mut state: SystemState<(Res<HudState>, ResMut<CivHistory>)> =
            SystemState::new(&mut world);

        // Ticks 0 and 5 are too early; 10 samples; 15 is mid-interval; 20 samples again.
        for tick in [0u64, 5, 10, 15, 20] {
            world.insert_resource(hud_with_tick(tick));
            let (hud, hist) = state.get_mut(&mut world);
            sample_history(hud, hist);
        }

        let hist = world.resource::<CivHistory>();
        assert_eq!(
            hist.population.iter().copied().collect::<Vec<_>>(),
            vec![1000, 1000],
            "exactly two samples: first at tick 10, second at tick 20"
        );
        assert_eq!(
            hist.faction_count.iter().copied().collect::<Vec<_>>(),
            vec![4, 4],
            "faction census sampled alongside population"
        );
        assert_eq!(hist.entropy.back(), Some(&0.5), "entropy_norm sampled");
        assert!(
            (hist.power_law.back().copied().unwrap() - 2.1).abs() < 1e-6,
            "power-law alpha sampled"
        );
        assert_eq!(hist.last_sampled_tick, 20, "sampling cursor follows the stream");
    }

    // FR-CIV-CLIENT-013 — the Y key toggles the history panel open/closed.
    #[test]
    fn fr_civ_client_013_y_key_toggles_history_panel() {
        let mut world = World::new();
        world.insert_resource(CivHistoryPanelOpen(false));
        let mut state: SystemState<(Res<ButtonInput<KeyCode>>, ResMut<CivHistoryPanelOpen>)> =
            SystemState::new(&mut world);

        // Frame 1: Y pressed → panel opens.
        let mut pressed = ButtonInput::<KeyCode>::default();
        pressed.press(KeyCode::KeyY);
        world.insert_resource(pressed);
        let (keys, open) = state.get_mut(&mut world);
        toggle_history_panel(keys, open);
        assert!(world.resource::<CivHistoryPanelOpen>().0, "Y opens the panel");

        // Frame 2: no press → state held.
        world.insert_resource(ButtonInput::<KeyCode>::default());
        let (keys, open) = state.get_mut(&mut world);
        toggle_history_panel(keys, open);
        assert!(world.resource::<CivHistoryPanelOpen>().0, "no press keeps it open");

        // Frame 3: Y pressed again → panel closes.
        let mut pressed = ButtonInput::<KeyCode>::default();
        pressed.press(KeyCode::KeyY);
        world.insert_resource(pressed);
        let (keys, open) = state.get_mut(&mut world);
        toggle_history_panel(keys, open);
        assert!(
            !world.resource::<CivHistoryPanelOpen>().0,
            "Y closes the panel again"
        );
    }

    // FR-CIV-CLIENT-013 — sampled history renders as ASCII sparklines.
    #[test]
    fn fr_civ_client_013_sparkline_renders_samples_as_ascii() {
        // Empty history renders the em-dash placeholder.
        let empty: VecDeque<u32> = VecDeque::new();
        assert_eq!(sparkline(&empty), "—");

        // A flat run renders one bar per sample, all at the same level.
        let flat: VecDeque<u32> = vec![5u32; 4].into();
        let rendered = sparkline(&flat);
        assert_eq!(rendered.chars().count(), 4, "one bar per sample");
        let level = rendered.chars().next().unwrap();
        assert!(
            rendered.chars().all(|c| c == level),
            "constant samples share one bar level: {rendered}"
        );

        // Extremes map to the lowest and full blocks.
        let ramp: VecDeque<u32> = vec![0u32, 10].into();
        let chars: Vec<char> = sparkline(&ramp).chars().collect();
        assert_eq!(chars, vec![' ', '\u{2588}'], "min→blank, max→full block");
    }

    // FR-CIV-CLIENT-013 — the ring buffer never exceeds HISTORY_CAP samples.
    #[test]
    fn fr_civ_client_013_history_ring_buffer_caps_at_history_cap() {
        let mut buf: VecDeque<u32> = VecDeque::new();
        for i in 0..(HISTORY_CAP as u32 + 5) {
            CivHistory::push(&mut buf, i);
        }
        assert_eq!(buf.len(), HISTORY_CAP, "buffer never exceeds HISTORY_CAP");
        assert_eq!(buf.front().copied(), Some(5), "oldest samples evicted");
        assert_eq!(
            buf.back().copied(),
            Some(HISTORY_CAP as u32 + 4),
            "newest sample retained"
        );
    }
}
