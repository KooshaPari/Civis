import assert from "node:assert/strict";
import { test } from "node:test";
import {
  chunkToMinimapUv,
  decodeChunkId,
  encodeChunkId,
  findChunkAtGrid,
  minimapBoundsFromKeys,
  minimapUvToChunkGrid,
  noteChunkIds,
} from "../src/minimap.mjs";

test("decodeChunkId unpacks grid coords", () => {
  const raw = (5n << 40n) | (7n << 16n) | 9n;
  assert.deepEqual(decodeChunkId(Number(raw)), [5, 7, 9]);
});

test("chunkToMinimapUv maps chunk centre in bounds", () => {
  const bounds = [0, 0, 3, 3];
  const origin = chunkToMinimapUv(0, bounds);
  assert.ok(Math.abs(origin[0] - 0.125) < Number.EPSILON);
  assert.ok(Math.abs(origin[1] - 0.125) < Number.EPSILON);

  const raw = (3n << 40n) | (9n << 16n) | 3n;
  const corner = chunkToMinimapUv(Number(raw), bounds);
  assert.ok(Math.abs(corner[0] - 0.875) < Number.EPSILON);
  assert.ok(Math.abs(corner[1] - 0.875) < Number.EPSILON);
});

test("chunkToMinimapUv centres single chunk bounds", () => {
  const raw = (2n << 40n) | (4n << 16n) | 6n;
  const bounds = [2, 6, 2, 6];
  const uv = chunkToMinimapUv(Number(raw), bounds);
  assert.ok(Math.abs(uv[0] - 0.5) < Number.EPSILON);
  assert.ok(Math.abs(uv[1] - 0.5) < Number.EPSILON);
});

test("minimapUvToChunkGrid inverts chunkToMinimapUv", () => {
  const bounds = [0, 0, 3, 3];
  const originUv = chunkToMinimapUv(0, bounds);
  assert.deepEqual(minimapUvToChunkGrid(originUv, bounds), [0, 0]);

  const raw = (3n << 40n) | (9n << 16n) | 3n;
  const cornerUv = chunkToMinimapUv(Number(raw), bounds);
  assert.deepEqual(minimapUvToChunkGrid(cornerUv, bounds), [3, 3]);
});

test("minimapUvToChunkGrid clamps to bounds", () => {
  const bounds = [1, 2, 4, 5];
  assert.deepEqual(minimapUvToChunkGrid([0, 0], bounds), [1, 2]);
  assert.deepEqual(minimapUvToChunkGrid([1, 1], bounds), [4, 5]);
});

test("minimapUvToChunkGrid coerces non-finite uv to minimum bounds", () => {
  const bounds = [1, 2, 4, 5];
  assert.deepEqual(minimapUvToChunkGrid([Number.NaN, Number.POSITIVE_INFINITY], bounds), [1, 2]);
});

test("minimapBoundsFromKeys returns inclusive XZ bounds", () => {
  const a = Number((1n << 40n) | (0n << 16n) | 2n);
  const b = Number((3n << 40n) | (0n << 16n) | 5n);
  assert.deepEqual(minimapBoundsFromKeys([a, b]), [1, 2, 3, 5]);
  assert.equal(minimapBoundsFromKeys([]), null);
});

test("findChunkAtGrid resolves loaded chunk id", () => {
  const id = encodeChunkId(2, 0, 4);
  assert.equal(findChunkAtGrid([id, encodeChunkId(1, 0, 1)], 2, 4), id);
  assert.equal(findChunkAtGrid([id], 9, 9), null);
});

test("noteChunkIds tracks count and last five ids newest-first", () => {
  const seen = new Set();
  let recent = [];

  let stats = noteChunkIds(seen, recent, [7, 42]);
  assert.equal(stats.count, 2);
  assert.deepEqual(stats.recentIds, [42, 7]);
  recent = stats.recentIds;

  stats = noteChunkIds(seen, recent, [7]);
  assert.equal(stats.count, 2);
  assert.deepEqual(stats.recentIds, [7, 42]);
});

test("noteChunkIds clamps invalid recent caps to zero or higher", () => {
  const seen = new Set([1]);
  const stats = noteChunkIds(seen, [1], [2], -3);
  assert.equal(stats.count, 2);
  assert.deepEqual(stats.recentIds, []);
});

test("chunkToMinimapUv stays clamped inside the minimap", () => {
  const uv = chunkToMinimapUv(encodeChunkId(99, 0, 99), [0, 0, 3, 3]);
  assert.deepEqual(uv, [1, 1]);
});
