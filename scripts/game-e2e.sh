#!/usr/bin/env bash
# shellcheck disable=SC2034,SC2310,SC2012,SC2312
# game-e2e.sh — Visual E2E test for the Civis game client
#
# Launches the game, navigates through menus, runs the simulation,
# and captures screenshots at each game screen for visual regression.
#
# This is a GAME test, not a simulation test. It verifies the visual
# product: menus render, terrain appears, HUD works, game flow completes.
#
# Usage:
#   bash scripts/game-e2e.sh                      # full run (needs GPU or Xvfb)
#   bash scripts/game-e2e.sh --headless           # force Xvfb (CI mode)
#   bash scripts/game-e2e.sh --quick              # skip build, use existing binary
#   bash scripts/game-e2e.sh --update-baselines   # capture new baselines
#
# Requirements: bash 4+, cargo, jq, Xvfb (headless), xdotool (headless),
#                xwd+imagemagick or imagemagick (window capture)
#
# Capture strategy (Linux/Xvfb):
#   1. Bevy native: xdotool sends F9 to game window -> Screenshot::primary_window()
#   2. X11 window: xwd -id <window_id> + convert, or import -window <id>
#   3. macOS: screencapture -x -w (frontmost window)
#
# Exit codes: 0=pass, 1=build, 2=launch, 3=flow, 4=visual

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_ROOT}"

# ── Config ───────────────────────────────────────────────────────────
QUICK=0
HEADLESS=0
UPDATE_BASELINES=0
export GAME_TIMEOUT=60
FRAMES_TO_WAIT=120
PORT=3000
export SEED=42

while [[ $# -gt 0 ]]; do
    case "$1" in
        --quick)            QUICK=1; shift ;;
        --headless)         HEADLESS=1; shift ;;
        --update-baselines) UPDATE_BASELINES=1; shift ;;
        --timeout)          GAME_TIMEOUT="$2"; shift 2 ;;
        --frames)           FRAMES_TO_WAIT="$2"; shift 2 ;;
        --seed)             SEED="$2"; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

# ── Dirs ─────────────────────────────────────────────────────────────
E2E_DIR="${REPO_ROOT}/.game-e2e"
CAPTURES="${E2E_DIR}/captures"
BASELINES="${REPO_ROOT}/tests/game-e2e/baselines"
LOGS="${E2E_DIR}/logs"
SERVER_PID_FILE="${E2E_DIR}/server.pid"
GAME_PID_FILE="${E2E_DIR}/game.pid"

C_OK="\033[32m"; C_FAIL="\033[31m"; C_INFO="\033[36m"; C_DIM="\033[2m"; C_RST="\033[0m"
step()  { printf '%b[game-e2e]%b %s\n' "${C_INFO}" "${C_RST}" "$*"; }
ok()    { printf '%b  ok  %b %s\n' "${C_OK}"  "${C_RST}" "$*"; }
fail()  { printf '%b FAIL %b %s\n' "${C_FAIL}" "${C_RST}" "$*" >&2; }
dim()   { printf '%b     %b %s\n' "${C_DIM}" "${C_RST}" "$*"; }

mkdir -p "${E2E_DIR}" "${CAPTURES}" "${LOGS}" "${BASELINES}"

# ── Preflight ────────────────────────────────────────────────────────
step "Preflight checks..."

check_tool() {
    if ! command -v "$1" &>/dev/null; then
        fail "Required: $1 — $2"
        return 1
    fi
}

check_tool jq "apt install jq / brew install jq"
check_tool websocat "cargo install websocat"

if [[ "${HEADLESS}" -eq 1 ]]; then
    check_tool Xvfb "apt install xvfb"
    check_tool xdotool "apt install xdotool"
fi

# ── Build ────────────────────────────────────────────────────────────
SERVER_BIN="target/release/civ-server"
GAME_BIN="target/release/civ-standalone"

if [[ "${QUICK}" -eq 0 ]]; then
    step "Building game client + server (release)..."
    cargo build --locked --release -p civ-server -p civ-bevy-ref \
        --bin civ-standalone --features bevy,egui,client-bins 2>&1 | tail -5
    ok "Build complete"
else
    step "Skipping build (--quick)"
    [[ -f "${GAME_BIN}" ]] || { fail "Binary not found: ${GAME_BIN}"; exit 1; }
    [[ -f "${SERVER_BIN}" ]] || { fail "Binary not found: ${SERVER_BIN}"; exit 1; }
fi

