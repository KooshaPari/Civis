# NuGet Publishing Guide

## Overview

DINOForge publishes multiple NuGet packages to nuget.org for community developers. This guide covers prerequisites, publishing workflow, verification, and rollback procedures.

## Prerequisites

### GitHub Actions Setup (One-time)

1. **Obtain NuGet API Key**
   - Go to https://www.nuget.org/account/apikeys
   - Sign in with your NuGet.org account
   - Create a new API Key with scope: Push new packages and package versions
   - Copy the key

2. **Add NUGET_API_KEY to GitHub Secrets**
   - Navigate to GitHub repo: Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `NUGET_API_KEY`
   - Value: Paste your NuGet API Key
   - Save

### Local Requirements

- .NET 8.0 SDK or later
- Git with tag support
- PowerShell 7+ (for dry-run script)

## Publishing Workflow

### Step 1: Prepare Release

Ensure all changes are committed and tests pass:

```bash
dotnet test src/DINOForge.CI.NoRuntime.sln --configuration Release
```

### Step 2: Create Version Tag

Tag follows semantic versioning (e.g., `v0.24.0`):

```bash
# Create tag
git tag v0.24.0

# Push tag to GitHub (triggers release.yml)
git push origin v0.24.0
```

For pre-release versions (alpha, beta, rc):

```bash
git tag v0.24.0-beta.1
git push origin v0.24.0-beta.1
```

### Step 3: Monitor CI/CD

- GitHub Actions automatically triggers the `release.yml` workflow
- Workflow builds, tests, and publishes packages
- Check workflow status: https://github.com/KooshaPari/Dino/actions/workflows/release.yml

## Published Packages

The following packages are automatically published to nuget.org:

| Package | Description | TFM |
|---------|-------------|-----|
| `DINOForge.SDK` | Public mod API & registries | netstandard2.0 |
| `DINOForge.Bridge.Protocol` | JSON-RPC 2.0 message types | netstandard2.0 |
| `DINOForge.Bridge.Client` | GameClient for bridge communication | net8.0 |
| `DINOForge.Tools.Installer` | InstallerLib for setup automation | net8.0 |
| `DINOForge.Templates` | Project templates | (N/A) |
| `DINOForge.Domains.Warfare` | Warfare domain plugin | net8.0 |
| `DINOForge.Domains.Economy` | Economy domain plugin | net8.0 |
| `DINOForge.Domains.Scenario` | Scenario domain plugin | net8.0 |
| `DINOForge.Domains.UI` | UI domain plugin | net8.0 |

## Verification

### Check Package on NuGet.org

After publishing (typically within 5 minutes), verify your package appears:

```powershell
# Via nuget.org web
# https://www.nuget.org/packages/DINOForge.Bridge.Protocol

# Via CLI
dotnet package search DINOForge.Bridge.Protocol --exact-match
```

### Verify Symbol Packages

All publishable packages include symbol packages (`.snupkg`):

- `DINOForge.Bridge.Protocol.0.24.0.nupkg` - Release package
- `DINOForge.Bridge.Protocol.0.24.0.snupkg` - Symbols package

Symbol packages enable debugging via NuGet.org's symbol server:

```powershell
# In Visual Studio, Tools → Options → Debugging → Symbols
# Add: https://symbols.nuget.org/download/symbols
```

### Verify Package Contents

```powershell
# Install the package locally
dotnet add package DINOForge.Bridge.Protocol --version 0.24.0

# Check installed package
dir $env:USERPROFILE\.nuget\packages\dinoforge.bridge.protocol\0.24.0
```

## Rollback Procedure

### If a Package Has Issues

1. **Unlist the Package** (Mark as unlisted, still downloadable)
   - Go to https://www.nuget.org/packages/DINOForge.Bridge.Protocol/0.24.0
   - Sign in
   - Click "Edit" → Unlist
   - Save

2. **Or Push a Patch Release**
   ```bash
   # Fix the issue
   # Bump version in csproj files (e.g., 0.24.0 → 0.24.1)
   # Commit changes
   git add .
   git commit -m "fix: address NuGet package issue"
   git tag v0.24.1
   git push origin v0.24.1
   ```

3. **Contact NuGet.org Support**
   - For security issues or critical problems
   - Email: support@nuget.org
   - Reference package ID and version

## Dry-Run (Local Testing)

Use the provided `scripts/nuget-dry-run.ps1` to test packaging without uploading:

```powershell
./scripts/nuget-dry-run.ps1
```

This script:
- Builds all NuGet packages locally
- Displays package metadata
- Lists generated files and sizes
- Does NOT upload to nuget.org

## Troubleshooting

### Package Push Fails

**Issue**: `403 Forbidden` when pushing to NuGet.org

**Solution**:
- Verify NUGET_API_KEY secret is set in GitHub Actions
- Check API key is not expired (regenerate if needed)
- Ensure API key has "Push new packages" scope

**Issue**: Package not appearing on nuget.org

**Solution**:
- NuGet.org indexing takes 5-15 minutes
- Check GitHub Actions workflow completed successfully
- Review workflow logs for any upload errors

### Symbol Package Not Generated

**Issue**: `.snupkg` file not in release

**Solution**:
- Verify `IncludeSymbols=true` in .csproj
- Verify `SymbolPackageFormat=snupkg` in .csproj
- Re-run workflow with corrected .csproj

## References

- [NuGet.org Publishing Guide](https://docs.microsoft.com/en-us/nuget/nuget-org/publish-a-package)
- [Symbol Packages](https://docs.microsoft.com/en-us/nuget/create-packages/symbol-packages-snupkg)
- [GitHub Secrets Documentation](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [DINOForge release.yml Workflow](.github/workflows/release.yml)

---

**Last Updated**: 2026-04-23
**Maintainer**: KooshaPari
