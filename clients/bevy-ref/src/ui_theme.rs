#![cfg(all(feature = "bevy", feature = "egui"))]

//! "Console Holo" design-language theme primitives for the Civis HUD.
//!
//! Implements `docs/design/ui-design-language.md` — the binding HUD design
//! language that supersedes the old cyan/gold "AAA dark-glass" theme (judged
//! too generic / too color-dominated). The fix is a single inversion: the
//! surface is **mostly neutral graphite glass**, and neon appears **only as
//! line/edge/glow accents on active or live elements** (≤8% of any panel's
//! area — never as a fill).
//!
//! The language fuses five exact references and ships them as five signature
//! moves:
//! 1. **Two-tier surface** — a calm graphite *Console* (chrome, ≈92%) vs. a
//!    glowing cyan *Projection* (holo, ≈8%, see [`crate::ui_holo`]).
//! 2. **Neon as edge, never fill** — accent ([`NEON`]) lives only on
//!    active/focused edges + glow.
//! 3. **Xbox-2001 2-tone bevel** — every blade gets a [`STEEL_400`] top-left
//!    highlight + [`INK_1`] bottom-right shadow, inverting on press
//!    ([`blade_frame`]).
//! 4. **Geist mono numerics** — every value/count/coord in a tabular monospace
//!    style ([`apply_type_scale`]).
//! 5. **Star Wars hologram instruments** — the inspector/minimap projections,
//!    painted by [`crate::ui_holo`] using the [`HOLO_CORE`]/[`HOLO_GLOW`] family.
//!
//! This module owns the **chrome tier** tokens (the 10-step graphite ramp,
//! [`NEON`], [`AMBER`], semantic statuses) and the blade/glass painters. The
//! holo-cyan family is defined here too so [`crate::ui_holo`] can share it, but
//! it must never appear in chrome.
//!
//! Legacy symbol names (`ACCENT`, `GOLD`, `SURFACE`, …) are kept as
//! **re-export aliases** mapped onto the new roles so the rest of the HUD
//! compiles unchanged while the per-module re-theme lands incrementally.

use bevy_egui::egui;

// ===========================================================================
// 1. Color tokens (locked hex, from §1 of the design language)
// ===========================================================================

// --- 1.1 Neutral graphite ramp — the entire chrome surface -----------------

/// Void / deepest scrim behind floating panels (`#05070A`).
pub const INK_0: egui::Color32 = egui::Color32::from_rgba_premultiplied(5, 7, 10, 244);
/// Panel bottom / blade base, viewport letterbox (`#0A0D12`) — also the B/R bevel shadow.
pub const INK_1: egui::Color32 = egui::Color32::from_rgba_premultiplied(10, 13, 18, 240);
/// **Primary panel fill** — the console face (`#0F131A`).
pub const GRAPHITE_900: egui::Color32 = egui::Color32::from_rgba_premultiplied(15, 19, 26, 236);
/// Inset wells, list rows, slider track (`#161B23`).
pub const GRAPHITE_800: egui::Color32 = egui::Color32::from_rgba_premultiplied(22, 27, 35, 236);
/// Chips / inactive buttons — mid surface (`#1E242E`).
pub const GRAPHITE_700: egui::Color32 = egui::Color32::from_rgba_premultiplied(30, 36, 46, 238);
/// Hover surface / raised nested cards (`#272E3A`).
pub const GRAPHITE_600: egui::Color32 = egui::Color32::from_rgba_premultiplied(39, 46, 58, 240);
/// Active-but-neutral surface, pressed blade (`#333C4A`).
pub const GRAPHITE_500: egui::Color32 = egui::Color32::from_rgb(51, 60, 74);
/// **Bevel highlight** — top/left lit edge (`#4A5564`).
pub const STEEL_400: egui::Color32 = egui::Color32::from_rgb(74, 85, 100);
/// Hairline borders, inactive widget stroke (`#5E6A7A`).
pub const STEEL_300: egui::Color32 = egui::Color32::from_rgb(94, 106, 122);
/// Tick marks, disabled glyphs (`#7C8898`).
pub const STEEL_200: egui::Color32 = egui::Color32::from_rgb(124, 136, 152);

