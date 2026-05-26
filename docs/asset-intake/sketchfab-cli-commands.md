# Sketchfab CLI Commands Integration Guide

## Overview

This document specifies the new CLI commands for Sketchfab integration that should be added to `AssetctlCommand.cs`. These commands enable automated asset discovery, download, and manifest generation for the DINOForge asset intake pipeline.

**Implementation Status**: Pseudocode/interface design (agents implement per design)

---

## New CLI Commands

### 1. `assetctl search-sketchfab <query>`

**Purpose**: Search Sketchfab for candidate assets matching query and filters

**Usage**:
```bash
assetctl search-sketchfab "clone wars b1 droid" \
  --license cc-by,cc0 \
  --max-poly 2000 \
  --limit 10 \
  --sort-by relevance \
  --format json

# Output: Ranked candidates with confidence scores
```

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `<query>` | string | (required) | Search keywords |
| `--license` | string | `cc0,cc-by,cc-by-sa` | CSV of allowed licenses |
| `--max-poly` | int | `5000` | Max polygon count filter |
| `--min-poly` | int | (none) | Min polygon count filter |
| `--limit` | int | `10` | Max results to return |
| `--sort-by` | string | `relevance` | Sort order: relevance\|likeCount\|viewCount\|publishedAt |
| `--exclude-paid` | bool | `true` | Exclude paid/exclusive models |
| `--format` | string | `text` | Output format: text\|json |

**Output (text)**:
```
╔═══════════════════════════════════════════════════════════════╗
║ Sketchfab Search: clone wars b1 droid                        ║
╚═══════════════════════════════════════════════════════════════╝

┌─────┬──────────────────────┬──────────┬────────────┬──────────┐
│ #   │ Model Name           │ License  │ Poly Count │ Score    │
├─────┼──────────────────────┼──────────┼────────────┼──────────┤
│ 1   │ Star Wars B1 Droid   │ CC-BY    │ 1,200      │ 0.92     │
│ 2   │ B1 Battle Droid      │ CC-BY-SA │ 2,400      │ 0.87     │
│ 3   │ Clone Trooper B1     │ CC-0     │ 800        │ 0.85     │
└─────┴──────────────────────┴──────────┴────────────┴──────────┘

Found 3 results matching criteria.
✓ All models use allowed licenses.
```

**Output (json)**:
```json
{
  "query": "clone wars b1 droid",
  "filters": {
    "license": "cc0,cc-by,cc-by-sa",
    "max_poly": 5000,
    "min_poly": null,
    "sort_by": "relevance",
    "exclude_paid": true
  },
  "results_count": 3,
  "results": [
    {
      "model_id": "a1b2c3d4e5f6",
      "name": "Star Wars B1 Droid",
      "creator": {
        "display_name": "3d_artist_123",
        "profile_url": "https://sketchfab.com/3d_artist_123"
      },
      "license": "cc-by-4.0",
      "poly_count": 1200,
      "face_count": 600,
      "published_at": "2024-06-15T10:30:00Z",
      "model_url": "https://sketchfab.com/models/a1b2c3d4e5f6",
      "confidence_score": 0.92,
      "ranking_details": {
        "license_score": 0.95,
        "polycount_score": 0.90,
        "recency_score": 0.85,
        "calculation": "(0.95 * 0.5) + (0.90 * 0.3) + (0.85 * 0.2) = 0.92"
      }
    }
  ],
  "rate_limit": {
    "remaining": 42,
    "reset_at": "2026-03-12T00:00:00Z"
  }
}
```

**Implementation Notes**:
- Calls `AssetDownloader.SearchCandidatesAsync(query, criteria)`
- Results pre-ranked by confidence score
- License filter is intersection with `SKETCHFAB_LICENSE_FILTER` from `.env`
- Rate limit state printed in footer

---

### 2. `assetctl download-sketchfab <model_ref>`

**Purpose**: Download a specific model and generate intake manifest

**Usage**:
```bash
assetctl download-sketchfab sketchfab:a1b2c3d4e5f6 \
  --output packs/warfare-starwars/assets/raw/ \
  --format glb \
  --generate-manifest

# Output: Downloaded file path, manifest path, SHA256 hash
```

**Arguments**:
| Argument | Format | Description |
|----------|--------|-------------|
| `<model_ref>` | `source:modelId` | Model reference (e.g., `sketchfab:a1b2c3d4e5f6`) |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--output` | path | `.` | Output directory for asset |
| `--format` | string | `glb` | Download format (glb, zip, fbx) |
| `--generate-manifest` | bool | `true` | Generate asset_manifest.json |
| `--franchise` | string | `custom` | Franchise tag for assetId |
| `--ip-status` | string | `private` | IP status label (e.g., fan_star_wars_private_only) |
| `--format` | string | `text` | Output format: text\|json |

**Output (text)**:
```
╔═══════════════════════════════════════════════════════════════╗
║ Sketchfab Download: a1b2c3d4e5f6                            ║
╚═══════════════════════════════════════════════════════════════╝

