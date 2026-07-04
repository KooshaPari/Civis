import { summarizeFrameSamples } from "./lib/framePerf";
import { useDashboardStore } from "./store";

export function StatsPanel() {
  const { state, dispatch } = useDashboardStore();
  const {
    frame3dTick,
    loadedChunkCount,
    frameSamples,
    recentChunkIds,
    inspectedChunkId,
    terrain,
    snapshot,
  } = state;
  const summary = summarizeFrameSamples(frameSamples);
  const hasMinimap = terrain != null;

  return (
    <section className="inspector-section stats-panel" aria-labelledby="stats-heading">
      <h3 id="stats-heading">Stream stats</h3>
      <div className="stats-metrics">
        <Stat
          label="Tick"
          value={snapshot?.tick ?? (frame3dTick != null ? frame3dTick : "—")}
        />
        <Stat label="Population" value={snapshot?.population ?? "—"} />
        <Stat label="Chunks loaded" value={loadedChunkCount} />
        <Stat label="FPS" value={frameSamples.length ? summary.fps.toFixed(0) : "—"} />
      </div>
      {snapshot ? (
        <div className="stats-metrics">
          <Stat label="Births" value={snapshot.births_this_tick} />
          <Stat label="Deaths" value={snapshot.deaths_this_tick} />
          <Stat label="Voxel chunks" value={snapshot.voxel_chunk_count} />
        </div>
      ) : null}
      <div className="stats-detail">
        <span>Detail</span>
        <strong>{inspectedChunkId != null ? formatChunkId(inspectedChunkId) : "—"}</strong>
      </div>
      {!hasMinimap && recentChunkIds.length > 0 ? (
        <ul className="chunk-id-list" aria-label="Recent loaded chunks">
          {recentChunkIds.map((id) => (
            <li key={id}>
              <button
                type="button"
                className="chunk-id-row"
                onClick={() => dispatch({ type: "set_inspected_chunk", chunkId: id })}
              >
                <code>{formatChunkId(id)}</code>
              </button>
            </li>
          ))}
        </ul>
      ) : (
        !hasMinimap && (
          <p className="stats-empty">No VoxelDelta chunks yet — click a row when chunks arrive</p>
        )
      )}
    </section>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="perf-metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function formatChunkId(chunkId: number) {
  return `#${chunkId}`;
}
