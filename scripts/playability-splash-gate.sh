#!/usr/bin/env bash
set -euo pipefail

# Headless contract for the first playable loading screen. This intentionally
# checks the checked-in source/runtime pair without rasterising or launching a
# GPU client, so it is safe on CI and produces actionable packaging evidence.
readonly ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly UI_ROOT="$ROOT/clients/bevy-ref/assets/ui"
readonly CYAN="#50C8F0"
readonly GOLD="#E8B84B"

fail() {
  echo "playability-splash-gate: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing asset: ${path#$ROOT/}"
}

assert_png() {
  local name="$1"
  local minimum_bytes="$2"
  local path="$UI_ROOT/$name"
  require_file "$path"
  local size
  size="$(wc -c < "$path" | tr -d '[:space:]')"
  (( size > minimum_bytes )) || fail "asset too small: $name (${size} bytes)"
  local magic
  magic="$(od -An -tx1 -N8 "$path" | tr -d '[:space:]')"
  [[ "$magic" == "89504e470d0a1a0a" ]] || fail "not a PNG: $name"
}

assert_svg() {
  local name="$1"
  local path="$UI_ROOT/$name"
  require_file "$path"
  grep -Eq 'viewBox="0 0 (2560 1440|120 120)"' "$path" \
    || fail "unexpected viewBox: $name"
  grep -Fq "$CYAN" "$path" || fail "missing cyan brand token: $name"
  if [[ "$name" == "loading-spinner.svg" ]]; then
    grep -Fq "$GOLD" "$path" || fail "missing gold brand token: $name"
  fi
  grep -q '<animate\|<set ' "$path" && fail "runtime animation in $name" || true
  grep -q 'CURRENT_TIMESTAMP\|Date(' "$path" && fail "nondeterminism in $name" || true
  grep -Eq '</svg>[[:space:]]*$' "$path" || fail "incomplete SVG: $name"
}

assert_svg loading-bg.svg
assert_svg loading-spinner.svg
assert_png loading-bg.png 1024
assert_png loading-spinner.png 256

echo "playability-splash-gate: pass (source/runtime pair is present and deterministic)"
