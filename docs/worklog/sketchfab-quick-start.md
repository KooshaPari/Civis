# Sketchfab Integration Quick Start

This guide walks you through setting up the Sketchfab API integration for DINOForge's automated asset intake pipeline.

## Prerequisites

- DINOForge CLI built and installed
- A Sketchfab account (free or pro)
- Command-line access to `assetctl` commands

## Step 1: Get a Sketchfab API Token

1. Go to [Sketchfab Settings > API](https://sketchfab.com/settings/api)
2. Click **"Generate New Token"** (or use an existing token if available)
3. Configure token scope:
   - Scope: **Read** (search, metadata, downloads)
   - Expiration: 1 year or longer
   - Name: "DINOForge Asset CLI" (optional, for identification)
4. Copy the token (UUID format, ~36 characters)

## Step 2: Create `.env` File

The `.env.example` file is provided in the repo. Copy it to create your local configuration:

```bash
# From repo root:
cp .env.example .env
```

## Step 3: Fill in Your API Token

Edit `.env` and replace the placeholder:

```bash
# BEFORE:
SKETCHFAB_API_TOKEN=your_personal_access_token_here

# AFTER:
SKETCHFAB_API_TOKEN=abc123def456...your-actual-token...
```

**Important**: The `.env` file is in `.gitignore` and will NOT be committed to version control. Your token is safe locally.

## Step 4: Validate the Token

Run the validation command to verify your token works:

```bash
dotnet run --project src/Tools/Cli -- assetctl validate-sketchfab-token --format json
```

Expected output (success):
```json
{
  "success": true,
  "command": "validate-sketchfab-token",
  "status": "valid",
  "token_length": 36,
  "message": "Token format valid (API validation not yet implemented)"
}
```

If this fails:
- Double-check the token in `.env` (no extra spaces)
- Verify the token is still active at https://sketchfab.com/settings/api
- Try re-generating a token if needed

## Step 5: Search for Models

Search Sketchfab for candidate assets:

```bash
# Simple search
dotnet run --project src/Tools/Cli -- assetctl search-sketchfab "clone trooper"

# Advanced search with options
dotnet run --project src/Tools/Cli -- assetctl search-sketchfab "clone trooper" \
  --limit 10 \
  --license cc-by,cc-by-sa \
  --format json
```

Output example (text format):
```
┌────────────────────────────────────────────────────┐
│ Assetctl Result                                    │
├────────────────────────────────────────────────────┤
│ Search: clone trooper                              │
│ Source: sketchfab                                  │
│ Results: 5 candidates found                        │
│ ...                                                │
└────────────────────────────────────────────────────┘
```

Output example (JSON format):
```json
{
  "query": "clone trooper",
  "count": 5,
  "source_filter": "sketchfab",
  "results": [
    {
      "source": "sketchfab",
      "external_id": "abc123xyz",
      "title": "Clone Trooper Phase I",
      "license": "CC-BY-4.0",
      "confidence_score": 0.91
    }
  ]
}
```

## Step 6: Download a Model

Once you've found a model ID from search results, download it:

```bash
# Download single model
dotnet run --project src/Tools/Cli -- assetctl download-sketchfab "abc123xyz" \
  --format glb \
  --format json
```

**Note**: Downloads are saved to the directory specified in `.env`:
```
ASSET_DOWNLOAD_DIR=packs/warfare-starwars/assets/raw
```

## Step 7: Batch Download Multiple Models

Create a manifest JSON file with multiple model IDs:

```json
{
  "models": [
    { "id": "model-id-1", "title": "Clone Trooper Phase I" },
    { "id": "model-id-2", "title": "B1 Battle Droid" },
    { "id": "model-id-3", "title": "AAT Walker" }
  ]
}
```

Then run batch download:

```bash
dotnet run --project src/Tools/Cli -- assetctl download-batch-sketchfab manifest.json \
  --parallel 2 \
  --format json
```

**Rate limit note**: Free tier allows ~6 requests/hour. Use `--parallel 1` for sequential downloads to stay under limits.

## Step 8: Check API Quota

Monitor your Sketchfab API quota at any time:

```bash
dotnet run --project src/Tools/Cli -- assetctl sketchfab-quota --format json
```

Output:
```json
{
  "success": true,
  "command": "sketchfab-quota",
  "requests_remaining": 45,
  "requests_per_hour": "6",
  "reset_at": "2026-03-12T15:30:00Z",
  "message": "Quota info not yet implemented"
}
```

## Configuration Reference

All settings in `.env` control the behavior of asset commands:

### API Settings
| Variable | Default | Notes |
|----------|---------|-------|
| `SKETCHFAB_API_TOKEN` | (required) | Your personal access token from Sketchfab |
| `SKETCHFAB_API_BASE_URL` | https://api.sketchfab.com/v3 | API endpoint (don't change) |
| `SKETCHFAB_DOWNLOAD_FORMAT` | glb | Format: glb, fbx, usdz |
| `SKETCHFAB_RATE_LIMIT_PER_HOUR` | 6 | Free: 6, Pro: 60+ |
| `SKETCHFAB_CONCURRENT_REQUESTS` | 1 | Parallel downloads (1-5 recommended) |
| `SKETCHFAB_HTTP_TIMEOUT_SECONDS` | 30 | Request timeout |

### Search Defaults
| Variable | Default | Notes |
|----------|---------|-------|
| `SKETCHFAB_LICENSE_FILTER` | cc0,cc-by,cc-by-sa | Allowed licenses |
| `SKETCHFAB_POLY_MAX` | 5000 | Max polygon count filter |
| `SKETCHFAB_SORT_BY` | relevance | Sort: relevance, likeCount, viewCount |
| `SKETCHFAB_EXCLUDE_PAID` | true | Exclude paid models |

### Pipeline Settings
| Variable | Default | Notes |
|----------|---------|-------|
| `ASSET_DOWNLOAD_DIR` | packs/warfare-starwars/assets/raw | Where downloads go |
| `ASSET_CACHE_DIR` | .cache/sketchfab | API response cache |
| `ASSET_INTEGRITY_CHECK` | true | Enable SHA256 verification |
| `ASSET_GENERATE_HASHES` | true | Auto-generate checksums |
| `ASSET_AUTO_MANIFEST` | true | Generate asset_manifest.json |
| `ASSET_BATCH_SIZE_MAX` | 10 | Max downloads per batch |

### Logging
| Variable | Default | Notes |
|----------|---------|-------|
| `SKETCHFAB_LOG_LEVEL` | info | debug, info, warn, error |
| `SKETCHFAB_LOG_HTTP_HEADERS` | false | Log HTTP headers |
| `SKETCHFAB_LOG_HTTP_BODY` | false | Log request/response bodies |
| `SKETCHFAB_ENABLE_METRICS` | true | Enable performance metrics |
| `VERBOSE` | false | Enable verbose console output |

## Rate Limit Strategy

Sketchfab has different limits based on your account tier:

### Free Tier
- ~50 requests/day
- ~6 requests/hour
- 2-3 second delays recommended between requests

### Pro Tier
- ~500 requests/day
- ~60 requests/hour
- Can do 2-3 parallel requests

### Best Practices
1. Set `SKETCHFAB_RATE_LIMIT_PER_HOUR` to your tier's limit
2. Set `SKETCHFAB_CONCURRENT_REQUESTS=1` for free tier
3. Use `assetctl sketchfab-quota` before batch downloads
4. Schedule batch downloads during off-peak hours
5. Cache search results to avoid duplicate queries

## Troubleshooting

### "SKETCHFAB_API_TOKEN not set"
- Verify `.env` file exists in repo root
- Check that `SKETCHFAB_API_TOKEN` line is not commented out (no `#` at start)
- Verify token value has no extra spaces

### "401 Unauthorized"
- Token is invalid or expired
- Go to https://sketchfab.com/settings/api and generate a new one
- Update `.env` with the new token

### "429 Too Many Requests"
- You've hit the API rate limit
- Wait for the reset time (shown in response headers)
- Check `assetctl sketchfab-quota` to see remaining quota
- Reduce `SKETCHFAB_CONCURRENT_REQUESTS` for batch downloads

### "Network timeout"
- Increase `SKETCHFAB_HTTP_TIMEOUT_SECONDS` in `.env` (e.g., 60)
- Check your internet connection
- Try again with `--format json` for more details

### "Connection refused"
- Verify you're online and can reach api.sketchfab.com
- Test with: `curl -H "Authorization: Bearer YOUR_TOKEN" https://api.sketchfab.com/v3/me`

## Next Steps

Once models are downloaded and validated:

1. **Normalize** in Blender — use `assetctl normalize` for LOD generation
2. **Validate** — run `assetctl validate` to check manifest completeness
3. **Stylize** — apply `assetctl stylize` with your profile
4. **Register** — add to asset index with `assetctl register`
5. **Export** — generate Unity-ready files with `assetctl export-unity`

See [Asset Intake Pipeline](./docs/asset-intake/) for the full workflow.

## Support & Resources

- **Sketchfab API Docs**: https://sketchfab.com/developers/api/v3
- **DINOForge Issues**: https://github.com/KooshaPari/Dino/issues
- **Clone Wars Sourcing**: See `packs/warfare-starwars/CLONE_WARS_SOURCING_MANIFEST.md`
