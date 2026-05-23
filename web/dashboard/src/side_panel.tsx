import { useDashboardStore } from "./store";

export function SidePanel() {
  const { state, dispatch } = useDashboardStore();

  return (
    <aside className={`side-panel ${state.inspectorOpen ? "open" : "closed"}`}>
      <button className="panel-toggle" onClick={() => dispatch({ type: "set_inspector_open", open: !state.inspectorOpen })}>
        {state.inspectorOpen ? "Hide" : "Show"}
      </button>
      {state.inspectorOpen && (
        <>
          <h2>Inspector</h2>
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
            <p className="inspector-empty">Click anywhere to inspect</p>
          )}
        </>
      )}
    </aside>
  );
}

