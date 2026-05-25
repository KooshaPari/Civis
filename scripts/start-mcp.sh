#!/usr/bin/env bash
set -euo pipefail

action="start"
foreground="0"
port="${DINOFORGE_MCP_PORT:-8765}"
host="${DINOFORGE_MCP_HOST:-127.0.0.1}"
watch="${DINOFORGE_MCP_WATCH:-0}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --action)
      action="${2:?missing value for --action}"
      shift 2
      ;;
    --foreground)
      foreground="1"
      shift
      ;;
    --port)
      port="${2:?missing value for --port}"
      shift 2
      ;;
    --host)
      host="${2:?missing value for --host}"
      shift 2
      ;;
    --watch)
      watch="1"
      shift
      ;;
    *)
      echo "[MCP] Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mcp_dir="${repo_root}/src/Tools/DinoforgeMcp"
state_dir="${TMPDIR:-/tmp}/DINOForge"
mcp_pid_file="${state_dir}/mcp-server.pid"
mcp_log_file="${state_dir}/mcp-server.log"
health_probe_url="http://${host}:${port}/health"

mkdir -p "${state_dir}"

if [[ -z "${DINO_GAME_DIR:-}" ]]; then
  export DINO_GAME_DIR="/path/to/Diplomacy is Not an Option"
fi

python_exe="${DINOFORGE_PYTHON:-python3}"
if ! command -v "${python_exe}" >/dev/null 2>&1; then
  echo "[MCP] python executable not found: ${python_exe}" >&2
  exit 1
fi

is_running() {
  local pid="$1"
  [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null
}

read_pid() {
  if [[ -f "${mcp_pid_file}" ]]; then
    tr -d '[:space:]' < "${mcp_pid_file}"
  fi
}

wait_for_health() {
  local retries=40
  while (( retries > 0 )); do
    if curl --silent --fail --max-time 1 "${health_probe_url}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
    retries=$((retries - 1))
  done
  return 1
}

stop_server() {
  local pid
  pid="$(read_pid || true)"
  if [[ -n "${pid}" ]] && is_running "${pid}"; then
    kill "${pid}" 2>/dev/null || true
    rm -f "${mcp_pid_file}"
    echo "[MCP] Stopped MCP PID ${pid}"
  else
    rm -f "${mcp_pid_file}"
    echo "[MCP] MCP is already stopped"
  fi
}

status_server() {
  local pid
  pid="$(read_pid || true)"
  if [[ -n "${pid}" ]] && is_running "${pid}"; then
    echo "[MCP] Status: running"
    echo "  PID: ${pid}"
  else
    echo "[MCP] Status: stopped"
  fi

  if curl --silent --fail --max-time 1 "${health_probe_url}" >/dev/null 2>&1; then
    echo "  Health /health: true"
  else
    echo "  Health /health: false"
  fi
}

case "${action}" in
  status)
    status_server
    exit 0
    ;;
  stop)
    stop_server
    exit 0
    ;;
  restart)
    stop_server
    ;;
  start)
    ;;
  *)
    echo "[MCP] Unsupported action: ${action}" >&2
    exit 1
    ;;
esac

existing_pid="$(read_pid || true)"
if [[ -n "${existing_pid}" ]] && is_running "${existing_pid}"; then
  echo "[MCP] Already running (PID=${existing_pid})"
  exit 0
fi

cd "${mcp_dir}"

if [[ "${watch}" == "1" ]]; then
  echo "[MCP] Watch mode is managed by the PowerShell launcher only; ignoring on POSIX." >&2
fi

echo "[MCP] Starting DINOForge MCP server (HTTP/SSE)..."
echo "[MCP] Port: ${port}"
echo "[MCP] Host: ${host}"
echo "[MCP] Game dir: ${DINO_GAME_DIR}"

if [[ "${foreground}" == "1" ]]; then
  echo "$$" > "${mcp_pid_file}"
  exec "${python_exe}" -m dinoforge_mcp.server --http --port "${port}" --host "${host}"
fi

"${python_exe}" -m dinoforge_mcp.server --http --port "${port}" --host "${host}" >>"${mcp_log_file}" 2>&1 &
server_pid=$!
echo "${server_pid}" > "${mcp_pid_file}"

if ! wait_for_health; then
  echo "[MCP] Warning: health check did not respond. Check log: ${mcp_log_file}" >&2
  exit 1
fi

echo "[MCP] PID file: ${mcp_pid_file}"
echo "[MCP] Log file: ${mcp_log_file}"
echo "[MCP] JSON-RPC endpoint: http://${host}:${port}/messages"
echo "[MCP] SSE endpoint: http://${host}:${port}/sse"
echo "[MCP] Health endpoint: ${health_probe_url}"