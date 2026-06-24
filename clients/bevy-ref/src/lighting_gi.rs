//! Real-time raytraced global illumination (Lumen-equivalent) via `bevy_solari`.
//!
//! WRAP > HANDROLL: this module wraps Bevy's **in-tree** `bevy_solari` — the
//! ReSTIR direct-illumination + ReSTIR GI (2nd bounce) + world-space irradiance
//! cache (further bounces) + specular-GI raytracing stack — rather than
//! hand-rolling VXGI / DDGI / surfels. Per
//! `docs/research/sota-tech/gfx.md` §1, Solari's ReSTIR + world-cache is *the*
//! GI answer for Civis and has effectively superseded probe/surfel GI in the
//! Bevy roadmap; VXGI is only worth keeping as a non-DXR fallback (not wired
//! here). `bevy_solari` also delivers RT reflections "for free-ish" through its
//! specular-GI pass (gfx.md §2), so this one plugin covers both GI and mirror
//! reflections.
//!
//! # Bevy 0.18 compatibility
//!
//! `bevy_solari` ships *inside* Bevy 0.18 behind the `bevy_solari` cargo
//! feature (NOT a separate crates.io crate). This module is gated behind a
//! `gi` cargo feature which enables `bevy/bevy_solari`. The public API used
//! here was verified on 2026-05-30 against the Bevy 0.18 `examples/3d/solari.rs`
//! reference:
//!
//! - `bevy::solari::prelude::SolariPlugins` — the umbrella plugin group
//!   (raytracing scene/BLAS/TLAS build + lighting passes).
//! - `bevy::solari::prelude::SolariLighting` — the per-camera component that
//!   turns on realtime ReSTIR DI + GI for that view.
//! - `bevy::camera::CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING)`
//!   and `Msaa::Off` — both **required** on any Solari camera (the example file
//!   states this explicitly: Solari writes to the view target via a storage
//!   binding and is incompatible with hardware MSAA).
//!
//! # Hardware requirement + graceful no-op
//!
//! Solari needs a GPU and backend exposing **ray-tracing acceleration
//! structures**: DXR on DX12 (our primary desktop target — see project memory
//! "Civis DESKTOP IS PRIMARY", DX12 Ultimate + DXR) or `VK_KHR_ray_tracing` on
//! Vulkan. On adapters without RT (older GPUs, most software/CI adapters like
//! llvmpipe, Metal without the RT family, WebGPU) `SolariPlugins` cannot run.
//!
//! Per the project charter (CLAUDE.md "Optionality and failure behavior"): we
//! do **not** silently degrade. [`SolariGiPlugin`] checks the render adapter's
//! advertised features at startup and, when RT acceleration structures are
//! absent, **logs a loud, explicit warning naming the missing feature** and
//! skips adding the Solari camera components — the app keeps running with the
//! standard rasterized lighting path instead of panicking the renderer. Callers
//! that consider GI mandatory should treat that warning as a hard failure in
//! their own preflight.
//!
//! # Wiring (the caller does these — this module touches no other file)
//!
//! In `lib.rs`:
//! ```ignore
//! #[cfg(all(feature = "bevy", feature = "gi"))]
//! pub mod lighting_gi;
//! ```
//!
//! In the app builder (e.g. `bin/standalone.rs`):
//! ```ignore
//! #[cfg(feature = "gi")]
//! app.add_plugins(civ_bevy_ref::lighting_gi::SolariGiPlugin);
//! ```
//!
//! The plugin attaches the required Solari components to every `Camera3d`
//! automatically (when RT is available), so no change to `setup_camera` is
//! strictly needed. If you prefer to wire the camera explicitly instead, insert
//! these on the camera entity in `setup_camera`:
//! ```ignore
//! # use bevy::prelude::*;
//! # use bevy::core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass};
//! # use bevy::camera::CameraMainTextureUsages;
//! # use bevy::render::render_resource::TextureUsages;
//! # use bevy::solari::prelude::SolariLighting;
//! // ... alongside Camera3d::default() ...
//! SolariLighting::default(),
//! CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
//! Msaa::Off,
//! ```

