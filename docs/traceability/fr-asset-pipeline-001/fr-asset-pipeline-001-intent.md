# Intent: FR-ASSET-PIPELINE-001 — Asset pipeline scaffold

> Date: 2026-09-20
> FR: FR-ASSET-PIPELINE-001
> Epic: FR-ASSET-PIPELINE

## What This FR Captures

The `asset-pipeline` crate's initial scaffold — a working skeleton of
[`asset_pipeline::export_svg`] plus the [`ExportError`] enum — that
validates the crate's API contract and error path before any external
encoder is wired in. Encoder wiring (resvg → PNG, image → ICO, codec →
WEBM) is gated on the first-asset PR per FR-ASSET-PIPELINE-001.

## User Intent

A working scaffold is needed to lock in the crate's public API and
error surface before downstream code starts depending on it.

## Acceptance Signal

- `cargo build -p asset-pipeline` succeeds with the stub.
- `export_svg` validates input/output paths and returns
  `ExportError::Encode` until real encoders are wired.
- The CLI binary `svg_export` (feature `cli`) compiles.

## Implementing Code

- `crates/asset-pipeline/src/lib.rs:3` — crate header w/ FR ref
- `crates/asset-pipeline/src/lib.rs:18` — first-asset-gating note
- `crates/asset-pipeline/src/lib.rs:57` — scaffold stub returning
  `ExportError::Encode`

## Test Coverage

> Scaffold only; encoder test matrix lands with the first-asset PR.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-asset-pipeline-001-intent.md` |
| Implementing crate | `crates/asset-pipeline/src/` |