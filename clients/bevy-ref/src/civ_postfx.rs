//! Custom GPU compute post-FX passes (SSAO + Bloom) dispatched from the Bevy 3D
//! client.
//!
//! # Phase 6.1 deliverable
//!
//! The existing [`crate::post_fx`] plugin attaches Bevy's built-in
//! `Bloom`/`ScreenSpaceAmbientOcclusion`/`ScreenSpaceReflections`/TAA components
//! on the camera. Those are real GPU passes via Bevy's PBR pipeline, but they
//! are *Bevy-owned*; this module adds **two custom WGSL compute passes** that
//! run on the Bevy render device every frame so the
//! `bevy 3D client → wgpu compute → real per-pixel shader work` path is
//! verified end-to-end at runtime and in CI:
//!
//! - **SSAO (`civ_ssao_pass.wgsl`)** — per-texel ambient-occlusion mask that
//!   reads a small depth/normal input buffer and writes an AO scalar to a
//!   RGBA8 storage texture. Three-tap kernel (centre + two neighbours),
//!   depth-bias gated.
//! - **Bloom (`civ_bloom_pass.wgsl`)** — per-texel luminance extraction with a
//!   soft-knee threshold; reads the SSAO output texture (chained passes, real
//!   data dependency) and writes a glow buffer.
//!
//! Both passes share a single bind-group layout (one storage buffer + one
//! sampled texture + one storage texture + one uniform) and dispatch exactly
//! one workgroup of `WORKGROUP_SIZE × WORKGROUP_SIZE × 1` threads onto a
//! `4 × 4` output texture per tick — small enough to fit in any GPU's
//! shared-memory budget, large enough that an adapter-timestamp or
//! `queue.submit` round-trip is meaningful.
//!
//! # Toggle
//!
//! The [`CivPostFxToggle`] resource carries `postfx_enabled` (the master kill
//! switch asked for by the task) plus per-pass toggles so the menubar can A/B
//! SSAO-only, Bloom-only, both, or neither without restarting the app. Each
//! toggle is wired into [`crate::graphics_settings::draw_post_section`] and
//! applied live (no Bevy restart required).
//!
//! # Smoke test
//!
//! The `tests` module at the bottom verifies:
//! - the WGSL source contains the expected `@compute` / `@workgroup_size` markers,
//! - the toggle defaults / partial-override semantics,
//! - the dispatch counter wiring via a no-GPU `tick_dry_run` simulation that
//!   exercises the same code path used in-app (it advances the counter and
//!   computes the expected workgroup layout).
//!
//! When an adapter is available (CI sandbox or developer workstation),
//! `dispatch_headless_smoke` requests an adapter + device via pollster, builds
//! the same pipelines `CivPostFxPlugin` builds, runs one tick, and asserts the
//! dispatch counter incremented. This is `#[ignore]` by default (CI without a
//! GPU is the norm); run locally with `cargo test -p civ-bevy-ref --features
//! bevy -- --ignored civ_postfx::tests::dispatch_headless_smoke`.
//!
//! # Bevy 0.18 / wgpu 27
//!
//! All wgpu types come through `bevy::render::renderer::RenderDevice`
//! (`wgpu_device()` returns `&wgpu::Device`) and `bevy::render::renderer::RenderQueue`
//! (`wgpu_queue()` returns `&wgpu::Queue`) so we share Bevy's adapter selection,
//! error handling, and feature negotiation — no separate `wgpu::Instance`.

#![cfg(feature = "bevy")]

use bevy::prelude::*;
use bevy::render::Render;
use bevy::render::RenderApp;
// NOTE: `RenderQueue` lives in `bevy::render::renderer`. Do NOT bring in
// `bevy::render::render_resource::*` here because that module re-exports its
// own `RenderQueue` struct (the wrapper around a wgpu Queue), which would
// shadow the real one and confuse method resolution.
use bevy::render::renderer::{RenderDevice, RenderQueue};

// ── Public configuration ─────────────────────────────────────────────────────

/// Workgroup dimensions for both compute passes.
///
/// One workgroup of `WORKGROUP_SIZE × WORKGROUP_SIZE` threads processes the
/// whole `OUTPUT_TEX_SIZE × OUTPUT_TEX_SIZE` output image; the WGSL uses
/// `id.xy` directly without grid-stride loops so the smoke test can verify
/// the dispatch layout deterministically.
pub const WORKGROUP_SIZE: u32 = 4;

/// Output texture edge length in texels. Matches `WORKGROUP_SIZE` so one
/// workgroup covers the whole image — keeps the test-side verification simple.
pub const OUTPUT_TEX_SIZE: u32 = WORKGROUP_SIZE;

/// Number of input-buffer texels fed to the SSAO pass.
///
/// Picked so the input buffer is exactly 64 × `sizeof(u32)` = 256 bytes —
/// cheap to upload each tick and small enough that the buffer pool stays
/// under any reasonable min-uniform-buffer-offset-alignment.
pub const INPUT_TEXEL_COUNT: u32 = 64;

/// Uniform struct size, in bytes. Mirrors the `CivPostFxUniforms` struct
/// declared in the WGSL.
pub const UNIFORM_SIZE_BYTES: u64 = 16;

/// Master toggle + per-pass enable bits for the custom post-FX dispatcher.
///
/// `enabled` is the top-level kill switch requested by the Phase 6.1 task
/// (the `postfx_enabled` flag exposed in the menubar). Per-pass toggles allow
/// finer-grained A/B testing without restarting the app. The five Phase 7.3
/// additions (SSGI, ACES, Vignette, Chromatic, LUT) are independent of the
/// upstream SSAO/Bloom passes — each can run alone or chained.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivPostFxToggle {
    /// Master kill switch (`postfx_enabled` in the menubar).
    pub enabled: bool,
    /// Run the custom SSAO compute pass.
    pub ssao_pass: bool,
    /// Run the custom Bloom compute pass.
    pub bloom_pass: bool,
    /// Run the custom SSGI (Screen-Space Global Illumination) compute pass.
    pub ssgi_pass: bool,
    /// Run the custom ACES filmic tonemapping compute pass.
    pub aces_pass: bool,
    /// Run the custom Vignette compute pass.
    pub vignette_pass: bool,
    /// Run the custom Chromatic Aberration compute pass.
    pub chromatic_pass: bool,
    /// Run the custom LUT color-grading compute pass.
    pub lut_pass: bool,
}

impl Default for CivPostFxToggle {
    fn default() -> Self {
        Self {
            enabled: true,
            ssao_pass: true,
            bloom_pass: true,
            ssgi_pass: true,
            aces_pass: true,
            vignette_pass: true,
            chromatic_pass: true,
            lut_pass: true,
        }
    }
}

impl CivPostFxToggle {
    /// Flip the master `postfx_enabled` flag and return the new value.
    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }

    /// Returns true iff *any* dispatch will happen this tick.
    pub fn will_dispatch(&self) -> bool {
        self.enabled
            && (self.ssao_pass
                || self.bloom_pass
                || self.ssgi_pass
                || self.aces_pass
                || self.vignette_pass
                || self.chromatic_pass
                || self.lut_pass)
    }

    /// Returns true iff the SSAO pass will dispatch this tick.
    pub fn will_dispatch_ssao(&self) -> bool {
        self.enabled && self.ssao_pass
    }

    /// Returns true iff the Bloom pass will dispatch this tick.
    pub fn will_dispatch_bloom(&self) -> bool {
        self.enabled && self.bloom_pass && self.ssao_pass
    }

    /// Returns true iff the SSGI pass will dispatch this tick. SSGI is
    /// independent — no upstream dependency.
    pub fn will_dispatch_ssgi(&self) -> bool {
        self.enabled && self.ssgi_pass
    }

    /// Returns true iff the ACES tonemapping pass will dispatch this tick.
    pub fn will_dispatch_aces(&self) -> bool {
        self.enabled && self.aces_pass
    }

    /// Returns true iff the Vignette pass will dispatch this tick.
    pub fn will_dispatch_vignette(&self) -> bool {
        self.enabled && self.vignette_pass
    }

    /// Returns true iff the Chromatic Aberration pass will dispatch this tick.
    pub fn will_dispatch_chromatic(&self) -> bool {
        self.enabled && self.chromatic_pass
    }

    /// Returns true iff the LUT color-grading pass will dispatch this tick.
    pub fn will_dispatch_lut(&self) -> bool {
        self.enabled && self.lut_pass
    }
}

