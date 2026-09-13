//! Runtime GPU capability detection for the Bevy reference client.
//!
//! This stays in the Bevy path only and is intended as a foundation for
//! native renderer escape hatches where the backend exposes extra features.

use bevy::prelude::*;
use bevy::render::render_resource::WgpuAdapterInfo;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::render::RenderApp;
use wgpu;

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

/// Pure feature flags derived from adapter info + feature set.
///
/// `max_vram_mb` is not part of this struct because it depends on the runtime
/// `wgpu::Device` limits; everything else can be classified off the cheap
/// adapter info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityFlags {
    pub ray_tracing: bool,
    pub mesh_shaders: bool,
    pub dlss_available: bool,
    pub fsr_available: bool,
    pub metal_fx: bool,
    pub backend_name: String,
}

/// Classify capabilities from a `(backend, vendor, features)` triple without
/// touching a live `wgpu::Device`. Pure function — fully testable.
#[must_use]
pub fn classify_capabilities(
    backend: wgpu::Backend,
    vendor: u32,
    features: wgpu::Features,
) -> CapabilityFlags {
    let backend_name = match backend {
        wgpu::Backend::Dx12 => "DX12",
        wgpu::Backend::Vulkan => "Vulkan",
        wgpu::Backend::Metal => "Metal",
        _ => "WebGPU",
    }
    .to_string();

    let is_nvidia = vendor == 0x10DE;
    let is_amd = vendor == 0x1002;
    let is_intel = vendor == 0x8086;
    let is_apple = vendor == 0x106B;

    let ray_tracing = features.contains(wgpu::Features::EXPERIMENTAL_RAY_QUERY)
        || (is_nvidia && matches!(backend, wgpu::Backend::Dx12))
        || (is_apple && matches!(backend, wgpu::Backend::Metal));

    let mesh_shaders = features.contains(wgpu::Features::EXPERIMENTAL_MESH_SHADER)
        || (is_nvidia && matches!(backend, wgpu::Backend::Dx12))
        || (is_amd && matches!(backend, wgpu::Backend::Dx12));

    let dlss_available = is_nvidia && matches!(backend, wgpu::Backend::Dx12);
    let metal_fx = is_apple && matches!(backend, wgpu::Backend::Metal);
    let fsr_available = is_amd
        || is_nvidia
        || is_intel
        || is_apple
        || matches!(backend, wgpu::Backend::BrowserWebGpu);

    CapabilityFlags {
        ray_tracing,
        mesh_shaders,
        dlss_available,
        fsr_available,
        metal_fx,
        backend_name,
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
    let flags = classify_capabilities(info.backend, info.vendor, features);

    // Best-effort estimate only: wgpu does not provide dedicated VRAM directly.
    // Adapter limits give us a stable runtime upper bound for uploadable buffers.
    let max_vram_mb = (render_device.limits().max_buffer_size / (1024 * 1024)) as u32;

    let caps = GpuCapabilities {
        ray_tracing: flags.ray_tracing,
        mesh_shaders: flags.mesh_shaders,
        dlss_available: flags.dlss_available,
        fsr_available: flags.fsr_available,
        metal_fx: flags.metal_fx,
        max_vram_mb,
        backend_name: flags.backend_name,
    };

    info!("gpu capabilities: {:?}", caps);
    caps
}

/// Plugin that captures GPU capabilities during render startup.
pub struct GpuFeaturesPlugin;

impl Plugin for GpuFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuCapabilities>();
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Startup, detect_and_store_capabilities);
            let mut default_extract = render_app.take_extract();
            render_app.set_extract(move |main_world, render_world| {
                if let Some(extract) = default_extract.as_mut() {
                    extract(main_world, render_world);
                }
                if let Some(caps) = render_world.get_resource::<GpuCapabilities>() {
                    main_world.insert_resource(caps.clone());
                }
            });
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

    const NVIDIA: u32 = 0x10DE;
    const AMD: u32 = 0x1002;
    const INTEL: u32 = 0x8086;
    const APPLE: u32 = 0x106B;

    #[test]
    fn defaults_are_conservative() {
        let caps = GpuCapabilities::default();
        assert!(!caps.ray_tracing);
        assert!(!caps.mesh_shaders);
        assert_eq!(caps.backend_name, "WebGPU");
    }

    // ---------------------------------------------------------------
    // classify_capabilities — exhaustive vendor/backend truth table
    // ---------------------------------------------------------------

    #[test]
    fn backend_name_matches_canonical_label() {
        assert_eq!(
            classify_capabilities(wgpu::Backend::Dx12, 0, wgpu::Features::empty()).backend_name,
            "DX12"
        );
        assert_eq!(
            classify_capabilities(wgpu::Backend::Vulkan, 0, wgpu::Features::empty())
                .backend_name,
            "Vulkan"
        );
        assert_eq!(
            classify_capabilities(wgpu::Backend::Metal, 0, wgpu::Features::empty()).backend_name,
            "Metal"
        );
        assert_eq!(
            classify_capabilities(
                wgpu::Backend::BrowserWebGpu,
                0,
                wgpu::Features::empty()
            )
            .backend_name,
            "WebGPU"
        );
    }

    #[test]
    fn nvidia_dx12_unlocks_rt_mesh_dlss_no_metal_fx() {
        let f = classify_capabilities(wgpu::Backend::Dx12, NVIDIA, wgpu::Features::empty());
        assert!(f.ray_tracing, "nvidia+dx12 should imply ray tracing");
        assert!(f.mesh_shaders, "nvidia+dx12 should imply mesh shaders");
        assert!(f.dlss_available, "nvidia+dx12 should imply dlss");
        assert!(f.fsr_available, "nvidia is in the fsr allowlist");
        assert!(!f.metal_fx, "nvidia+dx12 must NOT enable metal_fx");
    }

    #[test]
    fn nvidia_vulkan_loses_dlss_but_keeps_fsr() {
        let f = classify_capabilities(wgpu::Backend::Vulkan, NVIDIA, wgpu::Features::empty());
        assert!(!f.dlss_available, "dlss is dx12-only");
        assert!(f.fsr_available);
    }

    #[test]
    fn amd_dx12_enables_mesh_and_fsr() {
        let f = classify_capabilities(wgpu::Backend::Dx12, AMD, wgpu::Features::empty());
        assert!(f.mesh_shaders);
        assert!(f.fsr_available);
        assert!(!f.dlss_available);
    }

    #[test]
    fn intel_dx12_is_fsr_capable_only() {
        let f = classify_capabilities(wgpu::Backend::Dx12, INTEL, wgpu::Features::empty());
        assert!(f.fsr_available);
        assert!(!f.ray_tracing);
        assert!(!f.mesh_shaders);
        assert!(!f.dlss_available);
        assert!(!f.metal_fx);
    }

    #[test]
    fn apple_metal_unlocks_rt_and_metal_fx_only() {
        let f = classify_capabilities(wgpu::Backend::Metal, APPLE, wgpu::Features::empty());
        assert!(f.ray_tracing, "apple+metal should imply ray tracing");
        assert!(f.metal_fx, "apple+metal should imply metal_fx");
        assert!(f.fsr_available);
        assert!(!f.dlss_available, "metal must not imply dlss");
        assert!(!f.mesh_shaders, "apple+metal alone must not imply mesh shaders");
    }

    #[test]
    fn unknown_vendor_disables_all_experimental_features() {
        let f = classify_capabilities(wgpu::Backend::Vulkan, 0x9999, wgpu::Features::empty());
        assert!(!f.ray_tracing);
        assert!(!f.mesh_shaders);
        assert!(!f.dlss_available);
        assert!(!f.metal_fx);
        assert!(!f.fsr_available);
    }

    #[test]
    fn experimental_feature_flag_overrides_vendor_heuristics() {
        let f = classify_capabilities(
            wgpu::Backend::Vulkan,
            0x9999,
            wgpu::Features::EXPERIMENTAL_RAY_QUERY,
        );
        assert!(f.ray_tracing);
    }

    #[test]
    fn browser_webgpu_unlocks_fsr_only() {
        let f = classify_capabilities(
            wgpu::Backend::BrowserWebGpu,
            0x9999,
            wgpu::Features::empty(),
        );
        assert!(f.fsr_available, "BrowserWebGpu path must enable FSR fallback");
        assert!(!f.ray_tracing);
        assert!(!f.dlss_available);
        assert!(!f.metal_fx);
    }
}
