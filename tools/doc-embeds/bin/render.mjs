#!/usr/bin/env node
/**
 * Generic doc-embed renderer.
 *
 * Usage:
 *   node bin/render.mjs --annotations <path/to/annotations.json> [--format mp4|gif|both] [--out <dir>]
 *
 * Reads an EmbedSpec (see src/schema.ts), stages all referenced assets
 * (scene src + audio) into Remotion's public/ dir (paths in the spec are
 * resolved RELATIVE TO the annotations.json file), then invokes the Remotion
 * CLI to render an annotated mp4 and/or gif.
 *
 * Cross-project: copy the whole tools/doc-embeds/ folder into any repo, run
 * `npm install`, and point --annotations at a spec in that repo.
 */
import { spawnSync } from "node:child_process";
import { readFileSync, mkdirSync, copyFileSync, rmSync, existsSync, writeFileSync, readdirSync } from "node:fs";
const require_fs_readdir = (p) => readdirSync(p);
import { dirname, resolve, join, basename, isAbsolute } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..");
const PUBLIC = join(ROOT, "public");

function resolveBrowser() {
  // Remotion needs chrome-headless-shell (old headless mode). Full Chrome/Edge
  // removed old headless, so prefer a headless-shell binary. Playwright ships
  // one; reuse it to skip Remotion's flaky download.
  if (process.env.DOC_EMBEDS_BROWSER) return process.env.DOC_EMBEDS_BROWSER;
  const home = process.env.USERPROFILE || process.env.HOME || "";
  const pwRoot = join(home, "AppData", "Local", "ms-playwright");
  if (existsSync(pwRoot)) {
    // pick the newest chromium_headless_shell-* dir
    const dirs = require_fs_readdir(pwRoot).filter((d) => d.startsWith("chromium_headless_shell-"));
    dirs.sort();
    for (const d of dirs.reverse()) {
      for (const sub of [
        join(pwRoot, d, "chrome-headless-shell-win64", "chrome-headless-shell.exe"),
        join(pwRoot, d, "chrome-win", "headless_shell.exe"),
      ]) {
        if (existsSync(sub)) return sub;
      }
    }
  }
  return null; // fall back to Remotion's managed browser download
}

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  return i >= 0 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
}

const annotationsPath = arg("annotations");
if (!annotationsPath) {
  console.error("ERROR: --annotations <path/to/annotations.json> is required");
  process.exit(1);
}
const format = arg("format", "both"); // mp4 | gif | both
const specPath = resolve(process.cwd(), annotationsPath);
const specDir = dirname(specPath);
const spec = JSON.parse(readFileSync(specPath, "utf8"));
const outDir = resolve(process.cwd(), arg("out", join(specDir, "out")));

// --- Stage assets into public/ and rewrite spec paths to relative names ---
rmSync(PUBLIC, { recursive: true, force: true });
mkdirSync(PUBLIC, { recursive: true });

function stage(relOrAbs) {
  const srcAbs = isAbsolute(relOrAbs) ? relOrAbs : resolve(specDir, relOrAbs);
  if (!existsSync(srcAbs)) {
    console.error(`ERROR: asset not found: ${srcAbs}`);
    process.exit(1);
  }
  // Flatten to a unique-ish name to avoid collisions across scenes.
  const staged = basename(srcAbs);
  copyFileSync(srcAbs, join(PUBLIC, staged));
  return staged;
}

for (const scene of spec.scenes ?? []) {
  if (scene.src) scene.src = stage(scene.src);
}
if (spec.audioSrc) spec.audioSrc = stage(spec.audioSrc);

// Write the staged spec to a temp props file for the Remotion CLI.
const propsFile = join(ROOT, ".staged-props.json");
writeFileSync(propsFile, JSON.stringify(spec, null, 2), "utf8");

mkdirSync(outDir, { recursive: true });
const id = spec.id ?? "embed";

function render(fmt) {
  const ext = fmt === "gif" ? "gif" : "mp4";
  const output = join(outDir, `${id}.${ext}`);
  const args = [
    "remotion", "render", "src/index.tsx", "DocEmbed", output,
    `--props=${propsFile}`,
  ];
  // Allow reusing an installed Chrome/Edge to avoid the headless-shell
  // download (set DOC_EMBEDS_BROWSER, or it auto-detects common locations).
  const browser = resolveBrowser();
  if (browser) args.push(`--browser-executable="${browser}"`);
  if (fmt === "gif") {
    // Keep GIFs doc-friendly: half scale + decimated frames. Override via
    // --gif-scale / --gif-every-nth.
    args.push("--codec=gif");
    args.push(`--scale=${arg("gif-scale", "0.5")}`);
    args.push(`--every-nth-frame=${arg("gif-every-nth", "3")}`);
  }
  console.log(`\n[doc-embeds] rendering ${fmt} -> ${output}`);
  const r = spawnSync("npx", args, { cwd: ROOT, stdio: "inherit", shell: true });
  if (r.status !== 0) {
    console.error(`[doc-embeds] render ${fmt} FAILED (exit ${r.status})`);
    process.exit(r.status ?? 1);
  }
  return output;
}

const produced = [];
if (format === "mp4" || format === "both") produced.push(render("mp4"));
if (format === "gif" || format === "both") produced.push(render("gif"));

console.log(`\n[doc-embeds] DONE. Artifacts:\n  ${produced.join("\n  ")}`);