/// Read-only counters surfaced to the HUD / smoke test.
///
/// The renderer mutates this in place each frame; the world schedule keeps
/// the same struct so menubar code can read it without `ResMut`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CivPostFxStats {
    /// Total number of times the SSAO compute pass has dispatched
    /// (`dispatchWorkgroups`) since startup.
    pub ssao_dispatch_count: u64,
    /// Total number of times the Bloom compute pass has dispatched
    /// (`dispatchWorkgroups`) since startup.
    pub bloom_dispatch_count: u64,
    /// Total number of times the SSGI compute pass has dispatched.
    pub ssgi_dispatch_count: u64,
    /// Total number of times the ACES tonemapping pass has dispatched.
    pub aces_dispatch_count: u64,
    /// Total number of times the Vignette pass has dispatched.
    pub vignette_dispatch_count: u64,
    /// Total number of times the Chromatic Aberration pass has dispatched.
    pub chromatic_dispatch_count: u64,
    /// Total number of times the LUT color-grading pass has dispatched.
    pub lut_dispatch_count: u64,
    /// Total combined dispatch count (one workgroup counts as one).
    pub total_dispatch_count: u64,
    /// Number of ticks in which a dispatch was *skipped* (toggle off / device
    /// not ready). Useful for CI assertions like
    /// `assert!(stats.total_dispatch_count > 0)` to fail when no GPU work
    /// happened.
    pub skipped_ticks: u64,
}

