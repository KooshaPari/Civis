/** Module-level handle to the active `civ-server` WebSocket (operator controls). */

export type SocketState = "idle" | "connecting" | "open" | "reconnecting" | "closed" | "error";

export type SocketTelemetry = {
  state: SocketState;
  connectAttempts: number;
  reconnects: number;
  sentMessages: number;
  droppedMessages: number;
  bufferedAmount: number;
  reconnectDelayMs: number | null;
  lastError: string | null;
};

export type ReconnectPolicy = {
  baseDelayMs?: number;
  maxDelayMs?: number;
};

type SocketPayload = string | ArrayBufferLike | Blob | ArrayBufferView;

const DEFAULT_RECONNECT_POLICY: Required<ReconnectPolicy> = {
  baseDelayMs: 1_000,
  maxDelayMs: 30_000,
};
const MAX_BUFFERED_AMOUNT = 512 * 1024;

let activeWs: WebSocket | null = null;
let telemetry: SocketTelemetry = {
  state: "idle",
  connectAttempts: 0,
  reconnects: 0,
  sentMessages: 0,
  droppedMessages: 0,
  bufferedAmount: 0,
  reconnectDelayMs: null,
  lastError: null,
};
const telemetryListeners = new Set<(value: SocketTelemetry) => void>();

function publishTelemetry() {
  const snapshot = getSocketTelemetry();
  for (const listener of telemetryListeners) listener(snapshot);
}

function updateTelemetry(patch: Partial<SocketTelemetry>) {
  telemetry = { ...telemetry, ...patch };
  publishTelemetry();
}

export function setActiveServerSocket(ws: WebSocket | null) {
  activeWs = ws;
  updateTelemetry({
    state: ws ? "open" : "closed",
    bufferedAmount: ws?.bufferedAmount ?? 0,
    reconnectDelayMs: ws ? null : telemetry.reconnectDelayMs,
  });
}

export function getActiveServerSocket(): WebSocket | null {
  return activeWs;
}

export function getSocketTelemetry(): SocketTelemetry {
  return { ...telemetry };
}

export function subscribeSocketTelemetry(
  listener: (value: SocketTelemetry) => void,
): () => void {
  telemetryListeners.add(listener);
  listener(getSocketTelemetry());
  return () => telemetryListeners.delete(listener);
}

export function recordSocketAttempt() {
  updateTelemetry({
    state: "connecting",
    connectAttempts: telemetry.connectAttempts + 1,
    reconnectDelayMs: null,
    lastError: null,
  });
}

export function recordSocketConnected() {
  updateTelemetry({ state: "open", reconnectDelayMs: null, lastError: null });
}

export function recordSocketError(error: unknown) {
  updateTelemetry({
    state: "error",
    lastError: error instanceof Error ? error.message : String(error),
  });
}

export function recordSocketReconnectScheduled(delayMs: number) {
  updateTelemetry({
    state: "reconnecting",
    reconnects: telemetry.reconnects + 1,
    reconnectDelayMs: delayMs,
  });
}

/**
 * Compute a capped exponential reconnect delay. The caller owns the timer.
 * Keeping this deterministic makes reconnect behavior testable and observable.
 */
export function socketReconnectDelay(attempt: number, policy: ReconnectPolicy = {}): number {
  const { baseDelayMs, maxDelayMs } = { ...DEFAULT_RECONNECT_POLICY, ...policy };
  const safeAttempt = Math.max(0, Math.floor(attempt));
  return Math.min(maxDelayMs, baseDelayMs * 2 ** safeAttempt);
}

/**
 * Send only while the socket is open and its browser buffer is bounded.
 * Commands are intentionally dropped while reconnecting; replaying stale
 * mutations after a reconnect is less safe than exposing a bounded drop.
 */
export function sendActiveServerSocket(payload: SocketPayload): boolean {
  const ws = activeWs;
  const bufferedAmount = ws?.bufferedAmount ?? 0;
  if (!ws || ws.readyState !== WebSocket.OPEN || bufferedAmount > MAX_BUFFERED_AMOUNT) {
    updateTelemetry({
      droppedMessages: telemetry.droppedMessages + 1,
      bufferedAmount,
    });
    return false;
  }
  ws.send(payload);
  updateTelemetry({
    sentMessages: telemetry.sentMessages + 1,
    bufferedAmount: ws.bufferedAmount,
  });
  return true;
}
