import { useDashboardStore } from "./store";

export function TopBar() {
  const { state } = useDashboardStore();
  const connectionLabel = {
    live: "Live",
    reconnecting: "Reconnecting",
    disconnected: "Disconnected",
  }[state.connection];

  return (
    <header className="top-bar">
      <div className="brand-block">
        <p className="eyebrow">Civis 3D foundation</p>
        <h1>WorldBox-style playable sandbox</h1>
      </div>
      <div className="top-metrics">
        <Metric label="Tick" value={state.snapshot?.tick ?? 0} />
        <Metric label="Population" value={state.snapshot?.population ?? 0} />
        <Metric label="Voxel chunks" value={state.snapshot?.voxel_chunk_count ?? 0} />
        <Metric label="Voxel dirty" value={state.snapshot?.voxel_dirty_count ?? 0} />
        <Metric label="Day / Night" value={state.snapshot ? ((state.snapshot.tick % 24) < 12 ? "Day" : "Night") : "Day"} />
      </div>
      <span className={`connection-pill ${state.connection}`}>Connection: {connectionLabel}</span>
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