// --- 1.2 Text ramp (Geist neutral, high contrast) --------------------------

/// Primary body, headings, live values (`#ECEFF4`).
pub const TEXT_HI: egui::Color32 = egui::Color32::from_rgb(236, 239, 244);
/// Secondary text, field labels (`#9AA4B2`).
pub const TEXT_MID: egui::Color32 = egui::Color32::from_rgb(154, 164, 178);
/// Captions, units, inactive tabs, hints (`#646F7E`).
pub const TEXT_LOW: egui::Color32 = egui::Color32::from_rgb(100, 111, 126);
/// Disabled text (`#3C4450`).
pub const TEXT_DISABLED: egui::Color32 = egui::Color32::from_rgb(60, 68, 80);

// --- 1.3 Neon accent — the ONE signature (electric green) ------------------

/// Active tab underline, focused-widget 1px edge, selection ring, "live" pulse (`#3DF07A`).
/// **Line / edge / glow only — never a fill, never a large tint. ≤8% of panel area.**
pub const NEON: egui::Color32 = egui::Color32::from_rgb(61, 240, 122);
/// Hottest core of a neon glow — 1px inner line (`#8BFFB4`).
pub const NEON_HI: egui::Color32 = egui::Color32::from_rgb(139, 255, 180);
/// Neon at rest / pre-glow trace lines, scanline base in chrome (`#1E7A45`).
pub const NEON_DIM: egui::Color32 = egui::Color32::from_rgb(30, 122, 69);

// --- 1.4 Warm signal — restrained amber ------------------------------------

/// Treasury/era value, "confirm/positive", positive delta arrows (`#F2B33D`).
pub const AMBER: egui::Color32 = egui::Color32::from_rgb(242, 179, 61);
/// Amber glow core (`#FFE3A0`).
pub const AMBER_HI: egui::Color32 = egui::Color32::from_rgb(255, 227, 160);

// --- 1.5 Hologram cyan family — the PROJECTION tier ONLY -------------------

/// Holo line work, wireframe, primary projected text (`#5BE3FF`).
/// Reserved for true-3D projected layers (minimap / world / inspect).
pub const HOLO_CYAN: egui::Color32 = egui::Color32::from_rgb(91, 227, 255);
/// Legacy alias → [`HOLO_CYAN`].
pub const HOLO_CORE: egui::Color32 = HOLO_CYAN;
/// Holo glow / bloom color, scanline tint (desaturated [`HOLO_CYAN`]).
pub const HOLO_GLOW: egui::Color32 = egui::Color32::from_rgb(47, 191, 230);
/// Holo panel translucent fill — very low alpha (`#0E3A4A`).
pub const HOLO_DEEP: egui::Color32 = egui::Color32::from_rgb(14, 58, 74);
/// Chromatic-aberration **red** ghost channel (`#FF3B6B`).
pub const HOLO_ABERR_R: egui::Color32 = egui::Color32::from_rgb(255, 59, 107);
/// Chromatic-aberration **blue** ghost channel (`#3B7BFF`).
pub const HOLO_ABERR_B: egui::Color32 = egui::Color32::from_rgb(59, 123, 255);

// --- 1.6 Semantic status (world/state, not chrome decoration) --------------

/// Healthy / positive — fill-safe, less saturated than [`NEON`] (`#46D67A`).
pub const OK: egui::Color32 = egui::Color32::from_rgb(70, 214, 122);
/// Caution — shares [`AMBER`] (`#F2B33D`).
pub const WARN: egui::Color32 = AMBER;
/// Alert / negative / war (`#F0556B`).
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(240, 85, 107);
/// Disaster / magic / arcane category (`#9B7BF0`).
pub const MANA: egui::Color32 = egui::Color32::from_rgb(155, 123, 240);

// --- 1.7 Holocron command-deck tokens (holohud slice) ----------------------

