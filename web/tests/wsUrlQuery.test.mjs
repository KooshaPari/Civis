import assert from "node:assert/strict";
import { test } from "node:test";
import {
  defaultLiveWsUrl,
  parseWsBinaryEnvFlag,
  resolveWsUrlFromQuery,
  withTickFormatBinaryQuery,
  wsPreferBinary,
} from "../src/wsUrl.mjs";

test("resolveWsUrlFromQuery prefers ?ws= override", () => {
  assert.equal(
    resolveWsUrlFromQuery("?ws=ws://custom.test:4444/live"),
    "ws://custom.test:4444/live",
  );
});

test("resolveWsUrlFromQuery falls back to default attach URL", () => {
  assert.equal(resolveWsUrlFromQuery(""), defaultLiveWsUrl());
  assert.equal(resolveWsUrlFromQuery("?foo=bar"), defaultLiveWsUrl());
});

test("parseWsBinaryEnvFlag accepts common truthy values", () => {
  assert.equal(parseWsBinaryEnvFlag("1"), true);
  assert.equal(parseWsBinaryEnvFlag(" true "), true);
  assert.equal(parseWsBinaryEnvFlag("0"), false);
  assert.equal(parseWsBinaryEnvFlag("false"), false);
});

test("withTickFormatBinaryQuery appends once", () => {
  assert.equal(
    withTickFormatBinaryQuery("ws://127.0.0.1:3000/ws"),
    "ws://127.0.0.1:3000/ws?tick_format=binary",
  );
  assert.equal(
    withTickFormatBinaryQuery("ws://127.0.0.1:3000/ws?role=operator"),
    "ws://127.0.0.1:3000/ws?role=operator&tick_format=binary",
  );
});

test("wsPreferBinary honors ?binary=1 over env", () => {
  assert.equal(
    wsPreferBinary("?binary=1", { CIVIS_WS_BINARY: "0" }),
    true,
  );
  assert.equal(
    wsPreferBinary("?binary=0", { CIVIS_WS_BINARY: "1" }),
    false,
  );
});
