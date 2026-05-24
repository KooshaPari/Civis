/** Shared light/dark theme helpers (mirror of web/src/theme.mjs). */

export type ThemeMode = "dark" | "light";

export const THEME_STORAGE_KEY = "data-theme";
export const LEGACY_THEME_STORAGE_KEY = "civis-theme";
export const DEFAULT_THEME: ThemeMode = "dark";

export function normalizeTheme(value: string | null | undefined): ThemeMode | null {
  return value === "light" || value === "dark" ? value : null;
}

export function readThemeFromQuery(search = ""): ThemeMode | null {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const theme = new URLSearchParams(query).get("theme")?.trim().toLowerCase();
  return normalizeTheme(theme);
}

export function readStoredTheme(opts: { storage?: Storage | null; search?: string } = {}): ThemeMode {
  const fromUrl = opts.search != null ? readThemeFromQuery(opts.search) : null;
  if (fromUrl) return fromUrl;

  const storage =
    opts.storage ?? (typeof localStorage !== "undefined" ? localStorage : null);
  if (storage) {
    const stored =
      normalizeTheme(storage.getItem(THEME_STORAGE_KEY)) ??
      normalizeTheme(storage.getItem(LEGACY_THEME_STORAGE_KEY));
    if (stored) return stored;
  }

  return DEFAULT_THEME;
}

export function applyDocumentTheme(
  theme: ThemeMode,
  doc: Document | null = typeof document !== "undefined" ? document : null,
): void {
  if (!doc) return;
  doc.documentElement.dataset.theme = theme;
}

export function persistTheme(
  theme: ThemeMode,
  storage: Storage | null = typeof localStorage !== "undefined" ? localStorage : null,
): void {
  if (!storage) return;
  storage.setItem(THEME_STORAGE_KEY, theme);
  storage.removeItem(LEGACY_THEME_STORAGE_KEY);
}

export function toggleTheme(current: ThemeMode): ThemeMode {
  return current === "dark" ? "light" : "dark";
}

export function setTheme(
  theme: ThemeMode,
  opts: { storage?: Storage | null; doc?: Document | null } = {},
): void {
  persistTheme(theme, opts.storage);
  applyDocumentTheme(theme, opts.doc);
}

export function flipTheme(
  current: ThemeMode,
  opts: { storage?: Storage | null; doc?: Document | null } = {},
): ThemeMode {
  const next = toggleTheme(current);
  setTheme(next, opts);
  return next;
}

export function themeToggleLabel(theme: ThemeMode): string {
  return theme === "dark" ? "Switch to light theme" : "Switch to dark theme";
}
