/** Server JSON-RPC method for spawn palette (mirrors dashboard authoring.ts). */
export function serverSpawnMethod(kind) {
  return kind === "civilian" ? "sim.spawn_civilian" : "sim.spawn_entity";
}

/** civ-watch HTTP path for all spawn kinds. */
export function watchSpawnPath() {
  return "/control/spawn_entity";
}

/** Build spawn params for server or watch. */
export function spawnParams(kind, normX, normY, faction) {
  if (kind === "civilian") {
    return { x: normX, y: normY, faction };
  }
  return { kind, x: normX, y: normY, faction };
}
