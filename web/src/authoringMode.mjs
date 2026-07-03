/**
 * Web L2 authoring toggle (amends ADR-009 spectator default).
 * @param {string} search
 * @returns {boolean}
 */
export function resolveAuthoringEnabled(search) {
  const params = new URLSearchParams(search.startsWith("?") ? search : `?${search}`);
  const spectator = params.get("spectator")?.trim().toLowerCase();
  if (spectator === "1" || spectator === "true" || spectator === "yes") return false;
  const authoring = params.get("authoring")?.trim().toLowerCase();
  if (authoring === "0" || authoring === "false" || authoring === "no") return false;
  return true;
}

/**
 * Human-readable label for the current dashboard interaction mode.
 * @param {boolean} readOnly
 * @returns {string}
 */
export function authoringModeLabel(readOnly) {
  return readOnly ? "Spectator mode" : "Authoring enabled";
}
