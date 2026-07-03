import assert from "node:assert/strict";
import { test } from "node:test";
import {
  INFOVIEW_OVERLAYS,
  createInfoviewOverlayState,
  getInfoviewOverlay,
  getInfoviewOverlayEnabled,
  isKnownInfoviewOverlay,
  listInfoviewOverlayIds,
  setInfoviewOverlayEnabled,
  toggleInfoviewOverlay,
} from "../src/infoview.mjs";

test("infoview registry exposes stable overlay ids", () => {
  assert.deepEqual(listInfoviewOverlayIds(), ["agents", "mods", "perf"]);
  assert.equal(INFOVIEW_OVERLAYS.length, 3);
  assert.equal(isKnownInfoviewOverlay("mods"), true);
  assert.equal(isKnownInfoviewOverlay("unknown"), false);
});

test("infoview registry returns overlay metadata", () => {
  assert.deepEqual(getInfoviewOverlay("agents"), {
    id: "agents",
    label: "Agents",
    description: "Show agent-related status and debug overlays.",
    defaultEnabled: true,
  });
  assert.equal(getInfoviewOverlay("missing"), null);
});

test("createInfoviewOverlayState seeds defaults and preserves overrides", () => {
  assert.deepEqual(createInfoviewOverlayState(), {
    agents: true,
    mods: true,
    perf: false,
  });
  assert.deepEqual(
    createInfoviewOverlayState({ agents: false, perf: true }),
    { agents: false, mods: true, perf: true },
  );
});

test("toggleInfoviewOverlay flips state without mutating the original", () => {
  const state = createInfoviewOverlayState();
  const next = toggleInfoviewOverlay(state, "perf");
  assert.deepEqual(state, { agents: true, mods: true, perf: false });
  assert.deepEqual(next, { agents: true, mods: true, perf: true });
});

test("setInfoviewOverlayEnabled can force a value", () => {
  const state = createInfoviewOverlayState();
  const next = setInfoviewOverlayEnabled(state, "agents", false);
  assert.equal(getInfoviewOverlayEnabled(next, "agents"), false);
  assert.equal(getInfoviewOverlayEnabled(next, "mods"), true);
});

test("unknown overlay ids are ignored safely", () => {
  const state = createInfoviewOverlayState();
  assert.deepEqual(setInfoviewOverlayEnabled(state, "bogus"), state);
  assert.equal(getInfoviewOverlayEnabled(state, "bogus"), false);
});
