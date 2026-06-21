# Web Dashboard

This dashboard is served by the `civ-watch` process. Start the backend first, then run the Vite app.

## Launch Sequence

Terminal 1:

```powershell
cargo run -p civ-watch --release
```

Terminal 2:

```powershell
cd web/dashboard
bun install
bun run dev
```

Browser:

```text
http://localhost:5173
```

## Backend Endpoints

The dashboard reads the live simulation from `civ-watch`:

- `http://localhost:9090/events`
- `http://localhost:9090/snapshot`
- `http://localhost:9090/terrain`

If `localhost` resolves to a different listener on your machine, use `http://127.0.0.1:9090` for backend checks.

## Verification

Confirm the dashboard is serving HTML:

```powershell
curl http://localhost:5173
```

The page should return the dashboard HTML, and the snapshot should include civilians and buildings from the live simulation.
