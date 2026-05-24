/** Default Civis JSON-RPC WebSocket attach (matches `civ-server` `CIVIS_WS_ADDR` default). */
export const DEFAULT_WS_HOST = "127.0.0.1";
export const DEFAULT_WS_PORT = 3000;
export const DEFAULT_WS_PATH = "/ws";

/** When true, skip redundant JSON text tick frames (matches `civ-bevy-ref`). */
export const DEFAULT_WS_PREFER_BINARY = true;

/**
 * Build a WebSocket URL for the Civis live attach path (`ws://host:port/path`).
 * Mirrors `civ-bevy-ref::live_ws_url` (host/port/path only; port differs from bevy default).
 */
export function liveWsUrl(host, port, path = DEFAULT_WS_PATH) {
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `ws://${host}:${port}${normalizedPath}`;
}

/** Default local attach URL (`127.0.0.1:3000/ws`). */
export function defaultLiveWsUrl() {
  return liveWsUrl(DEFAULT_WS_HOST, DEFAULT_WS_PORT, DEFAULT_WS_PATH);
}

/**
 * Resolve the live WebSocket URL from environment variables.
 *
 * Precedence:
 * 1. `CIVIS_WS_URL` — full `ws://…` URL
 * 2. `CIVIS_WS_ADDR` — `host:port` (same as `civ-server` main), optional `CIVIS_WS_PATH`
 * 3. {@link defaultLiveWsUrl}
 *
 * @param {Record<string, string | undefined>} [env]
 */
export function resolveWsUrlFromEnv(env = {}) {
  const full = env.CIVIS_WS_URL?.trim();
  if (full) {
    return full;
  }

  const addr = env.CIVIS_WS_ADDR?.trim();
  if (addr) {
    const lastColon = addr.lastIndexOf(":");
    const host = lastColon > 0 ? addr.slice(0, lastColon) : addr;
    const port =
      lastColon > 0 ? Number(addr.slice(lastColon + 1)) : DEFAULT_WS_PORT;
    const path = env.CIVIS_WS_PATH?.trim() || DEFAULT_WS_PATH;
    return liveWsUrl(host, Number.isFinite(port) ? port : DEFAULT_WS_PORT, path);
  }

  return defaultLiveWsUrl();
}

/**
 * Resolve a WebSocket URL from a browser query string (`?ws=ws://…`).
 * @param {string} [search]
 * @param {string} [fallback]
 */
export function resolveWsUrlFromQuery(search = "", fallback) {
  const fb = fallback ?? defaultLiveWsUrl();
  const query = search.startsWith("?") ? search.slice(1) : search;
  const ws = new URLSearchParams(query).get("ws")?.trim();
  return ws || fb;
}

/**
 * Returns true for common truthy env strings used by `CIVIS_WS_BINARY`.
 * @param {string} value
 */
export function parseWsBinaryEnvFlag(value) {
  const trimmed = value.trim();
  if (["1", "true", "TRUE", "yes", "YES", "on", "ON"].includes(trimmed)) {
    return true;
  }
  if (["0", "false", "FALSE", "no", "NO", "off", "OFF"].includes(trimmed)) {
    return false;
  }
  return false;
}

/**
 * @param {Record<string, string | undefined>} [env]
 */
export function wsPreferBinaryFromEnv(env = {}) {
  const raw = env.CIVIS_WS_BINARY?.trim();
  if (raw) return parseWsBinaryEnvFlag(raw);
  return DEFAULT_WS_PREFER_BINARY;
}

/**
 * Parse `?binary=1` (or `0` / `false`) when present.
 * @param {string} [search]
 * @returns {boolean | undefined}
 */
export function parseWsBinaryQueryParam(search = "") {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const raw = new URLSearchParams(query).get("binary")?.trim();
  if (!raw) return undefined;
  return parseWsBinaryEnvFlag(raw);
}

/**
 * Resolve binary-first tick handling: `?binary=` overrides env, else `CIVIS_WS_BINARY`.
 * @param {string} [search]
 * @param {Record<string, string | undefined>} [env]
 */
export function wsPreferBinary(search = "", env = {}) {
  const fromQuery = parseWsBinaryQueryParam(search);
  if (fromQuery !== undefined) return fromQuery;
  return wsPreferBinaryFromEnv(env);
}

/**
 * Append `tick_format=binary` when absent so `civ-server` may switch to binary-only ticks.
 * @param {string} url
 */
export function withTickFormatBinaryQuery(url) {
  if (url.includes("tick_format=")) return url;
  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}tick_format=binary`;
}
