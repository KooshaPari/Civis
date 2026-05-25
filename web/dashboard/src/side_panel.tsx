import { ConnectionStatusCard } from "./connection_status";
import { AgentsPanel } from "./agents_panel";
import { EventFeed } from "./event_feed";
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
          <div className="side-panel-tabs">
            <button
              type="button"
              className={`side-panel-tab ${state.activeSideTab === "inspector" ? "active" : ""}`}
              onClick={() => dispatch({ type: "set_active_side_tab", tab: "inspector" })}
            >
              Inspector
            </button>
            <button
              type="button"
              className={`side-panel-tab ${state.activeSideTab === "events" ? "active" : ""}`}
              onClick={() => dispatch({ type: "set_active_side_tab", tab: "events" })}
            >
              Event Feed
            </button>
          </div>
          <p className="inspector-hint">
            {state.readOnly
              ? "Spectator mode (?spectator=1). Metrics and replay only."
              : "L2 web authoring: spawn, voxel, inspect. Full P-U1 palette in Godot."}
          </p>

          {state.activeSideTab === "inspector" ? (
            <>
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

              {state.selectedTool === "InspectAgent" && state.selectedMilitary ? (
                <div className="inspector-fields">
                  <InspectorRow label="unit_type" value={state.selectedMilitary.unit_type} />
                  <InspectorRow label="strength" value={state.selectedMilitary.strength.toFixed(2)} />
                  <InspectorRow label="faction" value={state.selectedMilitary.faction} />
                </div>
              ) : state.selectedTool === "InspectAgent" && state.selectedCivilian ? (
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
          ) : (
            <EventFeed />
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
