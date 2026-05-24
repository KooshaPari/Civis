import { flipTheme, themeToggleLabel } from "./lib/theme";
import { useDashboardStore } from "./store";

export function TopBar() {
  const { state, dispatch } = useDashboardStore();
  const connectionLabel = {
    live: "Live",
    reconnecting: "Reconnecting",
    disconnected: "Disconnected",
  }[state.connection];
  const metrics = state.serverMetrics;
  const tick = state.snapshot?.tick ?? metrics?.tick ?? 0;
  const modeLabel = state.attachMode === "server" ? "civ-server" : "civ-watch";

  return (
    <header className="top-bar">
      <div className="brand-block">
        <p className="eyebrow">Civis · ADR-009 spectator</p>
        <h1>Live simulation observer</h1>
        <p className="brand-sub">
          Attach: <strong>{modeLabel}</strong>
          {state.frame3dTick != null ? ` · F3D0 tick ${state.frame3dTick}` : null}
        </p>
      </div>
      <div className="top-metrics">
        <Metric label="Tick" value={tick} />
        <Metric label="Population" value={state.snapshot?.population ?? metrics?.population ?? 0} />
        {state.attachMode === "server" ? (
          <>
            <Metric label="Buildings" value={metrics?.building_count ?? 0} />
            <Metric
              label="Energy"
              value={
                metrics?.energy_budget != null
                  ? metrics.energy_budget.toFixed(1)
                  : "—"
              }
            />
            <Metric label="Speed" value={`${metrics?.speed_multiplier ?? state.speed}×`} />
          </>
        ) : (
          <>
            <Metric label="Voxel chunks" value={state.snapshot?.voxel_chunk_count ?? 0} />
            <Metric label="Voxel dirty" value={state.snapshot?.voxel_dirty_count ?? 0} />
          </>
        )}
      </div>
      <div className="top-actions">
        <button
          type="button"
          className="dark-light"
          title={themeToggleLabel(state.theme)}
          aria-label={themeToggleLabel(state.theme)}
          aria-pressed={state.theme === "light"}
          onClick={() => {
            const theme = flipTheme(state.theme);
            dispatch({ type: "set_theme", theme });
          }}
        >
          <span className="dark-light-track" aria-hidden>
            <span className="dark-light-icon sun">☀</span>
            <span className="dark-light-icon moon">☾</span>
            <span className="dark-light-thumb" />
          </span>
          <span className="dark-light-label">
            {state.theme === "dark" ? "Light" : "Dark"}
          </span>
        </button>
        <span className={`connection-pill ${state.connection}`}>Connection: {connectionLabel}</span>
        <a className="status-page-link" href="./status.html" title="WebSocket attach diagnostics">
          Status page
        </a>
      </div>
    </header>
  );
}

function Metric({ label, value }: { label: string; value: number | string }) {
  return (
    <article className="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}