/// Command-deck void / viewport letterbox (`#0B0E14`).
pub const DECK_BG: egui::Color32 = egui::Color32::from_rgb(11, 14, 20);
/// Frosted glass panel fill (`#141A24` @ ~68% alpha).
pub const DECK_GLASS: egui::Color32 = egui::Color32::from_rgba_premultiplied(20, 26, 36, 175);
/// Hairline glass border (`#FFFFFF` @ ~11% alpha).
pub const DECK_BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(255, 255, 255, 28);
/// Primary chrome accent — warm amber (`#FFB347`).
pub const DECK_AMBER: egui::Color32 = egui::Color32::from_rgb(255, 179, 71);
/// Restrained neon success (`#7CF5C4`).
pub const DECK_SUCCESS: egui::Color32 = egui::Color32::from_rgb(124, 245, 196);
/// Primary body text (`#E6EBF2`).
pub const DECK_TEXT: egui::Color32 = egui::Color32::from_rgb(230, 235, 242);
/// Secondary / label text (`#8A94A6`).
pub const DECK_TEXT_MID: egui::Color32 = egui::Color32::from_rgb(138, 148, 166);

/// Spacing scale (px): 4 / 8 / 12 / 16 / 24 / 32.
pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const SPACE_XXL: f32 = 32.0;

/// Panel corner radius (10–14px band).
pub const RADIUS_PANEL: u8 = 12;
/// Button / chip corner radius.
pub const RADIUS_BTN: u8 = 8;
/// Documented backdrop-blur intent (egui has no real blur; shadow + scrim sell depth).
pub const DECK_BLUR_PX: f32 = 20.0;

// --- Corner radii -----------------------------------------------------------

/// Shared corner radius for E0/E1 glass (`8px`).
pub const RADIUS: u8 = 8;
/// Tighter radius for chips / small buttons.
pub const RADIUS_SM: u8 = 6;
/// Modal (E2) corner radius (`12px`).
pub const RADIUS_LG: u8 = 12;

// ---------------------------------------------------------------------------
// Legacy aliases — keep the old public names compiling while the per-module
// re-theme lands. Each maps onto its new role (handoff §9.1).
// ---------------------------------------------------------------------------

/// Legacy alias → [`NEON`] (was the cyan primary accent).
pub const ACCENT: egui::Color32 = NEON;
/// Legacy alias → [`NEON_HI`] (was the bright cyan glow).
pub const ACCENT_HI: egui::Color32 = NEON_HI;
/// Legacy alias → [`AMBER`] (was the gold secondary accent).
pub const GOLD: egui::Color32 = AMBER;
/// Legacy alias → [`OK`] (fill-safe positive green).
pub const GREEN: egui::Color32 = OK;
/// Legacy alias → [`DANGER`] (negative / war red).
pub const RED: egui::Color32 = DANGER;
/// Legacy alias → [`MANA`] (disaster / magic violet).
pub const VIOLET: egui::Color32 = MANA;

/// Legacy alias → [`INK_0`] (deepest backdrop / scrim).
pub const BG_DEEP: egui::Color32 = INK_0;
/// Legacy alias → [`GRAPHITE_900`] (primary panel fill).
pub const PANEL_FILL: egui::Color32 = GRAPHITE_900;
/// Legacy alias → [`GRAPHITE_700`] (mid surface for chips / inactive buttons).
pub const SURFACE: egui::Color32 = GRAPHITE_700;
/// Legacy alias → [`GRAPHITE_600`] (raised hover / nested cards).
pub const SURFACE_HI: egui::Color32 = GRAPHITE_600;
/// Legacy alias → [`GRAPHITE_800`] (inset wells / list rows).
pub const INSET_FILL: egui::Color32 = GRAPHITE_800;
/// Legacy alias → [`TEXT_MID`] (dimmed label color).
pub const DIM: egui::Color32 = TEXT_MID;
/// Legacy alias → [`TEXT_HI`] (primary near-white body text).
pub const TEXT: egui::Color32 = TEXT_HI;
/// Legacy alias → [`STEEL_300`] (inactive widget stroke).
pub const BORDER: egui::Color32 = STEEL_300;
/// Legacy alias → a faint graphite hairline for separators inside cards.
pub const HAIRLINE: egui::Color32 = egui::Color32::from_rgb(36, 42, 54);

