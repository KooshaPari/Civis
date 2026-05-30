# Rich Doc-Embeds Pipeline — Build + R&D Report

**Date**: 2026-05-30
**Scope**: Phase-3 R&D + build of the reusable, cross-project rich-doc-embed
pipeline — Remotion-edited + annotated screenshots/recordings → mp4/gif
embedded *in* docs, fed by E2E tests + phenotype-journeys.
**Author**: pipeline build agent (Phase-3). A separate agent owns Phase-1 stub-boxes.
**Branch**: `feat/unityexplorer-devtools-20260528` (local commits only, no push).

---

## TL;DR

A generic, parameterized pipeline now lives at **`tools/doc-embeds/`**. One
Remotion composition renders *any* embed from a data-only `annotations.json`
(`EmbedSpec`). It is fed three ways: hand-authored specs, an E2E (Playwright)
capture helper, or a `phenotype-journeys` manifest adapter. A **working example
is wired end-to-end** for the DINOForge MODS quick panel and embedded into
`docs/proof/iter146-ui-rendering-proof.md`.

Produced artifacts (committed under `docs/embeds/media/`):
- `mods-quick-panel.mp4` (2.2 MB, 1280×720, ~7.5 s, annotated)
- `mods-quick-panel.gif` (4.5 MB, 0.5× scale, decimated)

Verified by extracting frames from the rendered mp4: callouts, highlight rect
with label, cursor ripple, Ken-Burns zoom, and caption bar all render correctly.

---

## 1. Research findings

### 1.1 Remotion (chosen for UI/screenshot embeds)

- **Programmatic, data-driven render.** Remotion is React-in-a-video. Input
  props are passed via the CLI `--props='{...}'` or `--props=./file.json` and
  read with `getInputProps()`. This is the lever that makes ONE composition
  generic — the embed is entirely described by props, no per-feature `.tsx`.
- **Dynamic duration/dimensions.** `calculateMetadata({ props })` on a
  `<Composition>` returns `durationInFrames/fps/width/height` derived from the
  spec, so each embed sizes itself. (Older projects hardcode `durationInFrames`
  per composition — our generic one does not.)
- **Existing video into Remotion**: prefer `<OffthreadVideo>` (FFmpeg C-API
  frame extraction; more reliable under render than `<Video>`). Stills use
  `<Img>`. Both must reference `staticFile()` paths under `public/`.
- **Annotations** are just absolutely-positioned React elements animated with
  `useCurrentFrame()` + `spring()` / `interpolate()` — callout boxes, highlight
  rects, cursor rings, captions, Ken-Burns zoom.
- **GIF output** via `--codec=gif` (`@remotion/gif`). Raw gifs are huge (38 MB
  for our clip); `--scale 0.5 --every-nth-frame 3` brought it to 4.5 MB.
- **Minimal project structure** (what we built):
  `src/index.tsx` (one `<Composition id="DocEmbed">` + `calculateMetadata`),
  `src/DocEmbed.tsx` (scene `<Series>` + caption bar), `src/components/*`
  (Callout/Highlight/Cursor/SceneView), `src/schema.ts` (the `EmbedSpec`),
  `bin/render.mjs` (CLI wrapper that stages assets + renders).
- **Browser gotcha (important, documented in the wrapper).** Remotion 4.0.467
  needs *old headless mode* = `chrome-headless-shell`. Full Chrome/Edge removed
  old headless and fail to launch. The headless-shell auto-download was flaky in
  this sandbox (`ECONNRESET`/stall). Fix: `bin/render.mjs` auto-detects a
  **Playwright `chrome-headless-shell`** (we have `chromium_headless_shell-1208`
  installed) and passes `--browser-executable`; `DOC_EMBEDS_BROWSER` overrides;
  falls back to managed download otherwise.

### 1.2 E2E capture in this repo (what already exists)

`grep` found substantial prior art:
- **`scripts/video/`** — a Remotion project, but **hardcoded per-feature**
  (`ModsButtonFeature`, `F9OverlayFeature`, fixed 300-frame durations, raw clip
  paths baked into components). Good component patterns (CalloutBox spring,
  CaptionBar) but **not reusable/parameterized**. Our `tools/doc-embeds/`
  generalizes it.
- **`scripts/companion-playwright/`** — Playwright config + specs
  (`docs-site.spec.ts`, `mcp-health.spec.ts`). Demonstrates the repo already
  runs Playwright; tests there can adopt our `bin/capture.mjs` convention.
- **`scripts/game/capture-feature-clips.ps1`** + `tests/unit/Test-CaptureFeatureClips.ps1`
  — PowerShell that captures in-game feature clips (the raw mp4s under
  `scripts/video/public/`). These are the upstream source for game embeds.

