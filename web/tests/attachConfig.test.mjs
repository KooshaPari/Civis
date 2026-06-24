import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveAttachConfig } from "../src/attachConfig.mjs";

test("defaults to server without explicit attach", () => {
  const cfg = resolveAttachConfig({});
  assert.equal(cfg.mode, "server");
});

test("CIVIS_WS_URL selects server mode", () => {
  const cfg = resolveAttachConfig({
    env: { CIVIS_WS_URL: "ws://127.0.0.1:3000/ws" },
  });
  assert.equal(cfg.mode, "server");
  assert.equal(cfg.wsUrl, "ws://127.0.0.1:3000/ws?tick_format=binary");
});

test("query attach overrides", () => {
  assert.equal(
    resolveAttachConfig({ search: "?attach=server" }).mode,
    "server",
  );
  assert.equal(
    resolveAttachConfig({ search: "?attach=watch" }).mode,
    "watch",
  );
});

test("preferBinary defaults true and appends tick_format=binary", () => {
  const cfg = resolveAttachConfig({});
  assert.equal(cfg.preferBinary, true);
  assert.match(cfg.wsUrl, /tick_format=binary$/);
});

test("CIVIS_WS_BINARY=0 disables binary preference", () => {
  const cfg = resolveAttachConfig({ env: { CIVIS_WS_BINARY: "0" } });
  assert.equal(cfg.preferBinary, false);
  assert.doesNotMatch(cfg.wsUrl, /tick_format=binary/);
});

test("?binary=1 overrides env and enables binary URL hint", () => {
  const cfg = resolveAttachConfig({
    search: "?binary=1",
    env: { CIVIS_WS_BINARY: "0" },
  });
  assert.equal(cfg.preferBinary, true);
  assert.match(cfg.wsUrl, /tick_format=binary$/);
});

test("?attach=watch resolves civ-watch HTTP base for SSE/REST", () => {
  const cfg = resolveAttachConfig({ search: "?attach=watch" });
  assert.equal(cfg.mode, "watch");
  assert.equal(cfg.watchHttp, "http://127.0.0.1:9090");
});

test("watch HTTP base honors ?watch= then CIVIS_WATCH_HTTP", () => {
  assert.equal(
    resolveAttachConfig({ search: "?attach=watch&watch=http://custom:8080" })
      .watchHttp,
    "http://custom:8080",
  );
  assert.equal(
    resolveAttachConfig({
      search: "?attach=watch",
      env: { CIVIS_WATCH_HTTP: "http://env:9090" },
    }).watchHttp,
    "http://env:9090",
  );
});
