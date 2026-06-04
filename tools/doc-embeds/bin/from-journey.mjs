#!/usr/bin/env node
/**
 * Journey-record -> doc-embed adapter.
 *
 * Converts a phenotype-journeys manifest.json (see
 * tools/phenotype-journeys/schema/manifest.schema.json) into a doc-embed
 * EmbedSpec (src/schema.ts), then optionally renders it.
 *
 * A journey manifest already carries everything we need:
 *   - steps[].screenshot_path  -> scene still
 *   - steps[].intent           -> callout headline
 *   - steps[].description      -> callout subtext (Claude-described)
 *   - steps[].annotations[]    -> highlight rects / pointer rings
 *       (bbox [x,y,w,h], label, color, kind=region|pointer|highlight)
 *
 * Usage:
 *   node bin/from-journey.mjs --manifest <path/manifest.json> \
 *        [--keyframe-dir <dir>] [--accent #34d399] [--render]
 *
 * Writes annotations.json next to the manifest (annotations.embed.json) and,
 * with --render, produces mp4 + gif via render.mjs.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve, relative, join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  if (i >= 0 && (process.argv[i + 1] === undefined || process.argv[i + 1].startsWith("--"))) return true; // flag
  return i >= 0 ? process.argv[i + 1] : fallback;
}

const manifestArg = arg("manifest");
if (!manifestArg) {
  console.error("ERROR: --manifest <path/manifest.json> required");
  process.exit(1);
}
const manifestPath = resolve(process.cwd(), manifestArg);
const manifestDir = dirname(manifestPath);
const m = JSON.parse(readFileSync(manifestPath, "utf8"));
const accent = arg("accent", "#34d399");
// Journey keyframes are usually in a sibling keyframes/<id>/ dir.
const keyframeDir = arg("keyframe-dir", join(manifestDir, "..", "..", "keyframes", m.id));

const KIND_MAP = { pointer: "cursor", highlight: "highlight", region: "highlight" };

const scenes = (m.steps ?? []).map((step) => {
  const highlights = [];
  const cursors = [];
  for (const a of step.annotations ?? []) {
    const [x, y, width, height] = a.bbox;
    if (KIND_MAP[a.kind ?? "region"] === "cursor") {
      cursors.push({ x: x + width / 2, y: y + height / 2, atSec: 0.6, kind: "ripple", color: a.color ?? accent });
    } else {
      highlights.push({
        x, y, width, height,
        label: a.label,
        color: a.color ?? accent,
        atSec: 0.6,
        style: a.style === "dashed" ? "static" : "pulse",
      });
    }
  }
  // screenshot_path in a manifest is relative to its keyframe dir.
  const abs = resolve(keyframeDir, step.screenshot_path);
  return {
    src: relative(manifestDir, abs).split("\\").join("/"),
    holdSec: 3.5,
    callouts: [{ text: step.intent, subText: step.description ?? undefined, color: accent, atSec: 0.4, anchor: "top-right" }],
    highlights: highlights.length ? highlights : undefined,
    cursors: cursors.length ? cursors : undefined,
  };
});

const spec = {
  id: m.id,
  title: m.intent ?? m.id,
  subtitle: m.passed === false ? "journey: FAILED" : "journey",
  accent,
  width: 1280,
  height: 800,
  fps: 30,
  scenes,
};

const outPath = join(manifestDir, "annotations.embed.json");
writeFileSync(outPath, JSON.stringify(spec, null, 2), "utf8");
console.log(`[from-journey] wrote ${outPath} (${scenes.length} scenes)`);

if (arg("render", false)) {
  const r = spawnSync("node", [join(__dirname, "render.mjs"), "--annotations", outPath, "--format", "both"], {
    stdio: "inherit", shell: true,
  });
  process.exit(r.status ?? 0);
}
