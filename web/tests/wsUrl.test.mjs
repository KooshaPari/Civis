import assert from "node:assert/strict";
import { test } from "node:test";
import {
  defaultLiveWsUrl,
  liveWsUrl,
  resolveWsUrlFromEnv,
} from "../src/wsUrl.mjs";

test("defaultLiveWsUrl matches civ-server default bind", () => {
  assert.equal(defaultLiveWsUrl(), "ws://127.0.0.1:3000/ws");
});

test("liveWsUrl builds attach path", () => {
  assert.equal(liveWsUrl("127.0.0.1", 8765, "/ws"), "ws://127.0.0.1:8765/ws");
  assert.equal(liveWsUrl("localhost", 3000, "ws"), "ws://localhost:3000/ws");
});

test("resolveWsUrlFromEnv uses CIVIS_WS_URL when set", () => {
  assert.equal(
    resolveWsUrlFromEnv({ CIVIS_WS_URL: "ws://example.test:9999/custom" }),
    "ws://example.test:9999/custom",
  );
});

test("resolveWsUrlFromEnv builds from CIVIS_WS_ADDR", () => {
  assert.equal(
    resolveWsUrlFromEnv({ CIVIS_WS_ADDR: "10.0.0.5:4000" }),
    "ws://10.0.0.5:4000/ws",
  );
  assert.equal(
    resolveWsUrlFromEnv({
      CIVIS_WS_ADDR: "10.0.0.5:4000",
      CIVIS_WS_PATH: "/live",
    }),
    "ws://10.0.0.5:4000/live",
  );
});

test("resolveWsUrlFromEnv falls back to default", () => {
  assert.equal(resolveWsUrlFromEnv({}), defaultLiveWsUrl());
  assert.equal(resolveWsUrlFromEnv({ CIVIS_WS_URL: "  " }), defaultLiveWsUrl());
});