// ===========================================================================
// Theme application
// ===========================================================================

/// Apply the Console-Holo chrome theme + Geist-style typography to the egui ctx.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    let r = egui::CornerRadius::same(RADIUS);

    v.panel_fill = GRAPHITE_900;
    v.window_fill = GRAPHITE_900;
    v.window_corner_radius = r;
    v.window_stroke = egui::Stroke::new(1.0, STEEL_300);
    v.override_text_color = Some(TEXT_HI);
    apply_widget_visuals(&mut v, r);
    // Selection: a NEON ring + a *barely-there* tint (edge, not fill).
    v.selection.bg_fill = NEON.gamma_multiply(0.12);
    v.selection.stroke = egui::Stroke::new(1.0, NEON);
    v.window_shadow = panel_shadow();
    v.popup_shadow = panel_shadow();
    style.visuals = v;
    apply_type_scale(&mut style);
    ctx.set_style(style);
}

/// Configure interactive widget visuals as Xbox-2001 blades: graphite fills,
/// neutral strokes, with **neon only on strokes** (never a `bg_fill`).
fn apply_widget_visuals(v: &mut egui::Visuals, r: egui::CornerRadius) {
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MID);

    // Rest blade: GRAPHITE_700 fill, STEEL_300 border, TEXT_MID label. No accent.
    v.widgets.inactive.corner_radius = r;
    v.widgets.inactive.bg_fill = GRAPHITE_700;
    v.widgets.inactive.weak_bg_fill = GRAPHITE_700;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, STEEL_300);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_MID);

    // Hover blade: GRAPHITE_600 fill, TEXT_HI, a NEON@0.5 leading edge cue.
    v.widgets.hovered.corner_radius = r;
    v.widgets.hovered.bg_fill = GRAPHITE_600;
    v.widgets.hovered.weak_bg_fill = GRAPHITE_600;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, NEON.gamma_multiply(0.5));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_HI);

    // Active/pressed blade: GRAPHITE_500 fill, full 1px NEON border, TEXT_HI.
    // Accent lives on the *stroke* only — never the fill.
    v.widgets.active.corner_radius = r;
    v.widgets.active.bg_fill = GRAPHITE_500;
    v.widgets.active.weak_bg_fill = GRAPHITE_500;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, NEON);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_HI);

    v.widgets.open.corner_radius = r;
    v.widgets.open.bg_fill = GRAPHITE_600;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, STEEL_300);
}

/// Geist-style type scale (§4.1): tight neutral sans, distinct tabular mono for
/// every number. Uses egui's built-in Proportional/Monospace families (Geist
/// Sans/Mono font registration is a follow-up; the scale + mono-numerics
/// discipline is what counters the generic feel).
pub fn apply_type_scale(style: &mut egui::Style) {
    use egui::{FontFamily::Monospace, FontFamily::Proportional, FontId, TextStyle};
    style.text_styles = [
        // Display 22 / Heading 16 / Body 13.5 / Label(Button) 11.5 / Caption(Small) 10.5
        (TextStyle::Heading, FontId::new(16.0, Proportional)),
        (TextStyle::Name("Display".into()), FontId::new(22.0, Proportional)),
        (TextStyle::Body, FontId::new(13.5, Proportional)),
        (TextStyle::Button, FontId::new(11.5, Proportional)),
        (TextStyle::Small, FontId::new(10.5, Proportional)),
        // All values/counts/coords route through Monospace (tabular numerics).
        (TextStyle::Monospace, FontId::new(14.0, Monospace)),
        (TextStyle::Name("NumericSm".into()), FontId::new(11.5, Monospace)),
        (TextStyle::Name("Coord".into()), FontId::new(12.0, Monospace)),
    ]
    .into();
    // Generous Geist spacing (§4.2).
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.window_margin = egui::Margin::same(14);
}

