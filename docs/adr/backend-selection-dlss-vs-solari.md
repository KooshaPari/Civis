# ADR: Backend selection — DLSS (Vulkan framing) vs Solari (DX12) on Windows

Status: Accepted
Date: 2026-09-24
Trace: NFR-CIV-PORT-002 (FR-NFR-CIV-PORT-002)

## Context

NFR-CIV-PORT-002 (`docs/reference/non-functional-requirements.md`, §PORT-002)
requires this ADR: the two headline Windows features pull toward different
GPU backends, and without an explicit selection rule agent-driven work will
arbitrarily pick one and silently disable the other.

- **DLSS-requires-Vulkan (as framed by the NFR).** The NFR statement frames
  the Bevy upscaling DLSS plugin path as requiring Vulkan.
- **Solari-requires-DX12.** Bevy's in-tree global illumination
  (`bevy/bevy_solari`, enabled by the `solari` cargo feature of
  `civ-bevy-ref`) needs ray-tracing acceleration structures: DXR on DX12,
  or `VK_KHR_ray_tracing` on Vulkan. The NFR records the two framings as
  mutually exclusive on Windows.

The codebase of record reconciles them as follows.

## Decision — selection policy

1. **Windows default = DX12.** `clients/bevy-ref/src/native_backend.rs`
   restricts the Windows wgpu adapter search to DX12 (overriding wgpu's
   Vulkan-first preference) so the DXR + DLSS path is the landed backend.
   `CIV_BEVY_BACKEND=vulkan` forces the Vulkan fallback explicitly;
   `CIV_BEVY_BACKEND=metal`/`dx12` select the other sanctioned backends.
2. **DLSS is runtime-classified, not a cargo feature.** There is no
   `dlss` cargo feature on `civ-bevy-ref`; `clients/bevy-ref/src/gpu_features.rs`
   sets `dlss_available` only for NVIDIA GPUs on DX12 and reports FSR as
   the portable upscaling fallback elsewhere.
3. **`solari` cargo feature stays backend-honest.** The feature pulls
   `bevy/bevy_solari`; `SolariGiPlugin` degrades to a loud, logged no-op
   when no ray-tracing-capable backend is present (see the feature comment
   in `clients/bevy-ref/Cargo.toml`).

## Features unavailable on the non-selected backend

- **Vulkan selected (Windows fallback / Linux primary):** DLSS is
  unavailable — `gpu_features.rs` classifies `dlss_available` as DX12 +
  NVIDIA only. Use FSR as the upscaling fallback.
- **DX12 selected (Windows default):** MetalFX is unavailable (macOS-only
  path); non-NVIDIA GPUs additionally lose DLSS but keep FSR.
- **Metal (macOS):** neither DLSS nor Solari ray tracing is available;
  both degrade per policy 3.

## Consequences

- Feature-flag readers get the policy at the flag site: the
  `clients/bevy-ref/Cargo.toml` feature documentation references this ADR.
- `civ_server::portability::BACKEND_SELECTION_ADR` names this file as the
  single source of truth; CI doc lints assert the file exists, is
  substantive (≥ 200 words), and is referenced from the manifest.
- Related: `docs/adr/ADR-bevy-vulkan-primary-backend.md` (Vulkan as the
  portable primary fallback) and NFR-CIV-PORT-001's platform matrix.
