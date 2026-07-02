// rasterize.mjs — SVG -> PNG via resvg, at an explicit output size.
// Reproducible Civis UI art pipeline (Keycap teal/midnight design system).
//
// Usage: bun rasterize.mjs <in.svg> <out.png> <width> <height>
// Run with the jsui node_modules on the resolution path:
//   cd /c/Users/koosh/Dev/asset-engine-venv/jsui && bun /c/Users/koosh/Dev/Civis/tools/asset-gen/rasterize.mjs ...
// resolve resvg from the jsui asset-engine venv (override with RESVG_PKG)
const RESVG_PKG = process.env.RESVG_PKG
  || "file:///C:/Users/koosh/Dev/asset-engine-venv/jsui/node_modules/@resvg/resvg-js/index.js";
const { Resvg } = await import(RESVG_PKG);
import { readFileSync, writeFileSync } from "node:fs";

const [, , inPath, outPath, wStr, hStr] = process.argv;
if (!inPath || !outPath) {
  console.error("usage: rasterize.mjs <in.svg> <out.png> <width> [height]");
  process.exit(1);
}
const svg = readFileSync(inPath, "utf8");
const width = wStr ? parseInt(wStr, 10) : 0;

const resvg = new Resvg(svg, {
  fitTo: width ? { mode: "width", value: width } : { mode: "original" },
  background: "rgba(0,0,0,0)",
  shapeRendering: 2, // geometricPrecision
  textRendering: 2,
});
const png = resvg.render().asPng();
writeFileSync(outPath, png);
console.log(`ok ${outPath} (${png.length}B)`);
