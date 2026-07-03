#![cfg(all(feature = "bevy", feature = "egui"))]

//! Sandbox event-feed HUD panel — scrolling live emergence event log (P2.4).
//!
//! Reads [`EmergenceHudData`] and [`EventFeed`] to surface emergence-class
//! simulation events: regime changes, entropy spikes, novelty bursts, and
//! power-law shifts.  Toggle with **F8**.
//!
//! Implementation follows the exact pattern of `emergence_dashboard.rs`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::{
    event_feed::{EventFeed, EventKind, GameEvent},
    EmergenceHudData,
};

// ── Palette (mirrors emergence_dashboard.rs) ──────────────────────────────────

const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);

// ── Thresholds for emergence anomaly detection ────────────────────────────────

const ENTROPY_HIGH_THRESHOLD: f32 = 0.85;
const ENTROPY_LOW_THRESHOLD: f32 = 0.15;
const NOVELTY_BURST_THRESHOLD: f32 = 0.05;
const ALPHA_OUT_OF_RANGE_LOW: f32 = 1.5;
const ALPHA_OUT_OF_RANGE_HIGH: f32 = 4.0;

// ── Resource: open/closed toggle ──────────────────────────────────────────────

/// Whether the sandbox event-feed panel is open.  Bound to **F8**.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SandboxEventFeedOpen(pub bool);

impl Default for SandboxEventFeedOpen {
    fn default() -> Self {
        Self(false)
    }
}

// ── Resource: tracked emergence state for change detection ────────────────────

/// Internal state used to detect regime/metric transitions between frames.
#[derive(Resource, Default)]
pub struct EmergenceEventTracker {
    last_regime: String,
    last_entropy_band: EntropyBand,
    last_novelty_burst: bool,
    last_alpha_anomaly: bool,
}

/// Discretised entropy band for change detection.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyBand {
    /// Entropy below [`ENTROPY_LOW_THRESHOLD`].
    Low,
    /// Entropy in normal range.
    #[default]
    Normal,
    /// Entropy above [`ENTROPY_HIGH_THRESHOLD`].
    High,
}

impl EntropyBand {
    fn from_norm(norm: f32) -> Self {
        if norm < ENTROPY_LOW_THRESHOLD {
            Self::Low
        } else if norm > ENTROPY_HIGH_THRESHOLD {
            Self::High
        } else {
            Self::Normal
        }
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the sandbox event-feed panel and its driving systems.
pub struct SandboxEventFeedPlugin;

impl Plugin for SandboxEventFeedPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SandboxEventFeedOpen>()
            .init_resource::<EmergenceEventTracker>()
            .add_systems(Update, (toggle_sandbox_feed, detect_emergence_events))
            .add_systems(EguiPrimaryContextPass, draw_sandbox_event_feed);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_sandbox_feed(
    keys: Res<ButtonInput<KeyCode>>,
    mut open: ResMut<SandboxEventFeedOpen>,
) {
    if keys.just_pressed(KeyCode::F8) {
        open.0 = !open.0;
    }
}

/// Detect emergence anomalies and push descriptive events into [`EventFeed`].
fn detect_emergence_events(
    emergence_data: Option<Res<EmergenceHudData>>,
    mut tracker: ResMut<EmergenceEventTracker>,
    mut feed: ResMut<EventFeed>,
) {
    let Some(em) = emergence_data else {
        return;
    };

    // Regime transition
    if em.branching_regime != tracker.last_regime && !tracker.last_regime.is_empty() {
        feed.push(
            EventKind::System,
            format!(
                "Regime transition: {} → {}",
                tracker.last_regime, em.branching_regime
            ),
        );
    }
    tracker.last_regime = em.branching_regime.clone();

    // Entropy band transition
    let band = EntropyBand::from_norm(em.entropy_norm);
    if band != tracker.last_entropy_band {
        let desc = match band {
            EntropyBand::High => format!(
                "Entropy spike: {:.3} norm ({:.2} bits) — system highly disordered",
                em.entropy_norm, em.entropy_bits
            ),
            EntropyBand::Low => format!(
                "Entropy collapse: {:.3} norm ({:.2} bits) — system freezing",
                em.entropy_norm, em.entropy_bits
            ),
            EntropyBand::Normal => format!(
                "Entropy stabilised: {:.3} norm ({:.2} bits)",
                em.entropy_norm, em.entropy_bits
            ),
        };
        feed.push(EventKind::System, desc);
    }
    tracker.last_entropy_band = band;

    // Novelty burst
    let novelty_burst = em.novelty_rate > NOVELTY_BURST_THRESHOLD;
    if novelty_burst && !tracker.last_novelty_burst {
        feed.push(
            EventKind::Tech,
            format!(
                "Novelty burst: {:.4}/tick — civilisational innovation spike",
                em.novelty_rate
            ),
        );
    }
    tracker.last_novelty_burst = novelty_burst;

    // Power-law alpha anomaly
    let alpha_anomaly =
        em.power_law_alpha < ALPHA_OUT_OF_RANGE_LOW || em.power_law_alpha > ALPHA_OUT_OF_RANGE_HIGH;
    if alpha_anomaly && !tracker.last_alpha_anomaly {
        feed.push(
            EventKind::Disaster,
            format!(
                "Power-law α anomaly: {:.2} (target 2–3) — cluster dynamics unstable",
                em.power_law_alpha
            ),
        );
    }
    tracker.last_alpha_anomaly = alpha_anomaly;
}

fn draw_sandbox_event_feed(
    mut contexts: EguiContexts,
    open: Res<SandboxEventFeedOpen>,
    feed: Res<EventFeed>,
    emergence_data: Option<Res<EmergenceHudData>>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Sandbox Event Feed")
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(8.0, 8.0))
        .default_width(340.0)
        .default_height(420.0)
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
            draw_header(ui, &emergence_data);
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            draw_regime_badge(ui, &emergence_data);
            ui.add_space(6.0);
            draw_event_list(ui, &feed);
        });
}

