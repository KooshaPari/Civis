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

export type RemoteModEntry = {
  id: string;
  path: string;
  fetched_at: number;
  url: string;
};

function isRemoteSource(source: string): boolean {
  return source.startsWith("mods/remote/");
}

function catalogKindLabel(entry: ModCatalogEntry): string {
  return isRemoteSource(entry.source) ? "remote" : entry.kind;
}

function formatFetchedAt(epochSec: number): string {
  if (epochSec <= 0) {
    return "unknown";
  }
  return new Date(epochSec * 1000).toLocaleString();
}

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
  const [remoteMods, setRemoteMods] = useState<RemoteModEntry[]>([]);
  const [remoteError, setRemoteError] = useState<string | null>(null);
  const [fetchUrl, setFetchUrl] = useState("");
  const [fetchModId, setFetchModId] = useState("");
  const [fetching, setFetching] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
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

  const refreshRemoteMods = useCallback(async () => {
    if (state.attachMode === "server") {
      setRemoteMods([]);
      return;
    }
    try {
      const response = await fetch("/control/mods/remote");
      if (!response.ok) {
        throw new Error(`remote ${response.status}`);
      }
      const data = (await response.json()) as RemoteModEntry[];
      setRemoteMods(Array.isArray(data) ? data : []);
      setRemoteError(null);
    } catch (err) {
      setRemoteError(err instanceof Error ? err.message : "remote list fetch failed");
    }
  }, [state.attachMode]);

  const refreshInstallable = useCallback(async () => {
    await Promise.all([refreshCatalog(), refreshRemoteMods()]);
  }, [refreshCatalog, refreshRemoteMods]);

  useEffect(() => {
    void refreshInstallable();
  }, [refreshInstallable, mods.length]);

  const isRemoteInstalled = (source: string) =>
    catalog.some((entry) => entry.source === source && entry.installed);

  const installMod = async (source: string) => {
    setInstalling(source);
    try {
      await postControl("/control/mods/install", { source });
      await refreshInstallable();
    } catch (err) {
      setCatalogError(err instanceof Error ? err.message : "install failed");
    } finally {
      setInstalling(null);
    }
  };

  const uploadModFile = async (file: File) => {
    setUploading(true);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      let binary = "";
      for (let i = 0; i < bytes.length; i += 1) {
        binary += String.fromCharCode(bytes[i] ?? 0);
      }
      const data_base64 = btoa(binary);
      await postControl("/control/mods/upload", {
        filename: file.name,
        data_base64,
      });
      await refreshInstallable();
    } catch (err) {
      setCatalogError(err instanceof Error ? err.message : "upload failed");
    } finally {
      setUploading(false);
    }
  };

  const unloadMod = async (modId: string) => {
    setUnloading(modId);
    try {
      await postControl("/control/mods/unload", { mod_id: modId });
      setUnloadError(null);
      await refreshInstallable();
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
      await refreshInstallable();
    } catch (err) {
      setUnloadError(err instanceof Error ? err.message : "reload failed");
    } finally {
      setReloading(null);
    }
  };

  const fetchRemoteMod = async () => {
    const url = fetchUrl.trim();
    if (!url) {
      setRemoteError("URL required");
      return;
    }
    setFetching(true);
    try {
      const body: { url: string; mod_id?: string } = { url };
      const modId = fetchModId.trim();
      if (modId) {
        body.mod_id = modId;
      }
      await postControl("/control/mods/fetch", body);
      setRemoteError(null);
      setFetchUrl("");
      setFetchModId("");
      await refreshInstallable();
    } catch (err) {
      setRemoteError(err instanceof Error ? err.message : "fetch failed");
    } finally {
      setFetching(false);
    }
  };

  return (
    <section className="inspector-section">
      <h3>Mods</h3>
      {state.attachMode !== "server" ? (
        <>
          <div className="mods-catalog-header">
            <span className="mods-meta">Installable</span>
            <div className="mods-catalog-actions">
              <label className="mods-upload-label">
                <input
                  type="file"
                  accept=".civmod"
                  className="mods-upload-input"
                  disabled={uploading}
                  onChange={(event) => {
                    const file = event.target.files?.[0];
                    event.target.value = "";
                    if (file) {
                      void uploadModFile(file);
                    }
                  }}
                />
                {uploading ? "Uploading…" : "Upload .civmod"}
              </label>
              <button
                type="button"
                className="mods-refresh"
                onClick={() => void refreshInstallable()}
              >
                Refresh
              </button>
            </div>
          </div>
          <p className="mods-meta">
            Signed mods need manifest <code>author_pubkey_hex</code> and <code>mod.wasm.sig</code> in
            the archive.
          </p>
          {catalogError ? <p className="inspector-empty">{catalogError}</p> : null}
          {catalog.length === 0 ? (
            <p className="inspector-empty">No installable mods in catalog</p>
          ) : (
            <ul className="mods-list">
              {catalog.map((entry) => (
                <li key={entry.source} className="mods-list-item">
                  <strong>{entry.name || entry.id}</strong>
                  <span className="mods-meta">
                    {entry.source} · v{entry.version} · {entry.mod_type} ·{" "}
                    {catalogKindLabel(entry)}
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

          <h4 className="mods-loaded-title">Remote mods</h4>
          <form
            className="mods-fetch-form"
            onSubmit={(event) => {
              event.preventDefault();
              void fetchRemoteMod();
            }}
          >
            <label className="mods-meta">
              URL
              <input
                type="url"
                className="mods-fetch-input"
                placeholder="https://example.com/mods/demo.civmod"
                value={fetchUrl}
                disabled={fetching}
                onChange={(event) => setFetchUrl(event.target.value)}
              />
            </label>
            <label className="mods-meta">
              Mod id (optional)
              <input
                type="text"
                className="mods-fetch-input"
                placeholder="demo-mod"
                value={fetchModId}
                disabled={fetching}
                onChange={(event) => setFetchModId(event.target.value)}
              />
            </label>
            <div className="mods-fetch-actions">
              <button type="submit" className="mods-fetch" disabled={fetching}>
                {fetching ? "Fetching…" : "Fetch"}
              </button>
              <button
                type="button"
                className="mods-refresh"
                disabled={fetching}
                onClick={() => void refreshInstallable()}
              >
                Refresh
              </button>
            </div>
          </form>
          {remoteError ? <p className="inspector-empty">{remoteError}</p> : null}
          {remoteMods.length === 0 ? (
            <p className="inspector-empty">No remote mods cached</p>
          ) : (
            <ul className="mods-list">
              {remoteMods.map((entry) => (
                <li key={entry.path} className="mods-list-item">
                  <strong>{entry.id}</strong>
                  <span className="mods-meta">
                    {entry.path} · fetched {formatFetchedAt(entry.fetched_at)}
                  </span>
                  <span className="mods-meta">{entry.url}</span>
                  {isRemoteInstalled(entry.path) ? (
                    <span className="mods-installed">Installed</span>
                  ) : (
                    <button
                      type="button"
                      className="mods-install"
                      disabled={installing === entry.path}
                      onClick={() => void installMod(entry.path)}
                    >
                      {installing === entry.path ? "Installing…" : "Install"}
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
