import { useEffect, useRef, useState } from "react";
import { postControl } from "./control";
import {
  chunkToMinimapUv,
  encodeChunkId,
  findChunkAtGrid,
  minimapBoundsFromKeys,
  minimapUvToChunkGrid,
} from "./lib/minimap";
import {
  exportReplayBlob,
  importReplayBytes,
  jsonRpcCall,
  normalizeServerSnapshot,
} from "./lib/civisServer";
import { getActiveServerSocket } from "./lib/civisSocket";
import { mergeServerSnapshot } from "./lib/mergeSnapshot";
import { useDashboardStore, type SaveEntry, type TimeSpeed } from "./store";

export function BottomBar() {
  const { state, dispatch } = useDashboardStore();
  const miniMapRef = useRef<HTMLCanvasElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const autosaveBucketRef = useRef(-1);
  const [saveName, setSaveName] = useState("");
  const [loadEntries, setLoadEntries] = useState<SaveEntry[]>([]);
  const [loadOpen, setLoadOpen] = useState(false);

  const runWatchControl = async (path: string, body: object = {}) => {
    try {
      await postControl(path, body);
    } catch {
      dispatch({ type: "set_toast", message: `Failed: ${path}` });
    }
  };

  const saveGame = async (filename: string) => {
    if (state.attachMode !== "watch") {
      dispatch({ type: "set_toast", message: "Save/load is available in civ-watch mode" });
      return;
    }
    try {
      const response = await fetch("/control/save", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ filename }),
      });
      const data = (await response.json()) as { ok?: boolean; tick?: number; message?: string };
      if (!response.ok || !data.ok) {
        throw new Error(data.message ?? `save failed ${response.status}`);
      }
      const tick = Number(data.tick ?? 0);
      autosaveBucketRef.current = Math.floor(tick / 1000);
      dispatch({ type: "set_last_save_tick", tick });
      dispatch({ type: "set_toast", message: `Saved ${filename} @ tick ${tick}` });
    } catch (err) {
      dispatch({ type: "set_toast", message: err instanceof Error ? err.message : "Save failed" });
    }
  };

  const promptSave = () => {
    const tick = state.snapshot?.tick ?? state.serverMetrics?.tick ?? 0;
    const name = window.prompt("Save name:", saveName || `autosave-${tick}`);
    if (!name) return;
    setSaveName(name);
    void saveGame(name);
  };

  const openLoadDialog = async () => {
    if (state.attachMode !== "watch") {
      dispatch({ type: "set_toast", message: "Save/load is available in civ-watch mode" });
      return;
    }
    try {
      const response = await fetch("/control/saves");
      const entries = (await response.json()) as SaveEntry[];
      if (!response.ok) throw new Error(`load list failed ${response.status}`);
      setLoadEntries(entries);
      setLoadOpen(true);
    } catch (err) {
      dispatch({ type: "set_toast", message: err instanceof Error ? err.message : "Load failed" });
    }
  };

  const loadGame = async (name: string) => {
    try {
      const response = await fetch("/control/load", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ filename: name }),
      });
      const data = (await response.json()) as { ok?: boolean; tick?: number; message?: string };
      if (!response.ok || !data.ok) {
        throw new Error(data.message ?? `load failed ${response.status}`);
      }
      const tick = Number(data.tick ?? 0);
      autosaveBucketRef.current = Math.floor(tick / 1000);
      dispatch({ type: "set_last_save_tick", tick });
      dispatch({ type: "set_toast", message: `Loaded ${name} @ tick ${tick}` });
      setLoadOpen(false);
      setLoadEntries([]);
      const snap = await fetch("/snapshot").then((r) => r.json());
      dispatch({ type: "set_snapshot", snapshot: snap });
    } catch (err) {
      dispatch({ type: "set_toast", message: err instanceof Error ? err.message : "Load failed" });
    }
  };

  const setServerSpeed = async (multiplier: TimeSpeed) => {
    const ws = getActiveServerSocket();
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      dispatch({ type: "set_toast", message: "Not connected to civ-server" });
      return;
    }
    try {
      await jsonRpcCall(ws, "sim.set_speed", { multiplier });
      dispatch({ type: "set_speed", speed: multiplier });
      const snap = await jsonRpcCall<unknown>(ws, "sim.snapshot");
      const metrics = normalizeServerSnapshot(snap);
      dispatch({ type: "set_server_metrics", metrics });
      dispatch({
        type: "set_snapshot",
        snapshot: mergeServerSnapshot(snap, multiplier),
      });
    } catch {
      dispatch({ type: "set_toast", message: "sim.set_speed failed" });
    }
  };

  const runServerTick = async () => {
    const ws = getActiveServerSocket();
    if (!ws) return;
    try {
      await jsonRpcCall(ws, "sim.command", { action: "tick" });
      const snap = await jsonRpcCall<unknown>(ws, "sim.snapshot");
      const metrics = normalizeServerSnapshot(snap);
      dispatch({ type: "set_server_metrics", metrics });
      dispatch({
        type: "set_snapshot",
        snapshot: mergeServerSnapshot(snap, (metrics.speed_multiplier as TimeSpeed) ?? 1),
      });
    } catch {
      dispatch({ type: "set_toast", message: "sim.command tick failed" });
    }
  };

  const downloadReplay = async () => {
    try {
      const blob = await exportReplayBlob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "session.civreplay";
      a.click();
      URL.revokeObjectURL(url);
      dispatch({ type: "set_toast", message: "Replay exported" });
    } catch {
      dispatch({ type: "set_toast", message: "Replay export failed" });
    }
  };

  const onReplayFile = async (file: File) => {
    try {
      const buf = await file.arrayBuffer();
      const { tick } = await importReplayBytes(buf);
      dispatch({ type: "set_toast", message: `Replay imported @ tick ${tick}` });
      const ws = getActiveServerSocket();
      if (ws?.readyState === WebSocket.OPEN) {
        const snap = await jsonRpcCall<unknown>(ws, "sim.snapshot");
        const metrics = normalizeServerSnapshot(snap);
        dispatch({ type: "set_server_metrics", metrics });
        dispatch({
          type: "set_snapshot",
          snapshot: mergeServerSnapshot(snap, (metrics.speed_multiplier as TimeSpeed) ?? 1),
        });
      }
    } catch {
      dispatch({ type: "set_toast", message: "Replay import failed" });
    }
  };

  useEffect(() => {
    const canvas = miniMapRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const terrain = state.terrain;
    const snapshot = state.snapshot;
    const width = canvas.width;
    const height = canvas.height;
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = "#07101d";
    ctx.fillRect(0, 0, width, height);
    if (!terrain) return;

    const cellW = width / terrain.size;
    const cellH = height / terrain.size;
    for (let y = 0; y < terrain.size; y += 1) {
      for (let x = 0; x < terrain.size; x += 1) {
        const idx = y * terrain.size + x;
        ctx.fillStyle = biomeColor(terrain.biomes[idx]);
        ctx.fillRect(x * cellW, y * cellH, Math.ceil(cellW), Math.ceil(cellH));
      }
    }

    snapshot?.factions.forEach((faction) => {
      ctx.beginPath();
      ctx.fillStyle = `rgba(${faction.color[0]}, ${faction.color[1]}, ${faction.color[2]}, 0.95)`;
      ctx.arc(
        faction.capital[0] * width,
        faction.capital[1] * height,
        Math.max(2, faction.radius * 0.25),
        0,
        Math.PI * 2,
      );
      ctx.fill();
      ctx.font = "10px Segoe UI, sans-serif";
      ctx.textBaseline = "middle";
      ctx.textAlign = "left";
      ctx.fillStyle = "#ffffff";
      ctx.strokeStyle = "rgba(0, 0, 0, 0.7)";
      ctx.lineWidth = 3;
      const labelX = faction.capital[0] * width + 6;
      const labelY = faction.capital[1] * height - 8;
      const name = faction.name ?? `Faction ${faction.id + 1}`;
      ctx.strokeText(name, labelX, labelY);
      ctx.fillText(name, labelX, labelY);
    });

    drawCameraFrustum(ctx, width, height, state.cameraFocus);

    const bounds = minimapBoundsFromKeys(state.loadedChunkIds);
    if (bounds) {
      for (const chunkId of state.loadedChunkIds) {
        const [u, v] = chunkToMinimapUv(chunkId, bounds);
        ctx.fillStyle = "#b8b09e";
        ctx.fillRect(u * width - 2, v * height - 2, 4, 4);
      }
    }
  }, [state.snapshot, state.terrain, state.loadedChunkIds, state.cameraFocus]);

  useEffect(() => {
    const tick = state.snapshot?.tick ?? 0;
    const bucket = Math.floor(tick / 1000);
    if (state.attachMode !== "watch" || bucket <= 0 || bucket <= autosaveBucketRef.current) return;
    autosaveBucketRef.current = bucket;
    void saveGame("autosave");
  }, [state.attachMode, state.snapshot?.tick]);

  const inspectMinimapCell = (event: React.MouseEvent<HTMLCanvasElement>) => {
    if (!state.terrain) return;
    const canvas = miniMapRef.current;
    if (!canvas) return;

    const bounds = minimapBoundsFromKeys(state.loadedChunkIds);
    if (!bounds) return;

    const rect = canvas.getBoundingClientRect();
    const u = (event.clientX - rect.left) / rect.width;
    const v = (event.clientY - rect.top) / rect.height;
    const [cx, cz] = minimapUvToChunkGrid([u, v], bounds);
    const chunkId =
      findChunkAtGrid(state.loadedChunkIds, cx, cz) ?? encodeChunkId(cx, 0, cz);
    dispatch({ type: "set_inspected_chunk", chunkId });
    const size = state.terrain.size;
    dispatch({
      type: "set_camera_focus",
      focus: [Math.max(0, Math.min(size - 1, u * size)), Math.max(0, Math.min(size - 1, v * size))],
    });
  };

  const speedButtons = (
    <div className="time-row" role="group" aria-label="Simulation speed">
      {[0, 1, 2, 4, 8].map((speed) => (
        <button
          key={speed}
          type="button"
          className={`time-button ${state.speed === speed ? "active" : ""}`}
          title={speed === 0 ? "Pause" : `${speed}x speed`}
          onClick={() => {
            const s = speed as TimeSpeed;
            if (state.attachMode === "server") {
              void setServerSpeed(s);
            } else {
              dispatch({ type: "set_speed", speed: s });
              void runWatchControl("/control/speed", { speed: s });
            }
          }}
        >
          {speed === 0 ? "⏸ Pause" : speed === 1 ? "▶ 1×" : `⏩ ${speed}×`}
        </button>
      ))}
    </div>
  );

  return (
    <footer className="bottom-bar">
      <div className="control-group">
        <span className="control-label">View</span>
        <div className="tool-row">
          <ToolButton
            active={state.selectedTool === "InspectAgent"}
            title="Inspect terrain cell"
            emoji="🔍"
            onClick={() => dispatch({ type: "set_tool", tool: "InspectAgent" })}
          />
          <ToolButton
            active={state.selectedTool === "Camera"}
            title="Orbit camera"
            emoji="🎥"
            onClick={() => dispatch({ type: "set_tool", tool: "Camera" })}
          />
        </div>
        {state.selectedTool === "Camera" ? (
          <div className="tool-row" role="group" aria-label="Camera presets">
            <ToolButton
              active={state.cameraPreset === "wide"}
              title="Wide overview (FR-CIV-UX-005)"
              emoji="🌄"
              onClick={() => dispatch({ type: "set_camera_preset", preset: "wide" })}
            />
            <ToolButton
              active={state.cameraPreset === "close"}
              title="Close orbit"
              emoji="🔎"
              onClick={() => dispatch({ type: "set_camera_preset", preset: "close" })}
            />
            <ToolButton
              active={state.cameraPreset === "orbit"}
              title="Default orbit"
              emoji="🛰"
              onClick={() => dispatch({ type: "set_camera_preset", preset: "orbit" })}
            />
          </div>
        ) : null}
      </div>

      {!state.readOnly ? (
        <div className="control-group">
          <span className="control-label">
            Authoring ({state.attachMode === "server" ? "JSON-RPC" : "HTTP"})
          </span>
          <div className="tool-row">
            <ToolButton
              active={state.selectedTool === "PlaceVoxel"}
              title="Place voxel on terrain click"
              emoji="🧱"
              onClick={() => dispatch({ type: "set_tool", tool: "PlaceVoxel" })}
            />
            <ToolButton
              active={state.selectedTool === "SpawnCivilian"}
              title="Spawn: click civilian, drag-release vehicle/airport"
              emoji="🧍"
              onClick={() => dispatch({ type: "set_tool", tool: "SpawnCivilian" })}
            />
            <ToolButton
              active={state.selectedTool === "DamageBomb"}
              title="Tactical voxel damage (sim.damage / control/damage)"
              emoji="💥"
              onClick={() => dispatch({ type: "set_tool", tool: "DamageBomb" })}
            />
          </div>
          <div className="picker-row">
            <label>
              Material
              <input
                type="number"
                min={0}
                max={7}
                value={state.selectedMaterial}
                onChange={(e) =>
                  dispatch({ type: "set_material", material: Number(e.target.value) })
                }
              />
            </label>
            <label>
              Faction
              <input
                type="number"
                min={0}
                max={3}
                value={state.selectedFaction}
                onChange={(e) =>
                  dispatch({ type: "set_selected_faction", faction: Number(e.target.value) })
                }
              />
            </label>
            <label>
              Radius
              <input
                type="number"
                min={1}
                max={32}
                value={state.damageRadius}
                onChange={(e) =>
                  dispatch({ type: "set_damage_radius", radius: Number(e.target.value) })
                }
              />
            </label>
            <label>
              Spawn kind
              <select
                value={state.spawnKind}
                onChange={(e) =>
                  dispatch({
                    type: "set_spawn_kind",
                    kind: e.target.value as "civilian" | "vehicle" | "airport" | "port",
                  })
                }
              >
                <option value="civilian">Civilian</option>
                <option value="vehicle">Vehicle (drag on terrain)</option>
                <option value="airport">Airport (drag / convoy)</option>
                <option value="port">Port (drag / convoy)</option>
              </select>
            </label>
          </div>
        </div>
      ) : null}

      <div className="control-group">
        <span className="control-label">
          Operator {state.attachMode === "server" ? "(JSON-RPC)" : "(civ-watch HTTP)"}
        </span>
        {speedButtons}
        {state.attachMode === "server" ? (
          <div className="tool-row">
            <ToolButton title="Advance one tick" emoji="⏭" onClick={() => void runServerTick()} />
            <ToolButton title="Export .civreplay" emoji="💾" onClick={() => void downloadReplay()} />
            <ToolButton
              title="Import .civreplay"
              emoji="📂"
              onClick={() => fileInputRef.current?.click()}
            />
          </div>
        ) : (
          <div className="tool-row">
            <ToolButton title="Save game" emoji="💾" onClick={promptSave} />
            <ToolButton title="Load game" emoji="📂" onClick={() => void openLoadDialog()} />
          </div>
        )}
        <input
          ref={fileInputRef}
          type="file"
          accept=".civreplay,application/octet-stream"
          hidden
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) void onReplayFile(file);
            e.target.value = "";
          }}
        />
      </div>

      {loadOpen ? (
        <div className="modal-backdrop" onClick={() => setLoadOpen(false)}>
          <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
            <div className="modal-head">
              <strong>Load save</strong>
              <button type="button" onClick={() => setLoadOpen(false)}>
                Close
              </button>
            </div>
            <div className="modal-body">
              {loadEntries.length === 0 ? (
                <p>No saves found.</p>
              ) : (
                <div className="load-list">
                  {loadEntries.map((entry) => (
                    <button key={entry.name} type="button" className="load-item" onClick={() => void loadGame(entry.name)}>
                      <span>{entry.name}</span>
                      <small>{Math.round(entry.size_bytes / 1024)} KB</small>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      ) : null}

      <div className="minimap-shell">
        <div className="minimap-head">
          <span>Minimap</span>
          <strong>{state.snapshot?.factions.length ?? 0} factions</strong>
        </div>
        <canvas
          ref={miniMapRef}
          width={160}
          height={160}
          className="minimap"
          aria-label="Terrain minimap"
          onClick={inspectMinimapCell}
        />
      </div>
    </footer>
  );
}

function ToolButton({
  title,
  emoji,
  active,
  onClick,
}: {
  title: string;
  emoji: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button type="button" className={`tool-button ${active ? "active" : ""}`} title={title} onClick={onClick}>
      <span aria-hidden>{emoji}</span>
      <small>{title}</small>
    </button>
  );
}

function biomeColor(biome: string) {
  switch (biome) {
    case "deepwater":
      return "#0b2149";
    case "water":
      return "#294f86";
    case "sand":
      return "#cbbf73";
    case "grass":
      return "#567d39";
    case "forest":
      return "#254f2c";
    case "stone":
      return "#68645c";
    case "snow":
      return "#e7edf1";
    default:
      return "#334155";
  }
}

function drawCameraFrustum(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  cameraFocus: [number, number] | null,
) {
  const cx = (cameraFocus?.[0] ?? 0.5) * width;
  const cy = (cameraFocus?.[1] ?? 0.5) * height;
  const topW = width * 0.18;
  const bottomW = width * 0.42;
  const topH = height * 0.14;
  const bottomH = height * 0.28;
  const leftTop = cx - topW / 2;
  const rightTop = cx + topW / 2;
  const leftBottom = cx - bottomW / 2;
  const rightBottom = cx + bottomW / 2;
  const topY = cy - topH;
  const bottomY = cy + bottomH;
  ctx.save();
  ctx.strokeStyle = "rgba(255, 255, 255, 0.95)";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(leftTop, topY);
  ctx.lineTo(rightTop, topY);
  ctx.lineTo(rightBottom, bottomY);
  ctx.lineTo(leftBottom, bottomY);
  ctx.closePath();
  ctx.stroke();
  ctx.restore();
}
