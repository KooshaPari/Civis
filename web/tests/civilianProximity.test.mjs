import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createBuildingProximityIndex,
  isNearBuilding,
} from "../dashboard/src/lib/civilianProximity.mjs";

test("civilian proximity respects the indoor radius boundary", () => {
  const index = createBuildingProximityIndex(100, [{ x: 0.5, y: 0.5 }]);

  assert.equal(isNearBuilding(index, 100, { x: 0.505, y: 0.5 }), true);
  assert.equal(isNearBuilding(index, 100, { x: 0.52, y: 0.5 }), false);
});

test("civilian proximity queries reuse a built index", () => {
  const buildings = [{ x: 0.5, y: 0.5 }];
  const index = createBuildingProximityIndex(100, buildings);

  assert.equal(isNearBuilding(index, 100, { x: 0.5, y: 0.5 }), true);
  assert.equal(isNearBuilding(index, 100, { x: 0.9, y: 0.9 }), false);
  assert.strictEqual(index, index);
  assert.notStrictEqual(index, createBuildingProximityIndex(100, [{ x: 0.6, y: 0.5 }]));
});

test("civilian proximity handles neighboring buckets and empty snapshots", () => {
  const nearby = createBuildingProximityIndex(100, [{ x: 0.515, y: 0.5 }]);
  const empty = createBuildingProximityIndex(100, []);

  assert.equal(isNearBuilding(nearby, 100, { x: 0.5, y: 0.5 }), true);
  assert.equal(isNearBuilding(empty, 100, { x: 0.5, y: 0.5 }), false);
});
