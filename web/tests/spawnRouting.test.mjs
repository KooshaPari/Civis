import test from "node:test";
import assert from "node:assert/strict";
import {
  serverSpawnMethod,
  watchSpawnPath,
  spawnParams,
} from "../src/spawnRouting.mjs";

test("serverSpawnMethod routes civilian vs palette", () => {
  assert.equal(serverSpawnMethod("civilian"), "sim.spawn_civilian");
  assert.equal(serverSpawnMethod("vehicle"), "sim.spawn_entity");
  assert.equal(serverSpawnMethod("airport"), "sim.spawn_entity");
});

test("watchSpawnPath uses unified spawn_entity", () => {
  assert.equal(watchSpawnPath(), "/control/spawn_entity");
});

test("spawnParams includes kind for vehicle and airport", () => {
  assert.deepEqual(spawnParams("civilian", 0.5, 0.5, 1), { x: 0.5, y: 0.5, faction: 1 });
  assert.deepEqual(spawnParams("vehicle", 0.2, 0.8, 0), {
    kind: "vehicle",
    x: 0.2,
    y: 0.8,
    faction: 0,
  });
});
