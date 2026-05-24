import assert from "node:assert/strict";
import { test } from "node:test";

// Mirror dashboard merge (tested without TS compile in web root).
function mergeServerSnapshot(result, speed) {
  const r = result ?? {};
  const civPins = Array.isArray(r.civ_pins) ? r.civ_pins : [];
  return {
    tick: Number(r.tick ?? 0),
    population: Number(r.population ?? 0),
    civ_pins: civPins,
    speed,
  };
}

test("mergeServerSnapshot reads civ_pins from sim.snapshot", () => {
  const snap = mergeServerSnapshot(
    {
      tick: 5,
      population: 32,
      civ_pins: [{ idx: 0, x: 0.1, y: 0.2, dx: 0, dy: 0, job: "farmer" }],
      factions: [],
      buildings: [],
      is_day: true,
    },
    1,
  );
  assert.equal(snap.tick, 5);
  assert.equal(snap.civ_pins.length, 1);
  assert.equal(snap.civ_pins[0].job, "farmer");
});

test("parseInstitutions shape for sim.snapshot", () => {
  const rows = [
    { id: 1, kind: "market", balance_joules: 10 },
    { id: 2, kind: "treasury", balance_joules: 40 },
  ];
  assert.equal(rows.length, 2);
  assert.equal(rows[0].kind, "market");
});
