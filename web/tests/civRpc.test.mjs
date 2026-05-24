import assert from "node:assert/strict";
import { test } from "node:test";
import {
  buildJsonRpcRequest,
  normalizeServerSnapshot,
  parseJsonRpcResponse,
} from "../src/civRpc.mjs";

test("buildJsonRpcRequest increments id", () => {
  const a = buildJsonRpcRequest("health");
  const b = buildJsonRpcRequest("sim.snapshot");
  assert.notEqual(a.id, b.id);
  assert.equal(a.method, "health");
});

test("normalizeServerSnapshot maps fields", () => {
  const snap = normalizeServerSnapshot({
    tick: 9,
    population: 100,
    building_count: 3,
    energy_budget: 1.5,
    market_prices: { food: 1000 },
    hash_chain_root: "abc",
    speed_multiplier: 2,
  });
  assert.equal(snap.tick, 9);
  assert.equal(snap.population, 100);
  assert.equal(snap.market_prices.food, 1000);
  assert.equal(snap.hash_chain_root, "abc");
});

test("parseJsonRpcResponse", () => {
  const result = parseJsonRpcResponse(
    JSON.stringify({ jsonrpc: "2.0", id: 7, result: { ok: true } }),
    7,
  );
  assert.deepEqual(result, { ok: true });
});
