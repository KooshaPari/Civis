# Sketchfab API Integration - Quick Reference Card

## Setup (5 minutes)

```bash
# 1. Create Sketchfab account + API token
#    https://sketchfab.com/settings/api

# 2. Copy template and add token
cp .env.example .env
# Edit .env:
# SKETCHFAB_API_TOKEN=your_token_here

# 3. Validate
assetctl validate-sketchfab-token
```

## API Token

| Item | Value |
|------|-------|
| **Generate URL** | https://sketchfab.com/settings/api |
| **Scope** | Read-only (search, metadata, download) |
| **Expiration** | 1 year (rotate annually) |
| **Format** | UUID or alphanumeric string |
| **Storage** | `.env` file (NEVER commit) |

## Rate Limits

| Plan | Daily | Per Hour | Downloads/Day |
|------|-------|----------|---------------|
| **Free** | 50 | 6-8 | 20 |
| **Pro** | 500 | 60 | Unlimited |

**Avoid 429**: Wait `Retry-After` header or until `X-RateLimit-Reset`

## CLI Commands

### Search
```bash
assetctl search-sketchfab "query" \
  --license cc-by,cc0 \
  --max-poly 2000 \
  --limit 10 \
  --format json
```

### Download Single
```bash
assetctl download-sketchfab sketchfab:model_id \
  --output packs/warfare-starwars/assets/raw/ \
  --franchise star_wars
```

### Batch Download
```bash
assetctl download-batch-sketchfab "query" \
  --output packs/warfare-starwars/assets/raw/ \
  --limit 5 \
  --max-concurrent 1
```

### Check Quota
```bash
assetctl sketchfab-quota
assetctl validate-sketchfab-token
```

## Environment Variables

```bash
# Required
SKETCHFAB_API_TOKEN=your_token

# Optional (defaults shown)
SKETCHFAB_API_BASE_URL=https://api.sketchfab.com/v3
SKETCHFAB_DOWNLOAD_FORMAT=glb
SKETCHFAB_LICENSE_FILTER=cc0,cc-by,cc-by-sa
SKETCHFAB_RATE_LIMIT_PER_HOUR=6
ASSET_DOWNLOAD_DIR=packs/assets/raw
```

## File Structure

After download:
```
packs/{pack}/assets/raw/{assetId}/
├── raw/
│   ├── source_download.glb           # Model file
│   ├── asset_manifest.json           # Metadata
│   └── metadata.json                 # Full Sketchfab info
└── ...
```

## Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| **401 Unauthorized** | Invalid token | Check SKETCHFAB_API_TOKEN |
| **429 Too Many Requests** | Rate limited | Wait until reset, or upgrade plan |
| **404 Not Found** | Model deleted | Verify model ID exists |
| **400 Bad Request** | Invalid query param | Check filter values |
| **Timeout** | Network issue | Check connectivity, increase timeout |

## License Validation

**Allowed**: `cc0`, `cc-by`, `cc-by-sa`
**Blocked**: `cc-by-nc`, `cc-by-nd`, proprietary, commercial

## Code Examples

### C# (After Implementation)

```csharp
// Initialize client
var client = new SketchfabClient(apiToken);

// Search
var results = await client.SearchModelsAsync("clone trooper",
    new SketchfabSearchFilters {
        License = "cc-by",
        MaxPolyCount = 2000
    });

// Download
var result = await client.DownloadModelAsync(modelId, "glb");
Console.WriteLine($"Downloaded: {result.FilePath}");
Console.WriteLine($"Hash: {result.Sha256}");
```

## Testing

```bash
# With mock responses (no API calls)
dotnet test src/Tests/ -k "Sketchfab"

# With real API (if token set)
export SKETCHFAB_API_TOKEN=your_token
dotnet test --filter "Category=IntegrationReal"
```

## Documentation

| Document | Purpose |
|----------|---------|
| `SKETCHFAB_API_SETUP.md` | Full setup, security, rate limits |
| `SKETCHFAB_CLI_COMMANDS.md` | Command specs & examples |
| `IMPLEMENTATION_ROADMAP.md` | 5-sprint implementation plan |
| `SketchfabClient.cs` | API client interface (pseudocode) |
| `AssetDownloader.cs` | Orchestrator interface (pseudocode) |

## Useful Links

- **API Docs**: https://docs.sketchfab.com/api/index.html
- **Token Settings**: https://sketchfab.com/settings/api
- **API Status**: https://status.sketchfab.com
- **Search Sketchfab**: https://sketchfab.com/search

## Checklists

### Before Download
- [ ] Token valid: `assetctl validate-sketchfab-token`
- [ ] Quota available: `assetctl sketchfab-quota`
- [ ] License allowed (CC-0, CC-BY only)
- [ ] Polycount within limits
- [ ] Output directory exists

### After Download
- [ ] `asset_manifest.json` created
- [ ] `source_download.glb` present
- [ ] SHA256 hash in manifest
- [ ] `ip_status` set correctly (fan_star_wars_private_only)
- [ ] `technical_status` = "discovered"

### For Batch Operations
- [ ] Set `SKETCHFAB_CONCURRENT_REQUESTS=1` (free tier)
- [ ] Schedule off-peak hours (11pm-7am UTC)
- [ ] Monitor rate limit between batches
- [ ] Enable deduplication (--skip-duplicates)
- [ ] Keep logs for audit trail

## Performance Tips

- Cache search results (avoid duplicate queries)
- Use `--max-poly` to filter large models early
- Download during off-peak (reduce API congestion)
- Batch 5-10 models per session (free tier)
- Use `--max-concurrent 1` for free tier

## Troubleshooting Checklist

```
Is token valid?
├─ assetctl validate-sketchfab-token
├─ If NO → regenerate at https://sketchfab.com/settings/api

Is quota available?
├─ assetctl sketchfab-quota
├─ If NO → wait until reset or upgrade to Pro

Is network working?
├─ ping api.sketchfab.com
├─ If NO → check internet

Is file download path writable?
├─ ls -ld packs/warfare-starwars/assets/raw/
├─ If permission denied → check ownership/perms

Still stuck?
├─ Check full docs: docs/SKETCHFAB_API_SETUP.md
├─ Check Sketchfab API status: https://status.sketchfab.com
├─ Ask team for help
```

---

**Version**: 1.0
**Last Updated**: 2026-03-11
