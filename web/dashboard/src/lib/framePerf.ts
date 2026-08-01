/** Frame interval ring buffer + sparkline helpers (mirrors `web/src/framePerf.mjs`). */

export const FRAME_SAMPLE_CAP = 60;

/** Perf budget thresholds (dashboard alerts). */
export const PERF_FPS_WARN = 30;
export const PERF_FPS_CRITICAL = 15;
export const PERF_SPIKE_MS = 100;

/** Adaptive 3D renderer thresholds. Hysteresis avoids DPR changes from one-off spikes. */
export const ADAPTIVE_DPR_SLOW_FRAME_MS = 33;
export const ADAPTIVE_DPR_FAST_FRAME_MS = 18;
export const ADAPTIVE_DPR_SLOW_FRAME_COUNT = 30;
export const ADAPTIVE_DPR_FAST_FRAME_COUNT = 120;
export const ADAPTIVE_DPR_STEP = 0.5;

export type PerfBudgetLevel = "warn" | "critical";

export type PerfBudgetAlert = {
  id: string;
  level: PerfBudgetLevel;
  message: string;
};

export type PerfBudgetResult = {
  alerts: PerfBudgetAlert[];
  worstLevel: PerfBudgetLevel | null;
  fps: number;
  maxMs: number;
};

export type FrameSampleSummary = {
  fps: number;
  frameMs: number;
  count: number;
  latestMs: number;
  latestFps: number;
};

export function pushFrameSample(
  samples: number[],
  ms: number,
  cap = FRAME_SAMPLE_CAP,
): number[] {
  const next = [...samples, ms];
  if (next.length > cap) next.splice(0, next.length - cap);
  return next;
}

export function frameMsToFps(ms: number): number {
  if (!Number.isFinite(ms) || ms <= 0) return 0;
  return 1000 / ms;
}

export function averageFrameMs(samples: number[]): number {
  if (!samples.length) return 0;
  return samples.reduce((sum, value) => sum + value, 0) / samples.length;
}

export function averageFps(samples: number[]): number {
  return frameMsToFps(averageFrameMs(samples));
}

export function summarizeFrameSamples(samples: number[]): FrameSampleSummary {
  const count = samples.length;
  const frameMs = averageFrameMs(samples);
  const latestMs = count ? samples[count - 1] : 0;
  return {
    fps: frameMsToFps(frameMs),
    frameMs,
    count,
    latestMs,
    latestFps: frameMsToFps(latestMs),
  };
}

export function maxFrameMs(samples: number[]): number {
  if (!samples.length) return 0;
  return Math.max(...samples);
}

export function evaluatePerfBudget(samples: number[]): PerfBudgetResult {
  if (!samples.length) {
    return { alerts: [], worstLevel: null, fps: 0, maxMs: 0 };
  }

  const fps = averageFps(samples);
  const maxMs = maxFrameMs(samples);
  const alerts: PerfBudgetAlert[] = [];

  if (fps < PERF_FPS_CRITICAL) {
    alerts.push({
      id: "fps-critical",
      level: "critical",
      message: `Average FPS below ${PERF_FPS_CRITICAL} (${fps.toFixed(0)} fps)`,
    });
  } else if (fps < PERF_FPS_WARN) {
    alerts.push({
      id: "fps-warn",
      level: "warn",
      message: `Average FPS below ${PERF_FPS_WARN} (${fps.toFixed(0)} fps)`,
    });
  }

  if (maxMs > PERF_SPIKE_MS) {
    alerts.push({
      id: "spike-warn",
      level: "warn",
      message: `Frame interval spike ${maxMs.toFixed(0)} ms (> ${PERF_SPIKE_MS} ms)`,
    });
  }

  const worstLevel: PerfBudgetLevel | null = alerts.some((alert) => alert.level === "critical")
    ? "critical"
    : alerts.length
      ? "warn"
      : null;

  return { alerts, worstLevel, fps, maxMs };
}

export function sparklineScaleMax(samples: number[], floorMs = 100): number {
  if (!samples.length) return floorMs;
  return Math.max(floorMs, ...samples);
}

export function sparklinePoints(
  samples: number[],
  width: number,
  height: number,
  maxMs: number,
): { x: number; y: number }[] {
  if (!samples.length) return [];
  const pad = 1;
  const innerW = Math.max(1, width - pad * 2);
  const innerH = Math.max(1, height - pad * 2);
  const scale = maxMs > 0 ? maxMs : 1;

  return samples.map((ms, index) => {
    const x =
      samples.length === 1
        ? pad + innerW / 2
        : pad + (index / (samples.length - 1)) * innerW;
    const y = pad + Math.min(1, ms / scale) * innerH;
    return { x, y };
  });
}

export function createAttachFrameClock(cap = FRAME_SAMPLE_CAP) {
  let lastAt: number | null = null;
  let samples: number[] = [];

  return {
    record(nowMs: number) {
      if (lastAt != null) {
        const delta = Math.max(0, nowMs - lastAt);
        samples = pushFrameSample(samples, delta, cap);
      }
      lastAt = nowMs;
      return samples;
    },
    reset() {
      lastAt = null;
      samples = [];
    },
    getSamples() {
      return [...samples];
    },
  };
}

export function mockDevFrameMs(index: number): number {
  const base = 16 + Math.sin(index / 6) * 4;
  const spike = index % 23 === 0 ? 28 : 0;
  return Math.max(8, base + spike);
}

export type AdaptiveDprController = {
  record(frameMs: number): number;
  getDpr(): number;
  reset(): void;
};

/**
 * Adjusts a renderer DPR in response to sustained frame-time pressure.
 * The controller only returns a different value after a threshold is crossed.
 */
export function createAdaptiveDprController(
  initialDpr: number,
  minDpr = 1,
  slowFrameMs = ADAPTIVE_DPR_SLOW_FRAME_MS,
  fastFrameMs = ADAPTIVE_DPR_FAST_FRAME_MS,
): AdaptiveDprController {
  const lowerBound = Math.max(1, minDpr);
  const initial = Math.max(lowerBound, initialDpr);
  let dpr = initial;
  let slowFrames = 0;
  let fastFrames = 0;

  const reset = () => {
    dpr = initial;
    slowFrames = 0;
    fastFrames = 0;
  };

  return {
    record(frameMs: number) {
      if (!Number.isFinite(frameMs) || frameMs <= 0) return dpr;

      if (frameMs >= slowFrameMs) {
        slowFrames += 1;
        fastFrames = 0;
        if (slowFrames >= ADAPTIVE_DPR_SLOW_FRAME_COUNT) {
          dpr = Math.max(lowerBound, dpr - ADAPTIVE_DPR_STEP);
          slowFrames = 0;
        }
      } else if (frameMs <= fastFrameMs) {
        fastFrames += 1;
        slowFrames = 0;
        if (fastFrames >= ADAPTIVE_DPR_FAST_FRAME_COUNT) {
          dpr = Math.min(initial, dpr + ADAPTIVE_DPR_STEP);
          fastFrames = 0;
        }
      } else {
        slowFrames = 0;
        fastFrames = 0;
      }

      return dpr;
    },
    getDpr: () => dpr,
    reset,
  };
}
