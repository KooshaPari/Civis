/** @typedef {'idle'|'connecting'|'open'|'closed'|'error'} ConnectionStatus */

export const Status = {
  IDLE: "idle",
  CONNECTING: "connecting",
  OPEN: "open",
  CLOSED: "closed",
  ERROR: "error",
};

/**
 * Map a WebSocket `readyState` to a stable status label.
 * @param {number | undefined | null} readyState
 * @returns {ConnectionStatus}
 */
export function readyStateToStatus(readyState) {
  if (readyState === 0) return Status.CONNECTING;
  if (readyState === 1) return Status.OPEN;
  if (readyState === 2 || readyState === 3) return Status.CLOSED;
  return Status.ERROR;
}

/** @param {ConnectionStatus} status */
export function statusLabel(status) {
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

/** @param {ConnectionStatus} status */
export function statusClass(status) {
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

/**
 * Map dashboard store connection to status pill state.
 * @param {"live" | "reconnecting" | "disconnected"} connection
 * @returns {ConnectionStatus}
 */
export function attachConnectionToStatus(connection) {
  if (connection === "live") return Status.OPEN;
  if (connection === "reconnecting") return Status.CONNECTING;
  return Status.CLOSED;
}

/**
 * Human-readable detail line for attach status UI.
 * @param {ConnectionStatus} status
 * @param {"watch" | "server"} attachMode
 */
export function attachConnectionDetail(status, attachMode) {
  if (status === Status.IDLE || status === Status.CLOSED) {
    return "Not connected";
  }
  if (status === Status.OPEN) {
    return attachMode === "watch" ? "SSE stream active" : "WebSocket open";
  }
  if (status === Status.CONNECTING) {
    return attachMode === "watch" ? "Opening SSE stream…" : "Opening connection…";
  }
  return statusLabel(status);
}

export const connectionDetail = attachConnectionDetail;

/** JSON-RPC `health` probe (matches civ-server JSON-RPC surface). */
export function buildHealthProbe(id = 1) {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    method: "health",
    params: {},
  });
}

/**
 * Derive an HTTP base URL from a `ws://` attach URL (for `healthz` links).
 * @param {string} wsUrl
 */
export function httpBaseFromWsUrl(wsUrl) {
  const normalized = wsUrl.replace(/^wss:/i, "https:").replace(/^ws:/i, "http:");
  const url = new URL(normalized);
  return `${url.protocol}//${url.host}`;
}

/**
 * Track WebSocket connection lifecycle for dashboard status UI.
 * @param {string} url
 * @param {{
 *   WebSocketImpl?: typeof WebSocket;
 *   onChange?: (status: ConnectionStatus, detail?: { code?: number; reason?: string }) => void;
 * }} [options]
 */
export function createConnectionMonitor(url, options = {}) {
  const { WebSocketImpl = globalThis.WebSocket, onChange } = options;
  /** @type {WebSocket | null} */
  let ws = null;
  let connectionToken = 0;
  /** @type {ConnectionStatus} */
  let status = Status.IDLE;
  /** @type {{ code?: number; reason?: string }} */
  let detail = {};

  /**
   * @param {ConnectionStatus} next
   * @param {{ code?: number; reason?: string }} [nextDetail]
   */
  function setStatus(next, nextDetail = {}) {
    status = next;
    detail = nextDetail;
    onChange?.(status, detail);
  }

  function connect() {
    disconnect();
    if (!WebSocketImpl) {
      setStatus(Status.ERROR, { reason: "WebSocket unavailable" });
      return;
    }

    setStatus(Status.CONNECTING);
    try {
      ws = new WebSocketImpl(url);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      setStatus(Status.ERROR, { reason });
      return;
    }

    const socket = ws;
    const token = ++connectionToken;

    ws.addEventListener("open", () => {
      if (ws === socket && token === connectionToken) setStatus(Status.OPEN);
    });
    ws.addEventListener("close", (event) => {
      if (ws === socket && token === connectionToken) {
        setStatus(Status.CLOSED, { code: event.code, reason: event.reason });
      }
    });
    ws.addEventListener("error", () => {
      if (ws === socket && token === connectionToken && status !== Status.CLOSED) {
        setStatus(Status.ERROR);
      }
    });
  }

  function disconnect() {
    if (ws) {
      const socket = ws;
      ws = null;
      connectionToken += 1;
      socket.close();
    }
    if (status !== Status.IDLE) {
      setStatus(Status.IDLE);
    }
  }

  /** @param {string} message */
  function send(message) {
    if (ws?.readyState === 1) {
      ws.send(message);
      return true;
    }
    return false;
  }

  return {
    connect,
    disconnect,
    send,
    getStatus: () => status,
    getUrl: () => url,
    getDetail: () => ({ ...detail }),
  };
}
