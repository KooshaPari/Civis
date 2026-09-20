# Intent: FR-ASSET-PIPELINE-002 — WEBM codec integration

> Date: 2026-09-20
> FR: FR-ASSET-PIPELINE-002
> Epic: FR-ASSET-PIPELINE

## What This FR Captures

The planned WEBM video codec integration into `asset-pipeline`. PNG
(1x/2x/3x) and ICO encoders are wired today via `resvg` and `image`;
WEBM is the third output format required by the asset brief and is
still TBD. Code that mentions this FR today flags the codec as
"not yet wired" — e.g. `ExportError::Encode` and the CLI's `about`
string.

## User Intent

A single `export_svg` call should produce all raster + icon + video
variants. Today PNG/ICO work; WEBM is deferred to this FR.

## Acceptance Signal

- `cargo run -p asset-pipeline --features cli --bin svg_export`
  produces `.webm` files alongside `.png` and `.ico` once the codec
  is wired.
- `ExportError::Encode` is no longer triggered by WEBM-specific
  failures (a richer error variant may replace it).

## Implementing Code

- `crates/asset-pipeline/Cargo.toml:8` — crate description lists
  WEBM as gated behind this FR
- `crates/asset-pipeline/src/bin/svg_export.rs:20` — CLI `about`
  string references this FR
- `crates/asset-pipeline/src/error.rs:9` — module header notes the
  WEBM gap

## Test Coverage

> No tests yet — pending codec selection.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-asset-pipeline-002-intent.md` |
| Implementing crate | `crates/asset-pipeline/src/` |