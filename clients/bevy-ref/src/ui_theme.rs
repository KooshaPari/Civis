#![cfg(all(feature = "bevy", feature = "egui"))]

//! Keycap Palette + holocron dimensional chrome for the Civis HUD.
//!
//! Canonical tokens: Phenotype `keycap-palette` (`#7ebab5` accent, midnight
//! surfaces `#090a0c`–`#1a1e24`, WCAG-AA text ramp). Typography: Montserrat
//! (body/UI), Bricolage Grotesque (display/headings), JetBrains Mono (numeric).
//!
//! **Neon-as-signal:** teal lives on edges/glows/active strokes only — never
//! large fills. Cheap dimensional pass (egui): inner bevel, top-45% gloss,
//! colored rim glow, ease-out motion + scale-on-select + micro-jitter.
//!
//! TODO(holocron-3d-phase): tilt panels into perspective 3D quads; WGSL specular
//! sweep + fresnel rim; moving curved-perspective background + radial plasma.

use bevy::log::warn;
use bevy_egui::egui;
use egui::{FontData, FontFamily, FontId, TextStyle};

// ===========================================================================
// Keycap Palette — locked sRGB (dark mode primary)
// ===========================================================================

/// Primary teal accent (`#7ebab5`).
pub const KC_ACCENT: egui::Color32 = egui::Color32::from_rgb(126, 186, 181);
/// Accent hover (`#95ccc8`).
pub const KC_ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(149, 204, 200);
/// Accent active/pressed (`#6aa8a3`).
pub const KC_ACCENT_ACTIVE: egui::Color32 = egui::Color32::from_rgb(106, 168, 163);
/// Accent dim / trace (`#569691`).
pub const KC_ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(86, 150, 145);
/// Secondary surface / slate (`#353a40`).
pub const KC_SLATE: egui::Color32 = egui::Color32::from_rgb(53, 58, 64);

/// Deepest midnight background (`#090a0c`).
pub const KC_BG: egui::Color32 = egui::Color32::from_rgb(9, 10, 12);
/// Alt background (`#0e1014`).
pub const KC_BG_ALT: egui::Color32 = egui::Color32::from_rgb(14, 16, 20);
/// Soft surface (`#14171b`).
pub const KC_BG_SOFT: egui::Color32 = egui::Color32::from_rgb(20, 23, 27);
/// Elevated surface / glass base (`#1a1e24`).
pub const KC_BG_ELV: egui::Color32 = egui::Color32::from_rgb(26, 30, 36);
/// Primary text (`#f6f5f5`).
pub const KC_TEXT_1: egui::Color32 = egui::Color32::from_rgb(246, 245, 245);
/// Secondary text (`#a8adb5`).
pub const KC_TEXT_2: egui::Color32 = egui::Color32::from_rgb(168, 173, 181);
/// Muted text (`#6b7280`).
pub const KC_TEXT_3: egui::Color32 = egui::Color32::from_rgb(107, 114, 128);
/// Divider / hairline (`#1f2329`).
pub const KC_DIVIDER: egui::Color32 = egui::Color32::from_rgb(31, 35, 41);
/// Code / numeric well (`#060708`).
pub const KC_CODE_BG: egui::Color32 = egui::Color32::from_rgb(6, 7, 8);

// --- Surface ramp (aliases onto Keycap dark tokens) -------------------------

