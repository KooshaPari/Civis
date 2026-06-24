import assert from "node:assert/strict";
import { test } from "node:test";
import {
  DEFAULT_THEME,
  LEGACY_THEME_STORAGE_KEY,
  THEME_STORAGE_KEY,
  applyDocumentTheme,
  flipTheme,
  normalizeTheme,
  persistTheme,
  readStoredTheme,
  readThemeFromQuery,
  setTheme,
  themeToggleLabel,
  toggleTheme,
} from "../src/theme.mjs";

test("normalizeTheme accepts only dark and light", () => {
  assert.equal(normalizeTheme("dark"), "dark");
  assert.equal(normalizeTheme("light"), "light");
  assert.equal(normalizeTheme("auto"), null);
  assert.equal(normalizeTheme(null), null);
});

test("readThemeFromQuery reads ?theme= param", () => {
  assert.equal(readThemeFromQuery("?theme=light"), "light");
  assert.equal(readThemeFromQuery("?theme=dark&ws=ws://x"), "dark");
  assert.equal(readThemeFromQuery("?ws=ws://x"), null);
});

test("readStoredTheme prefers query, then data-theme, then legacy civis-theme", () => {
  const storage = {
    data: { [THEME_STORAGE_KEY]: "light", [LEGACY_THEME_STORAGE_KEY]: "dark" },
    getItem(key) {
      return this.data[key] ?? null;
    },
    setItem(key, value) {
      this.data[key] = value;
    },
    removeItem(key) {
      delete this.data[key];
    },
  };

  assert.equal(readStoredTheme({ search: "?theme=dark", storage }), "dark");
  assert.equal(readStoredTheme({ storage }), "light");
  delete storage.data[THEME_STORAGE_KEY];
  assert.equal(readStoredTheme({ storage }), "dark");
  assert.equal(readStoredTheme({}), DEFAULT_THEME);
});

test("persistTheme writes data-theme and clears legacy key", () => {
  const storage = {
    data: { [LEGACY_THEME_STORAGE_KEY]: "dark" },
    getItem(key) {
      return this.data[key] ?? null;
    },
    setItem(key, value) {
      this.data[key] = value;
    },
    removeItem(key) {
      delete this.data[key];
    },
  };

  persistTheme("light", storage);
  assert.equal(storage.data[THEME_STORAGE_KEY], "light");
  assert.equal(storage.data[LEGACY_THEME_STORAGE_KEY], undefined);
});

test("applyDocumentTheme sets html data-theme attribute", () => {
  const doc = { documentElement: { dataset: {} } };
  applyDocumentTheme("light", doc);
  assert.equal(doc.documentElement.dataset.theme, "light");
});

test("toggleTheme and themeToggleLabel", () => {
  assert.equal(toggleTheme("dark"), "light");
  assert.equal(toggleTheme("light"), "dark");
  assert.match(themeToggleLabel("dark"), /light/i);
  assert.match(themeToggleLabel("light"), /dark/i);
});

test("setTheme and flipTheme persist and apply", () => {
  const storage = {
    data: {},
    getItem(key) {
      return this.data[key] ?? null;
    },
    setItem(key, value) {
      this.data[key] = value;
    },
    removeItem(key) {
      delete this.data[key];
    },
  };
  const doc = { documentElement: { dataset: {} } };

  setTheme("dark", { storage, doc });
  assert.equal(storage.data[THEME_STORAGE_KEY], "dark");
  assert.equal(doc.documentElement.dataset.theme, "dark");

  assert.equal(flipTheme("dark", { storage, doc }), "light");
  assert.equal(storage.data[THEME_STORAGE_KEY], "light");
  assert.equal(doc.documentElement.dataset.theme, "light");
});
