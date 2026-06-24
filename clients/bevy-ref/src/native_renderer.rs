//! Bevy native renderer capability discovery.
//!
//! This runs inside the render app so it can read Bevy's `RenderDevice` and
//! `RenderAdapterInfo`, then branch into wgpu native escape hatches later.

use bevy::prelude::*;
use bevy::render::render_resource::WgpuAdapterInfo;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::render::RenderApp;

/// Runtime GPU capabilities detected from the active Bevy render device.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GpuCapabilities {
    /// `true` when ray tracing is available on the active adapter/backend.
    pub ray_tracing: bool,
    /// `true` when mesh shaders are available on the active adapter/backend.
    pub mesh_shaders: bool,
    /// Backend name in user-facing form: `DX12`, `Vulkan`, `Metal`, or `Other`.
    pub backend_name: String,
    /// Adapter name reported by wgpu.
    pub adapter_name: String,
    /// Adapter vendor ID reported by wgpu.
    pub vendor_id: u32,
}

impl Default for GpuCapabilities {
    fn default() -> Self {
        Self {
            ray_tracing: false,
            mesh_shaders: false,
            backend_name: "Other".to_string(),
            adapter_name: String::new(),
            vendor_id: 0,
        }
    }
}

/// Detect the active GPU capability set from Bevy's render resources.
#[must_use]
pub fn detect_capabilities(
    render_device: &RenderDevice,
    adapter_info: &RenderAdapterInfo,
) -> GpuCapabilities {
    let info: &WgpuAdapterInfo = &adapter_info.0;
    let features = render_device.features();
    let backend_name = match info.backend {
        wgpu::Backend::Dx12 => "DX12",
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        _ => "Other",
    }
    .to_string();

    let ray_tracing = features.contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY)
        || features.contains(wgpu::Features::SHADER_INT64)
        || matches!(info.backend, wgpu::Backend::Dx12 | wgpu::Backend::Vulkan);

    let mesh_shaders = features.contains(wgpu::Features::EXPERIMENTAL_MESH_SHADER)
        || matches!(info.backend, wgpu::Backend::Dx12 | wgpu::Backend::Vulkan);

    let caps = GpuCapabilities {
        ray_tracing,
        mesh_shaders,
        backend_name,
        adapter_name: info.name.clone(),
        vendor_id: info.vendor,
    };

    info!(
        "native gpu adapter={} vendor=0x{:04x} backend={} ray_tracing={} mesh_shaders={}",
        caps.adapter_name, caps.vendor_id, caps.backend_name, caps.ray_tracing, caps.mesh_shaders
    );

    caps
}

/// Plugin that captures GPU capabilities during render startup.
pub struct NativeRendererPlugin;

impl Plugin for NativeRendererPlugin {
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<GpuCapabilities>();
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
        assert_eq!(caps.backend_name, "Other");
    }
}