/// Void scrim behind modals (`KC_BG` @ high alpha).
pub const INK_0: egui::Color32 = egui::Color32::from_rgba_premultiplied(9, 10, 12, 244);
/// Blade shadow / letterbox (`KC_BG_ALT`).
pub const INK_1: egui::Color32 = egui::Color32::from_rgba_premultiplied(14, 16, 20, 240);
/// Primary panel fill (`KC_BG_ELV`).
pub const GRAPHITE_900: egui::Color32 = egui::Color32::from_rgba_premultiplied(26, 30, 36, 236);
/// Inset wells (`KC_BG_SOFT`).
pub const GRAPHITE_800: egui::Color32 = egui::Color32::from_rgba_premultiplied(20, 23, 27, 236);
/// Chips / inactive buttons (`KC_SLATE`).
pub const GRAPHITE_700: egui::Color32 = egui::Color32::from_rgba_premultiplied(53, 58, 64, 238);
/// Hover raised surface (between soft and slate).
pub const GRAPHITE_600: egui::Color32 = egui::Color32::from_rgba_premultiplied(42, 47, 54, 240);
/// Pressed blade.
pub const GRAPHITE_500: egui::Color32 = egui::Color32::from_rgb(36, 41, 48);
/// Bevel highlight (lifted edge).
pub const STEEL_400: egui::Color32 = egui::Color32::from_rgb(72, 78, 88);
/// Inactive widget stroke.
pub const STEEL_300: egui::Color32 = egui::Color32::from_rgb(58, 64, 74);
/// Tick / disabled chrome.
pub const STEEL_200: egui::Color32 = egui::Color32::from_rgb(90, 96, 106);

// --- Text -------------------------------------------------------------------

pub const TEXT_HI: egui::Color32 = KC_TEXT_1;
pub const TEXT_MID: egui::Color32 = KC_TEXT_2;
pub const TEXT_LOW: egui::Color32 = KC_TEXT_3;
pub const TEXT_DISABLED: egui::Color32 = egui::Color32::from_rgb(70, 76, 86);

// --- Active signal (teal edge/glow only) ------------------------------------

pub const NEON: egui::Color32 = KC_ACCENT;
pub const NEON_HI: egui::Color32 = KC_ACCENT_HOVER;
pub const NEON_DIM: egui::Color32 = KC_ACCENT_DIM;

/// Warm semantic gold (treasury / caution) — not the chrome accent.
pub const AMBER: egui::Color32 = egui::Color32::from_rgb(242, 179, 61);
pub const AMBER_HI: egui::Color32 = egui::Color32::from_rgb(255, 227, 160);

// --- Holo projection tier (teal-tinted, not chrome fill) --------------------

pub const HOLO_CYAN: egui::Color32 = KC_ACCENT_HOVER;
pub const HOLO_CORE: egui::Color32 = HOLO_CYAN;
pub const HOLO_GLOW: egui::Color32 = KC_ACCENT_DIM;
pub const HOLO_DEEP: egui::Color32 = egui::Color32::from_rgb(14, 40, 44);
pub const HOLO_ABERR_R: egui::Color32 = egui::Color32::from_rgb(255, 59, 107);
pub const HOLO_ABERR_B: egui::Color32 = egui::Color32::from_rgb(59, 123, 255);

// --- Semantic status --------------------------------------------------------

pub const OK: egui::Color32 = egui::Color32::from_rgb(90, 200, 140);
pub const WARN: egui::Color32 = AMBER;
pub const DANGER: egui::Color32 = egui::Color32::from_rgb(240, 85, 107);
pub const MANA: egui::Color32 = egui::Color32::from_rgb(155, 123, 240);

// --- Holocron deck (Keycap midnight glass) ------------------------------------

pub const DECK_BG: egui::Color32 = KC_BG;
/// Frosted glass (`KC_BG_ELV` @ ~86% alpha).
pub const DECK_GLASS: egui::Color32 = egui::Color32::from_rgba_premultiplied(26, 30, 36, 220);
/// Hairline on glass (`KC_DIVIDER` @ stronger contrast).
pub const DECK_BORDER: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 35, 41, 195);
/// Primary chrome accent — Keycap teal (was holocron amber).
pub const DECK_ACCENT: egui::Color32 = KC_ACCENT;
/// Back-compat alias → [`DECK_ACCENT`].
pub const DECK_AMBER: egui::Color32 = DECK_ACCENT;
pub const DECK_SUCCESS: egui::Color32 = OK;
pub const DECK_TEXT: egui::Color32 = KC_TEXT_1;
pub const DECK_TEXT_MID: egui::Color32 = KC_TEXT_2;

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const SPACE_XXL: f32 = 32.0;

