#![cfg(all(feature = "bevy", feature = "egui"))]

//! Star Wars **hologram** painters for the Civis HUD projection tier.
//!
//! Implements §3 of `docs/design/ui-design-language.md` — the signature
//! "projected light" layer reserved for live, data-dense readouts (inspector,
//! minimap, overlay legends, alert toasts, the boot/loading projection). This
//! is where color *intensity* lives, concentrated in a tiny area (≈8% of the
//! HUD); the rest is calm graphite chrome ([`crate::ui_theme`]).
//!
//! Everything is painted with egui [`egui::Painter`] primitives driven by a
//! per-frame `time` seconds value from the app clock, so the inspector /
//! minimap / overlays can call into these reusable helpers:
//!
//! - [`holo_panel`] — full recipe: scrim-free translucent volume + corner
//!   brackets + scanlines + scan-sweep + idle flicker/jitter + spawn flicker.
//! - [`holo_frame`] — just the corner brackets + border halo (no scanlines),
//!   for framing an existing widget (e.g. the minimap viewport rect).
//! - [`scanlines`] — the 3px scanline texture + travelling scan-sweep.
//! - [`holo_text`] — projected mono text with the 2-pass glow + ±1px chromatic
//!   aberration.
//!
//! The holo-cyan family (`HOLO_CORE`/`HOLO_GLOW`/`HOLO_DEEP`/aberration ghosts)
//! is defined in [`crate::ui_theme`] and re-used here; it must never appear in
//! the chrome tier.

use bevy_egui::egui;

use crate::ui_theme::{HOLO_ABERR_B, HOLO_ABERR_R, HOLO_CORE, HOLO_DEEP, HOLO_GLOW};

// ---------------------------------------------------------------------------
// Animation constants (§3.2 / §3.5)
// ---------------------------------------------------------------------------

/// Scanline pitch: 1px line + 2px gap (§3.2).
pub const SCANLINE_PITCH: f32 = 3.0;
/// Scan-sweep loop period in seconds (§3.2).
pub const SCAN_SWEEP_SECS: f32 = 2.2;
/// Spawn-flicker intro duration in seconds (§3.5).
pub const SPAWN_FLICKER_SECS: f32 = 0.22;
/// Idle vertical-jitter frequency in Hz (§3.5, "unstable beam").
pub const JITTER_HZ: f32 = 0.7;
/// Idle vertical-jitter amplitude in px (±).
pub const JITTER_PX: f32 = 0.5;
/// Resting chromatic-aberration offset in px (§3.4).
pub const ABERRATION_PX: f32 = 1.0;
/// Translucent holo fill alpha (§3.1, `HOLO_DEEP @ 0.22`).
pub const HOLO_FILL_ALPHA: f32 = 0.22;

/// Animation phase for a holo surface, derived from the app clock.
///
/// `time` is wall-clock seconds; `age` is seconds since the panel opened
/// (drives the §3.5 spawn flicker — pass a large value for a settled panel).
#[derive(Debug, Clone, Copy)]
pub struct HoloPhase {
    /// Wall-clock seconds (idle flicker, scan-sweep, jitter).
    pub time: f32,
    /// Seconds since this surface spawned (spawn-flicker intro).
    pub age: f32,
}

impl HoloPhase {
    /// A settled phase at `time` (no spawn flicker).
    #[must_use]
    pub fn settled(time: f32) -> Self {
        Self {
            time,
            age: SPAWN_FLICKER_SECS * 4.0,
        }
    }

    /// A spawning phase: `age` seconds into the §3.5 intro.
    #[must_use]
    pub fn spawning(time: f32, age: f32) -> Self {
        Self { time, age }
    }

    /// Whole-projection opacity multiplier: idle flicker × spawn intro (§3.5).
    #[must_use]
    pub fn opacity(&self) -> f32 {
        idle_flicker(self.time) * spawn_opacity(self.age)
    }

    /// Vertical beam jitter offset in px (§3.5).
    #[must_use]
    pub fn jitter_y(&self) -> f32 {
        (self.time * std::f32::consts::TAU * JITTER_HZ).sin() * JITTER_PX
    }

    /// Current chromatic-aberration offset in px — grows to ±2px during the
    /// spawn intro, then settles to ±1px (§3.4).
    #[must_use]
    pub fn aberration(&self) -> f32 {
        if self.age < SPAWN_FLICKER_SECS {
            let t = (self.age / SPAWN_FLICKER_SECS).clamp(0.0, 1.0);
            ABERRATION_PX + (1.0 - t) // 2.0 → 1.0
        } else {
            ABERRATION_PX
        }
    }
}

