# Asset Intake Pipeline - Sketchfab Integration

## Overview

DINOForge's asset intake pipeline enables **automated discovery, download, and ingestion** of 3D models from Sketchfab for use in mod packs. This documentation covers the complete setup, API integration, and operational workflows.

## Quick Links

| Document | Purpose |
|----------|---------|
| **[sketchfab-cli-commands.md](./sketchfab-cli-commands.md)** | CLI command specifications and examples |
| **[implementation-roadmap.md](./implementation-roadmap.md)** | 5-sprint implementation plan for agents |
| **[quick-reference.md](./quick-reference.md)** | Quick reference card |
| **[assetctl-prd.md](./assetctl-prd.md)** | Asset control PRD |

> Sketchfab API setup notes are tracked in the worklog (not part of the published site); see the source markdown under `docs/worklog/sketchfab-api-setup.md` in the repo.

## Status

| Component | Status | Notes |
|-----------|--------|-------|
| **API Setup Guide** | ✅ Complete | Full guide with OAuth vs token, credential mgmt, rate limits |
| **.env.example** | ✅ Complete | Template with all configuration options |
| **SketchfabClient.cs** | 🏗️ Pseudocode | Interface + method signatures, ready for implementation |
| **AssetDownloader.cs** | 🏗️ Pseudocode | Orchestrator interface, ready for implementation |
| **CLI Commands** | 📋 Designed | Spec document complete, commands awaiting implementation |
| **Unit Tests** | 📋 Designed | Test fixtures created, test cases specified |
| **Integration Tests** | 📋 Designed | Test scenarios defined |
| **Documentation** | ✅ Complete | All setup and reference docs ready |

## Architecture

### High-Level Flow

```
Search Query
    ↓
[AssetDownloader.SearchCandidatesAsync()]
    ↓
SketchfabClient.SearchModelsAsync() + Filter + Rank
    ↓
Ranked Candidates (confidence score 0.0-1.0)
    ↓
User selects or batch downloads
    ↓
[AssetDownloader.DownloadAssetAsync()] for each
    ↓
[SketchfabClient.DownloadModelAsync()] with:
    - Streaming download
    - SHA256 hash computation
    - Progress reporting
    ↓
Generate asset_manifest.json
    ↓
Store in packs/{pack-name}/assets/raw/{assetId}/
```

### Component Stack

```
CLI Commands (assetctl search-sketchfab, etc.)
    ↓
AssetDownloader (orchestrator)
    ↓
SketchfabClient (HTTP client)
    ↓
SketchfabHttpHandler (rate limit + retry)
    ↓
System.Net.Http.HttpClient (standard library)
    ↓
Sketchfab API v3 (https://api.sketchfab.com/v3)
```

## Getting Started

### 1. Setup API Token (5 minutes)

```bash
# 1. Create Sketchfab account (free): https://sketchfab.com/signup
# 2. Generate API token: https://sketchfab.com/settings/api
# 3. Copy .env.example to .env and fill in token
cp .env.example .env
# Edit .env and add your token:
# SKETCHFAB_API_TOKEN=your_token_here
```

See the Sketchfab API setup worklog (`docs/worklog/sketchfab-api-setup.md` in the source repo) for detailed instructions.

### 2. Validate Token (1 minute)

```bash
# Check that token works
dotnet run --project src/Tools/Cli -- assetctl validate-sketchfab-token
```

**Expected Output**:
```
✓ Token is valid
Plan: Free Tier
Remaining Today: 50 requests
```

### 3. Search for Assets (2 minutes)

```bash
# Search for Clone Trooper models (CC-BY licensed, max 2000 polygons)
dotnet run --project src/Tools/Cli -- \
  assetctl search-sketchfab "clone trooper" \
  --license cc-by \
  --max-poly 2000 \
  --limit 10 \
  --format json | jq
```

### 4. Download Single Asset (5 minutes)

```bash
# Download top candidate
MODEL_ID="a1b2c3d4e5f6"  # from search results
dotnet run --project src/Tools/Cli -- \
  assetctl download-sketchfab sketchfab:$MODEL_ID \
  --output packs/warfare-starwars/assets/raw/ \
  --franchise star_wars
```

**Result**: Asset stored with manifest
```
packs/warfare-starwars/assets/raw/
└── sw_sketchfab_clone_trooper_phase_1/
    ├── raw/
    │   ├── source_download.glb          # Downloaded model
    │   ├── asset_manifest.json          # Metadata manifest
    │   └── metadata.json                # Sketchfab metadata
    └── ...
```

### 5. Batch Download (10 minutes)