pub const RADIUS_PANEL: u8 = 12;
pub const RADIUS_BTN: u8 = 8;
pub const DECK_BLUR_PX: f32 = 20.0;

pub const RADIUS: u8 = 8;
pub const RADIUS_SM: u8 = 6;
pub const RADIUS_LG: u8 = 12;

pub const ACCENT: egui::Color32 = NEON;
pub const ACCENT_HI: egui::Color32 = NEON_HI;
pub const GOLD: egui::Color32 = AMBER;
pub const GREEN: egui::Color32 = OK;
pub const RED: egui::Color32 = DANGER;
pub const VIOLET: egui::Color32 = MANA;
pub const BG_DEEP: egui::Color32 = INK_0;
pub const PANEL_FILL: egui::Color32 = GRAPHITE_900;
pub const SURFACE: egui::Color32 = GRAPHITE_700;
pub const SURFACE_HI: egui::Color32 = GRAPHITE_600;
pub const INSET_FILL: egui::Color32 = GRAPHITE_800;
pub const DIM: egui::Color32 = TEXT_MID;
pub const TEXT: egui::Color32 = TEXT_HI;
pub const BORDER: egui::Color32 = STEEL_300;
pub const HAIRLINE: egui::Color32 = KC_DIVIDER;

/// Console ease-out: `cubic-bezier(0.16, 1, 0.3, 1)` — never linear HUD motion.
#[must_use]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// XMB-style scale: selected/hovered grow; idle slots shrink slightly.
#[must_use]
pub fn selection_scale(selected: bool, hovered: bool) -> f32 {
    if selected {
        1.04
    } else if hovered {
        1.02
    } else {
        0.97
    }
}

/// Sub-pixel jitter so motion reads "console" not "web slider".
#[must_use]
pub fn micro_jitter(time: f64, id: u64) -> egui::Vec2 {
    let t = time as f32 * 14.0 + id as f32 * 0.37;
    let amp = ease_out_cubic(((t * 0.25).fract() * 4.0).min(1.0));
    egui::vec2(t.sin() * 0.35 * amp, (t * 1.3).cos() * 0.28 * amp)
}

/// Id marking that Keycap fonts were installed (egui::Id::new is not const).
fn font_installed_id() -> egui::Id {
    egui::Id::new("kc_fonts_installed")
}

fn bricolage_loaded_id() -> egui::Id {
    egui::Id::new("kc_bricolage_loaded")
}

/// Safe display FontId: uses the Bricolage display family ONLY if it is bound
/// in this context's fonts, otherwise falls back to Proportional. Referencing
/// an unbound `FontFamily::Name` panics egui ("not bound to any fonts"), so all
/// display-font use sites MUST go through this helper.
#[must_use]
pub fn display_font(ctx: &egui::Context, size: f32) -> FontId {
    let bricolage_family_bound = ctx.fonts(|f| {
        f.families()
            .iter()
            .any(|fam| *fam == FontFamily::Name("bricolage".into()))
    });

    let bricolage_loaded = ctx
        .data(|d| d.get_temp::<bool>(bricolage_loaded_id()))
        .unwrap_or(false);

    if bricolage_family_bound && bricolage_loaded {
        FontId::new(size, FontFamily::Name("bricolage".into()))
    } else {
        FontId::new(size, FontFamily::Proportional)
    }
}

/// Register Montserrat / Bricolage / JetBrains from `assets/fonts/` (once).
pub fn install_keycap_fonts(ctx: &egui::Context) {
    let flag = font_installed_id();
    if ctx.data(|d| d.get_temp::<bool>(flag).unwrap_or(false)) {
        return;
    }
    let mut fonts = egui::FontDefinitions::default();
    let any_loaded = try_load_font_files(&mut fonts);
    if any_loaded {
        let bricolage_bound = wire_font_families(&mut fonts);
        ctx.set_fonts(fonts);
        ctx.data_mut(|d| d.insert_temp(bricolage_loaded_id(), bricolage_bound));
    } else {
        ctx.data_mut(|d| d.insert_temp(bricolage_loaded_id(), false));
        warn!("No Bevy HUD fonts loaded from {:?}", font_assets_dir());
    }
    ctx.data_mut(|d| d.insert_temp(flag, true));
}

