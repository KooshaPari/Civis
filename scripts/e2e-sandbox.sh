#!/usr/bin/env bash
# e2e-sandbox.sh — Run Civis E2E tests in a local sandbox
#
# Builds civ-server, starts it, runs the scripted playthrough,
# and runs post-playthrough verification. No GPU required.
#
# Usage:
#   bash scripts/e2e-sandbox.sh                    # full run
#   bash scripts/e2e-sandbox.sh --quick            # skip build if binary exists
#   bash scripts/e2e-sandbox.sh --seed 123         # custom seed
#
# Exit codes: 0=pass, 1=build, 2=server, 3=playthrough, 4=verification

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

QUICK=0
SEED="${CIVIS_PLAYTHROUGH_SEED:-42}"
SERVER_PORT="${CIV_SERVER_PORT:-3000}"
TIMEOUT="${HEALTH_TIMEOUT_SECONDS:-60}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --quick)  QUICK=1; shift ;;
        --seed)   SEED="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

export CIVIS_PLAYTHROUGH_SEED="$SEED"
export CIV_SERVER_PORT="$SERVER_PORT"
export CIV_WS_URL="ws://127.0.0.1:${SERVER_PORT}/ws"
export HEALTH_TIMEOUT_SECONDS="$TIMEOUT"

SANDBOX_DIR="${REPO_ROOT}/.e2e-sandbox"
LOG_DIR="${SANDBOX_DIR}/logs"
SAVE_DIR="${SANDBOX_DIR}/saves"
REPLAY_DIR="${SANDBOX_DIR}/replays"

C_OK="\033[32m"; C_FAIL="\033[31m"; C_INFO="\033[36m"; C_RST="\033[0m"
step()  { printf '%b[e2e]%b %s\n' "$C_INFO" "$C_RST" "$*"; }
ok()    { printf '%b  ok  %b %s\n' "$C_OK"  "$C_RST" "$*"; }
fail()  { printf '%b FAIL %b %s\n' "$C_FAIL" "$C_RST" "$*" >&2; }

# Preflight
step "Setting up sandbox..."
mkdir -p "$LOG_DIR" "$SAVE_DIR" "$REPLAY_DIR"

if lsof -ti :"$SERVER_PORT" &>/dev/null 2>&1; then
    step "Killing existing server on port $SERVER_PORT..."
    kill $(lsof -ti :"$SERVER_PORT") 2>/dev/null || true
    sleep 1
fi

for tool in websocat jq; do
    if ! command -v "$tool" &>/dev/null; then
        fail "Required: $tool"
        case "$tool" in
            websocat) echo "  cargo install websocat";;
            jq)       echo "  brew install jq";;
        esac
        exit 1
    fi
done

# Build
SERVER_BINARY="target/release/civ-server"
if [[ "$QUICK" -eq 1 ]] && [[ -f "$SERVER_BINARY" ]]; then
    step "Skipping build (--quick)"
else
    step "Building civ-server (release)..."
    if ! cargo build --locked --release -p civ-server 2>&1 | tail -3; then
        fail "Build failed"; exit 1
    fi
    ok "Build complete"
fi

# Start server
step "Starting civ-server on port $SERVER_PORT..."
"$SERVER_BINARY" > "${LOG_DIR}/server.log" 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "${SANDBOX_DIR}/server.pid"

step "Waiting for server..."
for i in $(seq 1 "$TIMEOUT"); do
    if grep -q "ws bridge listening" "${LOG_DIR}/server.log" 2>/dev/null; then
        ok "Server ready (${i}s)"; break
    fi
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        fail "Server died"; cat "${LOG_DIR}/server.log" >&2; exit 2
    fi
    [[ "$i" -eq "$TIMEOUT" ]] && { fail "Timeout"; cat "${LOG_DIR}/server.log" >&2; exit 2; }
    sleep 1
done

cleanup() {
    if [[ -f "${SANDBOX_DIR}/server.pid" ]]; then
        pid=$(cat "${SANDBOX_DIR}/server.pid")
        kill "$pid" 2>/dev/null || true
        sleep 1
        kill -9 "$pid" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Playthrough
step "Running playthrough (seed=$SEED)..."
PLAYTHROUGH_LOG="${LOG_DIR}/playthrough.log"
if bash "${REPO_ROOT}/scripts/playthrough.sh" 2>&1 | tee "$PLAYTHROUGH_LOG"; then
    ok "Playthrough PASSED"
else
    fail "Playthrough FAILED"; exit 3
fi

# Post-playthrough verification
step "Verifying..."
CENSUS_OUT=$("${REPO_ROOT}/target/release/civis-census" 2>/dev/null || echo '{}')
if echo "$CENSUS_OUT" | jq -e '.tick // .simulation.tick // .result.tick' &>/dev/null; then
    ok "Census: valid snapshot"
else
    fail "Census: no tick data"; exit 4
fi

if kill -0 "$SERVER_PID" 2>/dev/null; then
    ok "Server: still running"
else
    fail "Server: died"; exit 4
fi

echo ""
ok "=== E2E Sandbox: ALL PASSED ==="
echo "  Seed:  $SEED"
echo "  Logs:  ${LOG_DIR}/"
echo "  Saves: ${SAVE_DIR}/"
