import React, { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

// ---------------------------------------------------------------------------
// Types mirrored from crates/watch/src/main.rs + terrain.rs
// ---------------------------------------------------------------------------

type Biome =
  | "deepwater"
  | "water"
  | "sand"
  | "grass"
  | "forest"
  | "stone"
  | "snow";

type Terrain = {
  size: number;
  heights: number[];
  biomes: Biome[];
};

type Job =
  | "farmer"
  | "warrior"
  | "scholar"
  | "trader"
  | "priest"
  | "admin"
  | "unemployed";

type CivPin = {
  idx: number;
  x: number;
  y: number;
  job: Job | null;
};

type Snapshot = {
  tick: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: Array<{
    age: number;
    health: number;
    ideology: number;
    welfare: number;
    job: Job | null;
  }>;
  civ_pins: CivPin[];
  is_day: boolean;
  speed: number;
};

const BIOME_COLOR: Record<Biome, string> = {
  deepwater: "rgb(16, 38, 90)",
  water: "rgb(44, 100, 168)",
  sand: "rgb(222, 200, 132)",
  grass: "rgb(104, 154, 60)",
  forest: "rgb(44, 100, 52)",
  stone: "rgb(128, 124, 116)",
  snow: "rgb(240, 240, 240)",
};

const JOB_COLOR: Record<Job, string> = {
  farmer: "#7ed957",
  warrior: "#e74c3c",
  scholar: "#5b9bd5",
  trader: "#f1c40f",
  priest: "#9b59b6",
  admin: "#95a5a6",
  unemployed: "#34495e",
};

type Tool =
  | "place_voxel"
  | "spawn_civilian"
  | "damage"
  | "inspect"
  | "camera";

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async function postControl(path: string, body: unknown): Promise<boolean> {
  try {
    const res = await fetch(`/control/${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const data = await res.json().catch(() => ({ ok: false }));
    return Boolean(data?.ok);
  } catch {
    return false;
  }
}

async function fetchTerrain(): Promise<Terrain | null> {
  try {
    const res = await fetch("/terrain");
    if (!res.ok) return null;
    return (await res.json()) as Terrain;
  } catch {
    return null;
  }
}

// ---------------------------------------------------------------------------
// Top-down terrain canvas with civilian overlay
// ---------------------------------------------------------------------------

function GodView(props: {
  terrain: Terrain | null;
  snapshot: Snapshot | null;
  tool: Tool;
  material: number;
  radius: number;
  faction: number;
  onToast: (msg: string) => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { terrain, snapshot } = props;

  // Render heightmap base into an offscreen pattern when terrain changes.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !terrain) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const size = terrain.size;
    canvas.width = size * 5;
    canvas.height = size * 5;
    // Paint biome cells.
    const img = ctx.createImageData(canvas.width, canvas.height);
    for (let y = 0; y < size; y++) {
      for (let x = 0; x < size; x++) {
        const biome = terrain.biomes[y * size + x];
        const h = terrain.heights[y * size + x];
        const [r, g, b] = parseRgb(BIOME_COLOR[biome]);
        const shade = 0.65 + h * 0.35;
        const cellR = Math.round(r * shade);
        const cellG = Math.round(g * shade);
        const cellB = Math.round(b * shade);
        for (let dy = 0; dy < 5; dy++) {
          for (let dx = 0; dx < 5; dx++) {
            const px = (y * 5 + dy) * canvas.width + (x * 5 + dx);
            const i = px * 4;
            img.data[i] = cellR;
            img.data[i + 1] = cellG;
            img.data[i + 2] = cellB;
            img.data[i + 3] = 255;
          }
        }
      }
    }
    ctx.putImageData(img, 0, 0);
  }, [terrain]);

  // Overlay civilian pins + day/night tint every snapshot frame.
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !terrain || !snapshot) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    // Re-paint base (cheap; ImageData is cached by the engine).
    const w = canvas.width;
    const h = canvas.height;
    // Day/night veil — bluish at night, neutral at day.
    if (!snapshot.is_day) {
      ctx.save();
      ctx.fillStyle = "rgba(10, 14, 40, 0.35)";
      ctx.fillRect(0, 0, w, h);
      ctx.restore();
    }
    // Civilian pins.
    for (const pin of snapshot.civ_pins) {
      const px = pin.x * w;
      const py = pin.y * h;
      ctx.beginPath();
      ctx.arc(px, py, 3, 0, Math.PI * 2);
      ctx.fillStyle = pin.job ? JOB_COLOR[pin.job] : "white";
      ctx.fill();
      ctx.lineWidth = 0.5;
      ctx.strokeStyle = "rgba(0,0,0,0.6)";
      ctx.stroke();
    }
  }, [snapshot, terrain]);

  function onClick(e: React.MouseEvent<HTMLCanvasElement>) {
    const canvas = canvasRef.current;
    if (!canvas || !terrain) return;
    const rect = canvas.getBoundingClientRect();
    const cx = (e.clientX - rect.left) / rect.width;
    const cy = (e.clientY - rect.top) / rect.height;
    const gridX = Math.floor(cx * terrain.size);
    const gridY = Math.floor(cy * terrain.size);
    const SCALE = 1_000_000;
    const worldX = BigInt(gridX) * BigInt(SCALE);
    const worldZ = BigInt(gridY) * BigInt(SCALE);
    void (async () => {
      switch (props.tool) {
        case "place_voxel": {
          const ok = await postControl("place_voxel", {
            x: Number(worldX),
            y: 0,
            z: Number(worldZ),
            material: props.material,
          });
          props.onToast(
            ok ? `placed voxel @ ${gridX},${gridY}` : "place_voxel failed",
          );
          break;
        }
        case "spawn_civilian": {
          const ok = await postControl("spawn_civilian", {
            x: cx,
            y: cy,
            faction: props.faction,
          });
          props.onToast(ok ? `spawned civilian @ ${gridX},${gridY}` : "spawn failed");
          break;
        }
        case "damage": {
          const ok = await postControl("damage", {
            x: Number(worldX),
            y: 0,
            z: Number(worldZ),
            radius: props.radius,
            energy: 100,
          });
          props.onToast(
            ok ? `boom @ ${gridX},${gridY} r=${props.radius}` : "damage failed",
          );
          break;
        }
        case "inspect":
          props.onToast(`inspect ${gridX},${gridY} (not wired yet)`);
          break;
        case "camera":
          // Camera tool is a no-op on this 2D view (placeholder for 3D mode).
          break;
      }
    })();
  }

  return (
    <canvas
      ref={canvasRef}
      onClick={onClick}
      className="god-canvas"
      width={640}
      height={640}
    />
  );
}

function parseRgb(s: string): [number, number, number] {
  const m = s.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/);
  if (!m) return [255, 0, 255];
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

// ---------------------------------------------------------------------------
// UI panes
// ---------------------------------------------------------------------------

function TopBar(props: { snapshot: Snapshot | null; status: string }) {
  const s = props.snapshot;
  return (
    <div className="top-bar">
      <div className="stat">
        <div className="stat-label">Tick</div>
        <div className="stat-value">{s?.tick ?? "—"}</div>
      </div>
      <div className="stat">
        <div className="stat-label">Population</div>
        <div className="stat-value">{s?.population?.toLocaleString() ?? "—"}</div>
      </div>
      <div className="stat">
        <div className="stat-label">Voxel chunks</div>
        <div className="stat-value">{s?.voxel_chunk_count ?? "—"}</div>
      </div>
      <div className="stat">
        <div className="stat-label">Dirty / tick</div>
        <div className="stat-value">{s?.voxel_dirty_count ?? "—"}</div>
      </div>
      <div className="stat">
        <div className="stat-label">Day / Night</div>
        <div className="stat-value">{s?.is_day ? "☀ Day" : "🌙 Night"}</div>
      </div>
      <div className="connection-pill" data-status={props.status}>
        {props.status === "live"
          ? "● Live"
          : props.status === "reconnecting"
            ? "● Reconnecting"
            : "● Disconnected"}
      </div>
    </div>
  );
}

function BottomBar(props: {
  tool: Tool;
  setTool: (t: Tool) => void;
  speed: number;
  setSpeed: (s: number) => void;
  material: number;
  setMaterial: (m: number) => void;
  radius: number;
  setRadius: (r: number) => void;
  faction: number;
  setFaction: (f: number) => void;
}) {
  const tools: Array<{ key: Tool; icon: string; label: string }> = [
    { key: "place_voxel", icon: "🧱", label: "Voxel" },
    { key: "spawn_civilian", icon: "👤", label: "Civilian" },
    { key: "damage", icon: "💥", label: "Damage" },
    { key: "inspect", icon: "🔍", label: "Inspect" },
    { key: "camera", icon: "🎥", label: "Camera" },
  ];
  const speeds = [0, 1, 2, 4, 8];

  async function applySpeed(s: number) {
    props.setSpeed(s);
    await postControl("speed", { speed: s });
  }

  return (
    <div className="bottom-bar">
      <div className="tool-group">
        {tools.map((t) => (
          <button
            key={t.key}
            className={`tool-btn ${props.tool === t.key ? "active" : ""}`}
            onClick={() => props.setTool(t.key)}
            title={t.label}
          >
            <span className="tool-icon">{t.icon}</span>
            <span className="tool-label">{t.label}</span>
          </button>
        ))}
      </div>

      <div className="divider" />

      <div className="picker">
        <label>Material</label>
        <select
          value={props.material}
          onChange={(e) => props.setMaterial(Number(e.target.value))}
        >
          {Array.from({ length: 8 }, (_, i) => (
            <option key={i} value={i}>
              {i === 0 ? "(air)" : `material #${i}`}
            </option>
          ))}
        </select>
      </div>

      <div className="picker">
        <label>Radius</label>
        <input
          type="range"
          min={1}
          max={32}
          value={props.radius}
          onChange={(e) => props.setRadius(Number(e.target.value))}
        />
        <span>{props.radius}</span>
      </div>

      <div className="picker">
        <label>Faction</label>
        <select
          value={props.faction}
          onChange={(e) => props.setFaction(Number(e.target.value))}
        >
          <option value={0}>Player</option>
          <option value={1}>AI A</option>
          <option value={2}>AI B</option>
          <option value={3}>AI C</option>
        </select>
      </div>

      <div className="divider" />

      <div className="speed-group">
        {speeds.map((s) => (
          <button
            key={s}
            className={`speed-btn ${props.speed === s ? "active" : ""}`}
            onClick={() => void applySpeed(s)}
            title={s === 0 ? "Pause" : `${s}×`}
          >
            {s === 0 ? "⏸" : `${s}×`}
          </button>
        ))}
      </div>
    </div>
  );
}