fn try_load_font_files(fonts: &mut egui::FontDefinitions) -> bool {
    let base = font_assets_dir();
    let entries = [
        ("montserrat", "Montserrat-Regular.ttf"),
        ("montserrat-bold", "Montserrat-SemiBold.ttf"),
        ("jetbrains", "JetBrainsMono-Regular.ttf"),
        ("bricolage", "BricolageGrotesque-SemiBold.ttf"),
    ];
    let mut ok = false;
    for (name, file) in entries {
        let path = base.join(file);
        match std::fs::read(&path) {
            Ok(bytes) => {
                fonts.font_data.insert(
                    name.into(),
                    std::sync::Arc::new(FontData::from_owned(bytes)),
                );
                ok = true;
            }
            Err(err) => {
                warn!("Failed to load HUD font {}: {}", file, err);
            }
        }
    }
    ok
}

fn font_assets_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts")
}

fn wire_font_families(fonts: &mut egui::FontDefinitions) -> bool {
    // CRITICAL: only bind a family to a font key that was actually loaded into
    // `font_data`. Binding FontFamily::Name to a missing key panics egui on the
    // first text render ("not bound to any fonts"). A custom Name family with
    // no valid font ALSO panics if referenced.
    let has = |k: &str| fonts.font_data.contains_key(k);

    if has("montserrat") {
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "montserrat".into());
    }
    if has("jetbrains") {
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "jetbrains".into());
    }

    let mut bricolage_bound = false;
    if has("bricolage") {
        fonts.families.insert(
            FontFamily::Name("bricolage".into()),
            vec!["bricolage".into()],
        );
        bricolage_bound = true;
    }

    if has("montserrat-bold") {
        let mut bold_stack: Vec<String> = vec!["montserrat-bold".into()];
        if has("montserrat") {
            bold_stack.push("montserrat".into());
        }
        fonts
            .families
            .insert(FontFamily::Name("montserrat-bold".into()), bold_stack);
    }

    bricolage_bound
}

/// Apply Keycap chrome theme + typography to the egui context.
pub fn apply_theme(ctx: &egui::Context) {
    install_keycap_fonts(ctx);
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    let r = egui::CornerRadius::same(RADIUS);
    v.panel_fill = GRAPHITE_900;
    v.window_fill = GRAPHITE_900;
    v.window_corner_radius = r;
    v.window_stroke = egui::Stroke::new(1.0, STEEL_300);
    v.override_text_color = Some(TEXT_HI);
    apply_widget_visuals(&mut v, r);
    v.selection.bg_fill = NEON.gamma_multiply(0.12);
    v.selection.stroke = egui::Stroke::new(1.0, NEON);
    v.window_shadow = panel_shadow();
    v.popup_shadow = panel_shadow();
    style.visuals = v;
    apply_type_scale(&mut style);
    ctx.set_style(style);
}

fn apply_widget_visuals(v: &mut egui::Visuals, r: egui::CornerRadius) {
    v.widgets.noninteractive.corner_radius = r;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_MID);
    v.widgets.inactive.corner_radius = r;
    v.widgets.inactive.bg_fill = GRAPHITE_700;
    v.widgets.inactive.weak_bg_fill = GRAPHITE_700;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, STEEL_300);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_MID);
    v.widgets.hovered.corner_radius = r;
    v.widgets.hovered.bg_fill = GRAPHITE_600;
    v.widgets.hovered.weak_bg_fill = GRAPHITE_600;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, NEON.gamma_multiply(0.5));
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_HI);
    v.widgets.active.corner_radius = r;
    v.widgets.active.bg_fill = GRAPHITE_500;
    v.widgets.active.weak_bg_fill = GRAPHITE_500;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, NEON);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_HI);
    v.widgets.open.corner_radius = r;
    v.widgets.open.bg_fill = GRAPHITE_600;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, STEEL_300);
}

