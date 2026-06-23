import assert from "node:assert/strict";
import { test } from "node:test";
import {
  FRAME_SAMPLE_CAP,
  PERF_FPS_CRITICAL,
  PERF_FPS_WARN,
  PERF_SPIKE_MS,
  averageFps,
  averageFrameMs,
  createAttachFrameClock,
  evaluatePerfBudget,
  frameMsToFps,
  maxFrameMs,
  mockDevFrameMs,
  pushFrameSample,
  sparklinePoints,
  sparklineScaleMax,
} from "../src/framePerf.mjs";

test("pushFrameSample keeps the newest cap entries", () => {
  let samples = [];
  for (let i = 1; i <= FRAME_SAMPLE_CAP + 5; i += 1) {
    samples = pushFrameSample(samples, i);
  }
  assert.equal(samples.length, FRAME_SAMPLE_CAP);
  assert.deepEqual(samples.slice(0, 3), [6, 7, 8]);
  assert.equal(samples.at(-1), FRAME_SAMPLE_CAP + 5);
});

test("frame timing helpers convert ms to fps", () => {
  assert.equal(frameMsToFps(16), 62.5);
  assert.equal(averageFrameMs([10, 20, 30]), 20);
  assert.equal(averageFps([10, 20, 30]), 50);
});

test("sparklinePoints map oldest-left to newest-right", () => {
  const points = sparklinePoints([20, 40], 100, 50, sparklineScaleMax([20, 40]));
  assert.equal(points.length, 2);
  assert.equal(points[0].x, 1);
  assert.equal(points[1].x, 99);
  assert.ok(points[0].y < points[1].y);
});

test("createAttachFrameClock records inter-arrival deltas", () => {
  const clock = createAttachFrameClock(4);
  assert.deepEqual(clock.record(0), []);
  assert.deepEqual(clock.record(16), [16]);
  clock.record(32);
  clock.record(48);
  clock.record(64);
  assert.deepEqual(clock.getSamples(), [16, 16, 16, 16]);
  clock.reset();
  assert.deepEqual(clock.getSamples(), []);
});

test("mockDevFrameMs stays in a plausible rAF band", () => {
  const sample = mockDevFrameMs(0);
  assert.ok(sample >= 8 && sample <= 60);
});

test("maxFrameMs returns the largest interval", () => {
  assert.equal(maxFrameMs([]), 0);
  assert.equal(maxFrameMs([16, 120, 20]), 120);
});

test("evaluatePerfBudget flags low average FPS", () => {
  const warnSamples = Array(10).fill(1000 / 25);
  const warn = evaluatePerfBudget(warnSamples);
  assert.equal(warn.worstLevel, "warn");
  assert.equal(warn.alerts.length, 1);
  assert.equal(warn.alerts[0].id, "fps-warn");

  const criticalSamples = Array(10).fill(1000 / 10);
  const critical = evaluatePerfBudget(criticalSamples);
  assert.equal(critical.worstLevel, "critical");
  assert.equal(critical.alerts.length, 1);
  assert.equal(critical.alerts[0].id, "fps-critical");
});

test("evaluatePerfBudget flags frame interval spikes", () => {
  const samples = [16, 16, 16, 120, 16];
  const budget = evaluatePerfBudget(samples);
  assert.equal(budget.worstLevel, "warn");
  assert.ok(budget.alerts.some((alert) => alert.id === "spike-warn"));
});

test("evaluatePerfBudget is clear when samples are healthy", () => {
  const samples = Array(10).fill(16);
  const budget = evaluatePerfBudget(samples);
  assert.equal(budget.worstLevel, null);
  assert.deepEqual(budget.alerts, []);
});

test("perf budget thresholds are exported", () => {
  assert.equal(PERF_FPS_WARN, 30);
  assert.equal(PERF_FPS_CRITICAL, 15);
  assert.equal(PERF_SPIKE_MS, 100);
});
