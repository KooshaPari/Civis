#![cfg(all(feature = "bevy", feature = "egui"))]

//! Cohesive AAA dark-glass theme primitives for the Civis HUD.
//!
//! Extracted from `game_ui.rs` so the palette, type scale, and reusable panel /
//! chip / card painters live in one place and can be shared across HUD modules
//! (`game_ui`, `tool_categories`, `event_feed`, `diplomacy_ui`, …). The visual
//! language draws on Cities Skylines (clean toolbars + flyout drawers), WorldBox
//! (chunky icon palette), Empire at War (command panels) and DINO.
//!
//! The look layers four ideas on top of the old flat translucent panels:
//! 1. **Depth tones** — a 4-step neutral ramp (`BG_DEEP`..`SURFACE_HI`) instead
//!    of a single fill, so panels read as stacked glass, not one flat sheet.
//! 2. **Accent system** — cyan primary + gold secondary, each with a soft glow
//!    helper for active/hover states.
//! 3. **Crisp 1px borders + inner glow** — [`accent_frame`] / [`inner_glow`].
//! 4. **Drop shadows** — [`panel_shadow`] under floating panels and flyouts.

use bevy_egui::egui;

// ---------------------------------------------------------------------------
// Palette — cohesive cyan/gold accents on a 4-step depth ramp.
// ---------------------------------------------------------------------------

/// Primary accent cyan (active widgets, highlights, primary text).
pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(86, 204, 242);
/// Brighter cyan for glow / lit edges.
pub const ACCENT_HI: egui::Color32 = egui::Color32::from_rgb(140, 224, 255);
/// Gold secondary accent (#E8B84B) for era/treasury/secondary highlights.
pub const GOLD: egui::Color32 = egui::Color32::from_rgb(232, 184, 75);
/// Friendly / positive green.
pub const GREEN: egui::Color32 = egui::Color32::from_rgb(120, 220, 130);
/// Warning / negative red.
pub const RED: egui::Color32 = egui::Color32::from_rgb(232, 100, 100);
/// Violet tertiary for disaster / magic categories.
pub const VIOLET: egui::Color32 = egui::Color32::from_rgb(178, 138, 246);

/// Deepest backdrop tone (behind everything, flyout scrims).
pub const BG_DEEP: egui::Color32 = egui::Color32::from_rgba_premultiplied(9, 11, 18, 240);
/// Base glass panel fill.
pub const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(16, 20, 30, 236);
/// Mid surface for chips / inactive buttons.
pub const SURFACE: egui::Color32 = egui::Color32::from_rgba_premultiplied(28, 34, 49, 238);
/// Raised surface for hover / nested cards.
pub const SURFACE_HI: egui::Color32 = egui::Color32::from_rgba_premultiplied(38, 46, 64, 240);
/// Deeper inset fill for list rows / inset wells.
pub const INSET_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(20, 25, 37, 238);

/// Dimmed label color for field names + secondary text.
pub const DIM: egui::Color32 = egui::Color32::from_rgb(152, 161, 182);
/// Primary near-white body text.
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(226, 232, 244);
/// Border color for inactive widgets (subtle, low-contrast).
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(58, 67, 90);
/// Faint hairline used for separators inside cards.
pub const HAIRLINE: egui::Color32 = egui::Color32::from_rgb(42, 50, 68);

/// Shared corner radius for the cohesive dark-glass look.
pub const RADIUS: u8 = 10;
/// Tighter radius for chips / small buttons.
pub const RADIUS_SM: u8 = 7;

// ---------------------------------------------------------------------------
// Theme application
// ---------------------------------------------------------------------------

/// Apply the cohesive dark-glass theme + typography to the egui context.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    let r = egui::CornerRadius::same(RADIUS);

    v.panel_fill = PANEL_FILL;
    v.window_fill = PANEL_FILL;
    v.window_corner_radius = r;
    v.window_stroke = egui::Stroke::new(1.0, BORDER);
    v.override_text_color = Some(TEXT);
    apply_widget_visuals(&mut v, r);
    v.selection.bg_fill = ACCENT.gamma_multiply(0.35);
    v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
    v.window_shadow = panel_shadow();
    v.popup_shadow = panel_shadow();
    style.visuals = v;
    apply_type_scale(&mut style);
    ctx.set_style(style);
}