```bash
# Search and download top 5 models in one command
dotnet run --project src/Tools/Cli -- \
  assetctl download-batch-sketchfab "clone wars infantry" \
  --output packs/warfare-starwars/assets/raw/ \
  --limit 5 \
  --max-poly 3000 \
  --max-concurrent 1
```

## Rate Limiting

### Quota by Plan

| Plan | Requests/Day | Requests/Hour | Downloads/Day |
|------|--------------|---------------|---------------|
| **Free** (recommended for dev) | 50 | ~6 | 20 |
| **Pro** | 500 | ~60 | Unlimited |

### Strategy

**For Free Tier**:
- ✅ Schedule batch downloads during off-peak hours (11pm-7am UTC)
- ✅ Cache search results to avoid duplicate queries
- ✅ Download 5-10 models per day (max)
- ✅ Use `assetctl sketchfab-quota` to monitor remaining requests

**For Pro Tier**:
- ✅ Can run batch operations during business hours
- ✅ Can parallelize downloads (2-3 concurrent)
- ✅ Higher search volume for research

### Monitoring

```bash
# Check remaining quota
dotnet run --project src/Tools/Cli -- assetctl sketchfab-quota --format json

# Output:
# { "remaining": 42, "limit": 50, "reset_at": "...", "reset_in_hours": 17.4 }
```

## Security

### Credential Management

⚠️ **IMPORTANT**: Never commit `.env` to git

```bash
# Safe approach (already set up)
echo ".env" >> .gitignore  # .env is already ignored
cp .env.example .env
# Edit .env locally only
```

### Token Rotation

- [ ] Set token expiration to 1 year
- [ ] Rotate annually or if leaked
- [ ] Use read-only scope (no write permissions)
- [ ] Store in environment variables, not code

See API Setup: Security Best Practices (in the worklog at `docs/worklog/sketchfab-api-setup.md`).

## Workflows

### Workflow 1: One-Off Search

```bash
assetctl search-sketchfab "B1 battle droid" \
  --license cc0,cc-by \
  --max-poly 2000 \
  --sort-by likeCount \
  --format text
```

Use this to:
- Explore available models
- Check license compatibility
- Validate poly counts before downloading

### Workflow 2: Bulk Intake (1-5 hours)

```bash
#!/bin/bash
# Batch intake script for multiple queries

QUERIES=("clone trooper" "B1 droid" "vehicle transport" "weapons blasters")
FRANCHISE="star_wars"
OUTPUT_DIR="packs/warfare-starwars/assets/raw/"

for query in "${QUERIES[@]}"; do
  echo "Processing: $query"
  dotnet run --project src/Tools/Cli -- \
    assetctl download-batch-sketchfab "$query" \
    --output "$OUTPUT_DIR" \
    --limit 3 \
    --max-concurrent 1 \
    --skip-duplicates

  # Respect rate limit
  sleep 300  # 5 min pause between batches
done
```

Use this to:
- Intake multiple asset types in one session
- Automate recurring asset discovery
- Spread downloads across day (rate limit friendly)

### Workflow 3: Resume After Interruption

```bash
# If batch was interrupted, re-run same command
# Deduplication will skip already-downloaded models
assetctl download-batch-sketchfab "clone wars infantry" \
  --output packs/warfare-starwars/assets/raw/ \
  --limit 10 \
  --skip-duplicates  # This will exclude already-downloaded
```

Use this to:
- Safely resume interrupted downloads
- Complete partial batches
- Add more models to existing set

### Workflow 4: License Audit

```bash
# Before pipeline: validate all assets have allowed licenses
# (CLI command planned for v1.1)
for manifest in packs/*/assets/raw/*/raw/asset_manifest.json; do
  jq '.license_label' "$manifest"
done | sort | uniq -c
```

Use this to:
- Ensure compliance (CC-0, CC-BY only)
- Flag non-compliant models
- Generate audit reports

## Manifest Format

Generated `asset_manifest.json`:

```json
{
  "assetId": "sw_sketchfab_clone_trooper_phase_1",
  "canonicalName": "sketchfab_clone_trooper_phase_1",
  "franchiseTag": "star_wars",
  "sourcePlatform": "sketchfab",
  "sourceUrl": "https://sketchfab.com/models/a1b2c3d4e5f6",
  "externalId": "a1b2c3d4e5f6",
  "authorName": "artist_name",
  "licenseLabel": "CC-BY-4.0",
  "licenseUrl": "https://creativecommons.org/licenses/by/4.0/",
  "acquisitionMode": "api",
  "acquiredAtUtc": "2026-03-11T20:45:30Z",
  "technicalStatus": "discovered",
  "ipStatus": "fan_star_wars_private_only",
  "provenanceConfidence": 0.92,
  "fileHash": "sha256:a1b2c3d4...",
  "fileSize": 3670016,
  "notes": [
    "Downloaded via Sketchfab API",
    "License verified: CC-BY-4.0",
    "Polycount: 1200 (within limits)"
  ]
}
```

