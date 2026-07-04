#!/usr/bin/env bash
# Reproducible Civis UI art build. Keycap teal #7ebab5 / midnight #090a0c.
# Requires: bun, the jsui node_modules (satori/resvg). Run from repo root or anywhere.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../.." && pwd)"
JSUI="/c/Users/koosh/Dev/asset-engine-venv/jsui"   # provides @resvg/resvg-js + satori
UI="$REPO/clients/bevy-ref/assets/ui"
RAST="$HERE/rasterize.mjs"

ras() { ( cd "$JSUI" && bun "$RAST" "$1" "$2" "$3" "$4" ); }

# 1) generate procedural crest SVGs
( cd "$HERE" && bun "$HERE/gen-crests.mjs" "$HERE/svg/crests" )

# 2) copy authored SVG sources into the asset tree (source-of-truth .svg the repo ships)
cp "$HERE/svg/logo.svg"            "$UI/logo.svg"
cp "$HERE/svg/loading-spinner.svg" "$UI/loading-spinner.svg"
cp "$HERE/svg/loading-bg.svg"      "$UI/loading-bg.svg"
for n in blue cyan gold green red violet; do
  cp "$HERE/svg/crests/crest-$n.svg" "$UI/faction-crests/crest-$n.svg"
done

# 3) rasterize PNGs at the exact dims the game loads
ras "$UI/logo.svg"            "$UI/logo.png"            1600 600
ras "$UI/loading-spinner.svg" "$UI/loading-spinner.png" 240  240
ras "$UI/loading-bg.svg"      "$UI/loading-bg.png"      2048 1152
for n in blue cyan gold green red violet; do
  ras "$UI/faction-crests/crest-$n.svg" "$UI/faction-crests/crest-$n.png" 256 256
done

echo "DONE"
