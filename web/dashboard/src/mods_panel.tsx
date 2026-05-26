import { useDashboardStore } from "./store";

/** Loaded mod row from civ-watch SSE or civ-server `sim.snapshot`. */
export type ModBrowserEntry = {
  id: string;
  name: string;
  version: string;
  mod_type: string;
  has_wasm: boolean;
  guest_memory_len: number;
};

function modsFromSnapshot(snapshot: Record<string, unknown> | null): ModBrowserEntry[] {
  if (!snapshot || !Array.isArray(snapshot.mods)) {
    return [];
  }
  return snapshot.mods as ModBrowserEntry[];
}

export function ModsPanel() {
  const { state } = useDashboardStore();
  const mods =
    state.attachMode === "server"
      ? modsFromSnapshot(state.serverMetrics as Record<string, unknown> | null)
      : modsFromSnapshot(state.snapshot as Record<string, unknown> | null);

  return (
    <section className="inspector-section">
      <h3>Mods</h3>
      {mods.length === 0 ? (
        <p className="inspector-empty">No mods loaded</p>
      ) : (
        <ul className="mods-list">
          {mods.map((mod) => (
            <li key={mod.id} className="mods-list-item">
              <strong>{mod.name}</strong>
              <span className="mods-meta">
                {mod.id} · v{mod.version} · {mod.mod_type}
                {mod.has_wasm ? " · wasm" : ""}
                {mod.guest_memory_len > 0 ? ` · mem ${mod.guest_memory_len}B` : ""}
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
