import { evaluatePerfBudget } from "./lib/framePerf";
import { formatPerfSummary, Sparkline } from "./sparkline";
import { useDashboardStore } from "./store";

export function PerfPanel() {
  const { state } = useDashboardStore();
  const { frameSamples, frameSampleSource } = state;
  const summary = formatPerfSummary(frameSamples);
  const budget = evaluatePerfBudget(frameSamples);
  const latestMs = frameSamples.length ? frameSamples[frameSamples.length - 1] : 0;
  const latestFps = latestMs > 0 ? 1000 / latestMs : 0;

  const sourceLabel =
    frameSampleSource === "mock" ? "mock (dev)" : frameSampleSource === "attach" ? "attach" : "idle";

  const panelClass = budget.worstLevel ? ` perf-panel--${budget.worstLevel}` : "";

  return (
    <section
      className={`inspector-section perf-panel${panelClass}`}
      aria-labelledby="perf-heading"
    >
      <div className="perf-head">
        <h3 id="perf-heading">Frame timing</h3>
        <span className="perf-cap">
          {sourceLabel} · {frameSamples.length}/60
        </span>
      </div>
      {budget.alerts.length > 0 ? (
        <ul className="perf-budget-alerts" role="status" aria-live="polite">
          {budget.alerts.map((alert) => (
            <li
              key={alert.id}
              className={`perf-budget-alert perf-budget-alert--${alert.level}`}
            >
              {alert.message}
            </li>
          ))}
        </ul>
      ) : null}
      <div className="perf-metrics">
        <div className="perf-metric">
          <span>Avg</span>
          <strong>{summary.fps.toFixed(0)} fps</strong>
          <small>{summary.frameMs.toFixed(1)} ms</small>
        </div>
        <div className="perf-metric">
          <span>Latest</span>
          <strong>{latestFps.toFixed(0)} fps</strong>
          <small>{latestMs.toFixed(1)} ms</small>
        </div>
      </div>
      <Sparkline samples={frameSamples} />
      <p className="perf-hint">
        Inter-arrival times from the attach stream (or rAF mock when offline in dev).
      </p>
    </section>
  );
}
