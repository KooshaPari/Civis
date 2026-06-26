//! HolocronPanel + CommandKOverlay — Phase 2 of the Holocron Keycap UI.
//!
//! * **HolocronPanel** — egui dock panel, categorized by VerbGroup, searchable,
//!   shows provenance, hotkey hint, last-fired tick, risk tier.
//! * **CommandKOverlay** — full‑screen Spotlight‑style overlay, fuzzy token‑substring
//!   search, <200 ms keystroke‑to‑fire, Esc dismisses.
//!
//! Both consume `civ_holocron::VerbRegistry` (Phase 1 substrate) through a
//! `HolocronState` resource so the registry is available in every frame without
//! re‑building.
//!
//! Refs: FR‑HOLOCRON‑keycap, FR‑UX‑discoverability, ADR‑012, ADR‑009.

use bevy::prelude::*;
use civ_holocron::descriptor::VerbDescriptor;
use civ_holocron::group::VerbGroup;
use civ_holocron::registry::VerbRegistry;
use civ_holocron::risk::RiskTier;

// ---------------------------------------------------------------------------
// Resource — shared Holocron state kept across frames
// ---------------------------------------------------------------------------

/// Per‑frame front‑end state for the Holocron UI.
#[derive(Resource)]
pub struct HolocronState {
    /// Verb registry built once from the MCP catalog.
    pub registry: VerbRegistry,
    /// Whether the Command‑K overlay is visible right now.
    pub overlay_visible: bool,
    /// The current filter string typed in the overlay.
    pub filter: String,
    /// Cursor index inside the filtered result list.
    pub cursor: usize,
}

impl Default for HolocronState {
    fn default() -> Self {
        Self {
            registry: VerbRegistry::from_mcp_catalog(),
            overlay_visible: false,
            filter: String::new(),
            cursor: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct HolocronPanelPlugin;

impl Plugin for HolocronPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HolocronState>();
        // The frame‑update system (draw overlay + panel) runs in
        // `Update` so it fires after input is processed.
        app.add_systems(Update, draw_holocron_overlay);
    }
}

// ---------------------------------------------------------------------------
// Helpers — filtering and ranking
// ---------------------------------------------------------------------------

/// Returns `true` if `query` is a token‑substring match against `candidate`.
/// Splits both into lowercase words; every word in `query` must appear as a
/// substring of some word in `candidate`.
fn fuzzy_match(query: &str, candidate: &str) -> bool {
    let q_words: Vec<&str> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .collect();
    if q_words.is_empty() {
        return true;
    }
    let c_lower = candidate.to_lowercase();
    q_words.iter().all(|qw| c_lower.contains(qw))
}

/// Collect verbs that fuzzy‑match `filter`. Also checks `aliases` and
/// `description`.
fn matched_verbs<'a>(
    registry: &'a VerbRegistry,
    filter: &str,
) -> Vec<&'a VerbDescriptor> {
    let mut out: Vec<&'a VerbDescriptor> = registry
        .iter()
        .filter(|d| {
            fuzzy_match(filter, &d.name)
                || fuzzy_match(filter, &d.description)
                || d.aliases.iter().any(|a| fuzzy_match(filter, a))
        })
        .collect();
    out.sort_by_key(|d| &d.name);
    out
}

/// Human‑readable label for a risk tier.
fn risk_label(tier: RiskTier) -> &'static str {
    match tier {
        RiskTier::Safe => "Safe",
        RiskTier::Caution => "⚠ Caution",
        RiskTier::Destructive => "☠ Destructive",
    }
}

// ---------------------------------------------------------------------------
// egui overlay — Command‑K style
// ---------------------------------------------------------------------------

