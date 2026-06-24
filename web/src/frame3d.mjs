/** @see crates/protocol-3d — `F3D0` magic + kind (1) + len BE (4) + JSON body */

export const FRAME3D_BINARY_MAGIC = "F3D0";
export const FRAME3D_HEADER_LEN = 9;

const KIND_VOXEL = 0;
const KIND_BUILDING = 1;
const KIND_AGENT = 2;

/**
 * @param {ArrayBuffer | Uint8Array} input
 * @returns {unknown}
 */
export function decodeFrame3dBinary(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.length < FRAME3D_HEADER_LEN) {
    throw new Error("frame3d: too short");
  }
  const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
  if (magic !== FRAME3D_BINARY_MAGIC) {
    throw new Error("frame3d: bad magic");
  }
  const len =
    (bytes[5] << 24) | (bytes[6] << 16) | (bytes[7] << 8) | bytes[8];
  const expected = FRAME3D_HEADER_LEN + len;
  if (bytes.length !== expected) {
    throw new Error("frame3d: length mismatch");
  }
  const jsonBytes = bytes.subarray(FRAME3D_HEADER_LEN);
  const text = new TextDecoder().decode(jsonBytes);
  return JSON.parse(text);
}

/** Alias for {@link decodeFrame3dBinary} (matches `civ-bevy-ref::parse_frame3d_binary`). */
export const parseFrame3dBinary = decodeFrame3dBinary;

/**
 * @param {string} text
 * @returns {unknown}
 */
export function parseFrame3dJson(text) {
  return JSON.parse(text);
}

/**
 * Decode a WebSocket payload: F3D0 binary first, then UTF-8 JSON fallback.
 * @param {string | ArrayBuffer | Uint8Array} payload
 * @returns {unknown}
 */
export function parseWsPayload(payload) {
  if (typeof payload === "string") {
    if (isFrame3dBinary(new TextEncoder().encode(payload))) {
      return parseFrame3dBinary(new TextEncoder().encode(payload));
    }
    return parseFrame3dJson(payload);
  }
  const bytes = payload instanceof Uint8Array ? payload : new Uint8Array(payload);
  if (isFrame3dBinary(bytes)) {
    return parseFrame3dBinary(bytes);
  }
  const text = new TextDecoder().decode(bytes);
  return parseFrame3dJson(text);
}

/**
 * @param {ArrayBuffer | Uint8Array} input
 * @returns {boolean}
 */
export function isFrame3dBinary(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.length < 4) return false;
  return (
    bytes[0] === 0x46 &&
    bytes[1] === 0x33 &&
    bytes[2] === 0x44 &&
    bytes[3] === 0x30
  );
}

/**
 * @param {unknown} frame
 * @returns {{ kind: "voxel" | "building" | "agent"; tick: number } | null}
 */
/**
 * @param {unknown} raw
 * @returns {number | null}
 */
function normalizeChunkId(raw) {
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string" && raw.trim() !== "") {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

/**
 * Extract chunk ids from a tagged `VoxelDelta` frame (unique per delta entry).
 * @param {unknown} frame
 * @returns {number[]}
 */
export function frame3dVoxelChunkIds(frame) {
  if (!frame || typeof frame !== "object" || !("VoxelDelta" in /** @type {object} */ (frame))) {
    return [];
  }
  const inner = /** @type {{ VoxelDelta?: { deltas?: Array<{ event?: { chunk_id?: unknown } }> } }} */ (
    frame
  ).VoxelDelta;
  const deltas = inner?.deltas;
  if (!Array.isArray(deltas) || deltas.length === 0) return [];

  const ids = [];
  for (const delta of deltas) {
    const chunkId = normalizeChunkId(delta?.event?.chunk_id);
    if (chunkId != null) ids.push(chunkId);
  }
  return ids;
}

export function frame3dSummary(frame) {
  if (!frame || typeof frame !== "object") return null;
  if ("VoxelDelta" in /** @type {object} */ (frame)) {
    const inner = /** @type {{ VoxelDelta: { tick?: number } }} */ (frame).VoxelDelta;
    return { kind: "voxel", tick: Number(inner?.tick ?? 0) };
  }
  if ("BuildingDiff" in /** @type {object} */ (frame)) {
    const inner = /** @type {{ BuildingDiff: { tick?: number } }} */ (frame).BuildingDiff;
    return { kind: "building", tick: Number(inner?.tick ?? 0) };
  }
  if ("AgentAppearance" in /** @type {object} */ (frame)) {
    const inner = /** @type {{ AgentAppearance: { tick?: number } }} */ (frame).AgentAppearance;
    return { kind: "agent", tick: Number(inner?.tick ?? 0) };
  }
  return null;
}

/**
 * Encode a minimal fixture frame for tests (mirrors Rust layout).
 * @param {unknown} payload — tagged Frame3d JSON object
 * @param {number} [kindTag]
 */
export function encodeFrame3dBinaryFixture(payload, kindTag = KIND_AGENT) {
  const json = new TextEncoder().encode(JSON.stringify(payload));
  const out = new Uint8Array(FRAME3D_HEADER_LEN + json.length);
  out[0] = 0x46;
  out[1] = 0x33;
  out[2] = 0x44;
  out[3] = 0x30;
  out[4] = kindTag;
  const len = json.length;
  out[5] = (len >>> 24) & 0xff;
  out[6] = (len >>> 16) & 0xff;
  out[7] = (len >>> 8) & 0xff;
  out[8] = len & 0xff;
  out.set(json, FRAME3D_HEADER_LEN);
  return out;
}

export { KIND_VOXEL, KIND_BUILDING, KIND_AGENT };
