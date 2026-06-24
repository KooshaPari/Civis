import {
  defaultLiveWsUrl,
  resolveWsUrlFromEnv,
  resolveWsUrlFromQuery,
  withTickFormatBinaryQuery,
  wsPreferBinary,
} from "./wsUrl.mjs";

/** @typedef {"watch" | "server"} AttachMode */

/**
 * Resolve how the dashboard attaches to Civis.
 *
 * Precedence:
 * 1. `?attach=watch|server`
 * 2. `CIVIS_ATTACH=watch|server` (Node / build-time)
 * 3. `server` when `CIVIS_WS_URL` or `CIVIS_WS_ADDR` is set
 * 4. `watch` (civ-watch SSE) for local dev without server
 *
 * @param {{ search?: string; env?: Record<string, string | undefined> }} [opts]
 * @returns {{ mode: AttachMode; wsUrl: string; watchHttp: string; preferBinary: boolean }}
 */
export function resolveAttachConfig(opts = {}) {
  const env = opts.env ?? {};
  const search = opts.search ?? "";
  const query = search.startsWith("?") ? search.slice(1) : search;
  const params = new URLSearchParams(query);

  const attachParam = params.get("attach")?.trim().toLowerCase();
  const envAttach = env.CIVIS_ATTACH?.trim().toLowerCase();

  let mode = /** @type {AttachMode} */ ("server");
  if (attachParam === "server" || attachParam === "watch") {
    mode = attachParam;
  } else if (envAttach === "server" || envAttach === "watch") {
    mode = envAttach;
  }

  const preferBinary = wsPreferBinary(search, env);
  let wsUrl = resolveWsUrlFromQuery(search, resolveWsUrlFromEnv(env));
  if (preferBinary) {
    wsUrl = withTickFormatBinaryQuery(wsUrl);
  }
  const watchHttp =
    env.CIVIS_WATCH_HTTP?.trim() ||
    params.get("watch")?.trim() ||
    "http://127.0.0.1:9090";

  return { mode, wsUrl, watchHttp, preferBinary };
}

export { defaultLiveWsUrl };