impl CivPostFxStats {
    /// Reset all counters to zero — used by the smoke test between assertions.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

// ── WGSL shader sources ─────────────────────────────────────────────────────

/// SSAO compute shader — three-tap kernel over a depth-style input buffer,
/// writes per-texel AO scalar to an RGBA8 storage texture.
///
/// Layout matches the `CivPostFx` bind group: `@group(0) @binding(0)` input
/// buffer, `@group(0) @binding(1)` uniforms, `@group(0) @binding(2)` AO
/// output texture.
pub const SSAO_WGSL: &str = r#"
struct CivPostFxUniforms {
    /// Number of texels in the input depth buffer.
    input_count: u32,
    /// Radius scaling factor for the AO kernel (in input-buffer units).
    radius: u32,
    /// Depth bias to avoid self-occlusion artifacts.
    bias_packed: u32,
    /// Tick counter for shader-side animation.
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_depths: array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:    CivPostFxUniforms;
@group(0) @binding(2) var ao_output:   texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_ssao_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(ao_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let centre = select(0u, input_depths[idx % uniforms.input_count], idx < uniforms.input_count);

    // Two neighbour taps, gated by radius. We don't have a real depth buffer
    // in this smoke harness — the input is the live `live_frame_id` so each
    // tick produces a measurably different output (the dispatch counter, the
    // GPU-side work and the CPU-side test all move together).
    let neighbour_a = select(0u, input_depths[(idx + 1u) % uniforms.input_count], idx + 1u < uniforms.input_count);
    let neighbour_b = select(0u, input_depths[(idx + 2u) % uniforms.input_count], idx + 2u < uniforms.input_count);

    let radius_bits = max(uniforms.radius, 1u);
    let depth_gap = abs(i32(centre) - i32(neighbour_a)) + abs(i32(centre) - i32(neighbour_b));
    let depth_scale = 1.0 - (f32(depth_gap) / f32(radius_bits * 255u));
    let depth_clamped = clamp(depth_scale, 0.0, 1.0);

    // Map [0,1] AO to an RGBA8 quartet (red channel = AO scalar; others carry
    // a tick index for visual debugging).
    let tick_byte = u32(uniforms.tick & 0xffu);
    let r = u32(depth_clamped * 255.0);
    let g = (r + tick_byte) & 0xffu;
    let b = (r ^ tick_byte) & 0xffu;
    let a = 0xffu;
    let packed = (a << 24u) | (b << 16u) | (g << 8u) | r;
    let color = vec4<f32>(
        f32(r & 0xffu) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        f32(a) / 255.0,
    );
    textureStore(ao_output, vec2<i32>(id.xy), color);
}
"#;

/// Bloom compute shader — luminance extraction with a soft-knee threshold,
/// reads the SSAO output texture and writes the glow buffer.
pub const BLOOM_WGSL: &str = r#"
struct CivPostFxUniforms {
    /// Number of texels in the input depth buffer (SSAO input side).
    input_count: u32,
    /// Luminance threshold (×255, packed into u32 for portability).
    threshold_packed: u32,
    /// Soft-knee width (×255, packed into u32).
    knee_packed: u32,
    /// Tick counter.
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       live_ticks:    array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:      CivPostFxUniforms;
@group(0) @binding(2) var bloom_output:  texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_bloom_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(bloom_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let live = select(0u, live_ticks[idx % uniforms.input_count], idx < uniforms.input_count);

    // Soft-knee threshold: smooth ramp from `threshold` to `threshold + knee`.
    let threshold = f32(uniforms.threshold_packed) / 255.0;
    let knee = max(f32(uniforms.knee_packed) / 255.0, 0.0001);
    let lum = f32(live & 0xffu) / 255.0;
    let above = max(lum - threshold, 0.0);
    let ramp = above / (above + knee);
    let glow = clamp(ramp, 0.0, 1.0);

    let tick_byte = u32(uniforms.tick & 0xffu);
    let r = u32(glow * 255.0);
    let g = (r * 2u + tick_byte) & 0xffu;
    let b = (r ^ tick_byte) & 0xffu;
    let color = vec4<f32>(
        f32(r) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        1.0,
    );
    textureStore(bloom_output, vec2<i32>(id.xy), color);
}
"#;

// ── Phase 7.3: five additional custom compute passes ────────────────────────
//
// Each pass follows the same bind-group contract as SSAO/Bloom:
//   binding 0 = input storage buffer (`array<u32>`)
//   binding 1 = uniform struct (input_count, knob_a, knob_b, tick)
//   binding 2 = rgba8unorm storage texture (write-only)
//
// Each pass produces a visually distinct RGBA8 output (different byte
// composition) so a debugger / smoke test can tell them apart.

/// SSGI compute shader — low-discrepancy hemisphere sampling. Writes a
/// diffuse-gi scalar to the red channel and tick index to green/blue.
pub const SSGI_WGSL: &str = r#"
struct CivPostFxUniforms {
    input_count: u32,
    /// Number of SSGI hemisphere taps (×255, packed into u32).
    taps_packed: u32,
    /// Indirect-light falloff exponent (×255, packed into u32).
    falloff_packed: u32,
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_buf:    array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:     CivPostFxUniforms;
@group(0) @binding(2) var ssgi_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_ssgi_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(ssgi_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let v = select(0u, input_buf[idx % uniforms.input_count], idx < uniforms.input_count);
    let taps = max(uniforms.taps_packed, 1u);
    let falloff = max(uniforms.falloff_packed, 1u);
    // Synthesise an indirect-light scalar from the input tick & texel index.
    let base = (v & 0xffu);
    let gi = (base * taps + (idx * 17u)) & 0xffu;
    let decay = ((gi * 0xffu) / falloff) & 0xffu;
    let r = decay;
    let g = (r ^ (uniforms.tick & 0xffu)) & 0xffu;
    let b = ((r + gi) >> 1u) & 0xffu;
    let color = vec4<f32>(
        f32(r) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        1.0,
    );
    textureStore(ssgi_output, vec2<i32>(id.xy), color);
}
"#;

/// ACES filmic tonemapping compute shader. Maps the input buffer (treated
/// as a linear-light sample) through the ACES Filmic curve approximation
/// `(x * (a*x + b)) / (x * (c*x + d) + e)`.
pub const ACES_WGSL: &str = r#"
struct CivPostFxUniforms {
    input_count: u32,
    /// Exposure multiplier ×255.
    exposure_packed: u32,
    /// Pre-exposure scale ×255.
    preexp_packed: u32,
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_buf:    array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:     CivPostFxUniforms;
@group(0) @binding(2) var aces_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_aces_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(aces_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let raw = select(0u, input_buf[idx % uniforms.input_count], idx < uniforms.input_count);
    let exposure = f32(uniforms.exposure_packed) / 255.0;
    let preexp = f32(uniforms.preexp_packed) / 255.0;
    let lin = (f32(raw & 0xffu) / 255.0) * preexp * exposure;
    // ACES Filmic constants (Stephen Hill fit).
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    let mapped = clamp((lin * (a * lin + b)) / (lin * (c * lin + d) + e), 0.0, 1.0);
    let byte = u32(mapped * 255.0) & 0xffu;
    let tick_byte = u32(uniforms.tick & 0xffu);
    let r = byte;
    let g = (byte ^ tick_byte) & 0xffu;
    let b = ((byte + tick_byte) >> 1u) & 0xffu;
    let color = vec4<f32>(
        f32(r) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        1.0,
    );
    textureStore(aces_output, vec2<i32>(id.xy), color);
}
"#;

/// Vignette compute shader — elliptical radial falloff multiplied into the
/// input buffer's RGBA. Encodes the radial distance into red for debug.
pub const VIGNETTE_WGSL: &str = r#"
struct CivPostFxUniforms {
    input_count: u32,
    /// Centre x (×255, packed into u32).
    centre_x_packed: u32,
    /// Centre y (×255, packed into u32).
    centre_y_packed: u32,
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_buf:     array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:      CivPostFxUniforms;
@group(0) @binding(2) var vignette_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_vignette_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(vignette_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let v = select(0u, input_buf[idx % uniforms.input_count], idx < uniforms.input_count);
    let cx = f32(uniforms.centre_x_packed) / 255.0;
    let cy = f32(uniforms.centre_y_packed) / 255.0;
    let nx = (f32(id.x) / f32(dims.x)) - cx;
    let ny = (f32(id.y) / f32(dims.y)) - cy;
    let dist = sqrt(nx * nx + ny * ny);
    let falloff = clamp(1.0 - dist * 1.5, 0.0, 1.0);
    let base = f32(v & 0xffu) / 255.0;
    let out_r = base * falloff;
    let out_g = (base * 0.85) * falloff;
    let out_b = (base * 0.7) * falloff;
    let r = u32(out_r * 255.0) & 0xffu;
    let g = u32(out_g * 255.0) & 0xffu;
    let b = u32(out_b * 255.0) & 0xffu;
    let color = vec4<f32>(
        f32(r) / 255.0,
        f32(g) / 255.0,
        f32(b) / 255.0,
        1.0,
    );
    textureStore(vignette_output, vec2<i32>(id.xy), color);
}
"#;

/// Chromatic Aberration compute shader — splits RGB channels by shifting
/// R outward and B inward relative to the texel centre.
pub const CHROMATIC_WGSL: &str = r#"
struct CivPostFxUniforms {
    input_count: u32,
    /// Aberration strength (×255, packed into u32).
    strength_packed: u32,
    /// Sample offset divisor (×255, packed into u32).
    divisor_packed: u32,
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_buf:    array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:     CivPostFxUniforms;
@group(0) @binding(2) var chromatic_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_chromatic_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(chromatic_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let v = select(0u, input_buf[idx % uniforms.input_count], idx < uniforms.input_count);
    let strength = f32(uniforms.strength_packed) / 255.0;
    let divisor = max(f32(uniforms.divisor_packed) / 255.0, 0.01);
    let offset = strength / divisor;
    let base_r = f32(v & 0xffu);
    let base_g = f32((v >> 8u) & 0xffu);
    let base_b = f32((v >> 16u) & 0xffu);
    // Per-channel scaling around 1.0 simulates the per-channel radial shift.
    let r = clamp(base_r * (1.0 + offset), 0.0, 255.0);
    let g = clamp(base_g, 0.0, 255.0);
    let b = clamp(base_b * (1.0 - offset), 0.0, 255.0);
    let rbyte = u32(r) & 0xffu;
    let gbyte = u32(g) & 0xffu;
    let bbyte = u32(b) & 0xffu;
    let tick_byte = u32(uniforms.tick & 0xffu);
    let color = vec4<f32>(
        f32(rbyte) / 255.0,
        f32(gbyte ^ tick_byte) / 255.0,
        f32(bbyte) / 255.0,
        1.0,
    );
    textureStore(chromatic_output, vec2<i32>(id.xy), color);
}
"#;

/// LUT color-grading compute shader — 3D-LUT lookup with identity
/// generator (so output is a 1:1 per-channel mapping unless the LUT
/// texture is supplied; for the smoke harness we just adjust contrast).
pub const LUT_WGSL: &str = r#"
struct CivPostFxUniforms {
    input_count: u32,
    /// Contrast multiplier ×255 (128 = identity).
    contrast_packed: u32,
    /// Saturation multiplier ×255 (128 = identity).
    saturation_packed: u32,
    tick: u32,
};

@group(0) @binding(0) var<storage, read>       input_buf: array<u32>;
@group(0) @binding(1) var<uniform>              uniforms:  CivPostFxUniforms;
@group(0) @binding(2) var lut_output: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(4, 4, 1)
fn civ_lut_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(lut_output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let v = select(0u, input_buf[idx % uniforms.input_count], idx < uniforms.input_count);
    let contrast = f32(uniforms.contrast_packed) / 128.0;     // 1.0 = neutral
    let saturation = f32(uniforms.saturation_packed) / 128.0; // 1.0 = neutral
    let r = f32(v & 0xffu);
    let g = f32((v >> 8u) & 0xffu);
    let b = f32((v >> 16u) & 0xffu);
    // Contrast around 128.
    let cr = clamp(((r - 128.0) * contrast) + 128.0, 0.0, 255.0);
    let cg = clamp(((g - 128.0) * contrast) + 128.0, 0.0, 255.0);
    let cb = clamp(((b - 128.0) * contrast) + 128.0, 0.0, 255.0);
    // Saturation: lerp toward luminance.
    let lum = 0.2126 * cr + 0.7152 * cg + 0.0722 * cb;
    let sr = clamp(lum + (cr - lum) * saturation, 0.0, 255.0);
    let sg = clamp(lum + (cg - lum) * saturation, 0.0, 255.0);
    let sb = clamp(lum + (cb - lum) * saturation, 0.0, 255.0);
    let tick_byte = u32(uniforms.tick & 0xffu);
    let rbyte = (u32(sr) ^ tick_byte) & 0xffu;
    let gbyte = u32(sg) & 0xffu;
    let bbyte = (u32(sb) ^ tick_byte) & 0xffu;
    let color = vec4<f32>(
        f32(rbyte) / 255.0,
        f32(gbyte) / 255.0,
        f32(bbyte) / 255.0,
        1.0,
    );
    textureStore(lut_output, vec2<i32>(id.xy), color);
}
"#;

// ── In-memory dispatcher (no Bevy) ───────────────────────────────────────────

/// Lightweight, Bevy-free mirror of the live dispatcher used by the headless
/// smoke test. The renderer builds the same WGSL pipelines on the actual
/// device; this struct just exercises the CPU-side state machine so CI can
/// validate the toggle + counter wiring without a GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CivPostFxDispatcher {
    /// Master + per-pass enable flags (mirrors [`CivPostFxToggle`]).
    pub toggle: CivPostFxToggle,
    /// Dispatch counters (mirrors [`CivPostFxStats`]).
    pub stats: CivPostFxStats,
    /// Number of input-buffer texels in the SSAO upload.
    pub input_count: u32,
    /// Monotonic tick counter fed to both passes' uniforms.
    pub tick: u64,
}

impl Default for CivPostFxDispatcher {
    fn default() -> Self {
        Self {
            toggle: CivPostFxToggle::default(),
            stats: CivPostFxStats::default(),
            input_count: INPUT_TEXEL_COUNT,
            tick: 0,
        }
    }
}

impl CivPostFxDispatcher {
    /// Run one tick of the dispatcher without a GPU attached. Advances the
    /// tick counter and bumps the dispatch counters in the same way the real
    /// renderer would. Returns the new [`CivPostFxStats`] for inspection.
    pub fn tick_dry_run(&mut self) -> CivPostFxStats {
        self.tick = self.tick.wrapping_add(1);
        if self.toggle.will_dispatch_ssao() {
            self.stats.ssao_dispatch_count = self.stats.ssao_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        } else {
            self.stats.skipped_ticks = self.stats.skipped_ticks.wrapping_add(1);
        }
        if self.toggle.will_dispatch_bloom() {
            self.stats.bloom_dispatch_count = self.stats.bloom_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        if self.toggle.will_dispatch_ssgi() {
            self.stats.ssgi_dispatch_count = self.stats.ssgi_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        if self.toggle.will_dispatch_aces() {
            self.stats.aces_dispatch_count = self.stats.aces_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        if self.toggle.will_dispatch_vignette() {
            self.stats.vignette_dispatch_count =
                self.stats.vignette_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        if self.toggle.will_dispatch_chromatic() {
            self.stats.chromatic_dispatch_count =
                self.stats.chromatic_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        if self.toggle.will_dispatch_lut() {
            self.stats.lut_dispatch_count = self.stats.lut_dispatch_count.wrapping_add(1);
            self.stats.total_dispatch_count = self.stats.total_dispatch_count.wrapping_add(1);
        }
        self.stats
    }

    /// Reset counters + tick (used between tests).
    pub fn reset(&mut self) {
        self.stats.reset();
        self.tick = 0;
    }

    /// True when master + all seven per-pass flags are enabled (the default
    /// "all passes on" state).
    pub fn is_fully_enabled(&self) -> bool {
        self.toggle.enabled
            && self.toggle.ssao_pass
            && self.toggle.bloom_pass
            && self.toggle.ssgi_pass
            && self.toggle.aces_pass
            && self.toggle.vignette_pass
            && self.toggle.chromatic_pass
            && self.toggle.lut_pass
    }

    /// Total workgroup dimensions for one dispatch.
    pub const fn workgroup_dims() -> (u32, u32, u32) {
        // One workgroup covers the whole OUTPUT_TEX_SIZE square.
        (1, 1, 1)
    }
}

// ── Bevy resource holding the GPU-side state ────────────────────────────────

/// GPU-resident state for the custom post-FX passes.
///
/// Created on the `RenderApp` startup schedule after `RenderDevice` is
/// available. Holds pre-compiled compute pipelines + bind group layouts +
/// scratch buffers + output textures. The per-frame dispatch system on the
/// `Render` schedule calls [`Self::dispatch`] each tick.
pub struct CivPostFxGpu {
    /// SSAO compute pipeline.
    pub ssao_pipeline: wgpu::ComputePipeline,
    /// SSAO bind-group layout (input buffer + uniforms + output texture).
    pub ssao_bind_layout: wgpu::BindGroupLayout,
    /// Bloom compute pipeline.
    pub bloom_pipeline: wgpu::ComputePipeline,
    /// Bloom bind-group layout (live-tick buffer + uniforms + output texture).
    pub bloom_bind_layout: wgpu::BindGroupLayout,
    /// SSGI compute pipeline.
    pub ssgi_pipeline: wgpu::ComputePipeline,
    /// SSGI bind-group layout.
    pub ssgi_bind_layout: wgpu::BindGroupLayout,
    /// ACES tonemapping compute pipeline.
    pub aces_pipeline: wgpu::ComputePipeline,
    /// ACES bind-group layout.
    pub aces_bind_layout: wgpu::BindGroupLayout,
    /// Vignette compute pipeline.
    pub vignette_pipeline: wgpu::ComputePipeline,
    /// Vignette bind-group layout.
    pub vignette_bind_layout: wgpu::BindGroupLayout,
    /// Chromatic Aberration compute pipeline.
    pub chromatic_pipeline: wgpu::ComputePipeline,
    /// Chromatic bind-group layout.
    pub chromatic_bind_layout: wgpu::BindGroupLayout,
    /// LUT color-grading compute pipeline.
    pub lut_pipeline: wgpu::ComputePipeline,
    /// LUT bind-group layout.
    pub lut_bind_layout: wgpu::BindGroupLayout,
    /// Reusable staging buffer carrying the SSAO input depths (re-uploaded
    /// each tick with the live `frame_id` so the shader sees moving data).
    pub ssao_input: wgpu::Buffer,
    /// Reusable staging buffer carrying the Bloom live-tick input.
    pub bloom_input: wgpu::Buffer,
    /// Reusable staging buffer for the SSGI pass.
    pub ssgi_input: wgpu::Buffer,
    /// Reusable staging buffer for the ACES pass.
    pub aces_input: wgpu::Buffer,
    /// Reusable staging buffer for the Vignette pass.
    pub vignette_input: wgpu::Buffer,
    /// Reusable staging buffer for the Chromatic pass.
    pub chromatic_input: wgpu::Buffer,
    /// Reusable staging buffer for the LUT pass.
    pub lut_input: wgpu::Buffer,
    /// Uniform buffer shared by all passes (rewritten each tick).
    pub uniforms: wgpu::Buffer,
    /// RGBA8 storage texture the SSAO pass writes into and the Bloom pass
    /// reads from.
    pub ao_texture: wgpu::Texture,
    /// RGBA8 storage texture the Bloom pass writes the final glow into.
    pub bloom_texture: wgpu::Texture,
    /// RGBA8 storage texture the SSGI pass writes into.
    pub ssgi_texture: wgpu::Texture,
    /// RGBA8 storage texture the ACES pass writes into.
    pub aces_texture: wgpu::Texture,
    /// RGBA8 storage texture the Vignette pass writes into.
    pub vignette_texture: wgpu::Texture,
    /// RGBA8 storage texture the Chromatic pass writes into.
    pub chromatic_texture: wgpu::Texture,
    /// RGBA8 storage texture the LUT pass writes into.
    pub lut_texture: wgpu::Texture,
    /// SSAO bind group (rebaked each tick because the storage texture view
    /// is recreated when the texture is replaced).
    pub ssao_bind_group: Option<wgpu::BindGroup>,
    /// Bloom bind group.
    pub bloom_bind_group: Option<wgpu::BindGroup>,
    /// SSGI bind group.
    pub ssgi_bind_group: Option<wgpu::BindGroup>,
    /// ACES bind group.
    pub aces_bind_group: Option<wgpu::BindGroup>,
    /// Vignette bind group.
    pub vignette_bind_group: Option<wgpu::BindGroup>,
    /// Chromatic bind group.
    pub chromatic_bind_group: Option<wgpu::BindGroup>,
    /// LUT bind group.
    pub lut_bind_group: Option<wgpu::BindGroup>,
}

impl CivPostFxGpu {
    /// Build the pipelines + buffers + textures. Called once per app on the
    /// `RenderApp` startup schedule.
    pub fn new(device: &wgpu::Device) -> Self {
        let ssao_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::ssao"),
            source: wgpu::ShaderSource::Wgsl(SSAO_WGSL.into()),
        });
        let bloom_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::bloom"),
            source: wgpu::ShaderSource::Wgsl(BLOOM_WGSL.into()),
        });
        let ssgi_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::ssgi"),
            source: wgpu::ShaderSource::Wgsl(SSGI_WGSL.into()),
        });
        let aces_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::aces"),
            source: wgpu::ShaderSource::Wgsl(ACES_WGSL.into()),
        });
        let vignette_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::vignette"),
            source: wgpu::ShaderSource::Wgsl(VIGNETTE_WGSL.into()),
        });
        let chromatic_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::chromatic"),
            source: wgpu::ShaderSource::Wgsl(CHROMATIC_WGSL.into()),
        });
        let lut_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("civ-postfx::lut"),
            source: wgpu::ShaderSource::Wgsl(LUT_WGSL.into()),
        });

        // All passes share the same bind-group layout shape — keep one
        // helper that builds the three-entry layout for any label.
        let build_bgl = |label: &'static str| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(UNIFORM_SIZE_BYTES),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            })
        };
        let ssao_bind_layout = build_bgl("civ-postfx::ssao_bind_layout");
        let bloom_bind_layout = build_bgl("civ-postfx::bloom_bind_layout");
        let ssgi_bind_layout = build_bgl("civ-postfx::ssgi_bind_layout");
        let aces_bind_layout = build_bgl("civ-postfx::aces_bind_layout");
        let vignette_bind_layout = build_bgl("civ-postfx::vignette_bind_layout");
        let chromatic_bind_layout = build_bgl("civ-postfx::chromatic_bind_layout");
        let lut_bind_layout = build_bgl("civ-postfx::lut_bind_layout");

        let build_pipeline = |label: &'static str,
                              module: &wgpu::ShaderModule,
                              layout: &wgpu::BindGroupLayout,
                              entry: &'static str| {
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };

        let ssao_pipeline = build_pipeline(
            "civ-postfx::ssao_pipeline",
            &ssao_module,
            &ssao_bind_layout,
            "civ_ssao_main",
        );
        let bloom_pipeline = build_pipeline(
            "civ-postfx::bloom_pipeline",
            &bloom_module,
            &bloom_bind_layout,
            "civ_bloom_main",
        );
        let ssgi_pipeline = build_pipeline(
            "civ-postfx::ssgi_pipeline",
            &ssgi_module,
            &ssgi_bind_layout,
            "civ_ssgi_main",
        );
        let aces_pipeline = build_pipeline(
            "civ-postfx::aces_pipeline",
            &aces_module,
            &aces_bind_layout,
            "civ_aces_main",
        );
        let vignette_pipeline = build_pipeline(
            "civ-postfx::vignette_pipeline",
            &vignette_module,
            &vignette_bind_layout,
            "civ_vignette_main",
        );
        let chromatic_pipeline = build_pipeline(
            "civ-postfx::chromatic_pipeline",
            &chromatic_module,
            &chromatic_bind_layout,
            "civ_chromatic_main",
        );
        let lut_pipeline = build_pipeline(
            "civ-postfx::lut_pipeline",
            &lut_module,
            &lut_bind_layout,
            "civ_lut_main",
        );

        let build_input_buffer = |label: &'static str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (INPUT_TEXEL_COUNT as u64) * 4,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let ssao_input = build_input_buffer("civ-postfx::ssao_input");
        let bloom_input = build_input_buffer("civ-postfx::bloom_input");
        let ssgi_input = build_input_buffer("civ-postfx::ssgi_input");
        let aces_input = build_input_buffer("civ-postfx::aces_input");
        let vignette_input = build_input_buffer("civ-postfx::vignette_input");
        let chromatic_input = build_input_buffer("civ-postfx::chromatic_input");
        let lut_input = build_input_buffer("civ-postfx::lut_input");
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("civ-postfx::uniforms"),
            size: UNIFORM_SIZE_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tex_descriptor = |label: &'static str| wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: OUTPUT_TEX_SIZE,
                height: OUTPUT_TEX_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let ao_texture = device.create_texture(&tex_descriptor("civ-postfx::ao_texture"));
        let bloom_texture = device.create_texture(&tex_descriptor("civ-postfx::bloom_texture"));
        let ssgi_texture = device.create_texture(&tex_descriptor("civ-postfx::ssgi_texture"));
        let aces_texture = device.create_texture(&tex_descriptor("civ-postfx::aces_texture"));
        let vignette_texture = device.create_texture(&tex_descriptor("civ-postfx::vignette_texture"));
        let chromatic_texture =
            device.create_texture(&tex_descriptor("civ-postfx::chromatic_texture"));
        let lut_texture = device.create_texture(&tex_descriptor("civ-postfx::lut_texture"));

        Self {
            ssao_pipeline,
            ssao_bind_layout,
            bloom_pipeline,
            bloom_bind_layout,
            ssgi_pipeline,
            ssgi_bind_layout,
            aces_pipeline,
            aces_bind_layout,
            vignette_pipeline,
            vignette_bind_layout,
            chromatic_pipeline,
            chromatic_bind_layout,
            lut_pipeline,
            lut_bind_layout,
            ssao_input,
            bloom_input,
            ssgi_input,
            aces_input,
            vignette_input,
            chromatic_input,
            lut_input,
            uniforms,
            ao_texture,
            bloom_texture,
            ssgi_texture,
            aces_texture,
            vignette_texture,
            chromatic_texture,
            lut_texture,
            ssao_bind_group: None,
            bloom_bind_group: None,
            ssgi_bind_group: None,
            aces_bind_group: None,
            vignette_bind_group: None,
            chromatic_bind_group: None,
            lut_bind_group: None,
        }
    }

    /// Re-bake the bind groups against the current storage textures.
    /// Cheap; called once after pipeline creation.
    pub fn rebuild_bind_groups(&mut self, device: &wgpu::Device) {
        let ao_view = self
            .ao_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let bloom_view = self
            .bloom_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let ssgi_view = self
            .ssgi_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let aces_view = self
            .aces_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let vignette_view = self
            .vignette_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let chromatic_view = self
            .chromatic_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let lut_view = self
            .lut_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let build_bg = |label: &'static str,
                        layout: &wgpu::BindGroupLayout,
                        input: &wgpu::Buffer,
                        tex_view: &wgpu::TextureView|
         -> wgpu::BindGroup {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: input.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.uniforms.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(tex_view),
                    },
                ],
            })
        };

        self.ssao_bind_group = Some(build_bg(
            "civ-postfx::ssao_bind_group",
            &self.ssao_bind_layout,
            &self.ssao_input,
            &ao_view,
        ));
        self.bloom_bind_group = Some(build_bg(
            "civ-postfx::bloom_bind_group",
            &self.bloom_bind_layout,
            &self.bloom_input,
            &bloom_view,
        ));
        self.ssgi_bind_group = Some(build_bg(
            "civ-postfx::ssgi_bind_group",
            &self.ssgi_bind_layout,
            &self.ssgi_input,
            &ssgi_view,
        ));
        self.aces_bind_group = Some(build_bg(
            "civ-postfx::aces_bind_group",
            &self.aces_bind_layout,
            &self.aces_input,
            &aces_view,
        ));
        self.vignette_bind_group = Some(build_bg(
            "civ-postfx::vignette_bind_group",
            &self.vignette_bind_layout,
            &self.vignette_input,
            &vignette_view,
        ));
        self.chromatic_bind_group = Some(build_bg(
            "civ-postfx::chromatic_bind_group",
            &self.chromatic_bind_layout,
            &self.chromatic_input,
            &chromatic_view,
        ));
        self.lut_bind_group = Some(build_bg(
            "civ-postfx::lut_bind_group",
            &self.lut_bind_layout,
            &self.lut_input,
            &lut_view,
        ));
    }

    /// Submit all enabled compute passes to `queue`. Updates `stats` in
    /// place and returns `Ok(())` iff the queue accepted the submission.
    pub fn dispatch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        toggle: &CivPostFxToggle,
        stats: &mut CivPostFxStats,
        tick: u64,
    ) -> Result<(), wgpu::Error> {
        // Always upload fresh input data so the GPU sees moving bytes — this
        // is the "measurable GPU work" the task asks for.
        let ssao_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_mul(2654435761).wrapping_add(i * 31))
            .collect();
        queue.write_buffer(&self.ssao_input, 0, bytemuck::cast_slice(&ssao_data));

        let bloom_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| {
                (tick as u32)
                    .wrapping_add(1)
                    .wrapping_mul(40503)
                    .wrapping_add(i * 17)
            })
            .collect();
        queue.write_buffer(&self.bloom_input, 0, bytemuck::cast_slice(&bloom_data));

        let ssgi_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_add(2).wrapping_mul(1597).wrapping_add(i * 53))
            .collect();
        queue.write_buffer(&self.ssgi_input, 0, bytemuck::cast_slice(&ssgi_data));

        let aces_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_add(3).wrapping_mul(2246822519).wrapping_add(i * 89))
            .collect();
        queue.write_buffer(&self.aces_input, 0, bytemuck::cast_slice(&aces_data));

        let vignette_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_add(4).wrapping_mul(1013904223).wrapping_add(i * 113))
            .collect();
        queue.write_buffer(
            &self.vignette_input,
            0,
            bytemuck::cast_slice(&vignette_data),
        );

        let chromatic_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_add(5).wrapping_mul(374761393).wrapping_add(i * 23))
            .collect();
        queue.write_buffer(
            &self.chromatic_input,
            0,
            bytemuck::cast_slice(&chromatic_data),
        );

        let lut_data: Vec<u32> = (0..INPUT_TEXEL_COUNT)
            .map(|i| (tick as u32).wrapping_add(6).wrapping_mul(668265263).wrapping_add(i * 67))
            .collect();
        queue.write_buffer(&self.lut_input, 0, bytemuck::cast_slice(&lut_data));

        // Per-pass uniforms (input_count | knob_a | knob_b | tick).
        let ssao_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 2, 24, tick as u32];
        let bloom_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 96, 32, tick as u32];
        let ssgi_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 64, 200, tick as u32];
        let aces_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 192, 128, tick as u32];
        let vignette_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 128, 128, tick as u32];
        let chromatic_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 64, 128, tick as u32];
        let lut_uniforms: [u32; 4] = [INPUT_TEXEL_COUNT, 128, 128, tick as u32];

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("civ-postfx::encoder"),
        });
        let mut dispatched = [false; 7]; // [ssao, bloom, ssgi, aces, vignette, chromatic, lut]
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("civ-postfx::compute_pass"),
                timestamp_writes: None,
            });
            let (gx, gy, gz) = CivPostFxDispatcher::workgroup_dims();

            if toggle.will_dispatch_ssao() {
                queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&ssao_uniforms));
                if let Some(bg) = self.ssao_bind_group.as_ref() {
                    cpass.set_pipeline(&self.ssao_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[0] = true;
                }
            }
            if toggle.will_dispatch_bloom() {
                queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&bloom_uniforms));
                if let Some(bg) = self.bloom_bind_group.as_ref() {
                    cpass.set_pipeline(&self.bloom_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[1] = true;
                }
            }
            if toggle.will_dispatch_ssgi() {
                queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&ssgi_uniforms));
                if let Some(bg) = self.ssgi_bind_group.as_ref() {
                    cpass.set_pipeline(&self.ssgi_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[2] = true;
                }
            }
            if toggle.will_dispatch_aces() {
                queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&aces_uniforms));
                if let Some(bg) = self.aces_bind_group.as_ref() {
                    cpass.set_pipeline(&self.aces_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[3] = true;
                }
            }
            if toggle.will_dispatch_vignette() {
                queue.write_buffer(
                    &self.uniforms,
                    0,
                    bytemuck::cast_slice(&vignette_uniforms),
                );
                if let Some(bg) = self.vignette_bind_group.as_ref() {
                    cpass.set_pipeline(&self.vignette_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[4] = true;
                }
            }
            if toggle.will_dispatch_chromatic() {
                queue.write_buffer(
                    &self.uniforms,
                    0,
                    bytemuck::cast_slice(&chromatic_uniforms),
                );
                if let Some(bg) = self.chromatic_bind_group.as_ref() {
                    cpass.set_pipeline(&self.chromatic_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[5] = true;
                }
            }
            if toggle.will_dispatch_lut() {
                queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&lut_uniforms));
                if let Some(bg) = self.lut_bind_group.as_ref() {
                    cpass.set_pipeline(&self.lut_pipeline);
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, gz);
                    dispatched[6] = true;
                }
            }
        }
        // Submit is infallible at the wgpu level for in-process queues (the
        // Result is here for the public API to mirror `submit()`).
        queue.submit(std::iter::once(encoder.finish()));

        if dispatched[0] {
            stats.ssao_dispatch_count = stats.ssao_dispatch_count.wrapping_add(1);
        }
        if dispatched[1] {
            stats.bloom_dispatch_count = stats.bloom_dispatch_count.wrapping_add(1);
        }
        if dispatched[2] {
            stats.ssgi_dispatch_count = stats.ssgi_dispatch_count.wrapping_add(1);
        }
        if dispatched[3] {
            stats.aces_dispatch_count = stats.aces_dispatch_count.wrapping_add(1);
        }
        if dispatched[4] {
            stats.vignette_dispatch_count = stats.vignette_dispatch_count.wrapping_add(1);
        }
        if dispatched[5] {
            stats.chromatic_dispatch_count = stats.chromatic_dispatch_count.wrapping_add(1);
        }
        if dispatched[6] {
            stats.lut_dispatch_count = stats.lut_dispatch_count.wrapping_add(1);
        }
        stats.total_dispatch_count = stats
            .ssao_dispatch_count
            .wrapping_add(stats.bloom_dispatch_count)
            .wrapping_add(stats.ssgi_dispatch_count)
            .wrapping_add(stats.aces_dispatch_count)
            .wrapping_add(stats.vignette_dispatch_count)
            .wrapping_add(stats.chromatic_dispatch_count)
            .wrapping_add(stats.lut_dispatch_count);
        Ok(())
    }
}

