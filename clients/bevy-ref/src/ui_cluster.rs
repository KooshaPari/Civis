#![cfg(all(feature = "bevy", feature = "egui"))]

//! Expanding-cluster widget — the toolbar/brush interaction model for the Civis
//! HUD overhaul.
//!
//! The pattern is two stacked **rectangles** (block-pills, never a smooth
//! trapezoid):
//!
//! * a small **category rect** — always visible as a rectangular block-pill
//!   carrying the category label + icon;
//! * clicking it expands a larger **items rect** (drawn *above* the category
//!   rect, growing upward out of the bottom bar) holding the sub-tool buttons.
//!   Items stack in a grid up to [`CLUSTER_VISIBLE_ROWS`] rows, after which the
//!   list becomes a vertically **scrollable** overflow.
//!
//! Only the small category rect is visible until the cluster is opened. The
//! material language is Liquid-Glass / Mica frosted glass pulled from
//! [`crate::ui_theme`] (teal accent on edges/glow only, layered translucency,
//! gloss + soft inner glow, ease-out motion).
//!
//! This module owns *only* the cluster widget primitives; `game_ui.rs` wires
//! them to [`crate::tool_categories`] data + the active-tool resources.

use bevy_egui::egui;

use crate::tool_categories::{Category, SubTool};
use crate::ui_theme::{
    liquid_glass_pill, motion_rect, paint_cluster_icon_label, DECK_ACCENT, DECK_BORDER, DECK_GLASS,
    DECK_TEXT, DECK_TEXT_MID, RADIUS_BTN, RADIUS_SM,
};

/// Width of one category block-pill in the bottom toolbar cluster.
pub const CLUSTER_PILL_W: f32 = 86.0;
/// Height of the always-visible category block-pill.
pub const CLUSTER_PILL_H: f32 = 46.0;
/// Width of one sub-tool tile inside an expanded items rect.
pub const CLUSTER_ITEM_W: f32 = 74.0;
/// Height of one sub-tool tile inside an expanded items rect.
pub const CLUSTER_ITEM_H: f32 = 56.0;
/// Sub-tool tiles per row inside the expanded items rect.
pub const CLUSTER_ITEMS_PER_ROW: usize = 3;
/// Rows shown before the items rect becomes a scrollable overflow.
pub const CLUSTER_VISIBLE_ROWS: usize = 3;
/// Gap between tiles / pills.
const GAP: f32 = 6.0;

/// Outcome of drawing one category cluster: did the player click the category
/// pill (toggle open/close) and/or pick a sub-tool?
#[derive(Debug, Default, Clone, Copy)]
pub struct ClusterResponse {
    /// The category block-pill was clicked (caller toggles `open`).
    pub category_clicked: bool,
    /// A sub-tool was picked inside the expanded items rect.
    pub picked: Option<SubTool>,
}

/// Pixel height of an expanded items rect for `count` sub-tools (capped to the
/// scrollable window). Used by the caller to reserve vertical space above the
/// bottom bar before painting the flyout.
#[must_use]
pub fn expanded_items_height(count: usize) -> f32 {
    let rows = count
        .div_ceil(CLUSTER_ITEMS_PER_ROW)
        .min(CLUSTER_VISIBLE_ROWS);
    let header = 26.0;
    header + rows as f32 * (CLUSTER_ITEM_H + GAP) + GAP
}

/// Width of an expanded items rect (always [`CLUSTER_ITEMS_PER_ROW`] tiles wide
/// so the larger rect reads as a clean block above the small category rect).
#[must_use]
pub fn expanded_items_width() -> f32 {
    CLUSTER_ITEMS_PER_ROW as f32 * (CLUSTER_ITEM_W + GAP) + GAP
}

