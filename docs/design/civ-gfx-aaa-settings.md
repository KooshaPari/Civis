# Civis — AAA Graphics Settings Panel (design)

**Status:** design. **Item:** gfx AAA settings + render-engine options.
**Builds on:** `clients/bevy-ref/src/graphics_settings.rs` (rich enum set already present),
`clients/bevy-ref/src/settings_ui.rs` (`GraphicsSettings` sub-struct, RON-persisted), and
`clients/bevy-ref/src/native_backend.rs` (`CIV_BEVY_BACKEND` backend selection).

Goal: a full, console-grade graphics options panel — including **render-engine options**
(backend, upscaler, RT) — that drives Bevy resources at runtime, persists to RON, and falls
back gracefully where the adapter can't support a feature.

## 1. What already exists (do not reinvent — extend)

`graphics_settings.rs` already defines:

| Enum | Variants | Maps to |
|------|----------|---------|
| `BackendPref` | Auto / Dx12 / Vulkan / Metal | `native_backend::CIV_BEVY_BACKEND` env (`env_token`) |
| `PresentMode` | Fifo / Mailbox / Immediate | `bevy::window::PresentMode` (`to_bevy`) |
| `QualityPreset` | Low / Medium / High / Ultra | master preset that drives the rest (`ALL`) |
| `ShadowResolution` | (tiers) | shadow-map `texels()` |
| `UpscalingMode` | Native / FSR / DLSS (tiers) | upscaler pipeline |
| `AaMode` | Off / MSAA / TAA | `msaa_samples()` → `Msaa` |
| `ToneCurve` | (filmic/etc) | `bevy::core_pipeline::tonemapping::Tonemapping` (`to_bevy`) |
| `WinMode` | Windowed / Borderless / Fullscreen | `bevy::window::WindowMode` (`to_bevy`) |

`settings_ui::GraphicsSettings` persists a subset and applies it to Bevy resources at runtime.

## 2. Settings matrix (target — full AAA)

One panel, categorized. `Apply` = live (system reacts to changed `GfxSettings`) or restart (needs
adapter/window re-init). Live changes are driven by a `react_to_gfx_changes` system watching
`Res<GfxSettings>` `is_changed()`.

| Category | Setting | Range / Options | Bevy target | Apply | Status |
|----------|---------|-----------------|-------------|-------|--------|
| Display | Resolution | adapter mode list | `Window.resolution` | live | add |
| Display | Window mode | `WinMode` | `Window.mode` | live | ✅ exists |
| Display | VSync / Present | `PresentMode` | `Window.present_mode` | live | ✅ exists |
| Display | Frame cap | Off / 30 / 60 / 120 / 144 / custom | `bevy_framepace` or `Time` limiter | live | add |
| Engine | Render backend | `BackendPref` (DX12/Vulkan/Metal/Auto) | `WgpuSettings.backends` via env | **restart** | ✅ exists |
| Engine | Upscaler | `UpscalingMode` Native/FSR/DLSS + scale | render-scale + upscale node | restart | ✅ enum, wire node |
| Engine | Ray-traced GI | Off / On | `bevy_solari` (`gi` feature) | restart | add (feature-gated) |
| Shadows | Cascade count | 1–4 | `DirectionalLightShadowMap` cascades | live | add |
| Shadows | Shadow resolution | `ShadowResolution` | `DirectionalLightShadowMap.size` | live | ✅ exists |
| Lighting | SSAO | Off / Low / High | `ScreenSpaceAmbientOcclusion` | live | add |
| Lighting | Bloom | Off / intensity | `Bloom` | live | add |
| AA | Anti-alias | `AaMode` Off/MSAA×{2,4,8}/TAA | `Msaa` / `TemporalAntiAliasing` | live (MSAA restart) | ✅ exists |
| AA | Tone mapping | `ToneCurve` | `Tonemapping` | live | ✅ exists |
| Textures | Texture quality | Low/Med/High/Ultra | mip bias / sampler | live | add |
| Textures | Anisotropic | Off / ×2..×16 | `ImageSamplerDescriptor.anisotropy_clamp` | restart (sampler) | add |
| World | Draw distance | meters / chunk rings | `voxel_stream` LOD ring count | live | add |
| World | LOD bias | -2..+2 | LOD selection threshold | live | add |
| FX | Particle density | 0–100% | `vfx` spawn-rate scalar | live | add |

