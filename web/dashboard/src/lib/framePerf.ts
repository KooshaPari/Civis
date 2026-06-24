/** Frame interval ring buffer + sparkline helpers (mirrors `web/src/framePerf.mjs`). */

export const FRAME_SAMPLE_CAP = 60;

/** Perf budget thresholds (dashboard alerts). */
export const PERF_FPS_WARN = 30;
export const PERF_FPS_CRITICAL = 15;
export const PERF_SPIKE_MS = 100;

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
