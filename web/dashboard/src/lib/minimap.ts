/** Minimap UV helpers — mirrors `web/src/minimap.mjs`. */

export type MinimapBounds = [number, number, number, number];

export function decodeChunkId(chunkId: number | bigint | string): [number, number, number] {
  const raw = BigInt.asUintN(64, BigInt(chunkId));
  let cx = Number((raw >> 40n) & 0xffffffn);
  let cy = Number((raw >> 16n) & 0xffffffn);
  let cz = Number(raw & 0xffffn);
  if (cx & 0x800000) cx |= ~0xffffff;
  if (cy & 0x800000) cy |= ~0xffffff;
  if (cz & 0x8000) cz |= ~0xffff;
  return [cx, cy, cz];
}

export function encodeChunkId(cx: number, cy: number, cz: number): number {
  const raw =
    (BigInt.asUintN(64, BigInt(cx)) << 40n) |
    (BigInt.asUintN(64, BigInt(cy)) << 16n) |
    BigInt.asUintN(64, BigInt(cz));
  return Number(raw);
}

export function chunkToMinimapUv(
  chunkId: number | bigint | string,
  bounds: MinimapBounds,
): [number, number] {
  const [cx, , cz] = decodeChunkId(chunkId);
  const [minX, minZ, maxX, maxZ] = bounds;
  const spanX = Math.max(maxX - minX + 1, 1);
  const spanZ = Math.max(maxZ - minZ + 1, 1);
  return [
    (cx - minX) / spanX + 0.5 / spanX,
    (cz - minZ) / spanZ + 0.5 / spanZ,
  ];
}

export function minimapUvToChunkGrid(
  uv: [number, number],
  bounds: MinimapBounds,
): [number, number] {
  const [minX, minZ, maxX, maxZ] = bounds;
  const spanX = Math.max(maxX - minX + 1, 1);
  const spanZ = Math.max(maxZ - minZ + 1, 1);
  const cx = Math.floor(uv[0] * spanX + minX);
  const cz = Math.floor(uv[1] * spanZ + minZ);
  return [
    Math.min(Math.max(cx, minX), maxX),
    Math.min(Math.max(cz, minZ), maxZ),
  ];
}

export function minimapBoundsFromKeys(
  chunkKeys: Array<number | bigint | string>,
): MinimapBounds | null {
  if (!chunkKeys.length) return null;
  let minX = Number.POSITIVE_INFINITY;
  let minZ = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxZ = Number.NEGATIVE_INFINITY;
  for (const key of chunkKeys) {
    const [cx, , cz] = decodeChunkId(key);
    minX = Math.min(minX, cx);
    minZ = Math.min(minZ, cz);
    maxX = Math.max(maxX, cx);
    maxZ = Math.max(maxZ, cz);
  }
  if (minX === Number.POSITIVE_INFINITY) return null;
  return [minX, minZ, maxX, maxZ];
}

export function findChunkAtGrid(chunkIds: number[], cx: number, cz: number): number | null {
  for (const id of chunkIds) {
    const [x, , z] = decodeChunkId(id);
    if (x === cx && z === cz) return id;
  }
  return null;
}

export function noteChunkIds(
  seen: Set<number>,
  recent: number[],
  ids: number[],
  recentCap = 5,
): { count: number; recentIds: number[] } {
  if (!ids.length) return { count: seen.size, recentIds: [...recent] };

  let nextRecent = [...recent];
  for (const id of ids) {
    seen.add(id);
    nextRecent = [id, ...nextRecent.filter((value) => value !== id)];
  }
  if (nextRecent.length > recentCap) nextRecent.length = recentCap;
  return { count: seen.size, recentIds: nextRecent };
}
