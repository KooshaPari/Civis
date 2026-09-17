// NFR/FR-CIV-ARCH-NOSVG-001 — no runtime SVG parsing in the asset bundle.
//
// Acceptance contract: the bundle scan finds zero runtime SVG parsers/loaders
// in the dashboard's load path. SVGs are rasterised to atlases at build time by
// the asset pipeline (see docs/specs/CIV-0600-2d-asset-pipeline-spec.md), so the
// shipped bundle must not be able to parse an SVG at runtime.
//
// The check reads the dashboard's own sources and enforces the contract
// statically: no SVG parser dependency may be imported, and no `.svg` asset may
// be imported as a module (which would make Vite treat it as an asset to parse
// or inline at runtime).

import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const DASHBOARD_SRC = resolve(here, "..", "dashboard", "src");
const DASHBOARD_PKG = resolve(here, "..", "dashboard", "package.json");

/** SVG parsers / rasterisers that must never reach the runtime bundle. */
const FORBIDDEN_SVG_RUNTIME_MODULES = [
  "resvg",
  "@resvg/resvg-js",
  "resvg-js",
  "svg.js",
  "@svgdotjs/svg.js",
  "svg-parser",
  "svgson",
  "canvg",
  "sharp",
  "svgo",
];

/** Recursively collect every file under `dir`. */
function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      out.push(...walk(full));
    } else {
      out.push(full);
    }
  }
  return out;
}

test("dashboard declares no SVG runtime parser dependency", () => {
  const pkg = JSON.parse(readFileSync(DASHBOARD_PKG, "utf8"));
  const declared = Object.keys({
    ...(pkg.dependencies ?? {}),
    ...(pkg.devDependencies ?? {}),
  });

  const offenders = declared.filter((dep) =>
    FORBIDDEN_SVG_RUNTIME_MODULES.some(
      (banned) => dep === banned || dep.startsWith(`${banned}/`),
    ),
  );
  assert.deepEqual(
    offenders,
    [],
    `runtime SVG parsers must not be declared: ${offenders.join(", ")}`,
  );
});

test("dashboard sources import no SVG parser and no .svg module", () => {
  const files = walk(DASHBOARD_SRC).filter((f) => /\.(mjs|js|ts|tsx|jsx)$/.test(f));
  assert.ok(files.length > 0, "expected dashboard sources to exist");

  const importRe = /(?:from|import)\s*\(?\s*["']([^"']+)["']/g;
  const offenders = [];

  for (const file of files) {
    const text = readFileSync(file, "utf8");
    for (const match of text.matchAll(importRe)) {
      const spec = match[1];
      const rel = file.slice(DASHBOARD_SRC.length + 1);

      if (spec.endsWith(".svg")) {
        offenders.push(`${rel} imports SVG asset module "${spec}"`);
        continue;
      }
      if (
        FORBIDDEN_SVG_RUNTIME_MODULES.some(
          (banned) => spec === banned || spec.startsWith(`${banned}/`),
        )
      ) {
        offenders.push(`${rel} imports SVG runtime parser "${spec}"`);
      }
    }
  }

  assert.deepEqual(
    offenders,
    [],
    `no runtime SVG parsing allowed in the bundle:\n  ${offenders.join("\n  ")}`,
  );
});

test("no inline SVG parser/DOMParser-based rasteriser in sources", () => {
  const files = walk(DASHBOARD_SRC).filter((f) => /\.(mjs|js|ts|tsx|jsx)$/.test(f));
  const offenders = [];

  for (const file of files) {
    const text = readFileSync(file, "utf8");
    const rel = file.slice(DASHBOARD_SRC.length + 1);
    // Constructing an SVG document for parsing is the runtime-parsing pattern
    // the requirement forbids.
    if (/new\s+DOMParser\s*\(/.test(text) && /svg/i.test(text)) {
      offenders.push(`${rel} uses DOMParser with SVG`);
    }
    if (/createElementNS\s*\(\s*["']http:\/\/www\.w3\.org\/2000\/svg/.test(text)) {
      offenders.push(`${rel} creates an SVG namespace node`);
    }
  }

  assert.deepEqual(offenders, [], `runtime SVG construction found: ${offenders.join(", ")}`);
});