**Recommended emission contract** (implemented in `bin/capture.mjs`): each E2E
test writes `docs/embeds/captures/<id>/{recording.webm, keyframes/frame-NNN.png,
annotations.json}`. The generated `annotations.json` is a ready-to-render
`EmbedSpec`; hand-edit to refine highlight coords/timing, then `render.mjs`.

### 1.3 phenotype-journeys / journey records (the user's concept)

`tools/phenotype-journeys/` is the user's extracted, project-agnostic **journey
harness** (origin: HWLedger's `cli-journeys` VHS tapes + `JourneyViewer.vue` +
recorder crate, generalized). Relevant pieces:
- **Canonical manifest** (`schema/manifest.schema.json`): `id`, `intent`,
  `recording`, `recording_gif`, `steps[]` with `index/slug/intent/
  screenshot_path/description/judge_score/assertions/annotations[]`.
- **`annotations[]`** already carries `bbox [x,y,w,h]`, `label`, `color`,
  `style (solid|dashed)`, `kind (region|pointer|highlight)` — a near-perfect
  match for our Highlight/Cursor schema.
- **`npm/journey-playwright`** records a web flow → manifest (screenshot/step).
- **`npm/journey-viewer`** (Vue 3) renders a playback/gallery viewer in VitePress.
- **`crates/phenotype-journey-core`** (Rust) + CLI: `record/verify/validate/
  sync/assert` with a Claude-describe + Claude-judge loop and OCR ground-truth.

**How a journey becomes an embed** (implemented in `bin/from-journey.mjs`): map
`steps[].screenshot_path → scene.src`, `intent → callout.text`, `description →
callout.subText`, and each `annotations[]` bbox → a highlight rect (or cursor
ripple for `kind=pointer`). Result is an `EmbedSpec` rendered to mp4/gif. So the
journey harness (which verifies correctness) and the embed pipeline (which makes
it pretty for docs) share the same recording — record once, verify *and* embed.

### 1.4 SOTA scan — chosen stack

| Surface | Best tool | Why |
|---|---|---|
| **Terminal/CLI** demos | **VHS** (charmbracelet) + asciinema | Scriptable `.tape` files, CI-friendly, tiny output; asciinema for true-text recording. phenotype-journeys already wraps VHS. |
| **UI / screenshot** embeds | **Remotion** | Programmatic React video; annotations as components; data-driven via props; mp4+gif. Best fit for callouts/zoom/cursor over screenshots. |
| **GIF size** | `--scale`/`--every-nth-frame` (Remotion) or `agg` (asciinema) | Keep gifs < ~5 MB for docs. |
| **Verification** | phenotype-journey-core (Claude describe+judge + OCR assert) | Hard gates so a broken frame can't pass. |

Decision: **VHS for terminals, Remotion for UI** — both emit into the same
`docs/embeds/` media dir; both can be driven from a journey manifest. Animated
SVG (sharper/smaller than GIF) is a noted future optimization for pure-CLI
README badges but is out of scope for annotated UI video.

