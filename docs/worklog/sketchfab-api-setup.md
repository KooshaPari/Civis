# Sketchfab API Setup & Integration Guide

## Overview

This guide covers setting up Sketchfab API access for DINOForge's automated asset intake pipeline. The Sketchfab v3 REST API enables searching, fetching metadata, and downloading 3D models programmatically.

**Integration Layer**: `src/Tools/Cli/Assetctl/Sketchfab/` (C# client wrapper + orchestrator)

**Related Commands**:
- `assetctl search-sketchfab <query>` — Search Sketchfab for models
- `assetctl download-sketchfab <model_ref>` — Download model by ID
- `assetctl ingest-batch <dir>` — Validate & manifest downloaded assets

---

## Part 1: Creating a Sketchfab Account

### Account Types

| Type | Free Tier | Pro Tier | Enterprise |
|------|-----------|----------|------------|
| **API Access** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Rate Limit** | 50 req/day | 500 req/day | Custom |
| **Download Limit** | 20 models/day | Unlimited | Unlimited |
| **Free Models Only** | ✅ Enforced | ❌ (can download paid) | ❌ |
| **Auth Method** | Token | Token | Token + OAuth2 |
| **Cost** | Free | $14/mo | Custom |

### Step 1: Sign Up for Sketchfab

1. Go to [sketchfab.com/signup](https://sketchfab.com/signup)
2. Create account (email + password or OAuth provider)
3. Verify email address
4. Complete profile setup (optional but recommended)

### Step 2: Verify Account Type

- **Free Tier** (recommended for initial development):
  - Login to https://sketchfab.com/settings/profile
  - Confirm "FREE" badge shows next to your username
  - Free tier access to 50k+ CC-licensed models

- **Pro Tier** (for production pipelines):
  - Upgrade at https://sketchfab.com/pro
  - Recommended if processing >20 models/day
  - Access to higher rate limits and detailed analytics

---

## Part 2: API Token Generation

### Generate Your Personal API Token

1. **Navigate to API Settings**:
   - Login to Sketchfab
   - Go to https://sketchfab.com/settings/api
   - Section: "Personal access tokens"

2. **Create Token**:
   - Click "Generate New Token"
   - **Scope** (select the following):
     - ✅ `Read` (search, metadata, download links)
     - Leave `Write` unchecked (DINOForge only reads)
   - **Token Name**: `dinoforge-intake-pipeline`
   - **Expiration**: Set to 1 year (rotate annually)
   - Click "Generate"

3. **Copy & Secure Token**:
   - Copy the token immediately (shown once)
   - Store in secure location (never commit to git)
   - See **Part 3** for credential management

**Example Token Format**: `8a4c2e9f-1b7d-48c3-9e2a-5f6c8d1a2b3c`

---

## Part 3: Credential Management

### Option A: Environment Variables (.env file)

**File**: `.env` (in repository root, listed in `.gitignore`)

```bash
# Sketchfab API Configuration
SKETCHFAB_API_TOKEN=your_token_here
SKETCHFAB_API_BASE_URL=https://api.sketchfab.com/v3
SKETCHFAB_DOWNLOAD_FORMAT=glb
SKETCHFAB_RATE_LIMIT_PER_HOUR=60

# Asset Pipeline Configuration
ASSET_DOWNLOAD_DIR=packs/warfare-starwars/assets/raw
ASSET_CACHE_DIR=.cache/sketchfab
ASSET_TEMP_DIR=/tmp/asset-downloads

# Logging & Observability
SKETCHFAB_LOG_LEVEL=info
SKETCHFAB_LOG_HTTP_HEADERS=false
```

### Option B: Local Secrets Manager

For Windows (recommended for CI/CD):
```powershell
# Store token in Windows Credential Manager
cmdkey /add:sketchfab.com /user:dinoforge-service /pass:your_token_here

# Retrieve in C# code:
var credential = new NetworkCredential("dinoforge-service", null);
var token = credential.Password;
```

For Linux/macOS:
```bash
# Store in local keyring
secret-tool store --label="Sketchfab DINOForge" sketchfab api_token your_token_here

# Retrieve in C#:
// Use libsecret or similar
```

### Option C: CI/CD Secrets (GitHub Actions / Azure DevOps)

**GitHub Actions** (`.github/workflows/asset-intake.yml`):
```yaml
env:
  SKETCHFAB_API_TOKEN: ${{ secrets.SKETCHFAB_API_TOKEN }}
  SKETCHFAB_DOWNLOAD_FORMAT: glb
```

Add secret via:
- GitHub Repo → Settings → Secrets and variables → Actions → New repository secret
- Name: `SKETCHFAB_API_TOKEN`
- Value: (paste your token)

### .env.example (Template)

Always commit `.env.example` with placeholder values:

```bash
# Copy to .env and fill in your credentials
# NEVER commit .env to version control

# Sketchfab API Token (get from https://sketchfab.com/settings/api)
# Token Format: UUID or alphanumeric string
SKETCHFAB_API_TOKEN=your_personal_access_token_here

# Sketchfab API Configuration
SKETCHFAB_API_BASE_URL=https://api.sketchfab.com/v3
SKETCHFAB_DOWNLOAD_FORMAT=glb
SKETCHFAB_RATE_LIMIT_PER_HOUR=60

# Asset Pipeline Directories
ASSET_DOWNLOAD_DIR=packs/warfare-starwars/assets/raw
ASSET_CACHE_DIR=.cache/sketchfab
ASSET_TEMP_DIR=/tmp/asset-downloads

# Search Filters (default values)
SKETCHFAB_LICENSE_FILTER=cc0,cc-by,cc-by-sa
SKETCHFAB_POLY_MAX=5000
SKETCHFAB_SORT_BY=relevance

# Logging
SKETCHFAB_LOG_LEVEL=info
SKETCHFAB_LOG_HTTP_HEADERS=false
SKETCHFAB_ENABLE_METRICS=true
```

### Security Best Practices

✅ **DO**:
- Use `.env` files (add to `.gitignore`)
- Rotate tokens annually
- Use environment variables in CI/CD
- Set token expiration dates
- Monitor token usage in API settings
- Use read-only scopes

❌ **DON'T**:
- Hardcode tokens in source code
- Commit `.env` files to git
- Share tokens via chat/email
- Reuse tokens across projects
- Leave tokens in test code
- Log tokens to console

---

## Part 4: Rate Limiting & Quotas

### API Rate Limits

| Plan | Requests/Day | Requests/Hour | Burst | Downloads/Day |
|------|--------------|---------------|-------|---------------|
| **Free** | 50 | ~6 | 10/min | 20 |
| **Pro** | 500 | ~60 | 30/min | Unlimited |
| **Enterprise** | Custom | Custom | Custom | Unlimited |

### Rate Limit Headers

Every API response includes:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 42
X-RateLimit-Reset: 1678536000
```

### Recommended Strategy

**For Free Tier** (50 req/day):
- Batch searches: 1 search query per 5 minutes (max 10/day)
- Cache results aggressively (deduplicate searches)
- Download only CC-0 and CC-BY models (validate license)
- Stagger downloads across 24 hours

**For Pro Tier** (500 req/day):
- Parallel searches: up to 5 concurrent requests
- Batch metadata fetches: group 10+ models per batch
- Download limit: 20-30 models/day to stay under quota
- Use HTTP/2 connection pooling for efficiency

### Rate Limit Detection & Retry Logic

The `SketchfabClient` automatically:
1. Detects `429 Too Many Requests` (rate limit exceeded)
2. Reads `Retry-After` header (wait time in seconds)
3. Implements exponential backoff: 1s → 2s → 4s → 8s (max 120s)
4. Logs rate limit events for monitoring
5. Queues requests if approaching limit (proactive throttling)

**Example Retry Flow**:
```
Request 1 → 200 OK (42 remaining)
Request 2 → 200 OK (41 remaining)
...
Request 50 → 429 Rate Limited (reset in 3600s)
         → Wait 3600s
         → Retry at next hour boundary
```

---

## Part 5: OAuth vs Token Auth

### Token Auth (Recommended for DINOForge)

**Best for**: Automated pipelines, service accounts, CI/CD

```http
GET /api/v3/search?query=clone-trooper HTTP/1.1
Host: api.sketchfab.com
Authorization: Bearer YOUR_API_TOKEN
```

**Pros**:
- ✅ Simple, stateless
- ✅ Works for automated scripts
- ✅ No callback server needed
- ✅ Revoke immediately if leaked

**Cons**:
- ❌ Must rotate manually
- ❌ Tied to single account

### OAuth 2.0 (Not Used in DINOForge v1)

**Best for**: Web apps, user delegation, multi-tenancy

**Flow**:
```
1. Redirect user to https://sketchfab.com/oauth/authorize?client_id=...&redirect_uri=...
2. User grants permission
3. Receive code, exchange for access token
4. Use access token for API calls
5. Refresh token when expired
```

**Pros**:
- ✅ User delegates specific permissions
- ✅ Can be used on behalf of others
- ✅ Automatic token refresh

**Cons**:
- ❌ Requires web server (redirect URI)
- ❌ More complex to implement
- ❌ Overkill for service accounts

**Decision**: DINOForge uses **token auth** for simplicity. OAuth support can be added in a future domain plugin if needed (e.g., for user-initiated downloads).

---

## Part 6: API Reference & Example Requests

### Base URL
```
https://api.sketchfab.com/v3
```

### Authentication Header (All Requests)
```
Authorization: Bearer YOUR_API_TOKEN
```

### Example 1: Search Models

```bash
curl -X GET \
  'https://api.sketchfab.com/v3/search?query=clone+trooper&license=cc0%2Ccc-by&max_poly=2000' \
  -H 'Authorization: Bearer YOUR_API_TOKEN'
```

**Query Parameters**:
- `query` (string): Search keywords
- `license` (csv): Filter by license (`cc0`, `cc-by`, `cc-by-sa`, `cc-by-nc`, etc.)
- `max_poly` (int): Max polygon count
- `sort_by` (string): `relevance` | `likeCount` | `viewCount` | `publishedAt`
- `animated` (bool): Filter animated models
- `face_count` (string): `8k`, `4k`, `2k`, `1k`, etc.
- `limit` (int): Results per page (default 20, max 40)
- `cursor` (string): Pagination token

**Response** (200 OK):
```json
{
  "results": [
    {
      "uid": "a1b2c3d4e5f6",
      "name": "Clone Trooper Phase 1",
      "creator": {
        "displayName": "artist_name"
      },
      "license": "cc-by-4.0",
      "vertexCount": 45000,
      "faceCount": 22500,
      "publishedAt": "2023-06-15T10:30:00Z",
      "modelUrl": "https://sketchfab.com/models/a1b2c3d4e5f6",
      "maturityRating": "general"
    }
  ],
  "next": "https://api.sketchfab.com/v3/search?cursor=...",
  "previous": null
}
```

### Example 2: Fetch Model Metadata

```bash
curl -X GET \
  'https://api.sketchfab.com/v3/models/a1b2c3d4e5f6' \
  -H 'Authorization: Bearer YOUR_API_TOKEN'
```

**Response** (200 OK):
```json
{
  "uid": "a1b2c3d4e5f6",
  "name": "Clone Trooper Phase 1",
  "description": "A low-poly Clone Trooper model from Star Wars...",
  "creator": {
    "uid": "user123",
    "displayName": "artist_name",
    "profileUrl": "https://sketchfab.com/artist_name"
  },
  "license": {
    "uid": "cc-by-4.0",
    "label": "Creative Commons Attribution 4.0 International"
  },
  "vertexCount": 45000,
  "faceCount": 22500,
  "publishedAt": "2023-06-15T10:30:00Z",
  "modelUrl": "https://sketchfab.com/models/a1b2c3d4e5f6",
  "downloadUrl": "https://media.sketchfab.com/models/a1b2c3d4e5f6/model.glb",
  "maturityRating": "general",
  "allowComments": true,
  "vertexCountPrecision": 44998
}
```

### Example 3: Download Model (Binary)

```bash
# Fetch metadata to get downloadUrl
MODEL_ID="a1b2c3d4e5f6"
curl -X GET \
  "https://api.sketchfab.com/v3/models/$MODEL_ID/download" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -o "clone_trooper.zip"
```

**Response** (200 OK):
- Binary GLB/ZIP file with textures and materials
- Content-Type: `application/octet-stream`
- Include `Content-Length` header for progress tracking

### Example 4: Handle Rate Limit

```bash
# Request hits rate limit
curl -X GET \
  'https://api.sketchfab.com/v3/search?query=clone' \
  -H 'Authorization: Bearer YOUR_API_TOKEN'
```

**Response** (429 Too Many Requests):
```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 50
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1678625200
Retry-After: 3600
Content-Type: application/json

{
  "detail": "Request was throttled. Expected available in 3600 seconds."
}
```

**Action**: Wait until `X-RateLimit-Reset` (Unix timestamp) before retrying.

### Example 5: Error Handling

**Invalid Token** (401):
```json
{
  "detail": "Authentication credentials were not provided."
}
```

**Model Not Found** (404):
```json
{
  "detail": "Not found."
}
```

**Bad Query** (400):
```json
{
  "detail": "Invalid query parameter: max_poly must be an integer."
}
```

---

## Part 7: Testing & Validation

### Local Testing Without API Token

Use mock HTTP responses for unit tests:

```csharp
// In SketchfabClient.Tests.cs
[Fact]
public async Task SearchModels_ReturnsResults_WhenValidQuery()
{
    var mockHttp = new MockHttpMessageHandler();
    mockHttp.When("GET", "https://api.sketchfab.com/v3/search*")
        .Respond("application/json", ResponseFixtures.SearchResultsMock);

    var client = new SketchfabClient(
        apiToken: "test-token",
        httpClient: new HttpClient(mockHttp));

    var results = await client.SearchModelsAsync("clone trooper");

    Assert.NotEmpty(results);
    Assert.All(results, r => Assert.NotNull(r.Uid));
}
```

### Rate Limit Testing

```csharp
[Fact]
public async Task DownloadModel_RetriesAfterRateLimit()
{
    var mockHttp = new MockHttpMessageHandler();
    var responses = new[]
    {
        new HttpResponseMessage(HttpStatusCode.TooManyRequests)
        {
            Headers = { { "Retry-After", "2" } }
        },
        new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new ByteArrayContent(new byte[] { /* glb bytes */ })
        }
    };

    mockHttp.When("GET", "*").Respond(request => responses[request.RequestUri.ToString().Contains("retry") ? 1 : 0]);

    var client = new SketchfabClient("test-token", new HttpClient(mockHttp));
    var result = await client.DownloadModelAsync("model-id");

    Assert.NotNull(result);
}
```

### Credential Validation

```bash
# Test token validity before running pipeline
assetctl validate-sketchfab-token

# Output:
# ✅ Token valid
# Plan: Free
# Rate Limit: 50 requests/day
# Remaining today: 47
```

---

## Part 8: Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| **401 Unauthorized** | Invalid/missing token | Check `SKETCHFAB_API_TOKEN` in `.env` |
| **429 Rate Limited** | Too many requests | Wait for `Retry-After` header, use backoff |
| **403 Forbidden** | Token lacks `Read` scope | Regenerate token with correct permissions |
| **404 Model Not Found** | Invalid model ID | Verify model ID from Sketchfab search |
| **500 Server Error** | Sketchfab API down | Check status.sketchfab.com, retry later |
| **Timeout** | Network/connectivity issue | Increase timeout, check internet connection |

### Debug Logging

Enable HTTP tracing:

```bash
# In .env
SKETCHFAB_LOG_LEVEL=debug
SKETCHFAB_LOG_HTTP_HEADERS=true
SKETCHFAB_LOG_HTTP_BODY=true

# Then run:
assetctl search-sketchfab "clone" --log-level debug 2>&1 | tee search.log
```

### Rate Limit Monitoring

Check remaining quota:

```bash
assetctl sketchfab-quota --format json

# Output:
# {
#   "remaining": 42,
#   "limit": 50,
#   "reset_at": "2026-03-12T00:00:00Z",
#   "hours_until_reset": 2.5
# }
```

---

## Part 9: Next Steps

### For Agents Implementing C# Client

1. Create `src/Tools/Cli/Assetctl/Sketchfab/SketchfabClient.cs`
   - Methods: `SearchModelsAsync()`, `GetModelMetadataAsync()`, `DownloadModelAsync()`
   - Handle rate limits with exponential backoff
   - Log all HTTP requests/responses

2. Create `src/Tools/Cli/Assetctl/Sketchfab/AssetDownloader.cs`
   - Orchestrate search → filter → download → manifest
   - Support batch operations
   - Generate SHA256 hashes for integrity

3. Add CLI commands to `AssetctlCommand.cs`
   - `search-sketchfab <query>`
   - `download-sketchfab <model-id>`
   - `validate-sketchfab-token`

4. Add integration tests
   - Mock Sketchfab API responses
   - Test rate limit handling
   - Test license filtering

5. Update documentation
   - Add API response examples to this file
   - Document error handling strategies
   - Add troubleshooting section (done)

### For Pipeline Designers

1. Create `manifests/asset-intake/sketchfab-intake-rules.yaml`
   - Define allowed licenses: `cc0`, `cc-by`, `cc-by-sa` only
   - Define excluded keywords: `paid`, `exclusive`, `commercial`
   - Define poly limits per asset type (unit: 5k, vehicle: 15k, prop: 2k)

2. Create intake pipeline workflow
   - Search → Deduplicate → Validate License → Download → Hash → Manifest

3. Set up scheduled jobs
   - Daily batch download (off-peak hours)
   - Weekly license audit
   - Monthly quota monitoring

---

## References

- **Sketchfab API Docs**: https://docs.sketchfab.com/api/index.html
- **API Status**: https://status.sketchfab.com
- **License Guide**: https://docs.sketchfab.com/api/index.html#api-licenses
- **Rate Limiting**: https://docs.sketchfab.com/api/index.html#authentication-rate-limiting
- **Token Management**: https://sketchfab.com/settings/api

---

**Last Updated**: 2026-03-11
**Maintained By**: DINOForge Agents
**Status**: In Progress (C# client implementation pending)
