# @phenotype/doc-embeds

Generic, **cross-project** rich-doc-embed pipeline. Turns screenshots and/or a
screen recording + an **annotations.json** (captions, highlight rects, cursor
rings, timing, zoom) into an **annotated mp4 + gif** ready to embed in docs.

One Remotion composition renders *any* embed — the entire embed is described by
data (the `EmbedSpec`), passed as input props. No per-feature React code.

```
tools/doc-embeds/
  src/
    schema.ts            # EmbedSpec — the only input contract
    index.tsx            # single generic <DocEmbed> composition (+ calculateMetadata)
    DocEmbed.tsx         # scene sequencer + caption bar
    components/          # Callout, HighlightRect, Cursor, SceneView
  bin/
    render.mjs           # CLI: annotations.json -> mp4/gif (stages assets, picks browser)
    capture.mjs          # E2E (Playwright) capture helper -> emits annotations.json
    from-journey.mjs     # phenotype-journeys manifest -> EmbedSpec -> render
  examples/
    mods-quick-panel/    # working DINOForge example (annotations.json + assets)
```

## Quickstart

```bash
cd tools/doc-embeds
npm install
node ./bin/render.mjs --annotations ./examples/mods-quick-panel/annotations.json --format both
# -> examples/mods-quick-panel/out/mods-quick-panel.{mp4,gif}
```

`--format` = `mp4` | `gif` | `both`. GIFs are auto-scaled to 0.5x and decimated
(`--gif-scale`, `--gif-every-nth` to override) to stay doc-friendly.

## Authoring an embed

Write an `annotations.json` (see `src/schema.ts` for the full contract):

```json
{
  "id": "my-feature",
  "title": "My Feature",
  "subtitle": "v1.2.3",
  "accent": "#34d399",
  "width": 1280, "height": 720, "fps": 30,
  "scenes": [
    {
      "src": "shots/step-1.png",
      "holdSec": 3.5,
      "zoom": [1.0, 1.06],
      "callouts": [{ "text": "Click Run", "atSec": 0.4, "anchor": "top-right" }],
      "cursors":  [{ "x": 840, "y": 525, "atSec": 1.2, "kind": "ripple" }]
    },
    {
      "src": "clips/result.mp4",
      "clipSec": 5,
      "highlights": [{ "x": 760, "y": 380, "width": 1040, "height": 680,
                       "label": "Result", "atSec": 0.7, "style": "pulse" }]
    }
  ]
}
```

Paths in `src`/`audioSrc` resolve **relative to the annotations.json file**.
A scene `src` ending in `.mp4/.mov/.webm/.mkv` is treated as video; anything
else is a held still. Highlight/cursor coords are in **source-image pixels**.

## Feeding it from E2E tests (Playwright)

`bin/capture.mjs` is the convention. In a `*.spec.ts`:

```ts
import { Capture } from "../tools/doc-embeds/bin/capture.mjs";

const cap = new Capture({ id: "mods-quick-panel", captureDir: "docs/embeds/captures" });
await page.goto(url);
await cap.step(page, "App launched");
await page.getByRole("button", { name: "MODS" }).click();
await cap.step(page, "MODS panel opens", { highlight: { x: 760, y: 380, width: 1040, height: 680, label: "Mod panel" } });
await cap.finish({ title: "MODS Quick Panel", accent: "#34d399" });
// writes docs/embeds/captures/mods-quick-panel/{keyframes,annotations.json}
```

Then render: `node bin/render.mjs --annotations docs/embeds/captures/mods-quick-panel/annotations.json`.
Enable `use: { video: "on" }` in `playwright.config` and pass `recordingSrc`
to `finish()` to also stash the session `.webm`.

## Feeding it from a journey record

`bin/from-journey.mjs` adapts a [`phenotype-journeys`](../phenotype-journeys)
`manifest.json` (steps with `intent`, `description`, bbox `annotations`) into an
`EmbedSpec` and optionally renders it:

```bash
node ./bin/from-journey.mjs --manifest path/to/manifests/<id>/manifest.json --render
```

`annotations[].kind` maps: `pointer` -> cursor ring, `region`/`highlight` ->
highlight rect. `intent`/`description` become the callout title/subtext.

## Embedding in docs (VitePress / GitHub markdown)

```md
<video src="../embeds/media/my-feature.mp4" controls loop muted width="720"></video>

![fallback](../embeds/media/my-feature.gif)
```

## Dropping into another repo

1. Copy the `tools/doc-embeds/` folder.
2. `npm install`.
3. Put an `annotations.json` + its assets anywhere; run `bin/render.mjs --annotations <path>`.
4. Copy the produced `out/*.mp4|gif` into that repo's docs media dir and embed.

Nothing is DINOForge-specific. The browser resolver reuses a Playwright
`chrome-headless-shell` if present (set `DOC_EMBEDS_BROWSER` to override), else
falls back to Remotion's managed browser download.