↓ Downloading: Star Wars B1 Droid... [████████████████────] 65%
  Size: 2.3 MB / 3.5 MB
  Speed: 2.1 MB/s
  Remaining: ~0.6s

✓ Download Complete
  Duration: 1.6s
  File Size: 3.5 MB
  SHA256: a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4
  Asset ID: sw_sketchfab_star_wars_b1_droid

✓ Manifest Generated
  Path: packs/warfare-starwars/assets/raw/sw_sketchfab_star_wars_b1_droid/raw/asset_manifest.json

Rate Limit: 41/50 remaining (reset in 17h)
```

**Output (json)**:
```json
{
  "success": true,
  "model_id": "a1b2c3d4e5f6",
  "asset_id": "sw_sketchfab_star_wars_b1_droid",
  "file_path": "/abs/path/to/packs/warfare-starwars/assets/raw/sw_sketchfab_star_wars_b1_droid/raw/source_download.glb",
  "manifest_path": "/abs/path/to/packs/warfare-starwars/assets/raw/sw_sketchfab_star_wars_b1_droid/raw/asset_manifest.json",
  "metadata_path": "/abs/path/to/packs/warfare-starwars/assets/raw/sw_sketchfab_star_wars_b1_droid/raw/metadata.json",
  "file_size_bytes": 3670016,
  "sha256": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4",
  "download_duration_ms": 1600,
  "download_speed_mbps": 2.29,
  "rate_limit": {
    "remaining": 41,
    "reset_at": "2026-03-12T00:00:00Z",
    "reset_in_hours": 17
  }
}
```

**Implementation Notes**:
- Calls `AssetDownloader.DownloadAssetAsync(candidate, outputDir)`
- Auto-generates asset_manifest.json with metadata
- Computes SHA256 during download for integrity checking
- Shows progress bar during download
- Logs rate limit state after completion

---

### 3. `assetctl download-batch-sketchfab <query>`

**Purpose**: Search and batch-download top N candidates matching criteria

**Usage**:
```bash
assetctl download-batch-sketchfab "clone wars infantry" \
  --output packs/warfare-starwars/assets/raw/ \
  --limit 5 \
  --max-poly 3000 \
  --license cc-by,cc0 \
  --max-concurrent 1 \
  --skip-duplicates \
  --format text

# Output: Batch progress, rate limit warnings, final report
```

**Arguments**:
| Argument | Format | Description |
|----------|--------|-------------|
| `<query>` | string | Search query |

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--output` | path | (current dir) | Output directory for all assets |
| `--limit` | int | `5` | Max models to download |
| `--max-poly` | int | `5000` | Max polygon filter |
| `--min-poly` | int | (none) | Min polygon filter |
| `--license` | string | `cc0,cc-by,cc-by-sa` | Allowed licenses (CSV) |
| `--max-concurrent` | int | `1` | Concurrent downloads (free=1, pro=2-3) |
| `--skip-duplicates` | bool | `true` | Skip already-downloaded models |
| `--format` | string | `text` | Output: text\|json |

**Output (text - progress)**:
```
╔═══════════════════════════════════════════════════════════════╗
║ Sketchfab Batch Download: clone wars infantry                ║
╚═══════════════════════════════════════════════════════════════╝

Search: "clone wars infantry" | Limit: 5 | License: cc-by,cc0
Max Poly: 5000 | Max Concurrent: 1 | Skip Duplicates: yes

Searching candidates...
✓ Found 8 results, 7 unique, 5 non-duplicate

[1/5] Star Wars Clone Trooper Phase 1
      ↓ Downloading... [████████████████────] 65%
      Size: 2.3 MB / 3.5 MB | Speed: 2.1 MB/s

[2/5] B1 Battle Droid Low Poly
      ⏳ Waiting (rate limit check)...

[3/5] Clone Trooper Phase 2
      ⏳ Queued

---

Final Report:
✓ Succeeded: 2 / 5
⏳ In Progress: 1
⏸️  Queued: 2

Elapsed: 3.2s
Remaining: ~6s
Rate Limit: 38/50 (reset in 17h)
```