pub fn apply_type_scale(style: &mut egui::Style) {
    use FontFamily::{Monospace, Proportional};
    // Headings use Proportional (Montserrat once loaded) — NOT a custom Name
    // family. Referencing `FontFamily::Name("bricolage")` in a TextStyle panics
    // egui if the family is not bound yet at render time (font load is async /
    // may fail). Deliberate Bricolage display use goes through `display_font()`,
    // which checks the binding per-frame. This keeps the HUD crash-proof.
    style.text_styles = [
        (TextStyle::Heading, FontId::new(16.0, Proportional)),
        (
            TextStyle::Name("Display".into()),
            FontId::new(22.0, Proportional),
        ),
        (TextStyle::Body, FontId::new(13.5, Proportional)),
        (TextStyle::Button, FontId::new(11.5, Proportional)),
        (TextStyle::Small, FontId::new(10.5, Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, Monospace)),
        (
            TextStyle::Name("NumericSm".into()),
            FontId::new(11.5, Monospace),
        ),
        (
            TextStyle::Name("Coord".into()),
            FontId::new(12.0, Monospace),
        ),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.window_margin = egui::Margin::same(14);
}

pub fn deck_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 4],
        blur: DECK_BLUR_PX as u8,
        spread: 0,
        color: egui::Color32::from_black_alpha(100),
    }
}

pub fn deck_rim_frame(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(DECK_GLASS)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .corner_radius(egui::CornerRadius::same(RADIUS_PANEL))
        .shadow(deck_shadow())
}

pub fn deck_chip(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32) {
    egui::Frame::NONE
        .fill(DECK_BG.gamma_multiply(0.55))
        .corner_radius(egui::CornerRadius::same(RADIUS_BTN))
        .stroke(egui::Stroke::new(1.0, DECK_BORDER))
        .inner_margin(egui::Margin::symmetric(SPACE_MD as i8, SPACE_XS as i8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label.to_uppercase())
                        .color(DECK_TEXT_MID)
                        .small(),
                );
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

pub fn panel_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 3],
        blur: 10,
        spread: 0,
        color: egui::Color32::from_black_alpha(90),
    }
}

pub fn floating_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 8],
        blur: 22,
        spread: 0,
        color: egui::Color32::from_black_alpha(135),
    }
}

pub fn modal_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 14],
        blur: 36,
        spread: 2,
        color: egui::Color32::from_black_alpha(170),
    }
}

pub fn frame_e0(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.6)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(panel_shadow())
}

pub fn frame_e1(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.7)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(floating_shadow())
}

pub fn frame_e2(margin: egui::Margin) -> egui::Frame {
    egui::Frame::NONE
        .fill(INK_1)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, STEEL_300.gamma_multiply(0.8)))
        .corner_radius(egui::CornerRadius::same(RADIUS_LG))
        .shadow(modal_shadow())
}

pub fn panel_frame(margin: egui::Margin) -> egui::Frame {
    frame_e0(margin)
}

pub fn accent_frame(margin: egui::Margin, accent: egui::Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(GRAPHITE_900)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, accent.gamma_multiply(0.55)))
        .corner_radius(egui::CornerRadius::same(RADIUS))
        .shadow(floating_shadow())
}

pub fn scrim(painter: &egui::Painter, rect: egui::Rect, alpha: u8) {
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(9, 10, 12, alpha),
    );
}

/// Xbox-style 2-tone inset bevel (Keycap divider + steel highlight).
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
    painter.line_segment([tl, tr], hi_stroke);
    painter.line_segment([tl, bl], hi_stroke);
    painter.line_segment([bl, br], lo_stroke);
    painter.line_segment([tr, br], lo_stroke);
}