/// Configure the interactive widget state visuals (inactive/hover/active).
fn apply_widget_visuals(v: &mut egui::Visuals, r: egui::CornerRadius) {
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);
    v.widgets.inactive.corner_radius = r;
    v.widgets.inactive.bg_fill = SURFACE;
    v.widgets.inactive.weak_bg_fill = SURFACE;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
    v.widgets.hovered.corner_radius = r;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, ACCENT_HI);
    v.widgets.hovered.bg_fill = SURFACE_HI;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.7));
    v.widgets.active.corner_radius = r;
    v.widgets.active.bg_fill = ACCENT.gamma_multiply(0.30);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.5, ACCENT);
}

/// Readable heading/body/small type scale + generous spacing.
pub fn apply_type_scale(style: &mut egui::Style) {
    use egui::{FontFamily::Proportional, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(19.0, Proportional)),
        (TextStyle::Body, FontId::new(14.5, Proportional)),
        (TextStyle::Button, FontId::new(14.5, Proportional)),
        (TextStyle::Small, FontId::new(11.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, egui::FontFamily::Monospace)),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(11.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
}

// ---------------------------------------------------------------------------
// Shadows + frames
// ---------------------------------------------------------------------------

/// Soft drop shadow used under floating panels / flyouts for depth.
pub fn panel_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 22,
        spread: 0,
        color: egui::Color32::from_black_alpha(135),
    }
}

/// Shared rounded glass frame for the HUD panels (subtle border).
pub fn panel_frame(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(RADIUS))
}

/// A panel frame with an accent-tinted border + drop shadow, for flyouts.
pub fn accent_frame(margin: egui::Margin, accent: egui::Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_FILL)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.7)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(panel_shadow())
}

// ---------------------------------------------------------------------------
// Reusable painters
// ---------------------------------------------------------------------------

/// Paint a soft inner glow (1px inset accent ring) inside `rect`.
pub fn inner_glow(painter: &egui::Painter, rect: egui::Rect, accent: egui::Color32, radius: u8) {
    let inset = rect.shrink(1.5);
    painter.rect_stroke(
        inset,
        radius as f32,
        egui::Stroke::new(1.0, accent.gamma_multiply(0.35)),
        egui::StrokeKind::Inside,
    );
}

/// A faint hairline section separator used inside cards.
pub fn hairline(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter().hline(rect.x_range(), y, egui::Stroke::new(1.0, HAIRLINE));
    ui.add_space(6.0);
}

/// A single rounded stat chip: `icon text` on a tinted pill.
pub fn chip(ui: &mut egui::Ui, icon: &str, text: &str, color: egui::Color32) {
    egui::Frame::NONE
        .fill(SURFACE)
        .corner_radius(egui::CornerRadius::same(RADIUS_SM))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(icon).color(color));
            ui.label(egui::RichText::new(text).color(color).strong());
        });
}

/// Format a large number compactly (`12.3K`, `4.5M`) for chips/counts.
pub fn compact(value: f64) -> String {
    let v = value.abs();
    if v >= 1.0e9 {
        format!("{:.1}B", value / 1.0e9)
    } else if v >= 1.0e6 {
        format!("{:.1}M", value / 1.0e6)
    } else if v >= 1.0e3 {
        format!("{:.1}K", value / 1.0e3)
    } else {
        format!("{:.0}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_number_formatting() {
        assert_eq!(compact(0.0), "0");
        assert_eq!(compact(950.0), "950");
        assert_eq!(compact(12_300.0), "12.3K");
        assert_eq!(compact(4_500_000.0), "4.5M");
        assert_eq!(compact(2_000_000_000.0), "2.0B");
    }

    #[test]
    fn palette_depth_ramp_is_ordered() {
        // Depth tones should brighten from deep backdrop to raised surface.
        let lum = |c: egui::Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
        assert!(lum(BG_DEEP) < lum(PANEL_FILL));
        assert!(lum(PANEL_FILL) < lum(SURFACE));
        assert!(lum(SURFACE) < lum(SURFACE_HI));
    }
}