// ===========================================================================
// 2. Shadows + glass frames (E0 / E1 / E2)
// ===========================================================================

/// Command-deck rim shadow: soft outer lift (`0,4 / 20 / 0 / 100`).
pub fn deck_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 4],
        blur: DECK_BLUR_PX as u8,
        spread: 0,
        color: egui::Color32::from_black_alpha(100),
    }
}

/// **Holocron deck rim** — top bar + bottom toolbar glass chrome.
pub fn deck_rim_frame(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(DECK_GLASS)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .corner_radius(egui::CornerRadius::same(RADIUS_PANEL))
        .shadow(deck_shadow())
}

/// Deck stat chip: glass inset well + mono value with a thin accent tick.
pub fn deck_chip(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32) {
    egui::Frame::NONE
        .fill(DECK_BG.gamma_multiply(0.55))
        .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .inner_margin(egui::Margin::symmetric(SPACE_MD as i8, SPACE_XS as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(label.to_uppercase()).color(DECK_TEXT_MID).small());
                ui.label(
                    egui::RichText::new(value)
                        .monospace()
                        .color(DECK_TEXT)
                        .strong(),
                );
            });
            let r = ui.min_rect();
            ui.painter().hline(
                r.x_range(),
                r.top() + 1.0,
                egui::Stroke::new(1.5, accent.gamma_multiply(0.85)),
            );
        });
}

/// E0 docked shadow (top bar, toolbar): `0,3 / 10 / 0 / 90`.
pub fn panel_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: egui::Color32::from_black_alpha(90),
    }
}

/// E1 floating shadow (flyouts, cards, dropdowns): `0,8 / 22 / 0 / 135`.
pub fn floating_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 22,
        spread: 0,
        color: egui::Color32::from_black_alpha(135),
    }
}

/// E2 modal shadow (menus, dialogs, loading): `0,14 / 36 / 2 / 170`.
pub fn modal_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 14],
        blur: 36,
        spread: 2,
        color: egui::Color32::from_black_alpha(170),
    }
}

/// **E0 — docked** graphite glass frame (top bar, toolbar, anchored panels).
pub fn frame_e0(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.6)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(panel_shadow())
}

/// **E1 — floating** graphite glass frame (flyouts, cards, dropdowns). Carries
/// a faint `NEON_DIM` inner trace; full accent inner-glow only enters on focus
/// via [`inner_glow`].
pub fn frame_e1(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.7)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(floating_shadow())
}

/// **E2 — modal** graphite glass frame (menus, dialogs, loading) over an
/// [`INK_0`] scrim painted by [`scrim`].
pub fn frame_e2(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(INK_1)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.8)))
        .corner_radius(egui::CornerRadius::same(RADIUS_LG))
        .shadow(modal_shadow())
}

/// Legacy alias → [`frame_e0`] (docked graphite glass).
pub fn panel_frame(margin: egui::Margin) -> egui::Frame {
    frame_e0(margin)
}

/// Legacy `accent_frame` → an E1 floating glass with an accent-tinted border.
/// The accent now lives on the **border stroke** (focus cue), not a fill — so
/// existing flyout callers keep their accent edge without an accent area.
pub fn accent_frame(margin: egui::Margin, accent: egui::Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.55)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(floating_shadow())
}

// ===========================================================================
// Reusable painters
// ===========================================================================

/// Paint a full-rect [`INK_0`] scrim (§2.1) behind floating/modal panels — the
/// cheap stand-in for a backdrop blur that separates the panel from the busy
/// 3D world. `alpha` is `0..=255` (spec uses ~180 for E2).
pub fn scrim(painter: &egui::Painter, rect: egui::Rect, alpha: u8) {
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(5, 7, 10, alpha),
    );
}

