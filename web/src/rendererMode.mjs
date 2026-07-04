/** FR-CIV-WEB-007 — optional Babylon.js viewer (rendering only). */

/**
 * @param {string} search
 * @param {Record<string, string | undefined>} [env]
 * @returns {"three" | "babylon"}
 */
export function resolveRendererMode(search, env = {}) {
  const params = new URLSearchParams(search.startsWith("?") ? search : `?${search}`);
  const query = params.get("renderer")?.trim().toLowerCase();
  if (query === "babylon" || query === "three") return query;
  const fromEnv = (env.CIVIS_RENDERER ?? env.VITE_CIVIS_RENDERER ?? "").trim().toLowerCase();
  if (fromEnv === "babylon" || fromEnv === "three") return fromEnv;
  return "three";
}
