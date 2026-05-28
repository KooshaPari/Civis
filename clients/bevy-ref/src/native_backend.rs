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
  #[cfg(not(any(target_os = "windows", target_os = "macos", all(unix, not(target_os = "macos")))))]
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
  let raw = std::env::var(BACKEND_ENV).ok()?;
  match raw.trim().to_ascii_lowercase().as_str() {
    "dx12" | "d3d12" | "directx" => Some(Backends::DX12),
    "vulkan" | "vk" => Some(Backends::VULKAN),
    "metal" => Some(Backends::METAL),
    _ => {
      bevy::log::warn!(
        "ignoring {BACKEND_ENV}={raw:?} (expected dx12, vulkan, or metal)"
      );
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
