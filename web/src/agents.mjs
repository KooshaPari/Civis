/** Agent helpers — mirrors `civ-bevy-ref::agent_color_from_id` and AgentAppearance parsing. */

const U64_MAX = 18446744073709551615n;
const SPLITMIX64_ADD = 0x9e3779b97f4a7c15n;
const SPLITMIX64_MUL1 = 0xbf58476d1ce4e5b9n;
const SPLITMIX64_MUL2 = 0x94d049bb133111ebn;

/** @param {bigint} value */
function u64ToF32(value) {
  const big = BigInt.asUintN(64, value);
  const lo = Number(big & 0xffffffffn);
  const hi = Number(big >> 32n);
  return Math.fround(hi * 4294967296 + lo);
}

/** @param {number | bigint | string} agentId */
function toAgentIdBigInt(agentId) {
  if (typeof agentId === "bigint") return BigInt.asUintN(64, agentId);
  if (typeof agentId === "number" && Number.isFinite(agentId)) {
    return BigInt.asUintN(64, BigInt(Math.trunc(agentId)));
  }
  if (typeof agentId === "string" && agentId.trim() !== "") {
    return BigInt.asUintN(64, BigInt(agentId.trim()));
  }
  throw new TypeError("agent id must be a finite number or integer string");
}

/** @param {number | bigint | string} agentId */
export function splitmix64(agentId) {
  let value = toAgentIdBigInt(agentId);
  value = BigInt.asUintN(64, value + SPLITMIX64_ADD);
  let z = value;
  z = BigInt.asUintN(64, (z ^ (z >> 30n)) * SPLITMIX64_MUL1);
  z = BigInt.asUintN(64, (z ^ (z >> 27n)) * SPLITMIX64_MUL2);
  return BigInt.asUintN(64, z ^ (z >> 31n));
}

/** @param {number} h @param {number} s @param {number} v @returns {[number, number, number]} */
export function hsvToRgb(h, s, v) {
  const hue = Math.max(0, h - Math.floor(h));
  const i = Math.floor(hue * 6);
  const f = hue * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);
  switch (i % 6) {
    case 0:
      return [v, t, p];
    case 1:
      return [q, v, p];
    case 2:
      return [p, v, t];
    case 3:
      return [p, q, v];
    case 4:
      return [t, p, v];
    default:
      return [v, p, q];
  }
}

/** Deterministic sRGB triple in `[0, 1]` for an opaque agent id (hash → hue). */
/** @param {number | bigint | string} agentId @returns {[number, number, number]} */
export function agentColorFromId(agentId) {
  const hash = splitmix64(agentId);
  const hue = (u64ToF32(hash) / u64ToF32(U64_MAX)) % 1;
  return hsvToRgb(hue, 0.62, 0.88);
}

/** @param {number | bigint | string} agentId */
export function agentColorCss(agentId) {
  const [r, g, b] = agentColorFromId(agentId);
  const ri = Math.round(r * 255);
  const gi = Math.round(g * 255);
  const bi = Math.round(b * 255);
  return `rgb(${ri}, ${gi}, ${bi})`;
}

/** @param {unknown} raw @returns {number | null} */
function normalizeAgentId(raw) {
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string" && raw.trim() !== "") {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

/**
 * Agent ids referenced by a tagged `AgentAppearance` frame.
 * @param {unknown} frame
 * @returns {number[]}
 */
export function frame3dAgentIds(frame) {
  if (!frame || typeof frame !== "object" || !("AgentAppearance" in /** @type {object} */ (frame))) {
    return [];
  }
  const inner = /** @type {{ AgentAppearance?: { updates?: unknown[]; agents?: unknown[] } }} */ (
    frame
  ).AgentAppearance;
  const updates = inner?.updates ?? inner?.agents;
  if (!Array.isArray(updates) || updates.length === 0) return [];

  const ids = [];
  for (const update of updates) {
    if (!update || typeof update !== "object") continue;
    const agentId = normalizeAgentId(/** @type {{ agent_id?: unknown }} */ (update).agent_id);
    if (agentId != null) ids.push(agentId);
  }
  return ids;
}

/**
 * Track seen agent ids and return the most recently observed ids (newest first).
 * @param {Set<number>} seen
 * @param {number[]} recent
 * @param {number[]} ids
 * @param {number} [recentCap]
 */
export function noteAgentIds(seen, recent, ids, recentCap = 5) {
  if (!ids.length) return { count: seen.size, recentIds: [...recent] };

  let nextRecent = [...recent];
  for (const id of ids) {
    seen.add(id);
    nextRecent = [id, ...nextRecent.filter((value) => value !== id)];
  }
  if (nextRecent.length > recentCap) nextRecent.length = recentCap;
  return { count: seen.size, recentIds: nextRecent };
}