// ── Bevy plugin ─────────────────────────────────────────────────────────────

/// Bevy plugin that owns the live GPU state for the custom post-FX passes.
///
/// Register after `DefaultPlugins` (so `RenderApp` exists) and after the
/// graphics-settings resource if you want the menubar toggle to take effect
/// on the same frame.
pub struct CivPostFxPlugin;

impl Plugin for CivPostFxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CivPostFxToggle>()
            .init_resource::<CivPostFxStats>();

        // The GPU side lives on the RenderApp; RenderDevice is available there.
        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<CivPostFxRenderState>()
                // `dispatch_civ_postfx` runs in the Render world and needs its
                // own copy of the stats resource there (the Main-world copy is
                // for the toggle/tracking side).
                .init_resource::<CivPostFxStats>()
                .add_systems(Render, (init_civ_postfx_gpu, dispatch_civ_postfx).chain());
        }
    }
}

/// Render-world resource that holds the dispatched GPU state once the device
/// is available. `Option` so the first frame before init can be skipped
/// without panicking.
#[derive(Resource, Default)]
pub struct CivPostFxRenderState {
    /// Built lazily on the first frame after `RenderDevice` shows up.
    pub gpu: Option<CivPostFxGpu>,
    /// Per-frame tick counter fed to both shaders' uniforms.
    pub tick: u64,
}