/// Top ~45% gloss gradient (white 35% → transparent) — holocron "wet glass".
pub fn gloss_sheen(painter: &egui::Painter, rect: egui::Rect) {
    let h = rect.height() * 0.45;
    if h < 1.0 {
        return;
    }
    let gloss = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), rect.top() + h));
    let mut mesh = egui::Mesh::default();
    let c_top = egui::Color32::from_white_alpha(110);
    let c_bot = egui::Color32::TRANSPARENT;
    let i = mesh.vertices.len() as u32;
    mesh.colored_vertex(gloss.left_top(), c_top);
    mesh.colored_vertex(gloss.right_top(), c_top);
    mesh.colored_vertex(gloss.right_bottom(), c_bot);
    mesh.colored_vertex(gloss.left_bottom(), c_bot);
    mesh.add_triangle(i, i + 1, i + 2);
    mesh.add_triangle(i, i + 2, i + 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// Colored outer rim bloom (teal) — not a white halo.
pub fn rim_glow(painter: &egui::Painter, rect: egui::Rect, accent: egui::Color32, radius: u8) {
    for (expand, alpha) in [(4.0, 0.45_f32), (9.0, 0.22)] {
        painter.rect_stroke(
            rect.expand(expand),
            radius as f32 + expand * 0.25,
            egui::Stroke::new(1.5, accent.gamma_multiply(alpha)),
            egui::StrokeKind::Outside,
        );
    }
}

/// Cheap dimensional finish: bevel + gloss; optional teal rim when focused.
pub fn panel_finish(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: u8,
    pressed: bool,
    focused: bool,
) {
    blade_frame(painter, rect, pressed);
    gloss_sheen(painter, rect);
    if focused {
        rim_glow(painter, rect, KC_ACCENT, radius);
        inner_glow(painter, rect, KC_ACCENT, radius);
    }
}

/// Legacy top sheen → full dimensional finish at rest.
pub fn top_sheen(painter: &egui::Painter, rect: egui::Rect) {
    panel_finish(painter, rect, RADIUS_PANEL, false, false);
}

pub fn inner_glow(painter: &egui::Painter, rect: egui::Rect, accent: egui::Color32, radius: u8) {
    let inset = rect.shrink(1.5);
    painter.rect_stroke(
        inset,
        radius as f32,
        egui::Stroke::new(1.0, accent.gamma_multiply(0.35)),
        egui::StrokeKind::Inside,
    );
}

pub fn hairline(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let y = ui.cursor().top();
    ui.painter().hline(
        rect.x_range(),
        y,
        egui::Stroke::new(1.0, KC_DIVIDER.gamma_multiply(0.85)),
    );
    ui.add_space(8.0);
}

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
            ui.label(egui::RichText::new(text).monospace().color(TEXT_HI));
        });
}

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

/// Scale + jitter a widget rect for console motion (call before painting).
#[must_use]
pub fn motion_rect(
    rect: egui::Rect,
    selected: bool,
    hovered: bool,
    time: f64,
    id: u64,
) -> egui::Rect {
    let scale = selection_scale(selected, hovered);
    let center = rect.center();
    let size = rect.size() * scale;
    let mut out = egui::Rect::from_center_size(center, size);
    if hovered {
        out = out.translate(micro_jitter(time, id));
    }
    out
}

/// Frosted-glass panel fill: translucent enough that the 3D scene clearly
/// reads through the panel (true Mica/Liquid-Glass), lighter than the opaque
/// graphite panels so layered depth shows. Alpha kept low on purpose.
pub const GLASS_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(22, 26, 32, 222);
/// Thin, dark rim edge that reads as midnight glass.
pub const GLASS_EDGE: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 35, 41, 180);

/// Frosted Liquid Glass frame for decks, sidebars, and pill shells.
///
/// Use [`liquid_glass_finish`] *after* drawing the panel content (it paints the
/// gloss sheen, soft inner glow, and colored teal rim that make the panel read
/// as frosted glass rather than a flat fill).
pub fn liquid_glass_frame(margin: egui::Margin, radius: u8) -> egui::Frame {
    egui::Frame::NONE
        .fill(GLASS_FILL)
        .inner_margin(margin)
        .stroke(egui::Stroke::new(1.0, GLASS_EDGE))
        .corner_radius(egui::CornerRadius::same(radius))
        .shadow(floating_shadow())
}

