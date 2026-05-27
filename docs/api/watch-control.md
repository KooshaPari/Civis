# civ-watch control API

HTTP control routes exposed by `civ-watch` on the default listen address
`http://127.0.0.1:9090` (`CIV_WATCH_PORT` overrides the port).

All control routes return JSON. Validation failures use HTTP `400` with:

```json
{ "ok": false, "message": "reason" }
```

Successful mod operations return route-specific payloads documented below.

## Mod catalog and lifecycle

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/control/mods/catalog` | List installable mods (examples, uploads, publish, remote cache) |
| `POST` | `/control/mods/upload` | Upload a `.civmod` archive (base64 body) to `mods/uploads/` |
| `POST` | `/control/mods/publish` | Copy a validated `.civmod` into `mods/publish/` |
| `GET` | `/control/mods/published` | List published mods |
| `POST` | `/control/mods/install` | Load a mod from a catalog `source` path |
| `POST` | `/control/mods/unload` | Unload a mod by stable `mod_id` |
| `POST` | `/control/mods/reload` | Hot-reload a loaded mod by `mod_id` |

## Remote mod cache

Fetch mods from HTTP(S) URLs into a local cache under `mods/remote/{id}/`.
Fetched archives appear in `/control/mods/catalog` like uploads and published mods.

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/control/mods/fetch` | Download and cache a remote `.civmod` / zip archive |
| `GET` | `/control/mods/remote` | List cached remote mods |

### `POST /control/mods/fetch`

Request body:

```json
{
  "url": "https://example.com/mods/demo.civmod",
  "mod_id": "demo-mod"
}
```

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `url` | `string` | yes | Must be `http://` or `https://`; empty URLs rejected |
| `mod_id` | `string` | no | Cache directory name; defaults to `url-{sha256-prefix}` |

Security and transport limits:

- Rejects non-HTTP schemes (`file://`, `ftp://`, etc.)
- Rejects path traversal in optional `mod_id` (`..`, `/`, `\`)
- Download timeout: 30 seconds
- Maximum payload size: 50 MiB
- Redirects: up to 5 hops

Validation:

- Payload must be a ZIP / `.civmod` archive with a root `manifest.toml`
- When `mod.wasm.sig` is present, Ed25519 verification uses the same rules as
  `civ-mod-host` (`author_pubkey_hex` required)

Success response:

```json
{
  "ok": true,
  "id": "demo-mod",
  "source": "mods/remote/demo-mod/mod.civmod",
  "path": "/abs/path/to/mods/remote/demo-mod/mod.civmod"
}
```

Cache layout:

```text
mods/remote/{id}/
  mod.civmod   # validated archive bytes
  meta.json    # { "id", "url", "fetched_at" }
```

Fetch failures from the origin server return HTTP `502` with `{ "ok": false, "message": "…" }`.
Invalid archives return HTTP `400`.

### `GET /control/mods/remote`

Returns cached remote mods:

```json
[
  {
    "id": "demo-mod",
    "path": "mods/remote/demo-mod/mod.civmod",
    "fetched_at": 1710000000,
    "url": "https://example.com/mods/demo.civmod"
  }
]
```

Install a cached mod with `POST /control/mods/install` using the returned `path`
as `source` (repo-relative, e.g. `mods/remote/demo-mod/mod.civmod`).
