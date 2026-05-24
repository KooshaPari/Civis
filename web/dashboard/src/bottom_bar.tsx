import { useEffect, useRef } from "react";
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
import { useDashboardStore, type TimeSpeed } from "./store";

export function BottomBar() {
  const { state, dispatch } = useDashboardStore();
  const miniMapRef = useRef<HTMLCanvasElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const runWatchControl = async (path: string, body: object = {}) => {
    try {
      await postControl(path, body);
    } catch {
      dispatch({ type: "set_toast", message: `Failed: ${path}` });
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
    });

    const bounds = minimapBoundsFromKeys(state.loadedChunkIds);
    if (bounds) {
      for (const chunkId of state.loadedChunkIds) {
        const [u, v] = chunkToMinimapUv(chunkId, bounds);
        ctx.fillStyle = "#b8b09e";
        ctx.fillRect(u * width - 2, v * height - 2, 4, 4);
      }
    }
  }, [state.snapshot, state.terrain, state.loadedChunkIds]);

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
        <span className="control-label">View (read-only)</span>
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
      </div>

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
        ) : null}
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
