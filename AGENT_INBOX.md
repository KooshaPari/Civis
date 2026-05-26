TL;DR
- Repo B has an existing but incomplete Vue3 `@phenotype/journey-viewer` package under `tools/phenotype-journeys/npm/journey-viewer/`.
- Repo A has 10 journey manifests under `docs/journeys/manifests/us-wsm-phase-*/manifest.json` with no capture/video/overlay fields yet.
- Convergence target: complete the shared viewer in this repo, then have WorldSphereMod consume it.

Shared need
- One cross-repo journey-records docs UI across both ecosystems with support for:
  - recorded videos
  - keyframe PNG sets
  - per-step screenshots
  - OCR overlays
  - optional SVG annotations
- Stable schema so both repos can author/integrate the same journey artifacts.

Prior art summary
- WorldSphereMod status:
  - `10` manifests exist in `docs/journeys/manifests/us-wsm-phase-*/manifest.json`.
  - manifest text scan does not show `captures`, `keyframes`, `screenshots`, `ocr`, `annotations`, or `svg` fields.
  - no local viewer implementation present in this repo root.
- Dino journey-viewer status:
  - Vue3 package directory exists with component surface (`src/JourneyViewer.vue`, `src/JourneyStep.vue`, `src/KeyframeGallery.vue`, `src/RecordingEmbed.vue`, `src/Shot.vue`, `src/types.ts`, etc.).
  - published tarball + `bun.lock` indicates prior packaging work but viewer is currently incomplete for all requested features.
- HWLedger-like repo scan:
  - no confirmed result in this pass; use `HWLedger location TBD`.

Proposed convergence
- Consolidate journey UI into a single shared library at:
  - `C:\Users\koosh\Dino\tools/phenotype-journeys/npm/journey-viewer/`
- Define and enforce a shared data contract for:
  - journey metadata
  - media timeline
  - keyframe arrays
  - per-step screenshot list
  - OCR results
  - optional SVG overlays
- Have WorldSphereMod consume the library with docs-facing integration only (no duplicate UI implementation).
- Link both notes for coordination:
  - `../Dev/WorldSphereMod/AGENT_INBOX.md`

Next actions for your current Dino-side session (3-5 concrete steps)
1. Finish/normalize the schema in `src/types.ts` and add strict validation defaults.
2. Implement the full render paths in components: video player, keyframe gallery, step screenshot timeline, OCR overlay layer, and optional SVG annotation overlay layer.
3. Define the package public API (props/events) and bump package metadata to `@phenotype/journey-viewer`.
4. Add a small fixture/example journey payload demonstrating all layers for manual smoke check.
5. Provide an integration contract note for WorldSphereMod and the expected manifest migration path.

Tech-stack hints
- Backend/tooling generators: prefer Rust or Go for reliable fast artifact generation/validation pipelines.
- UI surface: Vue3 + TanStack for list/selection/state composition and composable viewer blocks.