/// Bake the dimensional frosted-glass read onto a panel rect drawn with
/// [`liquid_glass_frame`]: top gloss sheen, layered soft inner glow, a thin
/// light inner highlight, and a subtle colored teal rim. Call once with the
/// panel's `ui.min_rect()` after the content is laid out.
pub fn liquid_glass_finish(painter: &egui::Painter, rect: egui::Rect, radius: u8) {
    gloss_sheen(painter, rect);
    soft_inner_glow(painter, rect, KC_ACCENT, radius);
    // Thin light inner highlight (the lifted glass edge) + a darker lower bevel.
    painter.rect_stroke(
        rect.shrink(1.0),
        radius as f32,
        egui::Stroke::new(1.2, egui::Color32::from_white_alpha(38)),
        egui::StrokeKind::Inside,
    );
    // Colored TEAL rim glow (2-pass, not white) so the panel reads as a lit holo
    // blade — the holocron "colored-glow-not-white" rule.
    rim_glow(painter, rect, KC_ACCENT, radius);
}

/// Draw a block-style Liquid Glass pill (fill + rim + sheen + optional accent bloom).
pub fn liquid_glass_pill(
    painter: &egui::Painter,
    rect: egui::Rect,
    radius: u8,
    lit: bool,
    hovered: bool,
) {
    let fill = if lit {
        GLASS_FILL.gamma_multiply(1.22)
    } else if hovered {
        GLASS_FILL.gamma_multiply(1.12)
    } else {
        GLASS_FILL
    };
    painter.rect_filled(rect, radius as f32, fill);
    painter.rect_stroke(
        rect,
        radius as f32,
        egui::Stroke::new(1.0, if lit { KC_ACCENT } else { GLASS_EDGE }),
        egui::StrokeKind::Outside,
    );
    gloss_sheen(painter, rect);
    let inner = rect.shrink(1.5);
    painter.rect_stroke(
        inner,
        radius as f32,
        egui::Stroke::new(1.5, egui::Color32::from_white_alpha(26)),
        egui::StrokeKind::Inside,
    );
    if lit {
        rim_glow(painter, rect, KC_ACCENT, radius);
        inner_glow(painter, rect, KC_ACCENT, radius);
    }
}

pub fn panel_glass_fill(hovered: bool, pressed: bool) -> egui::Color32 {
    if pressed {
        DECK_GLASS.gamma_multiply(1.14)
    } else if hovered {
        DECK_GLASS.gamma_multiply(1.07)
    } else {
        DECK_GLASS
    }
}

pub fn panel_edge_stroke(hovered: bool, focused: bool) -> egui::Stroke {
    if focused {
        egui::Stroke::new(1.2, DECK_ACCENT.gamma_multiply(0.72))
    } else if hovered {
        egui::Stroke::new(1.0, DECK_ACCENT.gamma_multiply(0.44))
    } else {
        egui::Stroke::new(1.0, DECK_BORDER)
    }
}

/// Subtle multi-pass inner glow inset for depth in glass panels and pills.
pub fn soft_inner_glow(
    painter: &egui::Painter,
    rect: egui::Rect,
    color: egui::Color32,
    radius: u8,
) {
    for (i, alpha) in [(1.0_f32, 0.18_f32), (2.7, 0.12), (4.6, 0.08)] {
        painter.rect_stroke(
            rect.shrink(i),
            radius as f32 * 0.9,
            egui::Stroke::new(1.0, color.gamma_multiply(alpha)),
            egui::StrokeKind::Inside,
        );
    }
}

