/** Agent helpers — mirrors `web/src/agents.mjs` and `civ-bevy-ref::agent_color_from_id`. */

const U64_MAX = 18446744073709551615n;
const SPLITMIX64_ADD = 0x9e3779b97f4a7c15n;
const SPLITMIX64_MUL1 = 0xbf58476d1ce4e5b9n;
const SPLITMIX64_MUL2 = 0x94d049bb133111ebn;

function u64ToF32(value: bigint): number {
  const big = BigInt.asUintN(64, value);
  const lo = Number(big & 0xffffffffn);
  const hi = Number(big >> 32n);
  return Math.fround(hi * 4294967296 + lo);
}

function toAgentIdBigInt(agentId: number | bigint | string): bigint {
  if (typeof agentId === "bigint") return BigInt.asUintN(64, agentId);
  if (typeof agentId === "number" && Number.isFinite(agentId)) {
    return BigInt.asUintN(64, BigInt(Math.trunc(agentId)));
  }
  if (typeof agentId === "string" && agentId.trim() !== "") {
    return BigInt.asUintN(64, BigInt(agentId.trim()));
  }
  throw new TypeError("agent id must be a finite number or integer string");
}

export function splitmix64(agentId: number | bigint | string): bigint {
  let value = toAgentIdBigInt(agentId);
  value = BigInt.asUintN(64, value + SPLITMIX64_ADD);
  let z = value;
  z = BigInt.asUintN(64, (z ^ (z >> 30n)) * SPLITMIX64_MUL1);
  z = BigInt.asUintN(64, (z ^ (z >> 27n)) * SPLITMIX64_MUL2);
  return BigInt.asUintN(64, z ^ (z >> 31n));
}

export function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
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

export function agentColorFromId(agentId: number | bigint | string): [number, number, number] {
  const hash = splitmix64(agentId);
  const hue = (u64ToF32(hash) / u64ToF32(U64_MAX)) % 1;
  return hsvToRgb(hue, 0.62, 0.88);
}

export function agentColorCss(agentId: number | bigint | string): string {
  const [r, g, b] = agentColorFromId(agentId);
  const ri = Math.round(r * 255);
  const gi = Math.round(g * 255);
  const bi = Math.round(b * 255);
  return `rgb(${ri}, ${gi}, ${bi})`;
}

function normalizeAgentId(raw: unknown): number | null {
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string" && raw.trim() !== "") {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

export function frame3dAgentIds(frame: unknown): number[] {
  if (!frame || typeof frame !== "object" || !("AgentAppearance" in frame)) return [];
  const inner = (frame as { AgentAppearance?: { updates?: unknown[]; agents?: unknown[] } })
    .AgentAppearance;
  const updates = inner?.updates ?? inner?.agents;
  if (!Array.isArray(updates) || updates.length === 0) return [];

  const ids: number[] = [];
  for (const update of updates) {
    if (!update || typeof update !== "object") continue;
    const agentId = normalizeAgentId((update as { agent_id?: unknown }).agent_id);
    if (agentId != null) ids.push(agentId);
  }
  return ids;
}

export function noteAgentIds(
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
