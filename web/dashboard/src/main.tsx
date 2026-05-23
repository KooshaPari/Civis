import React, { useEffect, useMemo, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";

type SampleCivilian = {
  age: number;
  health: number;
  ideology: number;
  welfare: number;
  job: string | null;
};

type Snapshot = {
  tick: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: SampleCivilian[];
};

const rootRoute = createRootRoute({
  component: Dashboard,
});

const router = createRouter({ routeTree: rootRoute });

function Dashboard() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [connected, setConnected] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const source = new EventSource("/events");
    source.onopen = () => setConnected(true);
    source.onerror = () => setConnected(false);
    source.addEventListener("snapshot", (event) => {
      const payload = (event as MessageEvent<string>).data;
      setSnapshot(JSON.parse(payload) as Snapshot);
    });
    return () => source.close();
  }, []);

  useEffect(() => {
    const handle = window.setInterval(async () => {
      if (snapshot) return;
      const response = await fetch("/snapshot");
      if (!response.ok) return;
      const data = (await response.json()) as Snapshot | null;
      setSnapshot(data);
    }, 1000);
    return () => window.clearInterval(handle);
  }, [snapshot]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      const ratio = window.devicePixelRatio || 1;
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;
      canvas.width = Math.max(1, Math.round(width * ratio));
      canvas.height = Math.max(1, Math.round(height * ratio));
      ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
      drawCanvas(ctx, width, height, snapshot);
    };

    resize();
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [snapshot]);

  const pill = connected ? "Live" : "Disconnected";
  return (
    <main className="shell">
      <section className="hero">
        <div>
          <p className="eyebrow">Civis 3D live watch</p>
          <h1>Simulation dashboard</h1>
        </div>
        <span className={`status ${connected ? "live" : "dead"}`}>Connection: {pill}</span>
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
            <h2>Voxel chunk map</h2>
            <p>Placeholder top-down view until full protocol meshes land.</p>
          </header>
          <canvas ref={canvasRef} className="map" />
        </article>

        <article className="panel">
          <header>
            <h2>Sample civilians</h2>
            <p>Snapshot payload from the background simulation worker.</p>
          </header>
          <div className="civilian-list">
            {snapshot?.sample_civilians.length ? (
              snapshot.sample_civilians.map((civilian, index) => (
                <div className="civilian" key={`${civilian.age}-${index}`}>
                  <strong>Citizen {index + 1}</strong>
                  <span>Age {civilian.age}</span>
                  <span>Health {civilian.health.toFixed(2)}</span>
                  <span>Welfare {civilian.welfare.toFixed(2)}</span>
                  <span>Job {civilian.job ?? "none"}</span>
                </div>
              ))
            ) : (
              <div className="empty">Waiting for first snapshot...</div>
            )}
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

function drawCanvas(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  snapshot: Snapshot | null,
) {
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#08111f";
  ctx.fillRect(0, 0, width, height);
  ctx.strokeStyle = "rgba(126, 198, 255, 0.18)";
  ctx.lineWidth = 1;

  for (let x = 0; x < width; x += 32) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }
  for (let y = 0; y < height; y += 32) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }

  if (!snapshot) return;

  const count = Math.max(snapshot.voxel_chunk_count, 1);
  for (let i = 0; i < count; i += 1) {
    const x = ((i * 97) % Math.max(width - 24, 1)) + 12;
    const y = ((i * 53) % Math.max(height - 24, 1)) + 12;
    ctx.fillStyle = i % 2 === 0 ? "#7ec6ff" : "#9bffcc";
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, Math.PI * 2);
    ctx.fill();
  }
}

document.body.style.margin = "0";
document.body.style.background = "#05070c";
document.body.style.color = "#eef3ff";
document.body.style.fontFamily = "Inter, system-ui, sans-serif";

const style = document.createElement("style");
style.textContent = `
  .shell { padding: 24px; max-width: 1280px; margin: 0 auto; }
  .hero { display: flex; justify-content: space-between; align-items: center; gap: 16px; }
  .eyebrow { margin: 0 0 8px; text-transform: uppercase; letter-spacing: 0.2em; color: #7ec6ff; font-size: 12px; }
  h1, h2, p { margin: 0; }
  h1 { font-size: clamp(2rem, 5vw, 4rem); }
  .status { border-radius: 999px; padding: 10px 14px; font-weight: 700; }
  .status.live { background: rgba(33, 197, 94, 0.16); color: #9ef7b6; }
  .status.dead { background: rgba(239, 68, 68, 0.16); color: #ffadad; }
  .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin: 24px 0; }
  .metric, .panel { background: rgba(10, 16, 28, 0.92); border: 1px solid rgba(126, 198, 255, 0.14); border-radius: 20px; box-shadow: 0 30px 90px rgba(0, 0, 0, 0.35); }
  .metric { padding: 18px; display: grid; gap: 8px; }
  .metric span { color: #90a4c6; font-size: 12px; text-transform: uppercase; letter-spacing: 0.14em; }
  .metric strong { font-size: 2rem; }
  .grid { display: grid; grid-template-columns: 2fr 1fr; gap: 16px; align-items: start; }
  .panel { padding: 18px; }
  .panel header { display: grid; gap: 6px; margin-bottom: 14px; }
  .panel header p { color: #90a4c6; }
  .map { width: 100%; height: 480px; display: block; border-radius: 16px; background: #08111f; }
  .civilian-list { display: grid; gap: 12px; }
  .civilian { display: grid; gap: 4px; padding: 12px; border-radius: 14px; background: rgba(255, 255, 255, 0.03); }
  .empty { color: #90a4c6; padding: 16px 0; }
  @media (max-width: 960px) {
    .metrics, .grid { grid-template-columns: 1fr; }
    .hero { align-items: start; flex-direction: column; }
  }
`;
document.head.appendChild(style);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
