# Asset Intake PRD (`assetctl`)

**Version**: 0.1.0  
**Status**: Draft  
**Owner**: docs/warfare-designer (pre-impl domain owner)  
**Created**: 2026-03-11  

## 1. Purpose

Build a deterministic, machine-driven intake pipeline for Star-Wars-style prototype assets that keeps technical viability and IP provenance separate.

The PRD scope is pre-implementation planning for a CLI-first tool chain and required data contracts; no runtime pack application logic is included in this phase.

## 2. Non-Goals

- No legal claims about public Star Wars compatibility.
- No full Unity asset optimization tooling in V1.
- No marketplace publishing or distribution controls in V1.
- No automatic source-specific scraping for every site.

## 3. Design Goals

- Machine-friendly discovery first (API over scrape over browser automation).
- Strict manifest-first asset lifecycle.
- Explicit provenance state machine.
- Shared risk and quality gates across agents and runs.
- Reproducible directory and file naming conventions.

## 4. Directory Model

```text
/assets-pipeline
  /sources
    /sketchfab
    /blendswap
    /moddb
  /registry
    asset_index.json
    provenance_index.json
  /raw
    /sw_b1_droid_sketchfab_001
      source_download.glb
      metadata.json
      asset_manifest.json
  /working
    /sw_b1_droid_sketchfab_001
      normalized.blend
      normalized.glb
      preview.png
      validation_report.json
  /export
    /unity
      /units
      /vehicles
      /props
  /logs
```

## 5. Core CLI Contract

`assetctl` is organized around a single noun and explicit lifecycle verbs.

- `assetctl search --source <tier> "<query>" [--limit N]`
- `assetctl intake <candidate_ref>`
- `assetctl normalize <asset_id>`
- `assetctl validate <asset_id>`
- `assetctl stylize <asset_id> --profile dinosw_lowpoly_v1`
- `assetctl register <asset_id>`
- `assetctl export-unity <asset_id> [--bundle unit|vehicle|prop]`

All commands output JSON by default when `--format json` is set.  
Machine parsing must not depend on colored terminal output.

## 6. Source Adapter Model

All source adapters implement a common contract:

- `source_name()`
- `search(query, filters) -> candidates`
- `fetch_metadata(external_id)`
- `can_download(external_id) -> bool`
- `download(external_id, out_dir) -> file list`
- `estimate_poly_and_style_score(candidate) -> score metadata`

Planned adapters:

- `SketchfabAdapter` (API-first, GLB/GTLF/USdz output).
- `BlendSwapAdapter` (scrape-assisted with cache of downloaded assets).
- `ModDbAdapter` (reference-only legacy source).
- `PlaywrightFallbackAdapter` (manual pages only).

## 7. Candidate Lifecycle

Technical and IP state are independent:

- Technical: `discovered`, `metadata_fetched`, `downloadable`, `downloaded`, `normalized`, `validated`, `ready_for_prototype`, `rejected_technical`.
- IP: `generic_safe`, `fan_star_wars_private_only`, `official_game_mod_tool_reference_only`, `unknown_provenance`, `high_risk_do_not_ship`.

An asset may progress technical stages while remaining blocked by IP state.

## 8. Scoring Model

Suggested initial ranking:

`0.30*style_fit + 0.20*faction_fit + 0.15*automation_ease + 0.10*topology_quality + 0.10*low_poly_fit + 0.10*provenance_confidence - 0.15*ip_risk_penalty - 0.10*cleanup_cost`

Score thresholds are enforced by policy and can be tuned in `manifests/asset-intake/source-rules.yaml`.

## 9. Policy and Data Contracts

- Manifest schema: `schemas/asset-manifest.schema.json`
- Source/risk policy: `manifests/asset-intake/source-rules.yaml`
- Validation report schema: defined in normalization worker and Unity export outputs.

## 10. V1 Scope (Pre-Impl Slice)

- Source support: Sketchfab only
- Candidate classes:
  - clone trooper
  - B1 battle droid
  - stormtrooper
  - TIE fighter
  - X-wing
  - AAT tank
  - desert / outer-rim structure props
- Outputs:
  - manifest
  - raw download
  - normalized GLB
  - preview
  - validation report

## 11. Open Decisions

- Whether to add an offline mirror cache for downloaded zip payloads in V1.
- Whether `normalize` performs geometry decimation internally or invokes external Blender profiles.
- Whether legal risk overrides are stored per pack or global policy.
