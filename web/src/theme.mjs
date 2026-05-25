/** Shared light/dark theme helpers for civ-watch dashboard and status page. */

export const THEME_STORAGE_KEY = "data-theme";
export const LEGACY_THEME_STORAGE_KEY = "civis-theme";
export const DEFAULT_THEME = "dark";

/** @typedef {"dark" | "light"} ThemeMode */

/** @param {string | null | undefined} value @returns {ThemeMode | null} */
export function normalizeTheme(value) {
  return value === "light" || value === "dark" ? value : null;
}

/** @param {string} search @returns {ThemeMode | null} */
export function readThemeFromQuery(search = "") {
  const query = search.startsWith("?") ? search.slice(1) : search;
  const theme = new URLSearchParams(query).get("theme")?.trim().toLowerCase();
  return normalizeTheme(theme);
}

/**
 * Read persisted theme. Precedence: `?theme=` → localStorage `data-theme` → legacy `civis-theme` → default.
 * @param {{ storage?: Storage | null; search?: string }} [opts]
 * @returns {ThemeMode}
 */
export function readStoredTheme(opts = {}) {
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

/**
 * @param {ThemeMode} theme
 * @param {Document | null} [doc]
 */
export function applyDocumentTheme(theme, doc = typeof document !== "undefined" ? document : null) {
  if (!doc) return;
  doc.documentElement.dataset.theme = theme;
}

/**
 * @param {ThemeMode} theme
 * @param {Storage | null} [storage]
 */
export function persistTheme(theme, storage = typeof localStorage !== "undefined" ? localStorage : null) {
  if (!storage) return;
  storage.setItem(THEME_STORAGE_KEY, theme);
  storage.removeItem(LEGACY_THEME_STORAGE_KEY);
}

/** @param {ThemeMode} current @returns {ThemeMode} */
export function toggleTheme(current) {
  return current === "dark" ? "light" : "dark";
}

/**
 * @param {ThemeMode} theme
 * @param {{ storage?: Storage | null; doc?: Document | null }} [opts]
 */
export function setTheme(theme, opts = {}) {
  persistTheme(theme, opts.storage);
  applyDocumentTheme(theme, opts.doc);
}

/**
 * @param {ThemeMode} current
 * @param {{ storage?: Storage | null; doc?: Document | null }} [opts]
 * @returns {ThemeMode}
 */
export function flipTheme(current, opts = {}) {
  const next = toggleTheme(current);
  setTheme(next, opts);
  return next;
}

/** @param {ThemeMode} theme @returns {string} */
export function themeToggleLabel(theme) {
  return theme === "dark" ? "Switch to light theme" : "Switch to dark theme";
}
