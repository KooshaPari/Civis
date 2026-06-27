/** Web-only infoview overlay registry + toggle helpers. */

export const INFOVIEW_OVERLAYS = Object.freeze([
  Object.freeze({
    id: "agents",
    label: "Agents",
    description: "Show agent-related status and debug overlays.",
    defaultEnabled: true,
  }),
  Object.freeze({
    id: "mods",
    label: "Mods",
    description: "Show mod lifecycle and attachment overlays.",
    defaultEnabled: true,
  }),
  Object.freeze({
    id: "perf",
    label: "Performance",
    description: "Show frame timing and render budget overlays.",
    defaultEnabled: false,
  }),
]);

const OVERLAY_BY_ID = new Map(INFOVIEW_OVERLAYS.map((overlay) => [overlay.id, overlay]));

/**
 * @param {string} id
 * @returns {boolean}
 */
export function isKnownInfoviewOverlay(id) {
  return OVERLAY_BY_ID.has(id);
}

/**
 * @returns {string[]}
 */
export function listInfoviewOverlayIds() {
  return INFOVIEW_OVERLAYS.map((overlay) => overlay.id);
}

/**
 * @param {string} id
 * @returns {{ id: string; label: string; description: string; defaultEnabled: boolean } | null}
 */
export function getInfoviewOverlay(id) {
  return OVERLAY_BY_ID.get(id) ?? null;
}

/**
 * @param {Record<string, boolean> | null | undefined} [state]
 * @returns {Record<string, boolean>}
 */
export function createInfoviewOverlayState(state = undefined) {
  const next = {};
  for (const overlay of INFOVIEW_OVERLAYS) {
    next[overlay.id] = state?.[overlay.id] ?? overlay.defaultEnabled;
  }
  return next;
}

/**
 * @param {Record<string, boolean>} state
 * @param {string} id
 * @returns {boolean}
 */
export function getInfoviewOverlayEnabled(state, id) {
  const overlay = OVERLAY_BY_ID.get(id);
  return overlay ? Boolean(state[overlay.id] ?? overlay.defaultEnabled) : false;
}

/**
 * @param {Record<string, boolean>} state
 * @param {string} id
 * @param {boolean} [enabled]
 * @returns {Record<string, boolean>}
 */
export function setInfoviewOverlayEnabled(state, id, enabled) {
  if (!OVERLAY_BY_ID.has(id)) return { ...state };
  return { ...state, [id]: enabled ?? !getInfoviewOverlayEnabled(state, id) };
}

/**
 * @param {Record<string, boolean>} state
 * @param {string} id
 * @returns {Record<string, boolean>}
 */
export function toggleInfoviewOverlay(state, id) {
  return setInfoviewOverlayEnabled(state, id);
}
