import { useCallback, useEffect, useState } from "react";
import type { ModBrowserEntry } from "./lib/civisServer";
import { postControl } from "./control";
import { useDashboardStore } from "./store";

export type ModCatalogEntry = {
  source: string;
  id: string;
  name: string;
  version: string;
  mod_type: string;
  kind: string;
  installed: boolean;
};

function modsFromSnapshot(snapshot: Record<string, unknown> | null): ModBrowserEntry[] {
  if (!snapshot || !Array.isArray(snapshot.mods)) {
    return [];
  }
  return snapshot.mods as ModBrowserEntry[];
}

export function ModsPanel() {
  const { state } = useDashboardStore();
  const [catalog, setCatalog] = useState<ModCatalogEntry[]>([]);
  const [catalogError, setCatalogError] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);
  const [unloadError, setUnloadError] = useState<string | null>(null);
  const [unloading, setUnloading] = useState<string | null>(null);
  const [reloading, setReloading] = useState<string | null>(null);

  const mods =
    state.attachMode === "server"
      ? (state.serverMetrics?.mods ?? [])
      : modsFromSnapshot(state.snapshot as Record<string, unknown> | null);

  const refreshCatalog = useCallback(async () => {
    if (state.attachMode === "server") {
      setCatalog([]);
      return;
    }
    try {
      const response = await fetch("/control/mods/catalog");
      if (!response.ok) {
        throw new Error(`catalog ${response.status}`);
      }
      const data = (await response.json()) as ModCatalogEntry[];
      setCatalog(Array.isArray(data) ? data : []);
      setCatalogError(null);
    } catch (err) {
      setCatalogError(err instanceof Error ? err.message : "catalog fetch failed");
    }
  }, [state.attachMode]);

  useEffect(() => {
    void refreshCatalog();
  }, [refreshCatalog, mods.length]);

  const installMod = async (source: string) => {
    setInstalling(source);
    try {
      await postControl("/control/mods/install", { source });
      await refreshCatalog();
    } catch (err) {
      setCatalogError(err instanceof Error ? err.message : "install failed");
    } finally {
      setInstalling(null);
    }
  };

  const unloadMod = async (modId: string) => {
    setUnloading(modId);
    try {
      await postControl("/control/mods/unload", { mod_id: modId });
      setUnloadError(null);
      await refreshCatalog();
    } catch (err) {
      setUnloadError(err instanceof Error ? err.message : "unload failed");
    } finally {
      setUnloading(null);
    }
  };

  const reloadMod = async (modId: string) => {
    setReloading(modId);
    try {
      await postControl("/control/mods/reload", { mod_id: modId });
      setUnloadError(null);
      await refreshCatalog();
    } catch (err) {
      setUnloadError(err instanceof Error ? err.message : "reload failed");
    } finally {
      setReloading(null);
    }
  };

  return (
    <section className="inspector-section">
      <h3>Mods</h3>
      {state.attachMode !== "server" ? (
        <>
          <div className="mods-catalog-header">
            <span className="mods-meta">Installable</span>
            <button type="button" className="mods-refresh" onClick={() => void refreshCatalog()}>
              Refresh
            </button>
          </div>
          {catalogError ? <p className="inspector-empty">{catalogError}</p> : null}
          {catalog.length === 0 ? (
            <p className="inspector-empty">No installable mods in catalog</p>
          ) : (
            <ul className="mods-list">
              {catalog.map((entry) => (
                <li key={entry.source} className="mods-list-item">
                  <strong>{entry.name || entry.id}</strong>
                  <span className="mods-meta">
                    {entry.source} · v{entry.version} · {entry.mod_type} · {entry.kind}
                  </span>
                  {entry.installed ? (
                    <span className="mods-installed">Installed</span>
                  ) : (
                    <button
                      type="button"
                      className="mods-install"
                      disabled={installing === entry.source}
                      onClick={() => void installMod(entry.source)}
                    >
                      {installing === entry.source ? "Installing…" : "Install"}
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
        </>
      ) : null}

      <h4 className="mods-loaded-title">Loaded</h4>
      {unloadError ? <p className="inspector-empty">{unloadError}</p> : null}
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
                {(mod.float_instruction_count ?? 0) > 0
                  ? ` · float ops ${mod.float_instruction_count}`
                  : ""}
                {(mod.float_contamination_site_count ?? 0) > 0
                  ? ` · float sites ${mod.float_contamination_site_count}`
                  : ""}
              </span>
              {state.attachMode !== "server" ? (
                <span className="mods-loaded-actions">
                  <button
                    type="button"
                    className="mods-reload"
                    disabled={reloading === mod.id || unloading === mod.id}
                    onClick={() => void reloadMod(mod.id)}
                  >
                    {reloading === mod.id ? "Reloading…" : "Reload"}
                  </button>
                  <button
                    type="button"
                    className="mods-unload"
                    disabled={unloading === mod.id || reloading === mod.id}
                    onClick={() => void unloadMod(mod.id)}
                  >
                    {unloading === mod.id ? "Unloading…" : "Unload"}
                  </button>
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
