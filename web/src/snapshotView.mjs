/**
 * FR-CIV-WEB-003 — derive read-only scene counts from a watch-style snapshot.
 * @param {Record<string, unknown> | null | undefined} snapshot
 */
export function sceneEntityCounts(snapshot) {
  if (!snapshot) {
    return { civilians: 0, buildings: 0, factions: 0, total: 0 };
  }
  const civs = Array.isArray(snapshot.civ_pins) ? snapshot.civ_pins.length : 0;
  const buildings = Array.isArray(snapshot.buildings) ? snapshot.buildings.length : 0;
  const factions = Array.isArray(snapshot.factions) ? snapshot.factions.length : 0;
  return {
    civilians: civs,
    buildings,
    factions,
    total: civs + buildings + factions,
  };
}