Sources: [Remotion passing-props/dynamic-metadata/OffthreadVideo docs](https://www.remotion.dev/docs/passing-props),
[asciinema for docs](https://dev.to/anderson_leite/asciinema-the-secret-weapon-for-better-documentation-training-and-handovers-1m9i),
[VHS / awesome-terminal-recorder](https://github.com/orangekame3/awesome-terminal-recorder),
[agg / asciicast2gif](https://github.com/asciinema/asciicast2gif),
[animated SVG vs GIF](https://dev.to/brpaz/make-your-project-readme-file-stand-out-with-animated-gifs-svgs-4kpe).

---

## 2. What was built (`tools/doc-embeds/`)

```
tools/doc-embeds/
  package.json            # @phenotype/doc-embeds, scripts: studio/render/render:*/build:example
  remotion.config.ts      # jpeg frames, overwrite, public dir
  tsconfig.json
  README.md               # authoring + cross-project drop-in guide
  src/
    schema.ts             # EmbedSpec contract (scenes, callouts, highlights, cursors, zoom, audio)
    index.tsx             # ONE generic <DocEmbed> Composition + calculateMetadata (dynamic size/duration)
    DocEmbed.tsx          # <Series> scene sequencer + caption bar; totalFrames()/sceneFrames()
    components/
      Callout.tsx         # spring-in title/subtitle box, 4 anchors, timed in/out
      Highlight.tsx       # HighlightRect (pulse/static, label) + Cursor (ripple/ring); source->canvas scaling
      SceneView.tsx       # OffthreadVideo|Img + Ken-Burns zoom + annotation layers
  bin/
    render.mjs            # CLI: stage assets into public/, resolve browser, render mp4/gif (auto-shrink gif)
    capture.mjs           # Capture class — E2E/Playwright helper emitting annotations.json + keyframes
    from-journey.mjs      # phenotype-journeys manifest -> EmbedSpec -> render
  examples/
    mods-quick-panel/
      annotations.json    # 2-scene annotated walkthrough
      assets/01-mods-button.png, 02-mods-panel.png
      out/                # (gitignored) rendered mp4+gif
```

Design choices:
- **Data-only contract.** `EmbedSpec` (in `src/schema.ts`) is the single input.
  Paths resolve relative to the `annotations.json` file → the whole spec+assets
  folder is portable to any repo.
- **One composition.** `calculateMetadata` derives size/fps/duration from the
  spec, so adding an embed = adding a JSON file, never new React.
- **Source-pixel annotation coords.** Highlight/cursor coords are authored in
  the source image's pixel space; `SceneView` maps them to canvas via a scale
  factor, so authors use coordinates straight off the screenshot.
- **Asset staging.** `render.mjs` copies referenced assets into Remotion's
  `public/` (required by `staticFile()`) and rewrites the spec paths, then
  renders — keeping source assets wherever they live in the repo.

---

## 3. Worked example (end-to-end, real)

Inputs: existing proof screenshots
`docs/screenshots/mods-button-FIXED-steamappid-20260529.png` (button injected)
and `docs/screenshots/iter146_mods_button_verified.png` (panel populated).

```bash
cd tools/doc-embeds
npm install
node ./bin/render.mjs --annotations ./examples/mods-quick-panel/annotations.json --format both
```

Output → `examples/mods-quick-panel/out/mods-quick-panel.{mp4,gif}`, copied to
**`docs/embeds/media/`** and embedded in
**`docs/proof/iter146-ui-rendering-proof.md`** (animated `<video>` + GIF
fallback + reproduce block).

The embed shows: Scene 1 — "MODS button injected" callout (top-right) + cursor
ripple + slow zoom; Scene 2 — "Quick panel opens" callout (top-left) + a pulsing
labeled highlight rect around the mod panel; persistent caption bar throughout.

---

## 4. How to reuse in another repo

1. Copy `tools/doc-embeds/` into the target repo.
2. `cd tools/doc-embeds && npm install`.
3. Author an `annotations.json` (+ assets) anywhere, OR generate one via
   `bin/capture.mjs` (Playwright E2E) or `bin/from-journey.mjs` (journey record).
4. `node ./bin/render.mjs --annotations <path> --format both`.
5. Copy the produced `out/*.mp4|gif` into that repo's docs media dir; embed with
   `<video>` (+ GIF fallback).

No DINOForge-specifics. Browser resolution reuses a Playwright headless-shell if
present (`DOC_EMBEDS_BROWSER` overrides), else Remotion's managed download.

Future consumers (per phenotype-journeys README): hwLedger, AgilePlus, thegent —
same drop-in. Pair with `phenotype-journey verify`/`assert` so an embed's source
recording is also a verified journey (record once → verify + embed).

---

## 5. Follow-ups / known limitations

- **Browser download flaky in sandbox** → mitigated by reusing Playwright's
  `chrome-headless-shell`. For CI, pre-install it or set `DOC_EMBEDS_BROWSER`.
- **Cross-resolution scenes**: annotation coords are per the spec's single
  `width/height` source space; scenes with differing native resolutions need
  coords expressed against that one space (documented). A future enhancement
  could allow per-scene `sourceWidth/Height`.
- **Audio/voiceover** wired in the schema (`audioSrc`) and `DocEmbed` but not
  exercised in the example.
- **Consolidation**: `scripts/video/` (hardcoded per-feature Remotion) should be
  migrated onto this generic pipeline and then retired to avoid two Remotion
  setups. Left intact for now to avoid colliding with other agents.

---

## Artifacts & paths

- Pipeline: `tools/doc-embeds/` (Remotion project + `bin/render.mjs`,
  `bin/capture.mjs`, `bin/from-journey.mjs`)
- Example spec: `tools/doc-embeds/examples/mods-quick-panel/annotations.json`
- Rendered embed: `docs/embeds/media/mods-quick-panel.mp4` (2.2 MB),
  `docs/embeds/media/mods-quick-panel.gif` (4.5 MB)
- Embedded in doc: `docs/proof/iter146-ui-rendering-proof.md`
- This report: `docs/embeds/PIPELINE-BUILD-20260530.md`

Commit SHA: _recorded after commit below._
