//! Window-icon wiring for the Civis standalone client.
//!
//! Bevy 0.18 / winit 0.30 do not expose a window icon through the `Window`
//! component, so we set it directly on the winit window via the `WinitWindows`
//! resource in a startup system (the documented Bevy pattern). The icon PNG is
//! embedded in the binary so a shortcut launch needs no asset path.
//!
//! The image is the project-owned Civis app icon
//! (`assets/icon/icon-64.png`, authored from `assets/icon/civis-icon.svg` —
//! graphite plate + electric-green voxel-world glyph + holo-cyan apex, per
//! `docs/design/ui-design-language.md`). The matching multi-size `civis.ico`
//! drives the release `.exe` icon (see `build.rs`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;

/// 64×64 RGBA PNG of the Civis app icon, embedded at compile time.
const ICON_PNG: &[u8] = include_bytes!("../assets/icon/icon-64.png");

/// Sets the Civis app icon on the primary winit window at startup.
///
/// Loud-failure stance (per CLAUDE.md): a decode failure is logged at `error!`
/// rather than silently ignored, but it is non-fatal — the app still runs with
/// the platform default icon.
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(primary_entity) = primary.single() else {
        return;
    };
    let Some(winit_window) = windows.get_window(primary_entity) else {
        return;
    };

    let decoded = match image::load_from_memory(ICON_PNG) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            error!("window_icon: failed to decode embedded icon PNG: {e}");
            return;
        }
    };
    let (w, h) = decoded.dimensions();
    match winit::window::Icon::from_rgba(decoded.into_raw(), w, h) {
        Ok(icon) => winit_window.set_window_icon(Some(icon)),
        Err(e) => error!("window_icon: winit rejected icon rgba: {e}"),
    }
}

/// Installs the window-icon startup system.
pub struct WindowIconPlugin;

impl Plugin for WindowIconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, set_window_icon);
    }
}