/// Draw a low-cost diagonal specular sweep across `rect` driven by `time`.
pub fn specular_sweep(painter: &egui::Painter, rect: egui::Rect, time: f64, radius: u8) {
    let mut mesh = egui::Mesh::default();
    let speed = 0.28_f64;
    let band = (rect.width() * 0.28).max(20.0);
    let phase = (time as f32 * speed as f32).fract();
    let drift = rect.width() + rect.height() + band * 1.2;
    let x = rect.left() - band * 0.6 + phase * drift;
    let y = rect.top() + rect.height() * 0.28;
    let up = egui::vec2(rect.height() * 0.08, -rect.height() * 0.08);
    let p0 = egui::pos2(x, y) + up;
    let p1 = egui::pos2(x + band, y) + up;
    let p2 = egui::pos2(p1.x + rect.height() * 0.12, p1.y + rect.height());
    let p3 = egui::pos2(p0.x + rect.height() * 0.12, p0.y + rect.height());
    let i = mesh.vertices.len() as u32;
    mesh.colored_vertex(p0, egui::Color32::from_white_alpha(0));
    mesh.colored_vertex(p1, egui::Color32::from_white_alpha(60));
    mesh.colored_vertex(p2, egui::Color32::from_white_alpha(0));
    mesh.colored_vertex(p3, egui::Color32::from_white_alpha(0));
    mesh.add_triangle(i, i + 1, i + 2);
    mesh.add_triangle(i, i + 2, i + 3);
    painter.add(egui::Shape::mesh(mesh));
    painter.rect_stroke(
        rect,
        radius as f32,
        egui::Stroke::new(1.0, DECK_BORDER),
        egui::StrokeKind::Inside,
    );
    panel_finish(painter, rect, radius, false, false);
}

/// Paint a centred icon (PNG when `icon_tex` is `Some`, else the unicode glyph)
/// above a small caption inside `rect`. Shared by the cluster pills + tiles so
/// category and sub-tool blocks render identically.
pub fn paint_cluster_icon_label(
    p: &egui::Painter,
    rect: egui::Rect,
    icon: &str,
    label: &str,
    lit: bool,
    accent: egui::Color32,
    icon_tex: Option<egui::TextureId>,
) {
    let icon_color = if lit { accent } else { DECK_TEXT };
    let icon_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.38);
    if let Some(tex) = icon_tex {
        let side = (rect.height() * 0.42).clamp(18.0, 28.0);
        let img_rect = egui::Rect::from_center_size(icon_at, egui::vec2(side, side));
        let tint = if lit {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_white_alpha(220)
        };
        p.image(
            tex,
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    } else {
        p.text(
            icon_at,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(20.0),
            icon_color,
        );
    }
    let label_color = if lit { accent } else { DECK_TEXT_MID };
    let label_at = rect.min + egui::vec2(rect.width() * 0.5, rect.height() * 0.80);
    p.text(
        label_at,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.5),
        label_color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_number_formatting() {
        assert_eq!(compact(0.0), "0");
        assert_eq!(compact(12_300.0), "12.3K");
    }

    #[test]
    fn keycap_ramp_is_ordered() {
        let lum = |c: egui::Color32| c.r() as u32 + c.g() as u32 + c.b() as u32;
        assert!(lum(KC_BG) < lum(KC_BG_ALT));
        assert!(lum(KC_BG_ALT) < lum(KC_BG_SOFT));
        assert!(lum(KC_BG_SOFT) < lum(KC_BG_ELV));
        assert!(lum(KC_BG_ELV) < lum(KC_SLATE));
    }

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
        ] {
            assert_ne!(fill, NEON);
            assert!(graphite.contains(&fill));
        }
        assert_eq!(v.widgets.active.bg_stroke.color, NEON);
    }

    #[test]
    fn legacy_aliases_and_deck_tokens() {
        assert_eq!(ACCENT, KC_ACCENT);
        assert_eq!(DECK_AMBER, KC_ACCENT);
        assert_eq!(PANEL_FILL, GRAPHITE_900);
        assert_eq!(HOLO_CORE, HOLO_CYAN);
        assert_eq!(DECK_BG, KC_BG);
    }

    #[test]
    fn ease_out_cubic_endpoints() {
        assert!((ease_out_cubic(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < f32::EPSILON);
    }
}
