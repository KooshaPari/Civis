#!/usr/bin/env bash
# POSIX manager for the Civis dev stack.
set -euo pipefail

ACTION="${1:-status}"
SERVICE="${2:-}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$REPO_ROOT/.process-compose/logs"
COMPOSE_FILE="$REPO_ROOT/process-compose.yaml"
PORT=18080

if ! command -v process-compose >/dev/null 2>&1; then
    echo "[dev] process-compose not installed" >&2
    exit 1
fi

is_running() { process-compose process list --port "$PORT" >/dev/null 2>&1; }

case "$ACTION" in
    up)
        mkdir -p "$LOG_DIR"
        if is_running; then
            echo "[dev] Stack already running."
            process-compose process list --port "$PORT"
            exit 0
        fi
        echo "[dev] Starting backing services..."
        cd "$REPO_ROOT"
        nohup process-compose up -f "$COMPOSE_FILE" --port "$PORT" --tui=false \
            >"$LOG_DIR/process-compose.log" 2>&1 &
        for _ in $(seq 1 60); do
            sleep 0.5
            if is_running; then
                echo "[dev] Stack ready."
                process-compose process list --port "$PORT"
                exit 0
            fi
        done
        echo "[dev] Timed out." >&2
        exit 1
        ;;
    down)
        is_running || { echo "[dev] Not running."; exit 0; }
        process-compose down --port "$PORT"
        ;;
    status)
        is_running || { echo "[dev] Not running."; exit 1; }
        process-compose process list --port "$PORT"
        ;;
    logs)
        if [[ -n "$SERVICE" ]]; then
            tail -f "$LOG_DIR/$SERVICE.log"
        else
            for f in "$LOG_DIR"/*.log; do
                echo "=== $(basename "$f") ==="
                tail -n 20 "$f"
            done
        fi
        ;;
    *) echo "Usage: $0 {up|down|status|logs} [service]" >&2; exit 2 ;;
esac
