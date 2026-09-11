//! Native-window lifecycle protection for the DX12 swapchain.
//!
//! Winit reports a minimized Windows window as a zero-sized client area. Bevy
//! clamps that to 1x1 before configuring the surface. On DX12/wgpu 27 that
//! `ResizeBuffers` call can fail with "window is in use" and panic the render
//! schedule. Keep the last usable surface extent while minimized, then permit
//! the normal resize path after the window is restored.

#![cfg(feature = "bevy")]

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::view::window::ExtractedWindows;
use bevy::render::{Render, RenderApp, RenderSystems};

/// Remembers the last non-minimized surface extent for each render window.
#[derive(Resource, Debug, Default)]
struct LastUsableSurfaceExtents(HashMap<Entity, (u32, u32)>);

/// Installs the native surface lifecycle safeguard.
#[derive(Default)]
pub struct NativeWindowLifecyclePlugin;

impl Plugin for NativeWindowLifecyclePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app
            .get_sub_app_mut(RenderApp)
            .expect("NativeWindowLifecyclePlugin requires Bevy's RenderApp");
        render_app
            .init_resource::<LastUsableSurfaceExtents>()
            .add_systems(
                Render,
                defer_minimized_surface_resize
                    .in_set(RenderSystems::ManageViews)
                    .before(bevy::render::view::window::create_surfaces),
            );
    }
}

fn defer_minimized_surface_resize(
    mut windows: ResMut<ExtractedWindows>,
    mut last_usable: ResMut<LastUsableSurfaceExtents>,
) {
    for window in windows.windows.values_mut() {
        let current = (window.physical_width, window.physical_height);
        if let Some(stable) = stabilized_extent(
            current,
            window.size_changed,
            last_usable.0.get(&window.entity).copied(),
        ) {
            window.physical_width = stable.0;
            window.physical_height = stable.1;
            window.size_changed = false;
            // A present-mode change would also ask Bevy to reconfigure this
            // minimized surface. Its requested value remains in the main
            // world and will be extracted again after restoration.
            window.present_mode_changed = false;
            continue;
        }
        last_usable.0.insert(window.entity, current);
    }
}

/// Returns a cached extent when a changed window has entered Bevy's 1x1
/// minimized representation. Any real non-minimized size updates the cache.
fn stabilized_extent(
    current: (u32, u32),
    size_changed: bool,
    cached: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    if size_changed && current.0 <= 1 && current.1 <= 1 {
        cached
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_changed_minimized_extent_uses_the_cached_surface_size() {
        assert_eq!(
            stabilized_extent((1, 1), true, Some((2560, 1440))),
            Some((2560, 1440))
        );
        assert_eq!(stabilized_extent((1, 1), false, Some((2560, 1440))), None);
        assert_eq!(
            stabilized_extent((2560, 1440), true, Some((1920, 1080))),
            None
        );
        assert_eq!(stabilized_extent((1, 1), true, None), None);
    }
}
