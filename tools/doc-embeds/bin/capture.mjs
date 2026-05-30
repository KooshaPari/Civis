#!/usr/bin/env node
/**
 * Capture helper / convention for E2E tests feeding the doc-embed pipeline.
 *
 * This is a thin, framework-agnostic library that E2E tests (Playwright) call
 * to emit capture artifacts into a known layout the renderer consumes:
 *
 *   <captureDir>/<id>/
 *     recording.webm        (full session video, if the test enabled it)
 *     keyframes/
 *       frame-001.png ...   (per-step screenshots)
 *     annotations.json      (auto-generated EmbedSpec stub from steps)
 *
 * The generated annotations.json is a ready-to-render EmbedSpec: each captured
 * keyframe becomes a held scene with a callout = the step intent. Hand-edit it
 * to add highlight rects / cursor rings / timing, then:
 *
 *   node bin/render.mjs --annotations <captureDir>/<id>/annotations.json
 *
 * --- Playwright usage (in a *.spec.ts) ---
 *
 *   import { Capture } from "../tools/doc-embeds/bin/capture.mjs";
 *   const cap = new Capture({ id: "mods-quick-panel", captureDir: "docs/embeds/captures" });
 *   await page.goto("...");
 *   await cap.step(page, "App launched");
 *   await page.getByRole("button", { name: "MODS" }).click();
 *   await cap.step(page, "MODS panel opens");
 *   await cap.finish({ title: "MODS Quick Panel", accent: "#34d399" });
 *
 * Enable video in playwright.config: use: { video: "on" } — then copy the
 * resulting .webm into <captureDir>/<id>/recording.webm (or pass recordingSrc
 * to finish()).
 */
import { mkdirSync, writeFileSync, copyFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

export class Capture {
  constructor({ id, captureDir = "docs/embeds/captures", sourceWidth = 1280, sourceHeight = 800 }) {
    this.id = id;
    this.dir = resolve(process.cwd(), captureDir, id);
    this.keyframes = join(this.dir, "keyframes");
    this.sourceWidth = sourceWidth;
    this.sourceHeight = sourceHeight;
    this.steps = [];
    mkdirSync(this.keyframes, { recursive: true });
  }

  /**
   * Capture one step. `page` is a Playwright Page (or anything with a
   * .screenshot({ path }) method). Returns the relative keyframe path.
   */
  async step(page, intent, opts = {}) {
    const n = this.steps.length + 1;
    const file = `frame-${String(n).padStart(3, "0")}.png`;
    const abs = join(this.keyframes, file);
    if (page && typeof page.screenshot === "function") {
      await page.screenshot({ path: abs, fullPage: false });
    }
    this.steps.push({ intent, file, holdSec: opts.holdSec ?? 3, highlight: opts.highlight });
    return join("keyframes", file);
  }

  /**
   * Write annotations.json (an EmbedSpec) from the captured steps.
   * Optionally copy a session recording into recording.webm.
   */
  async finish({ title, subtitle, accent = "#34d399", recordingSrc } = {}) {
    if (recordingSrc && existsSync(recordingSrc)) {
      copyFileSync(recordingSrc, join(this.dir, "recording.webm"));
    }
    const spec = {
      id: this.id,
      title: title ?? this.id,
      subtitle,
      accent,
      width: this.sourceWidth,
      height: this.sourceHeight,
      fps: 30,
      scenes: this.steps.map((s) => ({
        src: s.file ? `keyframes/${s.file}` : undefined,
        holdSec: s.holdSec,
        callouts: [{ text: s.intent, color: accent, atSec: 0.4, anchor: "top-right" }],
        highlights: s.highlight ? [{ ...s.highlight, atSec: 0.6, label: s.highlight.label }] : undefined,
      })),
    };
    writeFileSync(join(this.dir, "annotations.json"), JSON.stringify(spec, null, 2), "utf8");
    return spec;
  }
}