/// Idle flicker multiplier `0.92 + 0.08·noise(t)` with subtle ~80ms dips (§3.5).
#[must_use]
pub fn idle_flicker(time: f32) -> f32 {
    // Cheap deterministic pseudo-noise: layered sines + an occasional dip.
    let n = (time * 11.0).sin() * 0.5 + (time * 27.0 + 1.3).sin() * 0.5;
    let base = 0.92 + 0.08 * (n * 0.5 + 0.5);
    // Sparse brightness dips (~every 2.5s, ~80ms wide).
    let cycle = (time % 2.5) / 2.5;
    let dip = if cycle < 0.032 { 0.12 } else { 0.0 };
    (base - dip).clamp(0.0, 1.0)
}

/// Spawn-flicker opacity envelope: `0 → 1.1 → 1.0` over [`SPAWN_FLICKER_SECS`]
/// with 2–3 fast stutters — the Leia "projector switching on" moment (§3.5).
#[must_use]
pub fn spawn_opacity(age: f32) -> f32 {
    if age >= SPAWN_FLICKER_SECS {
        return 1.0;
    }
    let t = (age / SPAWN_FLICKER_SECS).clamp(0.0, 1.0);
    let ramp = if t < 0.6 {
        (t / 0.6) * 1.1 // up to 1.1 overshoot
    } else {
        1.1 - (t - 0.6) / 0.4 * 0.1 // settle 1.1 → 1.0
    };
    // Fast stutters early in the intro.
    let stutter = if t < 0.5 && (age * 90.0).sin() < -0.3 {
        0.45
    } else {
        1.0
    };
    (ramp * stutter).clamp(0.0, 1.1)
}

// ---------------------------------------------------------------------------
// Painters
// ---------------------------------------------------------------------------

/// Apply alpha (`0..=1`) to a color, multiplying into its existing alpha.
fn fade(color: egui::Color32, mul: f32) -> egui::Color32 {
    color.gamma_multiply(mul.clamp(0.0, 1.0))
}

/// Paint the **full holo panel** recipe (§3.1–3.5) into `rect`: translucent
/// [`HOLO_DEEP`] volume, a faint center→edge radial brighten, the corner-bracket
/// frame, scanlines + travelling sweep, and the whole projection modulated by
/// idle flicker (and the spawn intro when `phase.age` is small) plus a vertical
/// beam jitter.
///
/// Returns the jitter-adjusted content [`egui::Rect`] so the caller can draw
/// projected rows/wireframe inside the stable beam.
pub fn holo_panel(painter: &egui::Painter, rect: egui::Rect, phase: HoloPhase) -> egui::Rect {
    let op = phase.opacity();
    let rect = rect.translate(egui::vec2(0.0, phase.jitter_y()));

    // Translucent projected volume (you see the world faintly through it).
    painter.rect_filled(rect, 4.0, fade(HOLO_DEEP, HOLO_FILL_ALPHA * op));
    // Center-brighter radial sell: a second, inset, brighter fill.
    painter.rect_filled(
        rect.shrink(rect.width().min(rect.height()) * 0.22),
        4.0,
        fade(HOLO_GLOW, 0.05 * op),
    );

    scanlines(painter, rect, phase);
    holo_frame(painter, rect, phase);
    rect
}

/// Paint just the **corner-bracket frame** (§3.1): four 12px L-brackets in
/// [`HOLO_CORE`] with a 3px outer [`HOLO_GLOW`] halo — the targeting/blueprint
/// read. No closed rectangle; edges are implied by the scanlines.
pub fn holo_frame(painter: &egui::Painter, rect: egui::Rect, phase: HoloPhase) {
    let op = phase.opacity();
    let leg = 12.0_f32.min(rect.width() * 0.4).min(rect.height() * 0.4);
    let core = egui::Stroke::new(1.0, fade(HOLO_CORE, 0.8 * op));
    let halo = egui::Stroke::new(3.0, fade(HOLO_GLOW, 0.35 * op));

    for &(corner, dx, dy) in &[
        (rect.left_top(), 1.0, 1.0),
        (rect.right_top(), -1.0, 1.0),
        (rect.left_bottom(), 1.0, -1.0),
        (rect.right_bottom(), -1.0, -1.0),
    ] {
        let h = egui::pos2(corner.x + dx * leg, corner.y);
        let v = egui::pos2(corner.x, corner.y + dy * leg);
        // Glow halo under crisp core (2-pass, §3.3).
        painter.line_segment([corner, h], halo);
        painter.line_segment([corner, v], halo);
        painter.line_segment([corner, h], core);
        painter.line_segment([corner, v], core);
    }
}

