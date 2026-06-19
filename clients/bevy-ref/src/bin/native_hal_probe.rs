//! Short-lived Bevy app that logs whether the active wgpu device exposes a native HAL (DX12 on Windows).
#![allow(unsafe_code)]

use bevy::prelude::*;
use bevy::render::renderer::{RenderAdapterInfo, RenderDevice};
use bevy::render::RenderApp;
use civ_bevy_ref::{gpu_features::GpuFeaturesPlugin, native_backend::native_render_plugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(native_render_plugin())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "civ-native-hal-probe".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(GpuFeaturesPlugin)
        .add_plugins(NativeHalProbePlugin)
        .add_systems(Update, exit_after_probe)
        .run();
}

struct NativeHalProbePlugin;

impl Plugin for NativeHalProbePlugin {
    fn build(&self, app: &mut App) {
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.add_systems(Startup, log_native_hal_probe);
        }
    }
}

fn log_native_hal_probe(render_device: Res<RenderDevice>, adapter_info: Res<RenderAdapterInfo>) {
    let device = render_device.wgpu_device();
    let backend = adapter_info.0.backend;

    #[cfg(target_os = "windows")]
    {
        use wgpu::hal::api::Dx12;
        if backend == wgpu::Backend::Dx12 {
            if let Some(_hal) = unsafe { device.as_hal::<Dx12>() } {
                info!("native_hal_probe: DX12 HAL device acquired via wgpu::Device::as_hal");
            } else {
                warn!("native_hal_probe: backend is DX12 but as_hal::<Dx12>() returned None");
            }
        } else {
            info!(
                "native_hal_probe: active backend is {:?} (DX12 HAL probe skipped)",
                backend
            );
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        info!(
            "native_hal_probe: active backend {:?} (DX12 probe is Windows-only)",
            backend
        );
    }
}

fn exit_after_probe(mut frames: Local<u32>) {
    *frames += 1;
    if *frames >= 3 {
        info!("native_hal_probe: exiting");
        std::process::exit(0);
    }
}
