/** FR-CIV-WEB-007 — optional Babylon.js viewer (rendering only). */

export type RendererMode = "three" | "babylon";

export function resolveRendererMode(
  search: string,
  env: Record<string, string | undefined> = {},
): RendererMode {
  const params = new URLSearchParams(search.startsWith("?") ? search : `?${search}`);
  const query = params.get("renderer")?.trim().toLowerCase();
  if (query === "babylon" || query === "three") return query;
  const fromEnv = (env.CIVIS_RENDERER ?? import.meta.env.VITE_CIVIS_RENDERER ?? "")
    .trim()
    .toLowerCase();
  if (fromEnv === "babylon" || fromEnv === "three") return fromEnv;
  return "three";
}
