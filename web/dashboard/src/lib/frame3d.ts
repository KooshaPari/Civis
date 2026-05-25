/** F3D0 decode (FR-CIV-WEB-006) — mirrors `web/src/frame3d.mjs`. */

const MAGIC = "F3D0";
const HEADER_LEN = 9;

export function isFrame3dBinary(data: ArrayBuffer | Uint8Array | Blob): boolean {
  if (data instanceof Blob) return false;
  const view = data instanceof Uint8Array ? data : new Uint8Array(data);
  return (
    view.length >= 4 &&
    view[0] === 0x46 &&
    view[1] === 0x33 &&
    view[2] === 0x44 &&
    view[3] === 0x30
  );
}

export function decodeFrame3dBinary(input: ArrayBuffer | Uint8Array): unknown {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.length < HEADER_LEN) throw new Error("frame3d: too short");
  const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
  if (magic !== MAGIC) throw new Error("frame3d: bad magic");
  const len = (bytes[5] << 24) | (bytes[6] << 16) | (bytes[7] << 8) | bytes[8];
  const expected = HEADER_LEN + len;
  if (bytes.length !== expected) throw new Error("frame3d: length mismatch");
  const text = new TextDecoder().decode(bytes.subarray(HEADER_LEN));
  return JSON.parse(text) as unknown;
}

export const parseFrame3dBinary = decodeFrame3dBinary;

export function parseFrame3dJson(text: string): unknown {
  return JSON.parse(text) as unknown;
}

/** F3D0 binary first, then UTF-8 JSON fallback (matches `civ-bevy-ref::parse_ws_payload`). */
export function parseWsPayload(payload: string | ArrayBuffer | Uint8Array): unknown {
  if (typeof payload === "string") {
    const bytes = new TextEncoder().encode(payload);
    if (isFrame3dBinary(bytes)) return parseFrame3dBinary(bytes);
    return parseFrame3dJson(payload);
  }
  const bytes = payload instanceof Uint8Array ? payload : new Uint8Array(payload);
  if (isFrame3dBinary(bytes)) return parseFrame3dBinary(bytes);
  return parseFrame3dJson(new TextDecoder().decode(bytes));
}

export function frame3dTick(frame: unknown): number | null {
  if (!frame || typeof frame !== "object") return null;
  const f = frame as Record<string, { tick?: number }>;
  const inner = f.VoxelDelta ?? f.BuildingDiff ?? f.AgentAppearance;
  return inner?.tick != null ? Number(inner.tick) : null;
}

function normalizeChunkId(raw: unknown): number | null {
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string" && raw.trim() !== "") {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

/** Chunk ids referenced by a tagged `VoxelDelta` frame (one per delta entry). */
export function frame3dVoxelChunkIds(frame: unknown): number[] {
  if (!frame || typeof frame !== "object" || !("VoxelDelta" in frame)) return [];
  const inner = (frame as { VoxelDelta?: { deltas?: Array<{ event?: { chunk_id?: unknown } }> } })
    .VoxelDelta;
  const deltas = inner?.deltas;
  if (!Array.isArray(deltas) || deltas.length === 0) return [];

  const ids: number[] = [];
  for (const delta of deltas) {
    const chunkId = normalizeChunkId(delta?.event?.chunk_id);
    if (chunkId != null) ids.push(chunkId);
  }
  return ids;
}
