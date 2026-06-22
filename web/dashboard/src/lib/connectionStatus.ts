/** Display helpers aligned with `web/src/connectionStatus.mjs`. */

export const Status = {
  IDLE: "idle",
  CONNECTING: "connecting",
  OPEN: "open",
  CLOSED: "closed",
  ERROR: "error",
} as const;

export type ConnectionStatus = (typeof Status)[keyof typeof Status];

export function statusLabel(status: ConnectionStatus): string {
  switch (status) {
    case Status.IDLE:
      return "Idle";
    case Status.CONNECTING:
      return "Connecting…";
    case Status.OPEN:
      return "Connected";
    case Status.CLOSED:
      return "Disconnected";
    case Status.ERROR:
      return "Error";
    default:
      return "Unknown";
  }
}

export function statusClass(status: ConnectionStatus): string {
  switch (status) {
    case Status.IDLE:
    case Status.CONNECTING:
    case Status.OPEN:
    case Status.CLOSED:
    case Status.ERROR:
      return `status-${status}`;
    default:
      return "status-unknown";
  }
}

export function buildHealthProbe(id = 1): string {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    method: "health",
    params: {},
  });
}

export function httpBaseFromWsUrl(wsUrl: string): string {
  const normalized = wsUrl.replace(/^wss:/i, "https:").replace(/^ws:/i, "http:");
  const url = new URL(normalized);
  return `${url.protocol}//${url.host}`;
}

export function dashboardConnectionToStatus(
  connection: "live" | "reconnecting" | "disconnected",
): ConnectionStatus {
  if (connection === "live") return Status.OPEN;
  if (connection === "reconnecting") return Status.CONNECTING;
  return Status.CLOSED;
}

export function connectionDetail(
  status: ConnectionStatus,
  attachMode: "watch" | "server",
): string {
  if (status === Status.IDLE || status === Status.CLOSED) {
    return attachMode === "watch"
      ? "Not connected to civ-watch SSE"
      : "Not connected to civ-server WebSocket";
  }
  if (status === Status.OPEN) {
    return attachMode === "watch"
      ? "Connected to civ-watch SSE"
      : "Connected to civ-server WebSocket";
  }
  if (status === Status.CONNECTING) {
    return attachMode === "watch"
      ? "Opening civ-watch SSE stream…"
      : "Opening civ-server WebSocket…";
  }
  return statusLabel(status);
}

export const attachConnectionDetail = connectionDetail;
