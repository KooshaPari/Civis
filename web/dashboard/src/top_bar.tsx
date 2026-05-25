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
  const weatherLabel = formatWeather(state.snapshot?.weather);

  return (
    <header className="top-bar">
      <div className="brand-block">
        <p className="eyebrow">
          Civis · {state.readOnly ? "spectator" : "L2 authoring"}
        </p>
        <h1>{state.readOnly ? "Live simulation observer" : "Simulation sandbox"}</h1>
        <p className="brand-sub">
          Attach: <strong>{modeLabel}</strong>
          {state.frame3dTick != null ? ` · F3D0 tick ${state.frame3dTick}` : null}
          {weatherLabel ? ` · ${weatherLabel}` : null}
        </p>
      </div>
      <div className="top-metrics">
        <Metric label="Tick" value={tick} />
        <Metric label="Population" value={state.snapshot?.population ?? metrics?.population ?? 0} />
        <Metric label="⚔️ Soldiers" value={state.snapshot?.military_units?.length ?? 0} />
        <Metric
          label="🏠 Housing"
          value={`${state.snapshot?.housing_stats.occupied ?? 0}/${state.snapshot?.housing_stats.total_capacity ?? 0} (${Math.round((1 - (state.snapshot?.housing_stats.vacancy_rate ?? 0)) * 100)}%)`}
        />
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
      <div className="resource-strip" aria-label="Resource levels">
        <ResourceBar label="Food" value={state.snapshot?.economy.resources.food ?? 0} tone="food" />
        <ResourceBar label="Wood" value={state.snapshot?.economy.resources.wood ?? 0} tone="wood" />
        <ResourceBar label="Metal" value={state.snapshot?.economy.resources.metal ?? 0} tone="metal" />
        <ResourceBar label="Energy" value={state.snapshot?.economy.resources.energy ?? 0} tone="energy" />
        <ResourceBar
          label="Housing vacancy"
          value={(state.snapshot?.housing_stats.vacancy_rate ?? 0) * 100}
          tone="energy"
        />
      </div>
      <div className="top-actions">
        <span className="connection-pill">
          {state.lastSaveTick != null ? `💾 Last save: tick ${state.lastSaveTick}` : "💾 Last save: none"}
        </span>
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
        <button
          type="button"
          className="panel-toggle tech-tree-button"
          onClick={() => dispatch({ type: "set_tech_tree_open", open: true })}
        >
          🔬 Tech Tree
        </button>
        <span className={`connection-pill ${state.connection}`}>Connection: {connectionLabel}</span>
        <a className="status-page-link" href="./status.html" title="WebSocket attach diagnostics">
          Status page
        </a>
      </div>
    </header>
  );
}

function formatWeather(weather?: { season: string; temperature: number; precipitation: string } | null) {
  if (!weather) return "";
  const icon = weather.precipitation === "rain" ? "🌧️" : weather.precipitation === "snow" ? "❄️" : "☀️";
  const precipitation = weather.precipitation === "none" ? "" : ` ${capitalize(weather.precipitation)}`;
  return `${icon} ${weather.season} ${Math.round(weather.temperature)}°C${precipitation}`;
}

function capitalize(value: string) {
  return value.length ? value[0].toUpperCase() + value.slice(1) : value;
}

function Metric({ label, value }: { label: string; value: number | string }) {
  return (
    <article className="metric-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}

function ResourceBar({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone: "food" | "wood" | "metal" | "energy";
}) {
  const pct = Math.max(0, Math.min(100, value));
  return (
    <article className={`resource-bar ${tone}`}>
      <span>{label}</span>
      <div className="resource-track">
        <div className="resource-fill" style={{ width: `${pct}%` }} />
      </div>
    </article>
  );
}
