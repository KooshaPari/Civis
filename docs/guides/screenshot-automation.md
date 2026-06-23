# Screenshot automation

Reference visuals for ADR-007 docs, README assets, and regression baselines. Save repo copies under `docs/assets/<client>/`.

## Bevy (`civ-bevy-window`)

```bash
CIVIS_TICK_BROADCAST=binary cargo run -p civ-server
cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window   # or: just civ-3d-bevy-window
```

Wait for chunk meshes and the HUD to settle. Orbit with left-drag, scroll to zoom; **`R`** resets (see [`clients/bevy-ref/README.md`](../../clients/bevy-ref/README.md)). Capture the game window with your OS screenshot tool and save as `docs/assets/bevy-ref/<name>.png`. Default framing orbits chunk centre `(8, 8, 8)`.

## Godot (`godot-ref`)

From [`clients/godot-ref/README.md`](../../clients/godot-ref/README.md):

1. `cargo run -p civ-watch`; rebuild: `cd clients/godot-ref/rust && cargo build`.
2. Open `project.godot` → **World** → tune **Terrain Height Exaggeration** if needed (default `24`).
3. **F5**; wait for terrain from `GET /terrain`.
4. Capture the 3D view (`Camera3D` ~ `(32, 50, 32)`):
   - **Editor:** 3D viewport → **Editor → Take Screenshot** (Godot user `screenshots/`).
   - **Game window:** **Project → Tools → Capture Screenshot**, or the screenshot shortcut.
5. Copy to `docs/assets/godot-ref/` (e.g. `terrain-default.png`).

## VitePress docs site (Playwright)

Port **5199** (`docs/.vitepress/constants.mjs`; webServer in `docs/tests/playwright.config.ts`).

```bash
cd docs && bun install
bun run docs:dev   # optional — e2e starts VitePress automatically
bunx playwright screenshot http://localhost:5199/ assets/docs/home.png
bun run docs:test:e2e
```

Failures attach screenshots under `docs/test-results/`; report HTML in `docs/playwright-report/`. Batch helpers: [`scripts/capture/`](../../scripts/capture/) (stub).
