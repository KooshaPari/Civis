//! Native GPU backend selection for the Bevy reference client.
//!
//! Bevy still routes through `wgpu`, but we **restrict adapters to native HAL backends**
//! (DX12 / Vulkan / Metal) so future work can use [`wgpu::Device::as_hal`] for DXR, mesh
//! shaders, and vendor upscalers without GLES or browser WebGPU in the path.
//!
//! See `docs/research/wgpu-native-escape-hatches.md`.

use bevy::render::settings::{Backends, RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;

/// Environment variable to force a backend: `dx12`, `vulkan`, or `metal` (case-insensitive).
pub const BACKEND_ENV: &str = "CIV_BEVY_BACKEND";

/// Native HAL backends only — no GLES / browser WebGPU in the adapter search list.
#[must_use]
pub fn native_only_backends() -> Backends {
    if let Some(forced) = forced_backend_from_env() {
        return forced;
    }

    #[cfg(target_os = "windows")]
    {
        Backends::DX12 | Backends::VULKAN
    }
    #[cfg(target_os = "macos")]
    {
        Backends::METAL | Backends::VULKAN
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Backends::VULKAN
    }
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        all(unix, not(target_os = "macos"))
    )))]
    {
        Backends::all()
    }
}

/// `WgpuSettings` tuned for Civis: native backends + wireframe lines on chunk debug overlay.
#[must_use]
pub fn native_wgpu_settings() -> WgpuSettings {
    WgpuSettings {
        backends: Some(native_only_backends()),
        // Do not force experimental RT/mesh-shader features here — adapter may reject init.
        // Capability staging lives in [`crate::gpu_features::detect_capabilities`].
        features: WgpuFeatures::POLYGON_MODE_LINE,
        ..Default::default()
    }
}

/// `RenderPlugin` configured for native-only `wgpu` backends.
#[must_use]
pub fn native_render_plugin() -> RenderPlugin {
    RenderPlugin {
        render_creation: RenderCreation::Automatic(native_wgpu_settings()),
        ..Default::default()
    }
}

fn forced_backend_from_env() -> Option<Backends> {
    forced_backend_from_var(std::env::var(BACKEND_ENV).ok())
}

/// Resolve `CIV_BEVY_BACKEND` from an optional env string (used by [`forced_backend_from_env`] and tests).
fn forced_backend_from_var(raw: Option<String>) -> Option<Backends> {
    let raw = raw?;
    match parse_forced_backend_value(&raw) {
        Some(backends) => Some(backends),
        None => {
            bevy::log::warn!("ignoring {BACKEND_ENV}={raw:?} (expected dx12, vulkan, or metal)");
            None
        }
    }
}

/// Parse `CIV_BEVY_BACKEND` value (case-insensitive, trimmed). Returns `None` for unknown tokens.
fn parse_forced_backend_value(raw: &str) -> Option<Backends> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "dx12" | "d3d12" | "directx" => Some(Backends::DX12),
        "vulkan" | "vk" => Some(Backends::VULKAN),
        "metal" => Some(Backends::METAL),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_forced_backend_value_accepts_dx12_aliases() {
        for raw in ["dx12", "DX12", " d3d12 ", "DirectX"] {
            assert_eq!(
                parse_forced_backend_value(raw),
                Some(Backends::DX12),
                "raw={raw:?}"
            );
        }
    }

    #[test]
    fn parse_forced_backend_value_accepts_vulkan_aliases() {
        for raw in ["vulkan", "VULKAN", " vk ", "VK"] {
            assert_eq!(
                parse_forced_backend_value(raw),
                Some(Backends::VULKAN),
                "raw={raw:?}"
            );
        }
    }

    #[test]
    fn parse_forced_backend_value_accepts_metal() {
        assert_eq!(parse_forced_backend_value("metal"), Some(Backends::METAL));
        assert_eq!(parse_forced_backend_value(" Metal "), Some(Backends::METAL));
    }

    #[test]
    fn parse_forced_backend_value_rejects_gles_and_unknown() {
        for raw in ["", "gles", "gl", "webgpu", "browser_webgpu", "opengl"] {
            assert_eq!(parse_forced_backend_value(raw), None, "raw={raw:?}");
        }
    }


    #[test]
    fn forced_backend_from_var_unset_returns_none() {
        assert_eq!(forced_backend_from_var(None), None);
    }

    #[test]
    fn forced_backend_from_var_accepts_valid_tokens() {
        assert_eq!(
            forced_backend_from_var(Some("vulkan".into())),
            Some(Backends::VULKAN)
        );
        assert_eq!(
            forced_backend_from_var(Some(" DX12 ".into())),
            Some(Backends::DX12)
        );
        assert_eq!(
            forced_backend_from_var(Some("metal".into())),
            Some(Backends::METAL)
        );
    }

    #[test]
    fn forced_backend_from_var_rejects_gles_and_unknown() {
        for raw in ["gles", "webgpu", "not-a-backend"] {
            assert_eq!(
                forced_backend_from_var(Some(raw.into())),
                None,
                "raw={raw:?}"
            );
        }
    }

    #[test]
    fn native_backends_exclude_browser_webgpu_on_windows() {
        #[cfg(target_os = "windows")]
        {
            let b = native_only_backends();
            assert!(b.contains(Backends::DX12));
            assert!(b.contains(Backends::VULKAN));
            assert!(!b.contains(Backends::BROWSER_WEBGPU));
            assert!(!b.contains(Backends::GL));
        }
    }

    #[test]
    fn native_wgpu_settings_use_native_only_backends() {
        let settings = native_wgpu_settings();
        assert_eq!(settings.backends, Some(native_only_backends()));
        assert!(settings
            .features
            .contains(WgpuFeatures::POLYGON_MODE_LINE));
    }
}
