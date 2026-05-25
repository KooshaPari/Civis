import type { SpawnKind } from "../store";

/** Grid cells between convoy spawns along a drag path. */
export const CONVOY_SPACING_CELLS = 8;

/** Cap entities placed from one drag gesture. */
export const CONVOY_MAX_SPAWNS = 32;

const DRAG_MIN_CELLS = 4;

/** Kinds that support drag-release and convoy sampling. */
export function spawnKindUsesConvoy(kind: SpawnKind): boolean {
  return (
    kind === "vehicle" ||
    kind === "airport" ||
    kind === "port" ||
    kind === "hangar"
  );
}

/** Terrain cells to spawn along a drag segment (inclusive endpoints). */
export function convoyCells(
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  terrainSize: number,
): Array<{ cellX: number; cellY: number }> {
  const dist = Math.hypot(endX - startX, endY - startY);
  if (dist < DRAG_MIN_CELLS) {
    return [{ cellX: endX, cellY: endY }];
  }
  const steps = Math.min(
    CONVOY_MAX_SPAWNS - 1,
    Math.max(1, Math.floor(dist / CONVOY_SPACING_CELLS)),
  );
  const out: Array<{ cellX: number; cellY: number }> = [];
  for (let i = 0; i <= steps; i++) {
    const t = steps === 0 ? 1 : i / steps;
    out.push({
      cellX: Math.round(startX + (endX - startX) * t),
      cellY: Math.round(startY + (endY - startY) * t),
    });
  }
  const cap = out.slice(0, CONVOY_MAX_SPAWNS);
  const last = cap[cap.length - 1];
  if (last && (last.cellX !== endX || last.cellY !== endY)) {
    if (cap.length >= CONVOY_MAX_SPAWNS) {
      cap[cap.length - 1] = { cellX: endX, cellY: endY };
    } else {
      cap.push({ cellX: endX, cellY: endY });
    }
  }
  return cap;
}
