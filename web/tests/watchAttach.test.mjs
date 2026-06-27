import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveAttachConfig } from "../src/attachConfig.mjs";
import {
  DEFAULT_WATCH_HTTP,
  joinWatchHttp,
  resolveWatchAttachUrls,
  WATCH_API_ROUTES,
  WATCH_CLIENT_PATHS,
  WATCH_VITE_PROXY_PREFIXES,
} from "../src/watchAttach.mjs";

function readFixture(relativeUrl) {
  return readFileSync(new URL(relativeUrl, import.meta.url), "utf8");
}

test("watch client paths match civ-watch API routes", () => {
  assert.equal(WATCH_CLIENT_PATHS.sse, WATCH_API_ROUTES.events);
  assert.equal(WATCH_CLIENT_PATHS.snapshot, WATCH_API_ROUTES.snapshot);
  assert.equal(WATCH_CLIENT_PATHS.terrain, WATCH_API_ROUTES.terrain);
  assert.equal(WATCH_CLIENT_PATHS.controlSpeed, WATCH_API_ROUTES.controlSpeed);
});

test("civ-watch route table includes dashboard SSE/HTTP endpoints", () => {
  assert.deepEqual(
    [
      WATCH_API_ROUTES.terrain,
      WATCH_API_ROUTES.snapshot,
      WATCH_API_ROUTES.events,
      WATCH_API_ROUTES.controlSpeed,
    ],
    ["/terrain", "/snapshot", "/events", "/control/speed"],
  );
});

test("vite proxy prefixes cover watch client paths", () => {
  for (const path of Object.values(WATCH_CLIENT_PATHS)) {
    assert.ok(
      WATCH_VITE_PROXY_PREFIXES.some(
        (prefix) => path === prefix || path.startsWith(`${prefix}/`),
      ),
      `missing vite proxy for ${path}`,
    );
  }
});

test("?attach=watch resolves default civ-watch base URL", () => {
  const cfg = resolveAttachConfig({ search: "?attach=watch" });
  assert.equal(cfg.mode, "watch");
  assert.equal(cfg.watchHttp, DEFAULT_WATCH_HTTP);

  const urls = resolveWatchAttachUrls({ search: "?attach=watch" });
  assert.equal(urls.mode, "watch");
  assert.equal(urls.base, DEFAULT_WATCH_HTTP);
  assert.equal(urls.terrain, "http://127.0.0.1:9090/terrain");
  assert.equal(urls.snapshot, "http://127.0.0.1:9090/snapshot");
  assert.equal(urls.sse, "http://127.0.0.1:9090/events");
  assert.equal(urls.controlSpeed, "http://127.0.0.1:9090/control/speed");
});

test("?watch= query overrides default civ-watch base", () => {
  const cfg = resolveAttachConfig({
    search: "?attach=watch&watch=http://localhost:7777",
  });
  assert.equal(cfg.watchHttp, "http://localhost:7777");

  const urls = resolveWatchAttachUrls({
    search: "?attach=watch&watch=http://localhost:7777",
  });
  assert.equal(urls.terrain, "http://localhost:7777/terrain");
  assert.equal(urls.sse, "http://localhost:7777/events");
});

test("CIVIS_WATCH_HTTP env overrides default civ-watch base", () => {
  const cfg = resolveAttachConfig({
    search: "?attach=watch",
    env: { CIVIS_WATCH_HTTP: "http://127.0.0.1:9191" },
  });
  assert.equal(cfg.watchHttp, "http://127.0.0.1:9191");
});

test("joinWatchHttp strips trailing slash from base", () => {
  assert.equal(
    joinWatchHttp("http://127.0.0.1:9090/", "/terrain"),
    "http://127.0.0.1:9090/terrain",
  );
});

test("watch-mode attach docs stay aligned with code routes and defaults", () => {
  const readme = readFixture("../dashboard/README.md");
  const matrix = readFixture("../../docs/guides/client-attach-matrix.md");

  assert.ok(readme.includes("http://localhost:9090/snapshot"));
  assert.ok(readme.includes("http://localhost:9090/terrain"));
  assert.ok(readme.includes("http://localhost:9090/events"));
  assert.ok(matrix.includes("POST /control/*"));
  assert.ok(matrix.includes("http://127.0.0.1:9090/terrain"));
  assert.ok(matrix.includes("http://127.0.0.1:9090"));

  assert.equal(WATCH_CLIENT_PATHS.sse, WATCH_API_ROUTES.events);
  assert.equal(WATCH_CLIENT_PATHS.snapshot, WATCH_API_ROUTES.snapshot);
  assert.equal(WATCH_CLIENT_PATHS.terrain, WATCH_API_ROUTES.terrain);
  assert.equal(WATCH_CLIENT_PATHS.controlSpeed, WATCH_API_ROUTES.controlSpeed);
});
