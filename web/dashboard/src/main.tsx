import React, { useEffect, useMemo, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";

type JobLabel = "Farmer" | "Warrior" | "Scholar" | "Trader" | "Priest" | "Admin" | "Unemployed";

type SampleCivilian = {
  age: number;
  health: number;
  ideology: number;
  welfare: number;
  job: JobLabel | null;
};

type Snapshot = {
  tick: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: SampleCivilian[];
};

type ConnectionState = "live" | "reconnecting" | "disconnected";

const rootRoute = createRootRoute({
  component: Dashboard,
});

const router = createRouter({ routeTree: rootRoute });

const JOB_COLORS: Record<JobLabel, string> = {
  Farmer: "#53d36b",
  Warrior: "#ff6262",
  Scholar: "#5db2ff",
  Trader: "#ffd65a",
  Priest: "#c78bff",
  Admin: "#8c96a8",
  Unemployed: "#111111",
};

function Dashboard() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [connection, setConnection] = useState<ConnectionState>("disconnected");
  const [canvasTick, setCanvasTick] = useState(0);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const reconnectTimerRef = useRef<number | null>(null);
  const sourceRef = useRef<EventSource | null>(null);
  const closedByCleanupRef = useRef(false);

  useEffect(() => {
    const loadSnapshot = async () => {
      try {
        const response = await fetch("/snapshot");
        if (!response.ok) return;
        const data = (await response.json()) as Snapshot | null;
        if (data) setSnapshot(data);
      } catch {
        // Keep the SSE path as the primary live feed.
      }
    };

    void loadSnapshot();
  }, []);

  useEffect(() => {
    const scheduleReconnect = () => {
      if (closedByCleanupRef.current) return;
      if (reconnectTimerRef.current !== null) return;
      setConnection((current) => (current === "live" ? "reconnecting" : current));
      reconnectTimerRef.current = window.setTimeout(() => {
        reconnectTimerRef.current = null;
        connect();
      }, 3000);
    };

    const connect = () => {
      if (closedByCleanupRef.current) return;
      sourceRef.current?.close();
      const source = new EventSource("/events");
      sourceRef.current = source;

      source.onopen = () => {
        setConnection("live");
      };

      source.onmessage = () => {
        setConnection("live");
      };

      source.addEventListener("snapshot", (event) => {
        const payload = (event as MessageEvent<string>).data;
        setSnapshot(JSON.parse(payload) as Snapshot);
        setConnection("live");
      });

      source.onerror = () => {
        if (source.readyState === EventSource.CLOSED) {
          setConnection("disconnected");
          scheduleReconnect();
          return;
        }
        setConnection("reconnecting");
        scheduleReconnect();
      };
    };

    connect();

    return () => {
      closedByCleanupRef.current = true;
      sourceRef.current?.close();
      if (reconnectTimerRef.current !== null) {
        window.clearTimeout(reconnectTimerRef.current);
        reconnectTimerRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    const handle = window.setInterval(async () => {
      if (snapshot) return;
      try {
        const response = await fetch("/snapshot");
        if (!response.ok) return;
        const data = (await response.json()) as Snapshot | null;
        if (data) setSnapshot(data);
      } catch {
        // Polling fallback stays best-effort while SSE reconnects.
      }
    }, 3000);
    return () => window.clearInterval(handle);
  }, [snapshot]);

  useEffect(() => {
    const id = window.setInterval(() => setCanvasTick((value) => value + 1), 120);
    return () => window.clearInterval(id);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const draw = () => {
      drawCanvas(ctx, snapshot, canvasTick);
    };

    draw();
  }, [snapshot, canvasTick]);

  const connectionLabel = {
    live: "Live",
    reconnecting: "Reconnecting",
    disconnected: "Disconnected",
  }[connection];

  return (
    <main className="shell">
      <section className="hero">
        <div>
          <p className="eyebrow">Civis 3D live watch</p>
          <h1>Simulation dashboard</h1>
          <p className="subhead">Live snapshot feed with voxel activity and civilian sampling.</p>
        </div>
        <span className={`status ${connection}`}>Connection: {connectionLabel}</span>
      </section>

      <section className="metrics">
        <Metric label="Tick" value={snapshot?.tick ?? 0} />
        <Metric label="Population" value={snapshot?.population ?? 0} />
        <Metric label="Voxel dirty" value={snapshot?.voxel_dirty_count ?? 0} />
        <Metric label="Voxel chunks" value={snapshot?.voxel_chunk_count ?? 0} />
      </section>

      <section className="grid">
        <article className="panel canvas-panel">
          <header>
            <h2>Top-down field</h2>
            <p>Grid rendered at 50px intervals. Origin pulse reflects the current tick.</p>
          </header>
          <canvas ref={canvasRef} className="map" width={600} height={400} aria-label="Top-down simulation view" />
        </article>

        <article className="panel table-panel">
          <header>
            <h2>Sample civilians</h2>
            <p>Up to eight civilians sampled from the latest snapshot.</p>
          </header>
          <div className="table-wrap">
            <table className="civilian-table">
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
                {snapshot?.sample_civilians.length ? (
                  snapshot.sample_civilians.map((civilian, index) => (
                    <tr key={`${civilian.age}-${index}`}>
                      <td>{civilian.age}</td>
                      <td>{civilian.health.toFixed(2)}</td>
                      <td>{civilian.welfare.toFixed(2)}</td>
                      <td>{civilian.ideology.toFixed(2)}</td>
                      <td>
                        <span className={`job job-${jobClassName(civilian.job)}`}>{civilian.job ?? "Unemployed"}</span>
                      </td>
                    </tr>
                  ))
                ) : (
                  <tr>
                    <td colSpan={5} className="empty">
                      Waiting for first snapshot...
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </article>
      </section>
    </main>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <article className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  );
}

function jobClassName(job: JobLabel | null) {
  return (job ?? "Unemployed").toLowerCase();
}

function drawCanvas(ctx: CanvasRenderingContext2D, snapshot: Snapshot | null, canvasTick: number) {
  const width = 600;
  const height = 400;

  ctx.clearRect(0, 0, width, height);

  const background = ctx.createLinearGradient(0, 0, width, height);
  background.addColorStop(0, "#07101d");
  background.addColorStop(1, "#0d1728");
  ctx.fillStyle = background;
  ctx.fillRect(0, 0, width, height);

  ctx.strokeStyle = "rgba(148, 190, 255, 0.14)";
  ctx.lineWidth = 1;
  for (let x = 0; x <= width; x += 50) {
    ctx.beginPath();
    ctx.moveTo(x + 0.5, 0);
    ctx.lineTo(x + 0.5, height);
    ctx.stroke();
  }
  for (let y = 0; y <= height; y += 50) {
    ctx.beginPath();
    ctx.moveTo(0, y + 0.5);
    ctx.lineTo(width, y + 0.5);
    ctx.stroke();
  }

  const originX = width / 2;
  const originY = height / 2;
  ctx.fillStyle = "rgba(255, 255, 255, 0.16)";
  ctx.beginPath();
  ctx.arc(originX, originY, 3, 0, Math.PI * 2);
  ctx.fill();

  if (!snapshot) return;

  const pulse = 6 + Math.sin(canvasTick / 3) * 3;
  ctx.fillStyle = "rgba(120, 217, 255, 0.2)";
  ctx.beginPath();
  ctx.arc(originX, originY, pulse, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "rgba(120, 217, 255, 0.8)";
  ctx.beginPath();
  ctx.arc(originX, originY, pulse + 3, 0, Math.PI * 2);
  ctx.stroke();

  snapshot.sample_civilians.slice(0, 8).forEach((civilian, index) => {
    const job = civilian.job ?? "Unemployed";
    const color = JOB_COLORS[job];
    const x = derivePosition(
      width,
      40,
      20,
      160,
      civilian.age * 13 + civilian.health * 19 + index * 71 + snapshot.tick * 7,
    );
    const y = derivePosition(
      height,
      42,
      20,
      156,
      civilian.welfare * 31 + civilian.ideology * 23 + index * 53 + snapshot.tick * 5,
    );

    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.arc(x, y, 5.5, 0, Math.PI * 2);
    ctx.fill();

    ctx.strokeStyle = "rgba(255, 255, 255, 0.2)";
    ctx.beginPath();
    ctx.arc(x, y, 8.5, 0, Math.PI * 2);
    ctx.stroke();
  });
}

function derivePosition(limit: number, min: number, padding: number, range: number, seed: number) {
  const span = Math.max(limit - padding * 2, 1);
  const value = Math.abs(Math.sin(seed) * range) % span;
  return padding + min + value * 0.9;
}

document.body.style.margin = "0";
document.body.style.background = "radial-gradient(circle at top, #111b2f 0%, #05070c 60%)";
document.body.style.color = "#eef3ff";
document.body.style.fontFamily = "Inter, system-ui, sans-serif";

const style = document.createElement("style");
style.textContent = `
  * { box-sizing: border-box; }
  body { min-height: 100vh; }
  .shell { padding: 24px; max-width: 1320px; margin: 0 auto; }
  .hero { display: flex; justify-content: space-between; align-items: start; gap: 16px; margin-bottom: 20px; }
  .eyebrow { margin: 0 0 8px; text-transform: uppercase; letter-spacing: 0.22em; color: #7ec6ff; font-size: 12px; }
  .subhead { margin-top: 10px; color: #99a9c8; max-width: 60ch; }
  h1, h2, p { margin: 0; }
  h1 { font-size: clamp(2.2rem, 5vw, 4.2rem); line-height: 0.95; }
  .status { border-radius: 999px; padding: 10px 14px; font-weight: 700; white-space: nowrap; align-self: center; }
  .status.live { background: rgba(34, 197, 94, 0.16); color: #9ef7b6; }
  .status.reconnecting { background: rgba(249, 115, 22, 0.18); color: #ffc28f; }
  .status.disconnected { background: rgba(239, 68, 68, 0.16); color: #ffadad; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin: 18px 0 20px; }
  .metric, .panel { background: rgba(10, 16, 28, 0.88); border: 1px solid rgba(126, 198, 255, 0.14); border-radius: 20px; box-shadow: 0 30px 90px rgba(0, 0, 0, 0.35); backdrop-filter: blur(10px); }
  .metric { padding: 18px; display: grid; gap: 8px; }
  .metric span { color: #90a4c6; font-size: 12px; text-transform: uppercase; letter-spacing: 0.14em; }
  .metric strong { font-size: clamp(1.8rem, 3vw, 2.6rem); line-height: 1; }
  .grid { display: grid; grid-template-columns: minmax(0, 1.6fr) minmax(320px, 1fr); gap: 16px; align-items: start; }
  .panel { padding: 18px; }
  .panel header { display: grid; gap: 6px; margin-bottom: 14px; }
  .panel header p { color: #90a4c6; }
  .map { width: 100%; height: auto; aspect-ratio: 3 / 2; display: block; border-radius: 16px; background: #08111f; }
  .table-wrap { overflow: auto; border-radius: 16px; border: 1px solid rgba(126, 198, 255, 0.08); }
  .civilian-table { width: 100%; border-collapse: collapse; min-width: 440px; }
  .civilian-table th, .civilian-table td { padding: 12px 14px; text-align: left; border-bottom: 1px solid rgba(126, 198, 255, 0.08); }
  .civilian-table th { color: #9bb0d3; font-size: 12px; text-transform: uppercase; letter-spacing: 0.12em; background: rgba(255, 255, 255, 0.02); }
  .civilian-table td { color: #eef3ff; }
  .civilian-table tbody tr:last-child td { border-bottom: none; }
  .empty { color: #90a4c6; padding: 20px 14px; text-align: center; }
  .job { display: inline-flex; align-items: center; padding: 6px 10px; border-radius: 999px; color: #08111f; font-weight: 700; font-size: 12px; }
  .job-farmer { background: ${JOB_COLORS.Farmer}; }
  .job-warrior { background: ${JOB_COLORS.Warrior}; }
  .job-scholar { background: ${JOB_COLORS.Scholar}; }
  .job-trader { background: ${JOB_COLORS.Trader}; }
  .job-priest { background: ${JOB_COLORS.Priest}; }
  .job-admin { background: ${JOB_COLORS.Admin}; }
  .job-unemployed { background: ${JOB_COLORS.Unemployed}; color: #eef3ff; border: 1px solid rgba(255, 255, 255, 0.12); }
  @media (max-width: 1024px) {
    .metrics, .grid { grid-template-columns: 1fr; }
    .hero { flex-direction: column; }
    .status { align-self: flex-start; }
  }
`;
document.head.appendChild(style);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