/// Draw the always-visible small **category rect** as a block-pill. Lit when its
/// flyout is open or it owns the active tool. Returns the egui response.
pub fn category_pill(
    ui: &mut egui::Ui,
    cat: &Category,
    open: bool,
    active: bool,
    icon_tex: Option<egui::TextureId>,
) -> egui::Response {
    let size = egui::vec2(CLUSTER_PILL_W, CLUSTER_PILL_H);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let lit = open || active;
    let time = ui.input(|i| i.time);
    let paint_rect = motion_rect(rect, lit, resp.hovered(), time, ui.id().value());
    let p = ui.painter();
    liquid_glass_pill(p, paint_rect, RADIUS_BTN, lit, resp.hovered());
    // Animated specular sweep on lit/hovered pills — the "wet glass" highlight
    // sliding across the blade (holocron specular-sweep). Painted under the icon.
    if lit || resp.hovered() {
        crate::ui_theme::specular_sweep(p, paint_rect, time, RADIUS_BTN);
    }
    let accent = if lit { DECK_ACCENT } else { cat.accent };
    paint_cluster_icon_label(p, paint_rect, cat.icon, cat.label, lit, accent, icon_tex);
    // Caret hints the pill expands a larger items rect upward.
    let caret = paint_rect.center_top() + egui::vec2(0.0, 3.0);
    let caret_col = if open {
        DECK_ACCENT
    } else {
        DECK_TEXT_MID.gamma_multiply(0.7)
    };
    let caret_glyph = if open { "\u{25be}" } else { "\u{25b4}" };
    p.text(
        caret,
        egui::Align2::CENTER_TOP,
        caret_glyph,
        egui::FontId::proportional(9.0),
        caret_col,
    );
    resp.on_hover_text(format!("{}  [{}]", cat.label, cat.hotkey))
}

/// Draw the larger **items rect** (the expanded list) as a frosted block-pill
/// holding a grid of sub-tool tiles, scrollable past [`CLUSTER_VISIBLE_ROWS`].
/// Returns the picked sub-tool, if any.
pub fn items_rect(ui: &mut egui::Ui, cat: &Category, current: SubTool) -> Option<SubTool> {
    let mut picked = None;
    let frame = crate::ui_theme::liquid_glass_frame(
        egui::Margin::symmetric(GAP as i8, GAP as i8),
        RADIUS_SM,
    );
    let inner = frame.show(ui, |ui| {
        ui.set_width(expanded_items_width());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{}  {}", cat.icon, cat.label))
                    .color(cat.accent)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(format!("{} tools", cat.subtools.len()))
                    .color(DECK_TEXT_MID)
                    .small(),
            );
        });
        ui.add_space(4.0);
        let max_h = CLUSTER_VISIBLE_ROWS as f32 * (CLUSTER_ITEM_H + GAP);
        egui::ScrollArea::vertical()
            .max_height(max_h)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(GAP, GAP);
                    for &st in cat.subtools {
                        if item_tile(ui, st, st == current, cat.accent).clicked() {
                            picked = Some(st);
                        }
                    }
                });
            });
    });
    // Frost the larger items rect so it matches the pills below it.
    crate::ui_theme::liquid_glass_finish(ui.painter(), inner.response.rect, RADIUS_SM);
    picked
}

/// One sub-tool tile inside an expanded items rect, lit when it is current.
fn item_tile(
    ui: &mut egui::Ui,
    st: SubTool,
    active: bool,
    accent: egui::Color32,
) -> egui::Response {
    let size = egui::vec2(CLUSTER_ITEM_W, CLUSTER_ITEM_H);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let inert = !st.is_active_capable();
    let time = ui.input(|i| i.time);
    let paint_rect = motion_rect(rect, active, resp.hovered(), time, ui.id().value());
    let p = ui.painter();
    liquid_glass_pill(p, paint_rect, RADIUS_SM, active, resp.hovered());
    let lit = active && !inert;
    let tile_accent = if active { accent } else { DECK_TEXT };
    paint_cluster_icon_label(p, paint_rect, st.icon(), st.label(), lit, tile_accent, None);
    let tip = if inert {
        format!("{} \u{2014} coming soon", st.label())
    } else {
        st.label().to_string()
    };
    resp.on_hover_text(tip)
}

/// Silence unused-import lints for tokens kept for parity with `game_ui` styling.
#[allow(dead_code)]
fn _token_anchor() -> [egui::Color32; 2] {
    [DECK_GLASS, DECK_BORDER]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expanded_height_grows_then_caps() {
        let one = expanded_items_height(1);
        let full = expanded_items_height(CLUSTER_ITEMS_PER_ROW * CLUSTER_VISIBLE_ROWS);
        let overflow = expanded_items_height(CLUSTER_ITEMS_PER_ROW * CLUSTER_VISIBLE_ROWS + 9);
        assert!(full > one, "more tools => taller rect");
        assert_eq!(full, overflow, "height caps at the scrollable window");
    }

    #[test]
    fn width_is_a_fixed_grid() {
        assert!(expanded_items_width() > CLUSTER_ITEM_W);
    }

    #[test]
    fn default_response_is_inert() {
        let r = ClusterResponse::default();
        assert!(!r.category_clicked);
        assert!(r.picked.is_none());
    }
}
