import { useDashboardStore } from "./store";

export function TopBar() {
  const { state } = useDashboardStore();
  const connectionLabel = {
    live: "Live",
    reconnecting: "Reconnecting",
    disconnected: "Disconnected",
  }[state.connection];
  const tick = state.snapshot?.tick ?? 0;
  const eraIndex = Math.floor(tick / 600) % 6;
  const eras = ["Mud-brick", "Timber", "Stone", "Brick", "Concrete", "Arcology"];
  const dayPhase = state.snapshot
    ? deriveClockLabel(state.snapshot.is_day, tick)
    : "06:00 — Dawn";

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
        <Metric label="Era" value={`${eraIndex} · ${eras[eraIndex]}`} />
        <Metric label="Clock" value={dayPhase} />
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

function deriveClockLabel(isDay: boolean, tick: number) {
  const labels = isDay
    ? ["06:00 — Dawn", "12:00 — Noon"]
    : ["18:00 — Dusk", "00:00 — Midnight"];
  return labels[tick % 2];
}
