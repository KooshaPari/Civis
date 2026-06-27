import assert from "node:assert/strict";
import { test } from "node:test";
import { zoomDistanceFromWheel, zoomDistanceRoundTrip } from "../dashboard/src/lib/zoomMath.mjs";

test("dashboard wheel zoom round-trips without clamping", () => {
  const minDistance = 10;
  const maxDistance = 2000;
  const distance = 180;
  const deltaY = 120;
  const roundTrip = zoomDistanceRoundTrip({
    distance,
    deltaY,
    minDistance,
    maxDistance,
  });

  assert.ok(Math.abs(roundTrip - distance) < 1e-9, `${roundTrip} != ${distance}`);
});

test("dashboard wheel zoom clamps to bounds", () => {
  const minDistance = 24;
  const maxDistance = 240;

  assert.equal(
    zoomDistanceFromWheel({
      distance: 30,
      deltaY: 10_000,
      minDistance,
      maxDistance,
    }),
    maxDistance,
  );

  assert.equal(
    zoomDistanceFromWheel({
      distance: 30,
      deltaY: -10_000,
      minDistance,
      maxDistance,
    }),
    minDistance,
  );
});
