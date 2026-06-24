import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  decodeFrame3dBinary,
  encodeFrame3dBinaryFixture,
  frame3dSummary,
  frame3dVoxelChunkIds,
  isFrame3dBinary,
  parseFrame3dBinary,
  parseWsPayload,
  KIND_AGENT,
  KIND_VOXEL,
} from "../src/frame3d.mjs";

const __dir = dirname(fileURLToPath(import.meta.url));
const fixture = JSON.parse(
  readFileSync(join(__dir, "fixtures", "agent_frame.json"), "utf8"),
);
const voxelFixture = JSON.parse(
  readFileSync(join(__dir, "fixtures", "voxel_frame.json"), "utf8"),
);

test("decodeFrame3dBinary round-trips fixture", () => {
  const bytes = encodeFrame3dBinaryFixture(fixture, KIND_AGENT);
  assert.ok(isFrame3dBinary(bytes));
  const decoded = decodeFrame3dBinary(bytes);
  assert.deepEqual(decoded, fixture);
});

test("frame3dSummary reads agent tick", () => {
  const summary = frame3dSummary(fixture);
  assert.equal(summary?.kind, "agent");
  assert.equal(summary?.tick, 42);
});

test("frame3dVoxelChunkIds reads chunk ids from VoxelDelta", () => {
  assert.deepEqual(frame3dVoxelChunkIds(voxelFixture), [7, 42]);
  assert.deepEqual(frame3dVoxelChunkIds(fixture), []);
});

test("frame3dVoxelChunkIds works on binary VoxelDelta payloads", () => {
  const bytes = encodeFrame3dBinaryFixture(voxelFixture, KIND_VOXEL);
  const decoded = parseWsPayload(bytes);
  assert.deepEqual(frame3dVoxelChunkIds(decoded), [7, 42]);
});

test("decodeFrame3dBinary rejects bad magic", () => {
  const bytes = encodeFrame3dBinaryFixture(fixture);
  bytes[0] = 0x00;
  assert.throws(() => decodeFrame3dBinary(bytes), /bad magic/);
});

test("parseFrame3dBinary aliases decodeFrame3dBinary", () => {
  const bytes = encodeFrame3dBinaryFixture(fixture, KIND_AGENT);
  assert.deepEqual(parseFrame3dBinary(bytes), fixture);
});

test("parseWsPayload accepts binary and JSON text", () => {
  const bytes = encodeFrame3dBinaryFixture(fixture, KIND_AGENT);
  const json = JSON.stringify(fixture);
  assert.deepEqual(parseWsPayload(bytes), fixture);
  assert.deepEqual(parseWsPayload(json), fixture);
});

test("parseWsPayload prefers binary when magic present", () => {
  const bytes = encodeFrame3dBinaryFixture(fixture, KIND_AGENT);
  assert.deepEqual(parseWsPayload(bytes), fixture);
});
