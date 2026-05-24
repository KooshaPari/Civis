import { ConnectionStatusCard } from "./connection_status";
import { AgentsPanel } from "./agents_panel";
import { PerfPanel } from "./perf_panel";
import { StatsPanel } from "./stats_panel";
import { useDashboardStore } from "./store";

export function SidePanel() {
  const { state, dispatch } = useDashboardStore();
  const metrics = state.serverMetrics;

  return (
    <aside className={`side-panel ${state.inspectorOpen ? "open" : "closed"}`}>
      <button
        type="button"
        className="panel-toggle"
        onClick={() => dispatch({ type: "set_inspector_open", open: !state.inspectorOpen })}
      >
        {state.inspectorOpen ? "Hide" : "Show"}
      </button>
      {state.inspectorOpen && (
        <>
          <h2>Spectator</h2>
          <p className="inspector-hint">
            Read-only attach per ADR-009. Gameplay ships in Godot (P-U1).
          </p>

          <ConnectionStatusCard />
          <StatsPanel />
          <AgentsPanel />
          <PerfPanel />

          {state.attachMode === "server" && metrics ? (
            <section className="inspector-section">
              <h3>sim.snapshot</h3>
              <InspectorRow label="tick" value={metrics.tick} />
              <InspectorRow label="population" value={metrics.population} />
              <InspectorRow label="building_count" value={metrics.building_count} />
              <InspectorRow
                label="energy_budget"
                value={metrics.energy_budget?.toFixed(2) ?? "—"}
              />
              <InspectorRow
                label="hash_chain_root"
                value={metrics.hash_chain_root?.slice(0, 16) ?? "—"}
              />
              {Object.entries(metrics.market_prices).map(([good, price]) => (
                <InspectorRow key={good} label={`market.${good}`} value={price} />
              ))}
            </section>
          ) : null}

          {state.selectedTool === "InspectAgent" && state.selectedCivilian ? (
            <div className="inspector-fields">
              {Object.entries(state.selectedCivilian).map(([key, value]) => (
                <div key={key} className="inspector-row">
                  <span>{key}</span>
                  <strong>{String(value ?? "n/a")}</strong>
                </div>
              ))}
            </div>
          ) : (
            <p className="inspector-empty">Click terrain to inspect a cell</p>
          )}
        </>
      )}
    </aside>
  );
}

function InspectorRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="inspector-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
