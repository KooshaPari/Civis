/** Shared browser WebSocket guardrails for the Civis dashboard. */

const DEFAULT_RECONNECT_POLICY = { baseDelayMs: 1000, maxDelayMs: 30000 };
const MAX_BUFFERED_AMOUNT = 512 * 1024;

let activeWs = null;
let telemetry = {
  state: "idle",
  connectAttempts: 0,
  reconnects: 0,
  sentMessages: 0,
  droppedMessages: 0,
  bufferedAmount: 0,
  reconnectDelayMs: null,
  lastError: null,
};
const listeners = new Set();

function snapshot() {
  return { ...telemetry };
}

function update(patch) {
  telemetry = { ...telemetry, ...patch };
  const value = snapshot();
  for (const listener of listeners) listener(value);
}

export function setActiveServerSocket(ws) {
  activeWs = ws;
  update({
    state: ws ? "open" : "closed",
    bufferedAmount: ws?.bufferedAmount ?? 0,
    reconnectDelayMs: ws ? null : telemetry.reconnectDelayMs,
  });
}

export function getActiveServerSocket() {
  return activeWs;
}

export function getSocketTelemetry() {
  return snapshot();
}

export function subscribeSocketTelemetry(listener) {
  listeners.add(listener);
  listener(snapshot());
  return () => listeners.delete(listener);
}

export function recordSocketAttempt() {
  update({
    state: "connecting",
    connectAttempts: telemetry.connectAttempts + 1,
    reconnectDelayMs: null,
    lastError: null,
  });
}

export function recordSocketConnected() {
  update({ state: "open", reconnectDelayMs: null, lastError: null });
}

export function recordSocketError(error) {
  update({ state: "error", lastError: error instanceof Error ? error.message : String(error) });
}

export function recordSocketReconnectScheduled(delayMs) {
  update({
    state: "reconnecting",
    reconnects: telemetry.reconnects + 1,
    reconnectDelayMs: delayMs,
  });
}

export function socketReconnectDelay(attempt, policy = {}) {
  const { baseDelayMs, maxDelayMs } = { ...DEFAULT_RECONNECT_POLICY, ...policy };
  const safeAttempt = Math.max(0, Math.floor(attempt));
  return Math.min(maxDelayMs, baseDelayMs * 2 ** safeAttempt);
}

/** Drop stale commands while disconnected instead of replaying them later. */
export function sendActiveServerSocket(payload) {
  const ws = activeWs;
  const bufferedAmount = ws?.bufferedAmount ?? 0;
  if (!ws || ws.readyState !== globalThis.WebSocket.OPEN || bufferedAmount > MAX_BUFFERED_AMOUNT) {
    update({ droppedMessages: telemetry.droppedMessages + 1, bufferedAmount });
    return false;
  }
  ws.send(payload);
  update({ sentMessages: telemetry.sentMessages + 1, bufferedAmount: ws.bufferedAmount });
  return true;
}