**Output (json - final)**:
```json
{
  "success": true,
  "query": "clone wars infantry",
  "search_results": 8,
  "candidates_unique": 7,
  "candidates_downloaded": 5,
  "candidates_duplicates": 1,
  "results": [
    {
      "model_id": "uuid1",
      "name": "Clone Trooper Phase 1",
      "status": "completed",
      "asset_id": "sw_sketchfab_clone_trooper_phase_1",
      "file_path": "...",
      "manifest_path": "...",
      "file_size_bytes": 3670016,
      "sha256": "...",
      "duration_ms": 1600
    },
    {
      "model_id": "uuid2",
      "name": "B1 Droid",
      "status": "completed",
      "asset_id": "sw_sketchfab_b1_droid",
      "file_path": "...",
      "manifest_path": "...",
      "file_size_bytes": 2457600,
      "sha256": "...",
      "duration_ms": 1200
    }
  ],
  "failed": [],
  "summary": {
    "total_duration_ms": 8000,
    "total_downloaded_bytes": 6127616,
    "average_speed_mbps": 0.765,
    "rate_limit_remaining": 38
  }
}
```

**Implementation Notes**:
- Calls `AssetDownloader.SearchCandidatesAsync()` then `DownloadBatchAsync()`
- Deduplicates by checking existing asset_manifest.json files
- Respects rate limit (pauses if approaching quota)
- Progress callback updates display in real-time
- Rate limit state checked before and after batch

---

### 4. `assetctl validate-sketchfab-token`

**Purpose**: Validate API token and display quota info

**Usage**:
```bash
assetctl validate-sketchfab-token --format json

# Output: Token validity, plan type, remaining quota
```

**Options**:
| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--format` | string | `text` | Output format: text\|json |

**Output (text)**:
```
╔═══════════════════════════════════════════════════════════════╗
║ Sketchfab Token Validation                                   ║
╚═══════════════════════════════════════════════════════════════╝

✓ Token is valid

  Account Plan: Free Tier
  Daily Limit: 50 requests
  Remaining Today: 47 requests
  Reset Time: 2026-03-12T00:00:00Z (17 hours)

  Rate Limit Strategy: Sequential (max 1 req/10s)
  Recommended Batch Size: 5-10 models/day
  Recommended Usage: Off-peak hours (11pm-7am UTC)

  Token Expiration: 2026-12-31
  Token Scope: Read (search, metadata, download)

✓ All systems ready for asset downloads.
```

**Output (json)**:
```json
{
  "valid": true,
  "plan": "Free",
  "daily_limit": 50,
  "remaining_quota": 47,
  "reset_at": "2026-03-12T00:00:00Z",
  "reset_in_hours": 17,
  "token_expires_at": "2026-12-31T23:59:59Z",
  "token_scope": "Read",
  "recommended_batch_size": 10,
  "recommended_max_concurrent": 1,
  "recommendations": [
    "Use off-peak hours for batch downloads",
    "Cache search results to avoid duplicate queries",
    "Monitor rate limit state with assetctl sketchfab-quota"
  ]
}
```

**Implementation Notes**:
- Calls `SketchfabClient.ValidateTokenAsync()`
- Determines plan type from rate limit headers
- Provides recommendations based on plan
- Useful for debugging token/auth issues

---

### 5. `assetctl sketchfab-quota`

**Purpose**: Check current API rate limit quota

**Usage**:
```bash
assetctl sketchfab-quota --format json

# Output: Remaining requests, reset time, rate limit state
```

**Output (text)**:
```
╔═══════════════════════════════════════════════════════════════╗
║ Sketchfab API Quota                                          ║
╚═══════════════════════════════════════════════════════════════╝

Remaining Requests: 42 / 50
Reset Time: 2026-03-12T00:00:00Z
Reset In: 17 hours 23 minutes

Last Check: 2026-03-11T06:37:00Z

⚠️  Quota Warning: 8 requests remaining (16% of daily limit)
    Recommended action: Wait until reset or upgrade to Pro tier

📊 Estimated Availability:
   - If used now: 42 more requests available today
   - Next reset: 2026-03-12 00:00:00 UTC
   - Safe batch size: 10 models (accounting for search queries)
```

**Output (json)**:
```json
{
  "remaining": 42,
  "limit": 50,
  "reset_at": "2026-03-12T00:00:00Z",
  "reset_in_hours": 17.39,
  "reset_in_seconds": 62604,
  "percentage_remaining": 84,
  "last_checked_at": "2026-03-11T06:37:00Z"
}
```

---

## Integration Points

### 1. Update `AssetctlCommand.cs`

Add these methods to create the new commands:

```csharp
// In AssetctlCommand class

