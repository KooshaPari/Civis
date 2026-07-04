import assert from "node:assert/strict";
import { test } from "node:test";
import {
  Status,
  attachConnectionDetail,
  attachConnectionToStatus,
  buildHealthProbe,
  httpBaseFromWsUrl,
  statusClass,
  statusLabel,
} from "../src/connectionStatus.mjs";

test("attachConnectionToStatus maps dashboard store connection labels", () => {
  assert.equal(attachConnectionToStatus("live"), Status.OPEN);
  assert.equal(attachConnectionToStatus("reconnecting"), Status.CONNECTING);
  assert.equal(attachConnectionToStatus("disconnected"), Status.CLOSED);
});

test("attachConnectionDetail mirrors status.html detail strings", () => {
  assert.equal(attachConnectionDetail(Status.OPEN, "server"), "WebSocket open");
  assert.equal(attachConnectionDetail(Status.CONNECTING, "server"), "Opening connection…");
  assert.equal(attachConnectionDetail(Status.CLOSED, "server"), "Not connected");
  assert.equal(attachConnectionDetail(Status.OPEN, "watch"), "SSE stream active");
});

test("status helpers stay aligned for dashboard UI", () => {
  assert.equal(statusLabel(Status.OPEN), "Connected");
  assert.equal(statusClass(Status.OPEN), "status-open");
  assert.equal(
    httpBaseFromWsUrl("ws://127.0.0.1:3000/ws"),
    "http://127.0.0.1:3000",
  );
  assert.deepEqual(JSON.parse(buildHealthProbe(3)), {
    jsonrpc: "2.0",
    id: 3,
    method: "health",
    params: {},
  });
});
