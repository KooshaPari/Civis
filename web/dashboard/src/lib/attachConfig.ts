/** Web L2 authoring (spawn/voxel). Disabled with ?spectator=1 or ?authoring=0. */
export function resolveAuthoringEnabled(search = ""): boolean {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const params = new URLSearchParams(query);
  const spectator = params.get("spectator")?.trim().toLowerCase();
  if (spectator === "1" || spectator === "true" || spectator === "yes") return false;
  const authoring = params.get("authoring")?.trim().toLowerCase();
  if (authoring === "0" || authoring === "false" || authoring === "no") return false;
  return true;
}

/** Resolve attach mode for the dashboard. */

export type AttachMode = "watch" | "server";

export const DEFAULT_WS_PREFER_BINARY = true;

export function parseWsBinaryEnvFlag(value: string): boolean {
  const trimmed = value.trim();
  if (["1", "true", "TRUE", "yes", "YES", "on", "ON"].includes(trimmed)) {
    return true;
  }
  if (["0", "false", "FALSE", "no", "NO", "off", "OFF"].includes(trimmed)) {
    return false;
  }
  return false;
}

export function parseWsBinaryQueryParam(search = ""): boolean | undefined {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const raw = new URLSearchParams(query).get("binary")?.trim();
  if (!raw) return undefined;
  return parseWsBinaryEnvFlag(raw);
}

/** `?binary=` overrides `VITE_CIVIS_WS_BINARY`, else default true (bevy-ref). */
export function resolveWsPreferBinary(search = ""): boolean {
  const fromQuery = parseWsBinaryQueryParam(search);
  if (fromQuery !== undefined) return fromQuery;
  const fromEnv = import.meta.env.VITE_CIVIS_WS_BINARY?.trim();
  if (fromEnv) return parseWsBinaryEnvFlag(fromEnv);
  return DEFAULT_WS_PREFER_BINARY;
}

export function withTickFormatBinaryQuery(url: string): string {
  if (url.includes("tick_format=")) return url;
  const separator = url.includes("?") ? "&" : "?";
  return `${url}${separator}tick_format=binary`;
}

export function resolveAttachMode(search = ""): AttachMode {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const attach = new URLSearchParams(query).get("attach")?.trim().toLowerCase();
  if (attach === "server" || attach === "watch") return attach;
  return "server";
}

/** WebSocket URL: use Vite proxy in dev (`/ws` → civ-server). */
export function resolveBrowserWsUrl(search = ""): string {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const fromQuery = new URLSearchParams(query).get("ws")?.trim();
  let url: string;
  if (fromQuery) {
    url = fromQuery;
  } else {
    const fromEnv = import.meta.env.VITE_CIVIS_WS_URL as string | undefined;
    if (fromEnv?.trim()) {
      url = fromEnv.trim();
    } else if (typeof window !== "undefined") {
      const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
      url = `${proto}//${window.location.host}/ws`;
    } else {
      url = `ws://127.0.0.1:${import.meta.env.VITE_CIV_SERVER_PORT ?? "3000"}/ws`;
    }
  }
  if (resolveWsPreferBinary(search)) {
    return withTickFormatBinaryQuery(url);
  }
  return url;
}