#![cfg(all(feature = "bevy", feature = "gi"))]

use bevy::camera::CameraMainTextureUsages;
use bevy::prelude::*;
use bevy::render::render_resource::TextureUsages;
use bevy::render::renderer::RenderAdapter;
use bevy::solari::prelude::{SolariLighting, SolariPlugins};

/// Marker so we attach Solari components to each camera exactly once.
#[derive(Component)]
struct SolariCameraConfigured;

/// Plugin enabling real-time raytraced global illumination (`bevy_solari`).
///
/// Adds [`SolariPlugins`] and, on cameras, the required [`SolariLighting`] +
/// storage-binding main-texture usage + `Msaa::Off`. When the render adapter
/// lacks ray-tracing acceleration-structure support the plugin logs a loud
/// warning and becomes a no-op (no camera components added), leaving the
/// rasterized lighting path in place rather than degrading silently.
///
/// See the module docs for the exact wiring lines the caller adds in
/// `lib.rs` / the app builder.
pub struct SolariGiPlugin;

impl Plugin for SolariGiPlugin {
    fn build(&self, app: &mut App) {
        // SolariPlugins is a `PluginGroup` (not a `Plugin`), so it has no
        // `is_plugin_added::<T>` guard — `add_plugins` is idempotent for
        // groups (Bevy tracks the group itself once added), so a plain
        // unconditional add is the canonical pattern. Solari's own
        // `RaytracingScenePlugin` also self-skips when the adapter lacks the
        // required features (see `bevy_solari/src/scene/mod.rs:42`), so the
        // RT-unavailable no-op path below is redundant work; we keep the
        // warning + camera-skip here as the loud, user-visible failure signal
        // required by the project charter.
        app.add_plugins(SolariPlugins);
        app.add_systems(Update, configure_solari_cameras);
    }
}

/// True when the render adapter advertises the wgpu features required by
/// `bevy_solari` (DXR on DX12 / `VK_KHR_ray_tracing` on Vulkan surface as
/// `EXPERIMENTAL_RAY_QUERY` + binding-array extensions). Solari cannot run
/// without the full set — `SolariPlugins::required_wgpu_features()` is the
/// authoritative list maintained by `bevy_solari` itself.
fn adapter_supports_raytracing(adapter: &RenderAdapter) -> bool {
    let features = adapter.features();
    features.contains(SolariPlugins::required_wgpu_features())
}

/// Attach the Solari camera components to any not-yet-configured `Camera3d`.
///
/// Runs in `Update` (not `Startup`) so it also catches cameras spawned later by
/// other systems. Each camera is configured at most once via the
/// [`SolariCameraConfigured`] marker. When RT is unavailable this logs once and
/// marks cameras configured (a no-op) so the warning does not spam every frame.
fn configure_solari_cameras(
    mut commands: Commands,
    adapter: Res<RenderAdapter>,
    cameras: Query<Entity, (With<Camera3d>, Without<SolariCameraConfigured>)>,
) {
    if cameras.is_empty() {
        return;
    }

    let rt_ok = adapter_supports_raytracing(&adapter);
    if !rt_ok {
        let missing = SolariPlugins::required_wgpu_features() - adapter.features();
        warn!(
            "SolariGiPlugin: GI disabled — render adapter lacks required \
             Solari wgpu features (need DXR/DX12 or VK_KHR_ray_tracing on \
             Vulkan). Missing: {:?}. Falling back to rasterized lighting; \
             no Solari components attached. Adapter: {:?}",
            missing,
            adapter.get_info()
        );
    }

    for camera in &cameras {
        let mut ent = commands.entity(camera);
        if rt_ok {
            ent.insert((
                SolariLighting::default(),
                // Solari writes the view target via a storage binding and is
                // incompatible with hardware MSAA — both are REQUIRED.
                CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING),
                Msaa::Off,
            ));
        }
        // Mark configured either way so we don't re-check / re-warn each frame.
        ent.insert(SolariCameraConfigured);
    }
}
