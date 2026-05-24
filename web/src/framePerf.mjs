/** Frame interval ring buffer + sparkline helpers for dashboard perf UI. */

export const FRAME_SAMPLE_CAP = 60;

/** Perf budget thresholds (dashboard alerts). */
export const PERF_FPS_WARN = 30;
export const PERF_FPS_CRITICAL = 15;
export const PERF_SPIKE_MS = 100;

/**
 * Append a frame interval sample, keeping the newest `cap` entries.
 * @param {number[]} samples
 * @param {number} ms
 * @param {number} [cap]
 */
export function pushFrameSample(samples, ms, cap = FRAME_SAMPLE_CAP) {
  const next = [...samples, ms];
  if (next.length > cap) next.splice(0, next.length - cap);
  return next;
}

/** @param {number} ms */
export function frameMsToFps(ms) {
  if (!Number.isFinite(ms) || ms <= 0) return 0;
  return 1000 / ms;
}

/** @param {number[]} samples */
export function averageFrameMs(samples) {
  if (!samples.length) return 0;
  return samples.reduce((sum, ms) => sum + ms, 0) / samples.length;
}

/** @param {number[]} samples */
export function averageFps(samples) {
  return frameMsToFps(averageFrameMs(samples));
}

/** @param {number[]} samples */
export function maxFrameMs(samples) {
  if (!samples.length) return 0;
  return Math.max(...samples);
}

/**
 * Evaluate frame samples against perf budget thresholds.
 * @param {number[]} samples
 * @returns {{
 *   alerts: { id: string; level: "warn" | "critical"; message: string }[];
 *   worstLevel: "warn" | "critical" | null;
 *   fps: number;
 *   maxMs: number;
 * }}
 */
export function evaluatePerfBudget(samples) {
  if (!samples.length) {
    return { alerts: [], worstLevel: null, fps: 0, maxMs: 0 };
  }

  const fps = averageFps(samples);
  const maxMs = maxFrameMs(samples);
  /** @type {{ id: string; level: "warn" | "critical"; message: string }[]} */
  const alerts = [];

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

  const worstLevel = alerts.some((alert) => alert.level === "critical")
    ? "critical"
    : alerts.length
      ? "warn"
      : null;

  return { alerts, worstLevel, fps, maxMs };
}

/**
 * Y-axis scale for sparklines (frame time ms).
 * @param {number[]} samples
 * @param {number} [floorMs]
 */
export function sparklineScaleMax(samples, floorMs = 100) {
  if (!samples.length) return floorMs;
  return Math.max(floorMs, ...samples);
}

/**
 * Map samples to canvas coordinates (oldest left, newest right).
 * @param {number[]} samples
 * @param {number} width
 * @param {number} height
 * @param {number} maxMs
 * @returns {{ x: number; y: number }[]}
 */
export function sparklinePoints(samples, width, height, maxMs) {
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

/**
 * Track inter-arrival times for attach stream messages.
 * @param {number} [cap]
 */
export function createAttachFrameClock(cap = FRAME_SAMPLE_CAP) {
  /** @type {number | null} */
  let lastAt = null;
  /** @type {number[]} */
  let samples = [];

  return {
    /** @param {number} nowMs */
    record(nowMs) {
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

/**
 * Deterministic mock intervals for dev sparklines (no live attach).
 * @param {number} index
 */
export function mockDevFrameMs(index) {
  const base = 16 + Math.sin(index / 6) * 4;
  const spike = index % 23 === 0 ? 28 : 0;
  return Math.max(8, base + spike);
}
