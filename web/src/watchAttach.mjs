import { resolveAttachConfig } from "./attachConfig.mjs";

/** Default civ-watch listen URL (`CIV_WATCH_PORT` default 9090). */
export const DEFAULT_WATCH_HTTP = "http://127.0.0.1:9090";

/**
 * civ-watch HTTP/SSE routes — keep in sync with `crates/watch/src/main.rs`
 * `build_api_router`.
 */
export const WATCH_API_ROUTES = Object.freeze({
  events: "/events",
  snapshot: "/snapshot",
  terrain: "/terrain",
  controlPlaceVoxel: "/control/place_voxel",
  controlSpawnCivilian: "/control/spawn_civilian",
  controlDamage: "/control/damage",
  controlSpeed: "/control/speed",
});

/** Relative paths the dashboard uses in watch mode (same origin or Vite proxy). */
export const WATCH_CLIENT_PATHS = Object.freeze({
  sse: WATCH_API_ROUTES.events,
  snapshot: WATCH_API_ROUTES.snapshot,
  terrain: WATCH_API_ROUTES.terrain,
  controlSpeed: WATCH_API_ROUTES.controlSpeed,
});

/** Vite dev-server proxy prefixes that forward to civ-watch. */
export const WATCH_VITE_PROXY_PREFIXES = Object.freeze([
  WATCH_API_ROUTES.events,
  WATCH_API_ROUTES.snapshot,
  WATCH_API_ROUTES.terrain,
  "/control",
]);

export function joinWatchHttp(baseHttp, path) {
  const base = baseHttp.replace(/\/$/, "");
  return `${base}${path}`;
}

/**
 * Resolve absolute civ-watch URLs for attach smoke tests and tooling.
 *
 * @param {{ search?: string; env?: Record<string, string | undefined> }} [opts]
 */
export function resolveWatchAttachUrls(opts = {}) {
  const { mode, watchHttp } = resolveAttachConfig(opts);
  return {
    mode,
    base: watchHttp,
    sse: joinWatchHttp(watchHttp, WATCH_API_ROUTES.events),
    snapshot: joinWatchHttp(watchHttp, WATCH_API_ROUTES.snapshot),
    terrain: joinWatchHttp(watchHttp, WATCH_API_ROUTES.terrain),
    controlSpeed: joinWatchHttp(watchHttp, WATCH_API_ROUTES.controlSpeed),
    clientPaths: WATCH_CLIENT_PATHS,
  };
}
