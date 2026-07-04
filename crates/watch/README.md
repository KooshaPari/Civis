# civ-watch

Local hot-reload harness for Civis 3D. Runs a background `Simulation` at ~10 Hz and exposes a small HTTP API plus the built dashboard at `/`.

Default listen address: `http://127.0.0.1:9090` (`CIV_WATCH_PORT` overrides the port).

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/events` | Server-Sent Events stream; each event is named `snapshot` |
| `GET` | `/snapshot` | Latest snapshot JSON, or `null` before the first tick |
| `GET` | `/terrain` | Procedural heightmap generated once at startup |
| `POST` | `/control/place_voxel` | Write one voxel |
| `POST` | `/control/spawn_civilian` | Spawn a civilian |
| `POST` | `/control/damage` | Queue a damage event |
| `POST` | `/control/speed` | Set simulation speed multiplier |

Build the dashboard first: `cd web/dashboard && npm run build`. Static files are served from `web/dashboard/dist`.

## `GET /terrain`

Generated once at startup from seed `42`. Responses include an `ETag` derived from the heightmap fingerprint; send `If-None-Match` to receive `304 Not Modified` on repeat fetches.

```json
{
  "size": 128,
  "heights": [0.0, 0.12, "..."],
  "biomes": ["deepwater", "water", "sand", "grass", "forest", "stone", "snow"]
}
```

| Field | Type | Notes |
|-------|------|-------|
| `size` | `number` | Grid side length (currently `128`) |
| `heights` | `number[]` | Row-major normalized heights in `[0, 1]`; length `size × size` |
| `biomes` | `string[]` | Parallel biome per cell; lowercase enum values listed above |

## `GET /snapshot`

Returns the most recent simulation snapshot, or JSON `null` before the worker emits the first tick.

```json
{
  "tick": 42,
  "tick_dt_ms": 100,
  "population": 0,
  "voxel_dirty_count": 0,
  "voxel_chunk_count": 1,
  "sample_civilians": [],
  "civ_pins": [],
  "factions": [],
  "buildings": [],
  "is_day": true,
  "speed": 1
}
```

| Field | Type | Notes |
|-------|------|-------|
| `tick` | `number` | Current simulation tick |
| `tick_dt_ms` | `number` | Approximate ms between rendered frames at current speed |
| `population` | `number` | Total population |
| `voxel_dirty_count` | `number` | Voxel events in the last tick |
| `voxel_chunk_count` | `number` | Loaded voxel chunks |
| `sample_civilians` | `object[]` | Up to 8 citizens with `age`, `health`, `ideology`, `welfare`, `job` |
| `civ_pins` | `object[]` | Up to 256 map pins with `idx`, `x`, `y`, `dx`, `dy`, `job` (`x`/`y` in `[0, 1]`) |
| `factions` | `object[]` | `id`, `color` `[r,g,b]`, `capital` `[x,y]`, `radius` |
| `buildings` | `object[]` | `id`, `x`, `y`, `kind` (`Residential`…`Civic`), `era`, `faction_id` |
| `is_day` | `boolean` | Day/night flag from climate |
| `speed` | `number` | Active speed multiplier (`0`, `1`, `2`, `4`, or `8`) |

`job` values: `farmer`, `warrior`, `scholar`, `trader`, `priest`, `admin`, `unemployed`, or `null`.

## Control requests

All control routes return `{ "ok": true, "message": null }` on success, or `{ "ok": false, "message": "…" }` on validation failure.

### `POST /control/place_voxel`

```json
{ "x": 0, "y": 0, "z": 0, "material": 1 }
```

World coordinates use the kernel voxel grid (`WorldCoord`); `material` is a `MaterialId` (`u16`).

### `POST /control/speed`

```json
{ "speed": 1 }
```

Allowed values: `0` (pause), `1`, `2`, `4`, `8`.