/// Paint the Xbox-2001 **2-tone bevel** on `rect`: a 1px [`STEEL_400`] highlight
/// on the top + left edges and a 1px [`INK_1`] shadow on the bottom + right.
/// When `pressed` is true the bevel **inverts** (shadow on top/left) — the
/// pressed-in cue. This is the single trick that makes chrome read "extruded
/// tech panel" instead of "flat rounded rectangle" (§5, signature move 3).
pub fn blade_frame(painter: &egui::Painter, rect: egui::Rect, pressed: bool) {
    let (hi, lo) = if pressed {
        (INK_1, STEEL_400)
    } else {
        (STEEL_400, INK_1)
    };
    let hi_stroke = egui::Stroke::new(1.0, hi);
    let lo_stroke = egui::Stroke::new(1.0, lo);
    let (tl, tr, bl, br) = (
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    );
    // Top + left = highlight (or shadow when pressed).
    painter.line_segment([tl, tr], hi_stroke);
    painter.line_segment([tl, bl], hi_stroke);
    // Bottom + right = shadow (or highlight when pressed).
    painter.line_segment([bl, br], lo_stroke);
    painter.line_segment([tr, br], lo_stroke);
}

/// A 1px horizontal **top sheen line** at the panel's inner top edge —
/// `STEEL_400 @ 0.18`, the "light catching the glass" cue (§2.1).
pub fn top_sheen(painter: &egui::Painter, rect: egui::Rect) {
    let y = rect.top() + 1.0;
    painter.line_segment(
        [egui::pos2(rect.left() + 2.0, y), egui::pos2(rect.right() - 2.0, y)],
        egui::Stroke::new(1.0, STEEL_400.gamma_multiply(0.18)),
    );
}

/// Paint a soft inner glow (1.5px-inset 1px accent ring) inside `rect` (§2.1).
/// In chrome this is `STEEL_400 @ 0.25` at rest; pass [`NEON`]/[`AMBER`] @ ~0.35
/// for a focused/active panel — the only place accent enters a resting panel.
pub fn inner_glow(painter: &egui::Painter, rect: egui::Rect, accent: egui::Color32, radius: u8) {
    let inset = rect.shrink(1.5);
    painter.rect_stroke(
        inset,
        radius as f32,
        egui::Stroke::new(1.0, accent.gamma_multiply(0.35)),
        egui::StrokeKind::Inside,
    );
}

/// A faint hairline section separator used inside cards (`STEEL_300 @ 0.4`).
pub fn hairline(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter()
        .hline(rect.x_range(), y, egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.4)));
    ui.add_space(8.0);
}

/// A single stat chip (§5.4): `[glyph]  LABEL  value`. The glyph carries the
/// only color; `LABEL` is uppercased [`TEXT_LOW`]; the **value is mono
/// [`TEXT_HI`]**. The chip itself never glows. `color` tints the glyph.
///
/// Back-compat: the legacy call is `chip(ui, icon, text, color)` where `text`
/// is the value; we render the value in mono and skip a separate label when the
/// caller did not provide one (keeps every existing call-site working).
pub fn chip(ui: &mut egui::Ui, icon: &str, text: &str, color: egui::Color32) {
    egui::Frame::NONE
        .fill(GRAPHITE_700)
        .corner_radius(egui::CornerRadius::same(RADIUS_SM))
        .stroke(egui::Stroke::new(1.0, STEEL_300))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            if !icon.is_empty() {
                ui.label(egui::RichText::new(icon).color(color));
            }
            // Value in mono TEXT_HI — the precision-console numeric read.
            ui.label(egui::RichText::new(text).monospace().color(TEXT_HI));
        });
}