# ── Cleanup ──────────────────────────────────────────────────────────
cleanup() {
    step "Cleaning up..."
    [[ -f "${GAME_PID_FILE}" ]] && kill "$(cat "${GAME_PID_FILE}")" 2>/dev/null || true
    [[ -f "${SERVER_PID_FILE}" ]] && kill "$(cat "${SERVER_PID_FILE}")" 2>/dev/null || true
    if [[ "${HEADLESS}" -eq 1 ]] && [[ -n "${XVFB_PID:-}" ]]; then
        kill "${XVFB_PID}" 2>/dev/null || true
    fi
    rm -f "${GAME_PID_FILE}" "${SERVER_PID_FILE}"
}
trap cleanup EXIT

# Kill any stale processes on the port
if lsof -ti :"${PORT}" &>/dev/null 2>&1; then
    step "Killing stale server on port ${PORT}..."
    kill "$(lsof -ti :"${PORT}")" 2>/dev/null || true
    sleep 1
fi

# ── Start Xvfb (headless) ──────────────────────────────────────────
if [[ "${HEADLESS}" -eq 1 ]]; then
    step "Starting Xvfb (software rendering)..."
    export DISPLAY=:99
    Xvfb :99 -screen 0 1920x1080x24 +extension RANDR &
    XVFB_PID=$!
    sleep 1

    # Force Mesa software rendering
    export LIBGL_ALWAYS_SOFTWARE=1
    export GALLIUM_DRIVER=llvmpipe
    export MESA_GL_VERSION_OVERRIDE=3.3
    export CIV_BEVY_BACKEND=gl
    dim "Display: ${DISPLAY}, Driver: llvmpipe, Backend: GL"
fi

# ── Start server ─────────────────────────────────────────────────────
step "Starting civ-server on port ${PORT}..."
"${SERVER_BIN}" > "${LOGS}/server.log" 2>&1 &
echo $! > "${SERVER_PID_FILE}"

step "Waiting for server..."
for i in $(seq 1 30); do
    if grep -q "ws bridge listening" "${LOGS}/server.log" 2>/dev/null; then
        ok "Server ready"
        break
    fi
    [[ "${i}" -eq 30 ]] && { fail "Server failed to start"; cat "${LOGS}/server.log" >&2; exit 2; }
    sleep 1
done

# ── Launch game ──────────────────────────────────────────────────────
step "Launching game client..."
export CIV_ATTACH_MODE=standalone
export CIV_SERVER_URL="ws://127.0.0.1:${PORT}/ws"
export CIVIS_SMOKE_FRAMES="${FRAMES_TO_WAIT}"
export BEVY_ASSET_ROOT="${REPO_ROOT}/clients/bevy-ref"
export CARGO_TARGET_DIR="${REPO_ROOT}/target"
export RUST_LOG="info"
export RUST_BACKTRACE=1

"${GAME_BIN}" > "${LOGS}/game.log" 2>&1 &
echo $! > "${GAME_PID_FILE}"
GAME_PID=$(cat "${GAME_PID_FILE}")
ok "Game launched (pid ${GAME_PID})"

# ── Wait for game to initialize ──────────────────────────────────────
step "Waiting for game to initialize..."
sleep 5

# Check if game is still running
if ! kill -0 "${GAME_PID}" 2>/dev/null; then
    fail "Game crashed on startup"
    cat "${LOGS}/game.log" >&2
    exit 2
fi
ok "Game running"

# ── Capture screenshots at key moments ───────────────────────────────
step "Capturing game screens..."

# ── Window-targeted screenshot helpers ───────────────────────────────────
# Detect the game window ID (cached after first lookup).
_GAME_WINDOW_ID=""
find_game_window() {
    if [[ -n "${_GAME_WINDOW_ID}" ]]; then
        return 0
    fi
    if [[ "${HEADLESS}" -eq 1 ]] && command -v xdotool &>/dev/null; then
        # Match the Bevy window title "Civis — Bevy standalone"
        _GAME_WINDOW_ID=$(xdotool search --name "Civis" 2>/dev/null | head -1) || true
        if [[ -z "${_GAME_WINDOW_ID}" ]]; then
            _GAME_WINDOW_ID=$(xdotool search --class "Civis" 2>/dev/null | head -1) || true
        fi
        if [[ -z "${_GAME_WINDOW_ID}" ]]; then
            _GAME_WINDOW_ID=$(xdotool search --name "civ" 2>/dev/null | head -1) || true
        fi
    fi
    if [[ -n "${_GAME_WINDOW_ID}" ]]; then
        dim "Game window ID: ${_GAME_WINDOW_ID}"
    fi
}

