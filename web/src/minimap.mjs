/** Minimap UV helpers — mirrors `civ-bevy-ref::chunk_to_minimap_uv` and friends. */

/** @typedef {[number, number, number, number]} MinimapBounds */

/**
 * @param {number | bigint | string} chunkId
 * @returns {[number, number, number]}
 */
export function decodeChunkId(chunkId) {
  const raw = BigInt.asUintN(64, BigInt(chunkId));
  let cx = Number((raw >> 40n) & 0xffffffn);
  let cy = Number((raw >> 16n) & 0xffffffn);
  let cz = Number(raw & 0xffffn);
  if (cx & 0x800000) cx |= ~0xffffff;
  if (cy & 0x800000) cy |= ~0xffffff;
  if (cz & 0x8000) cz |= ~0xffff;
  return [cx, cy, cz];
}

/**
 * @param {number} cx
 * @param {number} cy
 * @param {number} cz
 * @returns {number}
 */
export function encodeChunkId(cx, cy, cz) {
  const raw =
    (BigInt.asUintN(64, BigInt(cx)) << 40n) |
    (BigInt.asUintN(64, BigInt(cy)) << 16n) |
    BigInt.asUintN(64, BigInt(cz));
  return Number(raw);
}

/**
 * @param {number | bigint | string} chunkId
 * @param {MinimapBounds} bounds
 * @returns {[number, number]}
 */
export function chunkToMinimapUv(chunkId, bounds) {
  const [cx, , cz] = decodeChunkId(chunkId);
  const [minX, minZ, maxX, maxZ] = bounds;
  const spanX = Math.max(maxX - minX + 1, 1);
  const spanZ = Math.max(maxZ - minZ + 1, 1);
  const clampUv = (value) => Math.min(Math.max(value, 0), 1);
  return [
    clampUv((cx - minX) / spanX + 0.5 / spanX),
    clampUv((cz - minZ) / spanZ + 0.5 / spanZ),
  ];
}

/**
 * @param {[number, number]} uv
 * @param {MinimapBounds} bounds
 * @returns {[number, number]}
 */
export function minimapUvToChunkGrid(uv, bounds) {
  const [minX, minZ, maxX, maxZ] = bounds;
  const spanX = Math.max(maxX - minX + 1, 1);
  const spanZ = Math.max(maxZ - minZ + 1, 1);
  const u = Number.isFinite(uv[0]) ? uv[0] : 0;
  const v = Number.isFinite(uv[1]) ? uv[1] : 0;
  const cx = Math.floor(u * spanX + minX);
  const cz = Math.floor(v * spanZ + minZ);
  return [
    Math.min(Math.max(cx, minX), maxX),
    Math.min(Math.max(cz, minZ), maxZ),
  ];
}

/**
 * @param {Array<number | bigint | string>} chunkKeys
 * @returns {MinimapBounds | null}
 */
export function minimapBoundsFromKeys(chunkKeys) {
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

/**
 * @param {number[]} chunkIds
 * @param {number} cx
 * @param {number} cz
 * @returns {number | null}
 */
export function findChunkAtGrid(chunkIds, cx, cz) {
  for (const id of chunkIds) {
    const [x, , z] = decodeChunkId(id);
    if (x === cx && z === cz) return id;
  }
  return null;
}

/**
 * Track seen chunk ids and return the most recently observed ids (newest first).
 * @param {Set<number>} seen
 * @param {number[]} recent
 * @param {number[]} ids
 * @param {number} [recentCap]
 */
export function noteChunkIds(seen, recent, ids, recentCap = 5) {
  if (!ids.length) return { count: seen.size, recentIds: [...recent] };
  const cap = Math.max(0, Math.trunc(recentCap));
  if (cap === 0) {
    for (const id of ids) seen.add(id);
    return { count: seen.size, recentIds: [] };
  }

  const nextRecent = [...recent];
  for (const id of ids) {
    seen.add(id);
    const existing = nextRecent.indexOf(id);
    if (existing !== -1) {
      nextRecent.splice(existing, 1);
    }
    nextRecent.unshift(id);
  }
  if (nextRecent.length > cap) nextRecent.length = cap;
  return { count: seen.size, recentIds: nextRecent };
}