// ── UI helpers ────────────────────────────────────────────────────────────────

fn draw_header(ui: &mut egui::Ui, emergence_data: &Option<Res<EmergenceHudData>>) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Sandbox Events")
                .color(ACCENT)
                .strong()
                .size(14.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let tick_label = emergence_data
                .as_ref()
                .map(|_| "[F8] hide")
                .unwrap_or("[F8] hide · no sim");
            ui.label(egui::RichText::new(tick_label).color(DIM).small().italics());
        });
    });
}

fn draw_regime_badge(ui: &mut egui::Ui, emergence_data: &Option<Res<EmergenceHudData>>) {
    match emergence_data.as_deref() {
        None => {
            ui.label(
                egui::RichText::new("Awaiting emergence data…")
                    .color(DIM)
                    .italics()
                    .small(),
            );
        }
        Some(em) => {
            let (label, color) = regime_badge_info(&em.branching_regime);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Regime").color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(label).color(color).strong().small());
                });
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Entropy").color(DIM).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.3}", em.entropy_norm))
                            .strong()
                            .small(),
                    );
                });
            });
        }
    }
    ui.add_space(4.0);
}

fn draw_event_list(ui: &mut egui::Ui, feed: &EventFeed) {
    let count = feed.events.len();
    ui.label(
        egui::RichText::new(format!("{count} events (newest first)"))
            .color(DIM)
            .small(),
    );
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .max_height(340.0)
        .show(ui, |ui| {
            if feed.events.is_empty() {
                ui.label(
                    egui::RichText::new("No events yet — simulation events will appear here.")
                        .color(DIM)
                        .italics()
                        .small(),
                );
                return;
            }
            for ev in feed.events.iter() {
                event_row(ui, ev);
                ui.add_space(2.0);
            }
        });
}

fn event_row(ui: &mut egui::Ui, ev: &GameEvent) {
    let color = ev.kind.color();
    egui::Frame::NONE
        .fill(CHIP_FILL)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(ev.kind.emoji()).color(color).size(13.0));
                ui.label(egui::RichText::new(&ev.text).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", ev.age))
                            .color(DIM)
                            .small(),
                    );
                });
            });
        });
}

fn regime_badge_info(regime: &str) -> (&'static str, egui::Color32) {
    let lower = regime.trim().to_ascii_lowercase();
    if lower.contains("supercritical") || lower.contains("explosion") {
        ("SUPERCRITICAL", RED)
    } else if lower.contains("heat-death")
        || lower.contains("heat death")
        || lower.contains("subcritical")
    {
        ("SUBCRITICAL", ACCENT)
    } else {
        ("EDGE OF CHAOS", GREEN)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-024 — sandbox feed panel starts closed by default.
    #[test]
    fn panel_default_state_is_closed() {
        let open = SandboxEventFeedOpen::default();
        assert!(!open.0, "panel must start closed (F8 opens it)");
    }

    #[test]
    fn entropy_band_classification() {
        assert_eq!(EntropyBand::from_norm(0.05), EntropyBand::Low);
        assert_eq!(EntropyBand::from_norm(0.5), EntropyBand::Normal);
        assert_eq!(EntropyBand::from_norm(0.95), EntropyBand::High);
        assert_eq!(
            EntropyBand::from_norm(ENTROPY_LOW_THRESHOLD),
            EntropyBand::Normal,
            "boundary value at low threshold is Normal"
        );
        assert_eq!(
            EntropyBand::from_norm(ENTROPY_HIGH_THRESHOLD),
            EntropyBand::Normal,
            "boundary value at high threshold is Normal"
        );
    }

    #[test]
    fn regime_badge_info_supercritical() {
        let (label, _color) = regime_badge_info("SUPERCRITICAL");
        assert_eq!(label, "SUPERCRITICAL");
    }

    #[test]
    fn regime_badge_info_subcritical() {
        let (label, _color) = regime_badge_info("SUBCRITICAL");
        assert_eq!(label, "SUBCRITICAL");
    }

    #[test]
    fn regime_badge_info_edge_of_chaos() {
        let (label, _color) = regime_badge_info("CRITICAL");
        assert_eq!(label, "EDGE OF CHAOS");
    }

    #[test]
    fn tracker_default_has_empty_regime() {
        let tracker = EmergenceEventTracker::default();
        assert!(tracker.last_regime.is_empty());
    }
}