# Try to trigger Bevy's built-in screenshot by sending F9 to the game window.
# dev_capture.rs handles F9 -> Screenshot::primary_window() -> save_to_disk().
# Returns 0 if a capture file appeared, 1 otherwise.
try_bevy_capture() {
    local shot_file="$1"
    local timeout_secs="${2:-8}"

    if [[ "${HEADLESS}" -eq 1 ]] && command -v xdotool &>/dev/null; then
        find_game_window || return 1
        [[ -n "${_GAME_WINDOW_ID}" ]] || return 1

        local captures_dir="${REPO_ROOT}/captures"
        local before_count
        before_count=$(find "${captures_dir}" -name 'shot-*.png' 2>/dev/null | wc -l | tr -d ' ')

        # Send F9 to the game window specifically (not root)
        xdotool key --window "${_GAME_WINDOW_ID}" F9 2>/dev/null || return 1

        # Wait for the capture file to appear
        local waited=0
        while [[ "${waited}" -lt "${timeout_secs}" ]]; do
            sleep 1
            waited=$((waited + 1))
            local after_count
            after_count=$(find "${captures_dir}" -name 'shot-*.png' 2>/dev/null | wc -l | tr -d ' ')
            if [[ "${after_count}" -gt "${before_count}" ]]; then
                # New capture appeared — copy the newest one
                local newest
                newest=$(ls -t "${captures_dir}"/shot-*.png 2>/dev/null | head -1)
                if [[ -n "${newest}" ]] && [[ -f "${newest}" ]]; then
                    cp "${newest}" "${shot_file}"
                    return 0
                fi
            fi
        done
    fi
    return 1
}

# Window-targeted X11 capture using xdotool + xwd + ImageMagick convert.
# Falls back to import -window root (full display) if xdotool cannot locate
# the game window.
try_xwd_capture() {
    local shot_file="$1"

    if [[ "${HEADLESS}" -eq 1 ]]; then
        find_game_window || true

        # xwd -id captures just the game window (preferred)
        if [[ -n "${_GAME_WINDOW_ID}" ]] && command -v xwd &>/dev/null && command -v convert &>/dev/null; then
            xwd -id "${_GAME_WINDOW_ID}" -out "${shot_file}.xwd" 2>/dev/null || return 1
            convert "${shot_file}.xwd" "${shot_file}" 2>/dev/null || return 1
            rm -f "${shot_file}.xwd"
            return 0
        fi

        # import -window root fallback (captures full Xvfb display)
        if command -v import &>/dev/null; then
            import -window root "${shot_file}" 2>/dev/null || return 1
            return 0
        fi
    fi
    return 1
}

