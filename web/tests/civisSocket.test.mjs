import assert from "node:assert/strict";
import { test } from "node:test";
import {
  getSocketTelemetry,
  recordSocketAttempt,
  recordSocketConnected,
  recordSocketError,
  recordSocketReconnectScheduled,
  sendActiveServerSocket,
  setActiveServerSocket,
  socketReconnectDelay,
  subscribeSocketTelemetry,
} from "../src/civisSocket.mjs";

globalThis.WebSocket = { OPEN: 1 };

test("reconnect delay is capped exponential backoff", () => {
  assert.deepEqual(
    [0, 1, 2, 8].map((attempt) => socketReconnectDelay(attempt)),
    [1000, 2000, 4000, 256000].map((delay) => Math.min(delay, 30000)),
  );
  assert.equal(socketReconnectDelay(-1), 1000);
  assert.equal(socketReconnectDelay(4, { baseDelayMs: 50, maxDelayMs: 125 }), 125);
});

test("send guard drops while disconnected and when browser buffer is high", () => {
  const sent = [];
  const ws = {
    readyState: 1,
    bufferedAmount: 0,
    send(value) {
      sent.push(value);
      this.bufferedAmount = 12;
    },
  };

  setActiveServerSocket(null);
  const before = getSocketTelemetry();
  assert.equal(sendActiveServerSocket("stale"), false);
  assert.equal(getSocketTelemetry().droppedMessages, before.droppedMessages + 1);

  setActiveServerSocket(ws);
  assert.equal(sendActiveServerSocket("health"), true);
  assert.deepEqual(sent, ["health"]);
  ws.bufferedAmount = 512 * 1024 + 1;
  assert.equal(sendActiveServerSocket("overloaded"), false);
  assert.equal(getSocketTelemetry().droppedMessages, before.droppedMessages + 2);
  setActiveServerSocket(null);
});

test("telemetry exposes lifecycle and supports unsubscribe", () => {
  const states = [];
  const unsubscribe = subscribeSocketTelemetry((value) => states.push(value.state));
  recordSocketAttempt();
  recordSocketReconnectScheduled(2000);
  recordSocketError(new Error("closed by peer"));
  recordSocketConnected();
  unsubscribe();
  const stateCount = states.length;
  recordSocketAttempt();
  assert.deepEqual(states.slice(0, 5), ["closed", "connecting", "reconnecting", "error", "open"]);
  assert.equal(states.length, stateCount);
  setActiveServerSocket(null);
});