**Fields**:
- `assetId` — Unique ID in DINOForge system
- `technicalStatus` — Pipeline stage (discovered → normalized → validated → ready)
- `ipStatus` — IP/copyright status (fan_star_wars_private_only = fan-made, not for commercial use)
- `provenanceConfidence` — Ranking score (0.0-1.0, how well this asset matches criteria)
- `licenseLabel` — Original license from Sketchfab
- `acquisitionMode` — How obtained (api, scrape, browser_automation, manual_copy)

## Troubleshooting

### Token Invalid

```bash
# Check token
assetctl validate-sketchfab-token

# Output: ✗ Token is invalid
# Solution:
# 1. Go to https://sketchfab.com/settings/api
# 2. Regenerate a new token
# 3. Update SKETCHFAB_API_TOKEN in .env
```

### Rate Limited

```bash
# assetctl search-sketchfab "clone" returns 429

# Check quota
assetctl sketchfab-quota

# Output: Remaining: 0 / 50
# Solution:
# 1. Wait until reset time (usually next day 00:00 UTC)
# 2. Upgrade to Pro tier if frequent heavy usage
# 3. Cache search results to avoid duplicate queries
```

### Download Fails

```bash
# Download interrupted or times out

# Try again with explicit output path
assetctl download-sketchfab sketchfab:model_id \
  --output packs/warfare-starwars/assets/raw/ \
  --format json  # JSON format shows detailed error

# If persistent:
# 1. Check internet connectivity
# 2. Increase SKETCHFAB_HTTP_TIMEOUT_SECONDS in .env
# 3. Check if model still exists on Sketchfab
# 4. Try different format (--format zip instead of glb)
```

See the Sketchfab API setup worklog (`docs/worklog/sketchfab-api-setup.md`, Troubleshooting section) for more.

## Implementation Status

### ✅ Completed
- [x] API setup guide (100% complete)
- [x] Credential management guide
- [x] Rate limiting documentation
- [x] .env.example template
- [x] SketchfabClient interface design
- [x] AssetDownloader interface design
- [x] CLI command specifications
- [x] Exception hierarchy design
- [x] Data model definitions
- [x] 5-sprint implementation roadmap
- [x] Test strategy document

### 🏗️ Pending Implementation (Agents)
- [ ] Sprint 1: Dependencies, fixtures, config
- [ ] Sprint 2: SketchfabClient (search, metadata, download)
- [ ] Sprint 3: AssetDownloader (orchestrator, batch, dedup)
- [ ] Sprint 4: CLI commands (search-sketchfab, download, batch, validate, quota)
- [ ] Sprint 5: Testing, documentation, release

### 📋 Planned Future Features
- [ ] OAuth support (user-initiated downloads)
- [ ] Resume/incremental downloads
- [ ] License audit tool
- [ ] Asset caching (SQLite metadata DB)
- [ ] Support for additional sources (BlendSwap, ModDB)
- [ ] Comprehensive reporting (CSV, PDF)

## Next Steps for Agents

1. **Start Sprint 1**: Create TaskList for implementation, assign to team
2. **Review Pseudocode**: Read SketchfabClient.cs and AssetDownloader.cs
3. **Set Up Environment**: Copy `.env.example` to `.env` with test token
4. **Begin Implementation**: Follow [implementation-roadmap.md](./implementation-roadmap.md)

## Support & Questions

- **API Issues**: See `docs/worklog/sketchfab-api-setup.md` (worklog) in the source repo
- **Command Usage**: See [sketchfab-cli-commands.md](./sketchfab-cli-commands.md)
- **Implementation**: See [implementation-roadmap.md](./implementation-roadmap.md)
- **Errors**: Check troubleshooting sections in API setup guide

## References

- **Sketchfab API Docs**: https://docs.sketchfab.com/api/index.html
- **API Status**: https://status.sketchfab.com
- **Token Management**: https://sketchfab.com/settings/api
- **License Guide**: https://sketchfab.com/licenses

---

**Last Updated**: 2026-03-11
**Maintained By**: DINOForge Agents
**Status**: Documentation complete, awaiting implementation
