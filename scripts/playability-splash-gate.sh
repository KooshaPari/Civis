#!/usr/bin/env bash
# shellcheck disable=SC2250,SC2295
set -euo pipefail

# Headless contract for the first playable loading screen. This intentionally
# checks the checked-in source/runtime pair without rasterising or launching a
# GPU client, so it is safe on CI and produces actionable packaging evidence.
root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT="$root_dir"
readonly UI_ROOT="$ROOT/clients/bevy-ref/assets/ui"
readonly CYAN="#50C8F0"
readonly GOLD="#E8B84B"

fail() {
    echo "playability-splash-gate: $*" >&2
    exit 1
}

require_file() {
    local path="$1"
    [[ -f $path ]] || fail "missing asset: ${path#$ROOT/}"
}

assert_png() {
    local name="$1"
    local minimum_bytes="$2"
    local path="$UI_ROOT/$name"
    require_file "$path"
    local size
    size="$(wc -c <"$path" | tr -d '[:space:]')"
    ((size > minimum_bytes)) || fail "asset too small: $name (${size} bytes)"
    local magic
    magic="$(od -An -tx1 -N8 "$path" | tr -d '[:space:]')"
    [[ $magic == "89504e470d0a1a0a" ]] || fail "not a PNG: $name"
}

assert_svg() {
    local name="$1"
    local path="$UI_ROOT/$name"
    require_file "$path"
    local svg_content
    svg_content="$(perl -0pe 's/<!--[\s\S]*?-->//g' "$path")"
    local expected_viewbox
    case "$name" in
    loading-bg.svg) expected_viewbox='viewBox="0 0 2560 1440"' ;;
    loading-spinner.svg) expected_viewbox='viewBox="0 0 120 120"' ;;
    *) fail "unexpected SVG asset: $name" ;;
    esac
    grep -Fq "$expected_viewbox" <<<"$svg_content" ||
        fail "unexpected viewBox: $name"
    grep -Fq "$CYAN" <<<"$svg_content" || fail "missing cyan brand token: $name"
    if [[ $name == "loading-spinner.svg" ]]; then
        grep -Fq "$GOLD" <<<"$svg_content" || fail "missing gold brand token: $name"
    fi
    if grep -Eiq '<[[:space:]]*(animate|set|animateMotion|animateTransform|mpath)([[:space:]>]|/>)' <<<"$svg_content"; then
        fail "runtime animation in $name"
    fi
    if grep -Eiq 'CURRENT_TIMESTAMP|date[[:space:]]*\(|new[[:space:]]+date|date[.]now|performance[.]now|math[.]random|randomuuid|uuid|timestamp|epoch' <<<"$svg_content"; then
        fail "nondeterminism in $name"
    fi
    grep -Eq '</svg>[[:space:]]*$' <<<"$svg_content" || fail "incomplete SVG: $name"
}

assert_svg loading-bg.svg
assert_svg loading-spinner.svg
assert_png loading-bg.png 1024
assert_png loading-spinner.png 256

echo "playability-splash-gate: pass (source/runtime pair is present and deterministic)"