function SidePanel(props: { snapshot: Snapshot | null }) {
  const sample = props.snapshot?.sample_civilians ?? [];
  return (
    <div className="side-panel">
      <h3>Civilians (first 8)</h3>
      <table>
        <thead>
          <tr>
            <th>Age</th>
            <th>Health</th>
            <th>Welfare</th>
            <th>Ideology</th>
            <th>Job</th>
          </tr>
        </thead>
        <tbody>
          {sample.length === 0 ? (
            <tr>
              <td colSpan={5} className="empty">
                waiting for first tick…
              </td>
            </tr>
          ) : (
            sample.map((c, i) => (
              <tr key={i}>
                <td>{c.age}</td>
                <td>{c.health.toFixed(2)}</td>
                <td>{c.welfare.toFixed(2)}</td>
                <td>{c.ideology.toFixed(2)}</td>
                <td>{c.job ?? "—"}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

// ---------------------------------------------------------------------------
// App shell
// ---------------------------------------------------------------------------

function App() {
  const [terrain, setTerrain] = useState<Terrain | null>(null);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [status, setStatus] = useState<"live" | "reconnecting" | "disconnected">(
    "reconnecting",
  );
  const [tool, setTool] = useState<Tool>("place_voxel");
  const [material, setMaterial] = useState<number>(1);
  const [radius, setRadius] = useState<number>(8);
  const [faction, setFaction] = useState<number>(0);
  const [speed, setSpeed] = useState<number>(1);
  const [toast, setToast] = useState<string | null>(null);

  useEffect(() => {
    void fetchTerrain().then((t) => {
      if (t) setTerrain(t);
    });
  }, []);

  // SSE subscription with auto-reconnect.
  useEffect(() => {
    let active = true;
    let es: EventSource | null = null;
    let backoff: number | null = null;

    function connect() {
      if (!active) return;
      setStatus("reconnecting");
      es = new EventSource("/events");
      es.addEventListener("snapshot", (ev: MessageEvent) => {
        if (!active) return;
        try {
          const snap = JSON.parse(ev.data) as Snapshot;
          setSnapshot(snap);
          setSpeed(snap.speed);
          setStatus("live");
        } catch {
          /* ignore */
        }
      });
      es.onerror = () => {
        setStatus("disconnected");
        es?.close();
        backoff = window.setTimeout(connect, 3000) as unknown as number;
      };
    }

    connect();

    return () => {
      active = false;
      es?.close();
      if (backoff) window.clearTimeout(backoff);
    };
  }, []);

  // Toast auto-dismiss.
  useEffect(() => {
    if (!toast) return;
    const id = window.setTimeout(() => setToast(null), 2500);
    return () => window.clearTimeout(id);
  }, [toast]);

  return (
    <div className="app">
      <TopBar snapshot={snapshot} status={status} />
      <div className="main">
        <div className="canvas-pane">
          <GodView
            terrain={terrain}
            snapshot={snapshot}
            tool={tool}
            material={material}
            radius={radius}
            faction={faction}
            onToast={setToast}
          />
        </div>
        <SidePanel snapshot={snapshot} />
      </div>
      <BottomBar
        tool={tool}
        setTool={setTool}
        speed={speed}
        setSpeed={setSpeed}
        material={material}
        setMaterial={setMaterial}
        radius={radius}
        setRadius={setRadius}
        faction={faction}
        setFaction={setFaction}
      />
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(<App />);
}