private static Command CreateSearchSketchfabCommand()
{
    // Argument: <query>
    // Options: --license, --max-poly, --limit, --sort-by, --format
    // Action: Call AssetDownloader.SearchCandidatesAsync()
    throw new NotImplementedException();
}

private static Command CreateDownloadSketchfabCommand()
{
    // Argument: <model_ref>
    // Options: --output, --format, --generate-manifest, --franchise, --ip-status
    // Action: Call AssetDownloader.DownloadAssetAsync()
    throw new NotImplementedException();
}

private static Command CreateDownloadBatchSketchfabCommand()
{
    // Argument: <query>
    // Options: --output, --limit, --max-poly, --license, --max-concurrent, --skip-duplicates
    // Action: Call AssetDownloader.DownloadBatchAsync()
    throw new NotImplementedException();
}

private static Command CreateValidateSketchfabTokenCommand()
{
    // No arguments
    // Options: --format
    // Action: Call SketchfabClient.ValidateTokenAsync()
    throw new NotImplementedException();
}

private static Command CreateSketchfabQuotaCommand()
{
    // No arguments
    // Options: --format
    // Action: Call SketchfabClient.GetRateLimitState()
    throw new NotImplementedException();
}
```

Then update `Create()` method to register:

```csharp
public static Command Create()
{
    Command command = new("assetctl", "Agent-friendly asset intake pipeline commands");

    // Existing commands...
    command.AddCommand(CreateSearchCommand());
    command.AddCommand(CreateIntakeCommand());

    // NEW: Sketchfab commands
    command.AddCommand(CreateSearchSketchfabCommand());
    command.AddCommand(CreateDownloadSketchfabCommand());
    command.AddCommand(CreateDownloadBatchSketchfabCommand());
    command.AddCommand(CreateValidateSketchfabTokenCommand());
    command.AddCommand(CreateSketchfabQuotaCommand());

    return command;
}
```

### 2. Environment Configuration

Agents should read from `.env`:
```csharp
var apiToken = Environment.GetEnvironmentVariable("SKETCHFAB_API_TOKEN");
var apiUrl = Environment.GetEnvironmentVariable("SKETCHFAB_API_BASE_URL")
    ?? "https://api.sketchfab.com/v3";
var maxConcurrent = int.TryParse(
    Environment.GetEnvironmentVariable("SKETCHFAB_CONCURRENT_REQUESTS"),
    out var value) ? value : 1;
```

### 3. Error Handling Pattern

Follow existing assetctl patterns:

```csharp
WriteResult(new
{
    success = false,
    command = "search-sketchfab",
    message = "Rate limited. Try again in 1 hour.",
    error = "rate_limit:3600"
}, outputFormat);
```

---

## Testing Strategy

### Unit Tests

```csharp
[Theory]
[InlineData("clone trooper", "cc-by,cc0", 2000, 5)]
[InlineData("b1 droid", "cc0", 1500, 3)]
public async Task SearchSketchfabCommand_ReturnsFilteredResults(
    string query, string license, int maxPoly, int limit)
{
    var mockClient = new Mock<SketchfabClient>();
    mockClient.Setup(c => c.SearchModelsAsync(query, It.IsAny<SketchfabSearchFilters>()))
        .ReturnsAsync(new[] { /* mock results */ });

    // Execute command
    // Assert results are filtered correctly
}

[Fact]
public async Task DownloadSketchfabCommand_GeneratesManifest()
{
    // Mock SketchfabClient.GetModelMetadataAsync() + DownloadModelAsync()
    // Execute download command
    // Assert asset_manifest.json is created correctly
}

[Fact]
public async Task DownloadBatchCommand_RespectRateLimit()
{
    // Mock rate limit state (remaining = 2)
    // Execute batch download of 10 models
    // Assert batch stops at rate limit, provides resume instructions
}
```

### Integration Tests

```bash
# Require valid SKETCHFAB_API_TOKEN in .env
dotnet test --filter "Category=SketchfabIntegration" \
    --environment SKETCHFAB_API_TOKEN=<your_token>
```

---

## Future Enhancements

1. **OAuth Support**: Allow user-initiated downloads (not service account)
2. **Incremental Downloads**: Resume partial downloads
3. **Filtering Rules**: Define filtering via YAML instead of CLI flags
4. **License Audit**: Periodic compliance checks of downloaded assets
5. **Asset Caching**: Local SQLite DB of model metadata for faster searches
6. **Reporting**: Generate intake reports (CSV, PDF) of batch operations

---

**Status**: Ready for implementation
**Assigned To**: Agent developers
**Estimated Effort**: 3-4 sprints (client + commands + tests)