/// Paint the **scanline texture** (§3.2): perfectly horizontal 1px lines every
/// [`SCANLINE_PITCH`] px at [`HOLO_GLOW`] @ 0.10, plus one brighter
/// [`HOLO_CORE`] @ 0.35 sweep line travelling top→bottom over
/// [`SCAN_SWEEP_SECS`], looping.
pub fn scanlines(painter: &egui::Painter, rect: egui::Rect, phase: HoloPhase) {
    let op = phase.opacity();
    let line = egui::Stroke::new(1.0, fade(HOLO_GLOW, 0.10 * op));
    let mut y = rect.top();
    while y < rect.bottom() {
        painter.hline(rect.x_range(), y, line);
        y += SCANLINE_PITCH;
    }
    // Travelling scan-sweep.
    let t = (phase.time % SCAN_SWEEP_SECS) / SCAN_SWEEP_SECS;
    let sweep_y = rect.top() + t * rect.height();
    painter.hline(
        rect.x_range(),
        sweep_y,
        egui::Stroke::new(1.5, fade(HOLO_CORE, 0.35 * op)),
    );
}

/// Paint **projected mono text** (§3.3–3.6) at `pos` (top-left): a 2-pass glow
/// ([`HOLO_GLOW`] soft under [`HOLO_CORE`] crisp) with ±`phase.aberration()` px
/// chromatic-aberration ghosts ([`HOLO_ABERR_R`] / [`HOLO_ABERR_B`]). Returns
/// the laid-out [`egui::Rect`] so callers can stack rows.
pub fn holo_text(
    painter: &egui::Painter,
    pos: egui::Pos2,
    text: &str,
    size: f32,
    phase: HoloPhase,
) -> egui::Rect {
    let op = phase.opacity();
    let ab = phase.aberration();
    let font = egui::FontId::monospace(size);
    let anchor = egui::Align2::LEFT_TOP;

    // Chromatic-aberration ghosts (text/wireframe only — §3.4).
    painter.text(
        pos + egui::vec2(ab, ab * 0.5),
        anchor,
        text,
        font.clone(),
        fade(HOLO_ABERR_R, 0.35 * op),
    );
    painter.text(
        pos + egui::vec2(-ab, -ab * 0.5),
        anchor,
        text,
        font.clone(),
        fade(HOLO_ABERR_B, 0.35 * op),
    );
    // Soft glow pass.
    painter.text(pos, anchor, text, font.clone(), fade(HOLO_GLOW, 0.4 * op));
    // Crisp core pass on top.
    painter.text(pos, anchor, text, font, fade(HOLO_CORE, op))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_flicker_stays_in_subtle_band() {
        // Never a strobe: opacity stays high except during sparse short dips.
        for i in 0..200 {
            let t = i as f32 * 0.05;
            let f = idle_flicker(t);
            assert!((0.0..=1.0).contains(&f));
            assert!(f > 0.78, "flicker dipped too hard at t={t}: {f}");
        }
    }

    #[test]
    fn spawn_opacity_settles_to_one() {
        assert_eq!(spawn_opacity(SPAWN_FLICKER_SECS), 1.0);
        assert_eq!(spawn_opacity(1.0), 1.0);
        // Overshoots above 1.0 somewhere in the intro (the 1.1 peak).
        let peak = (0..40)
            .map(|i| spawn_opacity(i as f32 / 40.0 * SPAWN_FLICKER_SECS))
            .fold(0.0_f32, f32::max);
        assert!(
            peak > 1.0,
            "spawn intro should overshoot to ~1.1, got {peak}"
        );
    }

    #[test]
    fn aberration_grows_during_spawn_then_settles() {
        let spawning = HoloPhase::spawning(0.0, 0.0);
        let settled = HoloPhase::settled(0.0);
        assert!(spawning.aberration() > settled.aberration());
        assert!((settled.aberration() - ABERRATION_PX).abs() < f32::EPSILON);
    }

    #[test]
    fn jitter_within_amplitude() {
        for i in 0..100 {
            let p = HoloPhase::settled(i as f32 * 0.1);
            assert!(p.jitter_y().abs() <= JITTER_PX + f32::EPSILON);
        }
    }
}