`QualityPreset` (Low/Med/High/Ultra) sets all `add` rows to sensible defaults; "Custom" appears
once any individual control is touched.

## 3. `GfxSettings` resource (sketch)

```rust
// clients/bevy-ref/src/graphics_settings.rs — promote settings_ui::GraphicsSettings to a
// first-class Bevy Resource (Serialize/Deserialize for RON persistence).
#[derive(Resource, Clone, PartialEq, Serialize, Deserialize)]
pub struct GfxSettings {
    pub preset: QualityPreset,
    pub backend: BackendPref,
    pub present: PresentMode,
    pub win_mode: WinMode,
    pub frame_cap: FrameCap,            // new enum
    pub upscaler: UpscalingMode,
    pub rt_gi: bool,
    pub shadow_cascades: u8,            // 1..=4
    pub shadow_res: ShadowResolution,
    pub ssao: SsaoQuality,             // new enum
    pub bloom: f32,                    // 0.0 = off
    pub aa: AaMode,
    pub tone: ToneCurve,
    pub texture_quality: QualityTier,  // new enum
    pub anisotropy: u8,               // 1,2,4,8,16
    pub draw_distance_rings: u8,
    pub lod_bias: i8,                 // -2..=2
    pub particle_density: f32,         // 0.0..=1.0
}
```

**Persistence:** RON at the existing settings path (`settings_ui` already writes
`<config-dir>/civis/settings.ron`); `GfxSettings` becomes a section there. Load on startup,
save on panel "Apply".

## 4. Panel layout (egui)

`settings_ui` panel gains tabbed categories: **Display · Engine · Shadows · Lighting · Anti-alias
· Textures · World · FX**. Preset selector at top; per-row control + a "(restart required)" badge on
backend/upscaler/RT/MSAA/anisotropy. Buttons: Apply (writes resource + RON), Revert, Reset-to-preset.

## 5. Apply policy + graceful fallback

- **Live** settings: a `react_to_gfx_changes` system on `Update` reads `Res<GfxSettings>` when
  `is_changed()` and patches the matching Bevy resource (`Msaa`, `Bloom`, `ScreenSpaceAmbientOcclusion`,
  `DirectionalLightShadowMap`, LOD ring count, particle scalar).
- **Restart** settings (backend, RT-GI, MSAA toggle, anisotropy sampler): write to RON + show a
  "restart to apply" toast; applied on next launch via `native_wgpu_settings()` + plugin gating.
- **Fallback:** every feature probes capability first (extend `gpu_features::detect_capabilities`):
  if the adapter rejects RT/DLSS/MSAA-level, the control is disabled with a tooltip and the value
  clamps to the nearest supported — never a hard panic (per project "fail clearly, not silently":
  surface the unsupported item in the tooltip).

## 6. Phased plan

1. Promote `GraphicsSettings` → `GfxSettings` Resource + add the new enums (`FrameCap`, `SsaoQuality`,
   `QualityTier`); RON load/save. (no rendering change yet)
2. `react_to_gfx_changes` live-apply system for the ✅/live rows (MSAA, bloom, shadows, tone, present, window).
3. Add the "add" live rows: SSAO, frame cap, draw distance, LOD bias, particle density.
4. Restart-tier wiring: RT-GI (`gi` feature gate), upscaler node, anisotropy sampler, backend (already env).
5. egui tabbed panel + preset defaults + capability-gated disable/tooltip.
6. Verify: `CIVIS_DUMP` reports active `GfxSettings`; toggle each live row and confirm the bound Bevy
   resource changed (programmatic), not by eye.

Charter-neutral (pure rendering/UX; no simulation law).
