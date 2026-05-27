//! Runtime GPU capability detection for the Bevy reference client.
//!
//! This stays in the Bevy path only and is intended as a foundation for
//! native renderer escape hatches where the backend exposes extra features.

use bevy::prelude::*;
use bevy::render::render_resource::WgpuAdapterInfo;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use wgpu;
use bevy::render::RenderApp;

/// Runtime GPU capabilities detected from the active Bevy render device.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GpuCapabilities {
    /// `true` when ray tracing is available on the active adapter/backend.
    pub ray_tracing: bool,
    /// `true` when mesh shaders are available on the active adapter/backend.
    pub mesh_shaders: bool,
    /// `true` when NVIDIA DLSS is likely available through a native SDK path.
    pub dlss_available: bool,
    /// `true` when AMD FSR is available or can be treated as available.
    pub fsr_available: bool,
    /// `true` when Apple MetalFX upscaling is available.
    pub metal_fx: bool,
    /// Estimated maximum VRAM in megabytes.
    pub max_vram_mb: u32,
    /// Backend name in user-facing form: `DX12`, `Vulkan`, `Metal`, `WebGPU`.
    pub backend_name: String,
}

impl Default for GpuCapabilities {
    fn default() -> Self {
        Self {
            ray_tracing: false,
            mesh_shaders: false,
            dlss_available: false,
            fsr_available: false,
            metal_fx: false,
            max_vram_mb: 0,
            backend_name: "WebGPU".to_string(),
        }
    }
}

/// Detect the active GPU capability set from Bevy's render resources.
#[must_use]
pub fn detect_capabilities(render_device: &RenderDevice, adapter_info: &RenderAdapterInfo) -> GpuCapabilities {
    let info: &WgpuAdapterInfo = &adapter_info.0;
    let features = render_device.features();
    let backend_name = match info.backend {
        wgpu::Backend::Dx12 => "DX12",
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        _ => "WebGPU",
    }
    .to_string();

    let vendor = info.vendor;
    let is_nvidia = vendor == 0x10DE;
    let is_amd = vendor == 0x1002;
    let is_intel = vendor == 0x8086;
    let is_apple = vendor == 0x106B;

    // wgpu 27 / Bevy 0.18: no stable RT feature flag; stage vendor/backend inference only.
    let ray_tracing = is_nvidia && matches!(info.backend, wgpu::Backend::Dx12)
        || (is_apple && matches!(info.backend, wgpu::Backend::Metal));

    // wgpu 0.20 does not expose mesh shader capability directly, so this is a
    // backend/vendor inference for the DX12 Ultimate path we want to stage.
    let mesh_shaders = is_nvidia && matches!(info.backend, wgpu::Backend::Dx12)
        || (is_amd && matches!(info.backend, wgpu::Backend::Dx12));

    let dlss_available = is_nvidia && matches!(info.backend, wgpu::Backend::Dx12);
    let metal_fx = is_apple && matches!(info.backend, wgpu::Backend::Metal);
    let fsr_available = is_amd
        || is_nvidia
        || is_intel
        || is_apple
        || matches!(info.backend, wgpu::Backend::BrowserWebGpu);

    // Best-effort estimate only: wgpu does not provide dedicated VRAM directly.
    // Adapter limits give us a stable runtime upper bound for uploadable buffers.
    let max_vram_mb = (render_device.limits().max_buffer_size / (1024 * 1024)) as u32;

    let caps = GpuCapabilities {
        ray_tracing,
        mesh_shaders,
        dlss_available,
        fsr_available,
        metal_fx,
        max_vram_mb,
        backend_name,
    };

    info!("gpu capabilities: {:?}", caps);
    caps
}

/// Plugin that captures GPU capabilities during render startup.
pub struct GpuFeaturesPlugin;

impl Plugin for GpuFeaturesPlugin {
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Startup, detect_and_store_capabilities);
        }
    }
}

fn detect_and_store_capabilities(
    render_device: Res<RenderDevice>,
    adapter_info: Res<RenderAdapterInfo>,
    mut commands: Commands,
) {
    let caps = detect_capabilities(&render_device, &adapter_info);
    commands.insert_resource(caps);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_conservative() {
        let caps = GpuCapabilities::default();
        assert!(!caps.ray_tracing);
        assert!(!caps.mesh_shaders);
        assert_eq!(caps.backend_name, "WebGPU");
    }
}