/// A labelled stat chip (§5.4 full layout): `[glyph] LABEL value`. The glyph is
/// `color`, the uppercase label is [`TEXT_LOW`], the value is mono [`TEXT_HI`]
/// (or `value_color` when semantically hot, e.g. [`AMBER`] for treasury).
pub fn chip_labeled(
    ui: &mut egui::Ui,
    glyph: &str,
    label: &str,
    value: &str,
    color: egui::Color32,
    value_color: egui::Color32,
) {
    egui::Frame::NONE
        .fill(GRAPHITE_700)
        .corner_radius(egui::CornerRadius::same(RADIUS_SM))
        .stroke(egui::Stroke::new(1.0, STEEL_300))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            if !glyph.is_empty() {
                ui.label(egui::RichText::new(glyph).color(color));
            }
            ui.label(
                egui::RichText::new(label.to_uppercase())
                    .color(TEXT_LOW)
                    .small(),
            );
            ui.label(egui::RichText::new(value).monospace().color(value_color));
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

    /// The 10-step graphite ramp must brighten monotonically from the void
    /// scrim up to the lit bevel highlight (design language §1.1 ordering).
    #[test]
    fn graphite_ramp_is_ordered() {
        let lum = |c: egui::Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
        assert!(lum(INK_0) < lum(INK_1));
        assert!(lum(INK_1) < lum(GRAPHITE_900));
        assert!(lum(GRAPHITE_900) < lum(GRAPHITE_800));
        assert!(lum(GRAPHITE_800) < lum(GRAPHITE_700));
        assert!(lum(GRAPHITE_700) < lum(GRAPHITE_600));
        assert!(lum(GRAPHITE_600) < lum(GRAPHITE_500));
        assert!(lum(GRAPHITE_500) < lum(STEEL_400));
        assert!(lum(STEEL_400) < lum(STEEL_300));
        assert!(lum(STEEL_300) < lum(STEEL_200));
    }

    /// Neon and amber are accents — they must NEVER be a widget `bg_fill`.
    /// Every interactive `bg_fill` must come from the neutral graphite ramp.
    #[test]
    fn no_accent_used_as_widget_bg_fill() {
        let mut v = egui::Visuals::dark();
        apply_widget_visuals(&mut v, egui::CornerRadius::same(RADIUS));
        let graphite = [
            GRAPHITE_500,
            GRAPHITE_600,
            GRAPHITE_700,
            GRAPHITE_800,
            GRAPHITE_900,
        ];
        for fill in [
            v.widgets.inactive.bg_fill,
            v.widgets.hovered.bg_fill,
            v.widgets.active.bg_fill,
            v.widgets.open.bg_fill,
        ] {
            assert_ne!(fill, NEON, "neon must not be a widget bg_fill");
            assert_ne!(fill, AMBER, "amber must not be a widget bg_fill");
            assert!(
                graphite.contains(&fill),
                "widget bg_fill must be a graphite ramp tone, got {fill:?}"
            );
        }
        // Accent is allowed on the active *stroke* (edge), which is the point.
        assert_eq!(v.widgets.active.bg_stroke.color, NEON);
    }

    /// Legacy aliases must still resolve onto the new roles so the rest of the
    /// HUD compiles unchanged during the incremental re-theme.
    #[test]
    fn legacy_aliases_map_to_new_roles() {
        assert_eq!(ACCENT, NEON);
        assert_eq!(GOLD, AMBER);
        assert_eq!(PANEL_FILL, GRAPHITE_900);
        assert_eq!(SURFACE, GRAPHITE_700);
        assert_eq!(SURFACE_HI, GRAPHITE_600);
        assert_eq!(INSET_FILL, GRAPHITE_800);
        assert_eq!(TEXT, TEXT_HI);
        assert_eq!(BG_DEEP, INK_0);
    }

    /// Holocron deck tokens match the holohud slice spec.
    #[test]
    fn deck_tokens_match_holohud_spec() {
        assert_eq!(HOLO_CORE, HOLO_CYAN);
        assert_eq!(HOLO_CYAN, egui::Color32::from_rgb(91, 227, 255));
        assert_eq!(DECK_AMBER, egui::Color32::from_rgb(255, 179, 71));
        assert_eq!(DECK_SUCCESS, egui::Color32::from_rgb(124, 245, 196));
        assert_eq!(DECK_BG, egui::Color32::from_rgb(11, 14, 20));
        assert!((10..=14).contains(&RADIUS_PANEL));
        assert_eq!(RADIUS_BTN, 8);
    }
}
