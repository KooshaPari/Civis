# Intent: FR-NFR-CIV-PORT-001 — Target platform matrix

> Date: 2026-09-20
> FR: FR-NFR-CIV-PORT-001
> Epic: FR-NFR-CIV-PORT

## What This FR Captures

The portability NFR that pins the platform/backend matrix the
codebase must compile and run on. Four configurations:

| Platform | GPU Backend | Build Target |
|----------|-------------|--------------|
| Windows 10/11 | Vulkan (primary) | `x86_64-pc-windows-msvc` |
| Windows 10/11 | DX12 (DLSS/Solari path) | `x86_64-pc-windows-msvc` |
| macOS 12+ | Metal | `aarch64-apple-darwin` |
| Linux (Ubuntu 22.04+) | Vulkan | `x86_64-unknown-linux-gnu` |

The full statement lives in
`docs/reference/non-functional-requirements.md:410-427`.

## User Intent

The active dev hardware is Windows RTX 3090 Ti + macOS M1; Linux
is required for CI and headless server. If a PR regresses any of
the four, the CI matrix catches it before it lands on `main`.

## Acceptance Signal

- All four configurations produce a `cargo build --release`
  success with zero errors in the CI matrix job
  `build/platform-matrix`.

## Implementing Code

- `docs/reference/non-functional-requirements.md:410` —
  statement.

## Test Coverage

- CI matrix job `build/platform-matrix` (see
  [`build/platform-matrix`](.github/workflows/)).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-civ-port-001-intent.md` |
| General spec | `docs/reference/non-functional-requirements.md` |