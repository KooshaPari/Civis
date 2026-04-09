# Release Automation Setup Guide

This guide walks through configuring DINOForge for automated NuGet package publishing on releases.

## NUGET_API_KEY Configuration

The release workflow requires a valid NuGet API key to publish packages. Follow these steps:

### 1. Create a NuGet API Key

1. Navigate to https://www.nuget.org (official NuGet package registry)
2. Sign in with your NuGet.org account (create one if needed)
3. Click your profile icon → **API keys**
4. Click **Create** to generate a new key
5. Select **Push** as the key scope (allows publishing packages)
6. Set an expiration (recommended: 1 year for security rotation)
7. Copy the full API key value (you will not be able to view it again)

### 2. Add Secret to GitHub Repository

1. Go to your GitHub repository: https://github.com/KooshaPari/Dino
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `NUGET_API_KEY`
5. Value: Paste the API key you copied from NuGet.org
6. Click **Add secret**

### 3. Verify Configuration

The release workflow (`release.yml`) will automatically:
- Build all packages (SDK, Bridge.Protocol, and Tier 2 domain packages)
- Pack each package with NuGet format
- Push to https://www.nuget.org using the `NUGET_API_KEY` secret
- Publish matching GitHub Release

**Expected result**: On next tag push (e.g., `git tag v0.19.0 && git push --tags`), all packages automatically appear on nuget.org.

## Verifying Publication

### Check NuGet.org for Published Packages

After a release completes:

1. Open https://www.nuget.org/packages/DINOForge.SDK/
2. Look for your new version in the version history
3. Repeat for other published packages:
   - `DINOForge.Bridge.Protocol`
   - `DINOForge.Domains.Warfare`
   - `DINOForge.Domains.Economy`
   - `DINOForge.Domains.Scenario`
   - `DINOForge.Domains.UI`

### Test Package Installation Locally

```bash
# Create a test project
dotnet new console -n TestPkg
cd TestPkg

# Add package from NuGet
dotnet add package DINOForge.SDK --version 0.19.0

# Verify it installed
dotnet restore
```

### View Package Details

```bash
# List local packages
dotnet nuget locals all --list

# Search NuGet from CLI
dotnet package search DINOForge.SDK
```

## Troubleshooting

### NuGet Push Fails with "Invalid API key"

- Verify `NUGET_API_KEY` secret is correctly set in GitHub Settings
- Check that the API key has not expired on nuget.org
- Ensure the API key scope is **Push** (not just Download or Org)
- Regenerate the key on nuget.org and update the GitHub secret

### "Package already exists" Error

- A version can only be pushed once to NuGet.org
- To fix a published package, increment the version and republish
- Old versions remain available for download

### Workflow Job Shows "Skipped"

- Verify you pushed a git tag: `git tag v0.19.0 && git push --tags`
- The release workflow only triggers on version tags matching `v*` pattern
- Check GitHub Actions page to see all workflow runs

### Manual Package Publishing (Fallback)

If automation fails and you need to publish manually:

```bash
# Navigate to project directory
cd src/SDK

# Build and pack
dotnet pack -c Release -o ./nupkg

# Push manually
dotnet nuget push ./nupkg/*.nupkg --api-key $env:NUGET_API_KEY --source https://api.nuget.org/v3/index.json
```

Repeat for each domain package in `src/Domains/Warfare/`, etc.

---

**Last updated**: 2026-04-08  
**Applies to**: v0.18.0+
