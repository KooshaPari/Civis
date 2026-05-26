# ADR-010: Deterministic Star-Wars-Style Asset Intake Pipeline

**Status**: Implementation in Progress
**Date**: 2026-03-11
**Deciders**: Agent Org

## Context

The current mod workflow can ingest assets through ad-hoc manual steps, but a repeatable low-poly conversion for a Star Wars conversion requires:

- deterministic source discovery (prefer API-driven),
- provenance-first manifests,
- explicit legal/IP state separate from technical quality,
- machine-readable scoring and rejection rules,
- and non-destructive registration before runtime use.

The immediate need is a pre-implementation contract for an `assetctl` toolchain, not a full production asset conversion subsystem.

## Decision

- Add an explicit agent-facing intake pipeline with machine-readable schemas and policy files.
- Add `assetctl` as a dedicated CLI surface for `search`, `intake`, `normalize`, `validate`, `stylize`, `register`, `export-unity`.
- Use a source-tier model (Sketchfab primary, BlendSwap secondary, ModDB reference, browser fallback last).
- Track two independent state dimensions on each asset:
  - `technical_status` (discovered → normalized → validated → ready_for_prototype),
  - `ip_status` (generic_safe vs high_risk_do_not_ship).
- Require a manifest conforming to `schemas/asset-manifest.schema.json` before promotion beyond prototype.

## Consequences

- Legal/provenance decisions become auditable and machine-enforced instead of informal.
- Asset quality variance is controlled through explicit technical gates and validation reports.
- Source/API strategy is configurable without code changes via policy documents.
- Pre-implementation artifacts remain non-invasive: no runtime coupling before design freeze.
- Future release-safe workflows can be added by changing policy and status transitions, not by rewriting the whole pipeline.

## Alternatives Considered

- One-off manual asset import:
  - rejected due to non-repeatability and weak provenance control.
- Full external DAM integration:
  - rejected in V1 due to complexity and scope, but preserved as a future extension.
- Browser automation as default:
  - rejected because it is brittle and harder to audit for policy and retries.

## M13: Asset Library Browser

The asset library browser (M13) extends the asset intake pipeline with a persistent catalog store and CLI surface for browsing and managing assets.

### Components

| Component | File | Responsibility |
|-----------|------|----------------|
| `AssetCatalogStore` | `AssetCatalogStore.cs` | SQLite-backed persistent catalog with CRUD operations |
| `ISourceAdapter` | `ISourceAdapter.cs` | Interface for asset source adapters |
| `LocalSourceAdapter` | `LocalSourceAdapter.cs` | Reads assets from local pack registry directories |
| `AssetLibraryCommand` | `AssetLibraryCommand.cs` | CLI surface for library operations |

### AssetCatalogStore Design

The catalog store provides:

- **SQLite persistence**: Assets are stored in a local SQLite database for fast queries
- **JSON export/import**: Portable catalog format via `schemas/asset-library.schema.json`
- **Search capabilities**: Filter by faction, type, status, and free-text query
- **Statistics**: Aggregate counts by faction, type, status, and pack

#### Database Schema

```sql
CREATE TABLE assets (
    asset_id TEXT PRIMARY KEY,
    name TEXT,
    faction TEXT,
    type TEXT,
    source_url TEXT,
    status TEXT,
    provenance TEXT,
    pack_id TEXT,
    metadata_json TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_assets_faction ON assets(faction);
CREATE INDEX idx_assets_type ON assets(type);
CREATE INDEX idx_assets_status ON assets(status);
CREATE INDEX idx_assets_pack_id ON assets(pack_id);
```

### CLI Commands

| Command | Description |
|---------|-------------|
| `dinoforge assetctl library list [--faction X] [--type Y] [--status Z]` | List assets from catalog |
| `dinoforge assetctl library search <query>` | Search assets by name or ID |
| `dinoforge assetctl library show <asset-id>` | Show detailed asset info |
| `dinoforge assetctl library stats` | Show catalog statistics |
| `dinoforge assetctl library sync` | Sync from pack registries |
| `dinoforge assetctl library import local` | Import from local pack registries |
| `dinoforge assetctl library export <path>` | Export catalog to JSON |

### Source Adapter Interface

The `ISourceAdapter` interface enables future extensibility:

```csharp
public interface ISourceAdapter
{
    Task<IEnumerable<AssetCandidate>> SearchAsync(string query, CancellationToken ct = default);
    Task<AssetCandidate?> GetByIdAsync(string id, CancellationToken ct = default);
    string SourceName { get; }
    bool SupportsSearch { get; }
    bool SupportsGetById { get; }
}
```

Implementations:
- `LocalSourceAdapter`: Reads from `packs/*/assets/registry/asset_index.json`
