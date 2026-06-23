#!/usr/bin/env bash
# POSIX fallback for `just play`. Mirrors Tools/play.ps1.
set -euo pipefail

PROFILE="${PROFILE:-release}"
LOG_LEVEL="${RUST_LOG:-info}"
NO_TAIL="${NO_TAIL:-0}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --debug) PROFILE=debug; shift ;;
        --log-level) LOG_LEVEL="$2"; shift 2 ;;
        --no-tail) NO_TAIL=1; shift ;;
        *) shift ;;
    esac
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$REPO_ROOT/.process-compose/logs"
PID_DIR="$REPO_ROOT/.process-compose/pids"
LOG_FILE="$LOG_DIR/civ-standalone.log"
ERR_FILE="$LOG_DIR/civ-standalone.err.log"
PID_FILE="$PID_DIR/civ-standalone.pid"

mkdir -p "$LOG_DIR" "$PID_DIR"

step()   { printf '\033[36m[play]\033[0m %s\n' "$*"; }
ok()     { printf '\033[32m[play]\033[0m %s\n' "$*"; }
err()    { printf '\033[31m[play]\033[0m %s\n' "$*" >&2; }

# 1. Kill stale
step "Killing any stale civ-standalone processes..."
if pgrep -f 'civ-standalone' >/dev/null 2>&1; then
    pkill -f 'civ-standalone' || true
    sleep 0.5
    ok "  killed"
else
    ok "  none running"
fi
[[ -f "$PID_FILE" ]] && rm -f "$PID_FILE"

# 2. Build
step "Building civ-standalone ($PROFILE)..."
cd "$REPO_ROOT"
BUILD_ARGS=(build -p civ-bevy-ref --features bevy,egui --bin civ-standalone)
[[ "$PROFILE" == "release" ]] && BUILD_ARGS+=(--release)
cargo "${BUILD_ARGS[@]}"

EXE="$REPO_ROOT/target/$PROFILE/civ-standalone"
[[ -x "$EXE" ]] || { err "Binary not found: $EXE"; exit 1; }
ok "Built: $EXE"

# 3. Launch
step "Launching civ-standalone (RUST_LOG=$LOG_LEVEL)..."
: > "$LOG_FILE"; : > "$ERR_FILE"

RUST_LOG="$LOG_LEVEL" RUST_BACKTRACE=1 nohup "$EXE" \
    >"$LOG_FILE" 2>"$ERR_FILE" &
GAME_PID=$!
echo "$GAME_PID" > "$PID_FILE"
ok "Launched pid $GAME_PID -> $LOG_FILE"

[[ "$NO_TAIL" == "1" ]] && exit 0

# 4. Tail
step "Tailing log (Ctrl+C to detach; game keeps running)..."
echo
tail -f "$LOG_FILE" "$ERR_FILE" &
TAIL_PID=$!
trap "kill $TAIL_PID 2>/dev/null || true" EXIT

wait $GAME_PID
RC=$?
kill $TAIL_PID 2>/dev/null || true
[[ $RC -eq 0 ]] && ok "civ-standalone exited cleanly." || err "civ-standalone exited with code $RC."
exit $RC