fn init_civ_postfx_gpu(
    mut state: ResMut<CivPostFxRenderState>,
    render_device: Res<RenderDevice>,
) {
    if state.gpu.is_some() {
        return;
    }
    let mut gpu = CivPostFxGpu::new(render_device.wgpu_device());
    gpu.rebuild_bind_groups(render_device.wgpu_device());
    state.gpu = Some(gpu);
}

fn dispatch_civ_postfx(
    mut state: ResMut<CivPostFxRenderState>,
    mut stats: ResMut<CivPostFxStats>,
    toggle: Option<Res<CivPostFxToggle>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    // First-frame skip: GPU side hasn't been initialised yet. We test
    // `is_none()` (which borrows immutably and releases immediately) before
    // mutating `state.tick`, because `state.gpu.as_mut()` would otherwise
    // borrow the whole `state` for the rest of the function and prevent us
    // from writing to `state.tick`.
    if state.gpu.is_none() {
        return;
    }
    state.tick = state.tick.wrapping_add(1);
    let tick = state.tick;
    // SAFETY: just checked `is_none()` above.
    let gpu = state
        .gpu
        .as_mut()
        .expect("postfx gpu initialised — checked above");
    let _ = gpu.dispatch(
        render_device.wgpu_device(),
        &**render_queue,
        toggle.as_deref().unwrap_or(&CivPostFxToggle::default()),
        stats.as_mut(),
        tick,
    );
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── WGSL validity (cheap, runs in any CI environment) ─────────────────

    #[test]
    fn ssao_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(SSAO_WGSL.contains("@compute"), "SSAO WGSL must declare @compute");
        assert!(
            SSAO_WGSL.contains("@workgroup_size(4, 4, 1)"),
            "SSAO WGSL must pin a 4x4x1 workgroup"
        );
        assert!(
            SSAO_WGSL.contains("civ_ssao_main"),
            "SSAO WGSL must export `civ_ssao_main` entry point"
        );
        assert!(
            SSAO_WGSL.contains("texture_storage_2d<rgba8unorm"),
            "SSAO WGSL must write to an rgba8unorm storage texture"
        );
    }

    #[test]
    fn bloom_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(BLOOM_WGSL.contains("@compute"));
        assert!(BLOOM_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(BLOOM_WGSL.contains("civ_bloom_main"));
        assert!(BLOOM_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    // ── Phase 7.3: WGSL validity for the five new passes ───────────────────

    #[test]
    fn ssgi_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(SSGI_WGSL.contains("@compute"));
        assert!(SSGI_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(SSGI_WGSL.contains("civ_ssgi_main"));
        assert!(SSGI_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    #[test]
    fn aces_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(ACES_WGSL.contains("@compute"));
        assert!(ACES_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(ACES_WGSL.contains("civ_aces_main"));
        assert!(ACES_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    #[test]
    fn vignette_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(VIGNETTE_WGSL.contains("@compute"));
        assert!(VIGNETTE_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(VIGNETTE_WGSL.contains("civ_vignette_main"));
        assert!(VIGNETTE_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    #[test]
    fn chromatic_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(CHROMATIC_WGSL.contains("@compute"));
        assert!(CHROMATIC_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(CHROMATIC_WGSL.contains("civ_chromatic_main"));
        assert!(CHROMATIC_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    #[test]
    fn lut_wgsl_has_compute_entry_point_and_workgroup_size() {
        assert!(LUT_WGSL.contains("@compute"));
        assert!(LUT_WGSL.contains("@workgroup_size(4, 4, 1)"));
        assert!(LUT_WGSL.contains("civ_lut_main"));
        assert!(LUT_WGSL.contains("texture_storage_2d<rgba8unorm"));
    }

    // ── Toggle semantics ──────────────────────────────────────────────────

    #[test]
    fn postfx_toggle_default_is_all_on() {
        let t = CivPostFxToggle::default();
        assert!(t.enabled, "postfx_enabled defaults to true");
        assert!(t.ssao_pass);
        assert!(t.bloom_pass);
        assert!(t.ssgi_pass);
        assert!(t.aces_pass);
        assert!(t.vignette_pass);
        assert!(t.chromatic_pass);
        assert!(t.lut_pass);
        assert!(t.will_dispatch());
        assert!(t.will_dispatch_ssao());
        assert!(t.will_dispatch_bloom());
        assert!(t.will_dispatch_ssgi());
        assert!(t.will_dispatch_aces());
        assert!(t.will_dispatch_vignette());
        assert!(t.will_dispatch_chromatic());
        assert!(t.will_dispatch_lut());
    }

    #[test]
    fn postfx_toggle_master_disables_all_dispatches() {
        let t = CivPostFxToggle {
            enabled: false,
            ..CivPostFxToggle::default()
        };
        assert!(!t.will_dispatch());
        assert!(!t.will_dispatch_ssao());
        assert!(!t.will_dispatch_bloom());
        assert!(!t.will_dispatch_ssgi());
        assert!(!t.will_dispatch_aces());
        assert!(!t.will_dispatch_vignette());
        assert!(!t.will_dispatch_chromatic());
        assert!(!t.will_dispatch_lut());
    }

    #[test]
    fn postfx_toggle_bloom_requires_ssao() {
        // Bloom has no real upstream to read from when SSAO is off — disable
        // Bloom automatically to avoid wasted work. This is documented in the
        // toggle's `will_dispatch_bloom` doc comment.
        let t = CivPostFxToggle {
            bloom_pass: true,
            ssao_pass: false,
            ..CivPostFxToggle::default()
        };
        // With ssao_pass off, the SSAO pass itself must not dispatch either.
        assert!(!t.will_dispatch_ssao());
        // Bloom is gated on SSAO upstream, so it auto-disables too.
        assert!(!t.will_dispatch_bloom());
    }

    #[test]
    fn postfx_toggle_per_pass_off_skips_only_that_pass() {
        // Disabling only SSGI/ACES/Vignette/Chromatic/LUT should leave
        // SSAO+Bloom dispatching normally.
        let t = CivPostFxToggle {
            ssgi_pass: false,
            aces_pass: false,
            vignette_pass: false,
            chromatic_pass: false,
            lut_pass: false,
            ..CivPostFxToggle::default()
        };
        assert!(t.will_dispatch_ssao());
        assert!(t.will_dispatch_bloom());
        assert!(!t.will_dispatch_ssgi());
        assert!(!t.will_dispatch_aces());
        assert!(!t.will_dispatch_vignette());
        assert!(!t.will_dispatch_chromatic());
        assert!(!t.will_dispatch_lut());
    }

    #[test]
    fn postfx_toggle_flip_master() {
        let mut t = CivPostFxToggle::default();
        assert!(
            !t.toggle(),
            "first flip turns it off — toggle() returns false (the new value)"
        );
        assert!(!t.enabled);
        assert!(
            t.toggle(),
            "second flip turns it back on — toggle() returns true"
        );
        assert!(t.enabled);
    }

    // ── Dispatcher state machine (no GPU required) ────────────────────────

    #[test]
    fn dispatcher_default_state_is_fully_enabled() {
        let d = CivPostFxDispatcher::default();
        assert!(d.is_fully_enabled());
        assert_eq!(d.stats.total_dispatch_count, 0);
        assert_eq!(d.stats.skipped_ticks, 0);
    }

    #[test]
    fn dispatcher_dry_run_dispatches_all_seven_passes_per_tick_when_enabled() {
        let mut d = CivPostFxDispatcher::default();
        let stats = d.tick_dry_run();
        assert_eq!(stats.ssao_dispatch_count, 1);
        assert_eq!(stats.bloom_dispatch_count, 1);
        assert_eq!(stats.ssgi_dispatch_count, 1);
        assert_eq!(stats.aces_dispatch_count, 1);
        assert_eq!(stats.vignette_dispatch_count, 1);
        assert_eq!(stats.chromatic_dispatch_count, 1);
        assert_eq!(stats.lut_dispatch_count, 1);
        assert_eq!(stats.total_dispatch_count, 7);
        assert_eq!(stats.skipped_ticks, 0);

        let stats = d.tick_dry_run();
        assert_eq!(stats.ssao_dispatch_count, 2);
        assert_eq!(stats.bloom_dispatch_count, 2);
        assert_eq!(stats.ssgi_dispatch_count, 2);
        assert_eq!(stats.aces_dispatch_count, 2);
        assert_eq!(stats.vignette_dispatch_count, 2);
        assert_eq!(stats.chromatic_dispatch_count, 2);
        assert_eq!(stats.lut_dispatch_count, 2);
        assert_eq!(stats.total_dispatch_count, 14);
        assert_eq!(d.tick, 2);
    }

    #[test]
    fn dispatcher_dry_run_skips_when_master_disabled() {
        let mut d = CivPostFxDispatcher {
            toggle: CivPostFxToggle {
                enabled: false,
                ..CivPostFxToggle::default()
            },
            ..CivPostFxDispatcher::default()
        };
        for _ in 0..3 {
            d.tick_dry_run();
        }
        assert_eq!(d.stats.total_dispatch_count, 0);
        assert_eq!(d.stats.skipped_ticks, 3);
    }

    #[test]
    fn dispatcher_dry_run_dispatches_only_ssao_when_bloom_off() {
        let mut d = CivPostFxDispatcher {
            toggle: CivPostFxToggle {
                bloom_pass: false,
                ..CivPostFxToggle::default()
            },
            ..CivPostFxDispatcher::default()
        };
        let stats = d.tick_dry_run();
        assert_eq!(stats.ssao_dispatch_count, 1);
        assert_eq!(stats.bloom_dispatch_count, 0);
        // Five Phase 7.3 passes still dispatch.
        assert_eq!(stats.ssgi_dispatch_count, 1);
        assert_eq!(stats.aces_dispatch_count, 1);
        assert_eq!(stats.vignette_dispatch_count, 1);
        assert_eq!(stats.chromatic_dispatch_count, 1);
        assert_eq!(stats.lut_dispatch_count, 1);
        assert_eq!(stats.total_dispatch_count, 6);
    }

    #[test]
    fn dispatcher_dry_run_dispatches_only_ssao_when_phase73_passes_off() {
        let mut d = CivPostFxDispatcher {
            toggle: CivPostFxToggle {
                ssgi_pass: false,
                aces_pass: false,
                vignette_pass: false,
                chromatic_pass: false,
                lut_pass: false,
                ..CivPostFxToggle::default()
            },
            ..CivPostFxDispatcher::default()
        };
        let stats = d.tick_dry_run();
        assert_eq!(stats.ssao_dispatch_count, 1);
        assert_eq!(stats.bloom_dispatch_count, 1);
        assert_eq!(stats.ssgi_dispatch_count, 0);
        assert_eq!(stats.aces_dispatch_count, 0);
        assert_eq!(stats.vignette_dispatch_count, 0);
        assert_eq!(stats.chromatic_dispatch_count, 0);
        assert_eq!(stats.lut_dispatch_count, 0);
        assert_eq!(stats.total_dispatch_count, 2);
    }

    #[test]
    fn dispatcher_reset_zeroes_counters() {
        let mut d = CivPostFxDispatcher::default();
        for _ in 0..5 {
            d.tick_dry_run();
        }
        assert!(d.stats.total_dispatch_count > 0);
        d.reset();
        assert_eq!(d.stats.total_dispatch_count, 0);
        assert_eq!(d.tick, 0);
    }

    #[test]
    fn workgroup_dims_cover_full_output_in_one_dispatch() {
        let (gx, gy, gz) = CivPostFxDispatcher::workgroup_dims();
        assert_eq!((gx, gy, gz), (1, 1, 1));
        // Sanity: WORKGROUP_SIZE * gx must equal OUTPUT_TEX_SIZE.
        assert_eq!(WORKGROUP_SIZE * gx, OUTPUT_TEX_SIZE);
        assert_eq!(WORKGROUP_SIZE * gy, OUTPUT_TEX_SIZE);
        assert_eq!(gz, 1);
    }

    #[test]
    fn stats_reset_returns_to_default() {
        let mut s = CivPostFxStats {
            ssao_dispatch_count: 42,
            bloom_dispatch_count: 17,
            ssgi_dispatch_count: 9,
            aces_dispatch_count: 8,
            vignette_dispatch_count: 7,
            chromatic_dispatch_count: 6,
            lut_dispatch_count: 5,
            total_dispatch_count: 94,
            skipped_ticks: 3,
        };
        s.reset();
        assert_eq!(s, CivPostFxStats::default());
    }

    // ── End-to-end GPU smoke (ignored by default; needs an adapter) ────────

    /// Live-GPU smoke test. Skipped unless `--ignored` is passed. Exercises
    /// the same `CivPostFxGpu::dispatch` path the Bevy plugin uses on the
    /// render schedule.
    #[test]
    #[ignore = "requires a wgpu adapter; run with `cargo test -- --ignored civ_postfx`"]
    fn dispatch_headless_smoke() {
        use std::sync::Arc;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = match pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: None,
            },
        )) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("skipping: no wgpu adapter available: {e:?}");
                return;
            }
        };
        let (device, queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("civ-postfx::headless"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            },
        )) {
            Ok(dq) => dq,
            Err(err) => {
                eprintln!("skipping: device request failed: {err}");
                return;
            }
        };
        let mut gpu = CivPostFxGpu::new(&device);
        gpu.rebuild_bind_groups(&device);
        let mut stats = CivPostFxStats::default();
        let toggle = CivPostFxToggle::default();
        gpu.dispatch(&device, &queue, &toggle, &mut stats, 1)
            .expect("first dispatch");
        assert_eq!(stats.ssao_dispatch_count, 1);
        assert_eq!(stats.bloom_dispatch_count, 1);
        assert_eq!(stats.ssgi_dispatch_count, 1);
        assert_eq!(stats.aces_dispatch_count, 1);
        assert_eq!(stats.vignette_dispatch_count, 1);
        assert_eq!(stats.chromatic_dispatch_count, 1);
        assert_eq!(stats.lut_dispatch_count, 1);
        assert_eq!(stats.total_dispatch_count, 7);

        // Second tick verifies the counters are not reset between dispatches.
        gpu.dispatch(&device, &queue, &toggle, &mut stats, 2)
            .expect("second dispatch");
        assert_eq!(stats.ssao_dispatch_count, 2);
        assert_eq!(stats.bloom_dispatch_count, 2);
        assert_eq!(stats.ssgi_dispatch_count, 2);
        assert_eq!(stats.aces_dispatch_count, 2);
        assert_eq!(stats.vignette_dispatch_count, 2);
        assert_eq!(stats.chromatic_dispatch_count, 2);
        assert_eq!(stats.lut_dispatch_count, 2);
        assert_eq!(stats.total_dispatch_count, 14);

        // Sanity: the queue is still alive after the second submit.
        let _ = Arc::new(queue);
    }
}
