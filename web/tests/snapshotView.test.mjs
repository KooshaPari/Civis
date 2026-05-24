import assert from "node:assert/strict";
import { test } from "node:test";
import { sceneEntityCounts } from "../src/snapshotView.mjs";

test("sceneEntityCounts from watch snapshot fixture", () => {
  const counts = sceneEntityCounts({
    tick: 1,
    population: 4,
    civ_pins: [{}, {}, {}],
    buildings: [{}, {}],
    factions: [{}],
  });
  assert.equal(counts.civilians, 3);
  assert.equal(counts.buildings, 2);
  assert.equal(counts.factions, 1);
  assert.ok(counts.total > 0);
});