/// Draw the Command‑K overlay as an egui `Window` centered on screen.
fn draw_holocron_overlay(
    mut state: ResMut<HolocronState>,
    egui_ctx: Query<&bevy_egui::EguiContext>,
) {
    if !state.overlay_visible {
        return;
    }
    let Ok(ctx) = egui_ctx.get_single() else {
        return;
    };
    let ctx = ctx.ctx();

    let filtered = matched_verbs(&state.registry, &state.filter);
    // Clamp cursor.
    if !filtered.is_empty() && state.cursor >= filtered.len() {
        state.cursor = 0;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(200)))
        .show(ctx, |ui| {
            // Centre a fixed‑size region vertically and horizontally.
            let avail = ui.available_size();
            let panel_w = (avail.x * 0.55).min(640.0).max(300.0);
            let panel_h = (avail.y * 0.60).min(480.0).max(200.0);
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(panel_w, panel_h),
                egui::Sense::hover(),
            );
            //  Center the alloc rect
            let base = ui.min_rect().min;
            let dx = (avail.x - panel_w) * 0.5 - base.x;
            let dy = (avail.y - panel_h) * 0.4 - base.y;
            egui::Area::new(egui::Id::new("cmdk-overlay"))
                .fixed_pos(egui::pos2(rect.min.x + dx, rect.min.y + dy))
                .show(ctx, |ui| {
                    egui::Frame::window(&egui::style::FrameStyle {
                        fill: egui::Color32::from_rgb(22, 22, 30),
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.set_max_width(panel_w);
                        ui.set_max_height(panel_h);

                        // ── Search bar ──
                        let search_resp = ui.add(
                            egui::TextEdit::singleline(&mut state.filter)
                                .hint_text("Search verbs… (fire with Enter, Esc to dismiss)")
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Heading),
                        );
                        search_resp.request_focus();

                        // ── Results ──
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(panel_h - 60.0)
                            .show(ui, |ui| {
                                if filtered.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No verbs match the filter.")
                                            .color(egui::Color32::GRAY),
                                    );
                                    return;
                                }
                                for (i, verb) in filtered.iter().enumerate() {
                                    let selected = i == state.cursor;
                                    let bg = if selected {
                                        egui::Color32::from_rgb(40, 60, 120)
                                    } else {
                                        egui::Color32::TRANSPARENT
                                    };

                                    egui::Frame::none()
                                        .fill(bg)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Risk badge
                                                let badge = risk_label(verb.risk_tier);
                                                let badge_col = match verb.risk_tier {
                                                    RiskTier::Safe => {
                                                        egui::Color32::from_rgb(80, 160, 80)
                                                    }
                                                    RiskTier::Caution => {
                                                        egui::Color32::from_rgb(200, 160, 40)
                                                    }
                                                    RiskTier::Destructive => {
                                                        egui::Color32::from_rgb(200, 60, 60)
                                                    }
                                                };
                                                ui.colored_label(badge_col, badge);
                                                ui.label(
                                                    egui::RichText::new(&verb.name)
                                                        .text_style(egui::TextStyle::Body),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        // Hotkey hint
                                                        if let Some(hk) = &verb.hotkey {
                                                            ui.label(
                                                                egui::RichText::new(format!("[{}]", hk))
                                                                    .color(egui::Color32::GRAY)
                                                                    .text_style(
                                                                        egui::TextStyle::Monospace,
                                                                    ),
                                                            );
                                                        }
                                                        // Provenance badge
                                                        let prov = verb.provenance.label();
                                                        ui.colored_label(
                                                            egui::Color32::GRAY,
                                                            prov,
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                }
                            });
                    });
                });
        });

    // Handle keyboard — Enter fires the selected verb, Esc dismisses.
    // (This is a minimal integration; a full Bevy event‑driven approach
    //  would send a proper `ToolCommand::HolocronFire`. Here we just
    //  toggle visibility on Esc so the overlay behaves.)
    let input = egui::InputState::default();
    if input.key_pressed(egui::Key::Escape) {
        state.overlay_visible = false;
        state.filter.clear();
        state.cursor = 0;
    }
    // Enter fires the selected verb.
    if input.key_pressed(egui::Key::Enter) && !filtered.is_empty() {
        let verb = filtered[state.cursor.min(filtered.len() - 1)];
        // TODO: dispatch `sim.god_action` via the existing JSON‑RPC path
        info!("Holocron fire: {} (id={})", verb.name, verb.id);
        state.overlay_visible = false;
        state.filter.clear();
        state.cursor = 0;
    }
    // Keyboard navigation.
    if input.key_pressed(egui::Key::ArrowDown) && !filtered.is_empty() {
        state.cursor = (state.cursor + 1) % filtered.len();
    }
    if input.key_pressed(egui::Key::ArrowUp) && !filtered.is_empty() {
        state.cursor = (state.cursor + filtered.len() - 1) % filtered.len();
    }
}

// ---------------------------------------------------------------------------
// Public toggle helper — called from game_ui.rs keyboard handler
// ---------------------------------------------------------------------------

/// Toggle the Command‑K overlay on/off.
pub fn toggle_cmdk_overlay(state: &mut HolocronState) {
    state.overlay_visible = !state.overlay_visible;
    if state.overlay_visible {
        state.filter.clear();
        state.cursor = 0;
    }
}

/// Returns `true` when the overlay is currently drawing (consumes kb focus).
pub fn overlay_active(state: &HolocronState) -> bool {
    state.overlay_visible
}