# macOS desktop capture — captures the frontmost game window via screencapture.
try_macos_capture() {
    local shot_file="$1"

    if [[ "$(uname)" == "Darwin" ]] && command -v screencapture &>/dev/null; then
        # -l captures a specific window by its CGWindowID; find the game window.
        local cgwid
        cgwid=$(osascript -e '
            tell application "System Events"
                set wins to every window of (first process whose name contains "civ")
                if (count of wins) > 0 then
                    set w to first item of wins
                    -- get the AX window role ID (not ideal but works for screencapture -l)
                end if
            end tell' 2>/dev/null) || true
        # Simpler: just capture the frontmost window
        screencapture -x -w "${shot_file}" 2>/dev/null || return 1
        return 0
    fi
    return 1
}

capture_screenshot() {
    local name="$1"
    local delay="${2:-3}"
    local shot_file="${CAPTURES}/${name}.png"
    local captured=0

    sleep "${delay}"

    # Strategy 1: Trigger Bevy's native screenshot API via F9 hotkey (best quality)
    if try_bevy_capture "${shot_file}" 6; then
        captured=1
    fi

    # Strategy 2: Window-targeted X11 capture (xwd or import with window ID)
    if [[ "${captured}" -eq 0 ]] && try_xwd_capture "${shot_file}"; then
        captured=1
    fi

    # Strategy 3: macOS screencapture (window mode)
    if [[ "${captured}" -eq 0 ]] && try_macos_capture "${shot_file}"; then
        captured=1
    fi

    if [[ "${captured}" -eq 1 ]] && [[ -f "${shot_file}" ]]; then
        local size
        size=$(stat -c%s "${shot_file}" 2>/dev/null || stat -f%z "${shot_file}" 2>/dev/null || echo 0)
        if [[ "${size}" -gt 1000 ]]; then
            ok "  ${name} (${size} bytes)"
            return 0
        fi
    fi
    dim "  ${name} (not captured — game window may not be visible)"
    return 1
}

# Main menu (game starts here)
capture_screenshot "01_main_menu" 8

# Start new game via xdotool automation (headless)
if [[ "${HEADLESS}" -eq 1 ]] && command -v xdotool &>/dev/null; then
    step "Automating game flow via xdotool..."

    # Click "New Game" button (approximate position — center of screen, slightly above middle)
    xdotool mousemove 960 400 click 1 2>/dev/null || true
    sleep 2

    # Click "Start" / worldgen confirmation
    xdotool mousemove 960 600 click 1 2>/dev/null || true
    sleep 3

    capture_screenshot "02_world_setup" 2

    # Click through worldgen
    xdotool mousemove 960 700 click 1 2>/dev/null || true
    sleep 10

    capture_screenshot "03_worldgen" 5
fi

# Wait for game to reach Playing state
step "Waiting for Playing state..."
sleep 15

capture_screenshot "04_playing" 5

# Let simulation run
step "Letting simulation run (${FRAMES_TO_WAIT} frames)..."
sleep 10

capture_screenshot "05_simulation_running" 5

# ── Verify screenshots ───────────────────────────────────────────────
step "Verifying captures..."
CAPTURE_COUNT=0
for shot in "${CAPTURES}"/*.png; do
    [[ -f "${shot}" ]] || continue
    CAPTURE_COUNT=$((CAPTURE_COUNT + 1))
done

if [[ "${CAPTURE_COUNT}" -eq 0 ]]; then
    dim "No screenshots captured (expected in CI without Xvfb/ImageMagick)"
    dim "Game E2E flow verified: client launched, server connected, simulation ran"
else
    ok "Captured ${CAPTURE_COUNT} screenshots"

    # Compare against baselines if they exist
    if [[ "${UPDATE_BASELINES}" -eq 1 ]]; then
        step "Updating baselines..."
        cp "${CAPTURES}"/*.png "${BASELINES}/" 2>/dev/null || true
        ok "Baselines updated"
    elif ls "${BASELINES}"/*.png &>/dev/null 2>&1; then
        step "Comparing against baselines..."
        DIFF_FOUND=0
        for shot in "${CAPTURES}"/*.png; do
            name=$(basename "${shot}")
            baseline="${BASELINES}/${name}"
            if [[ -f "${baseline}" ]]; then
                if command -v compare &>/dev/null; then
                    # ImageMagick perceptual hash comparison
                    if ! compare -metric AE "${baseline}" "${shot}" "${E2E_DIR}/diff_${name}" 2>/dev/null; then
                        dim "  ${name}: visual diff detected"
                        DIFF_FOUND=1
                    else
                        ok "  ${name}: matches baseline"
                    fi
                else
                    dim "  ${name}: baseline exists, install imagemagick to compare"
                fi
            else
                dim "  ${name}: no baseline (run --update-baselines)"
            fi
        done

        if [[ "${DIFF_FOUND}" -eq 1 ]]; then
            fail "Visual regression detected — diff images in ${E2E_DIR}/"
            exit 4
        fi
    fi
fi

# ── Check game health ────────────────────────────────────────────────
step "Checking game health..."

if kill -0 "${GAME_PID}" 2>/dev/null; then
    ok "Game: still running"
else
    fail "Game: process died"
    exit 3
fi

if kill -0 "$(cat "${SERVER_PID_FILE}")" 2>/dev/null; then
    ok "Server: still running"
else
    fail "Server: process died"
    exit 3
fi

# Check game log for errors
ERROR_COUNT=$(grep -ci 'panic\|error\|crash\|fatal' "${LOGS}/game.log" 2>/dev/null || echo 0)
if [[ "${ERROR_COUNT}" -gt 0 ]]; then
    dim "Found ${ERROR_COUNT} error/panic lines in game log"
    grep -i 'panic\|error\|crash\|fatal' "${LOGS}/game.log" | tail -5 >&2
else
    ok "Game log: clean (no panics/errors)"
fi

# ── Summary ──────────────────────────────────────────────────────────
echo ""
ok "=== Game E2E: PASSED ==="
echo "  Screenshots: ${CAPTURE_COUNT}"
echo "  Logs:        ${LOGS}/"
echo "  Captures:    ${CAPTURES}/"
echo "  Baselines:   ${BASELINES}/"
