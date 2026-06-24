# ADR: Bevy Vulkan Primary Backend

**Status:** Accepted
**Date:** 2026-05-30

## Context

The Bevy reference client already documents native GPU backend selection and backend-specific escape hatches. The project also tracks Bevy as the reference client in the 3D traceability matrix. Relevant docs:

- [`docs/research/wgpu-native-escape-hatches.md`](../research/wgpu-native-escape-hatches.md)
- [`clients/bevy-ref/src/native_backend.rs`](../../clients/bevy-ref/src/native_backend.rs)
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md)

## Decision

Bevy will use Vulkan as the primary native backend on platforms where it is available, with backend selection constrained to native HAL paths rather than GLES or browser WebGPU. The backend selection policy should keep Vulkan first-class for the Bevy reference client while preserving the ability to use DX12 or Metal where platform constraints require them.

## Consequences

- Vulkan becomes the main target for backend-specific Bevy validation and performance work.
- Native backend selection remains explicit and platform-aware rather than relying on a generic portable fallback.
- Backend-specific code paths must continue to honor the HAL/`wgpu` boundary.
- Platform support remains broader than Vulkan alone, but Vulkan is the default reference for native backend behavior.

