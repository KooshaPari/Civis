# Publishing DINOForge to PowerShell Gallery

This document describes the process for publishing the DINOForge PowerShell module to the official [PowerShell Gallery](https://www.powershellgallery.com/).

## Overview

Once the DINOForge PowerShell module is published to the gallery, users can install it with a single command:

```powershell
Install-Module -Name DINOForge -Scope CurrentUser
```

This eliminates the need for manual downloads or complex installation scripts.

## Prerequisites

1. **PowerShell Gallery Account**
   - Create account at https://www.powershellgallery.com/users/account/Register
   - Save your username and password

2. **API Key**
   - Generate at https://www.powershellgallery.com/users/account/ApiKeys
   - Create new API key (select "Push" for publish, or use default)
   - Store securely (never commit to git)

3. **PowerShell 5.1+**
   ```powershell
   $PSVersionTable.PSVersion
   ```

4. **PowerShellGet Module**
   ```powershell
   Get-Module PowerShellGet -ListAvailable
   Update-Module PowerShellGet -Force  # If outdated
   ```

## Publication Process

### Step 1: Verify Module Manifest

Ensure `tools/PSModule/DINOForge.psd1` is correct:

```powershell
# Test module manifest
Test-ModuleManifest -Path "tools/PSModule/DINOForge.psd1"

# Output should show no errors
```

Key fields to verify:
- `ModuleVersion` - Increment for each release
- `Author` - DINOForge Project
- `Description` - Clear, concise description
- `ProjectUri` - GitHub repo
- `ReleaseNotes` - Changelog for this version
- `Tags` - Relevant keywords

### Step 2: Version Bump

Update version in `DINOForge.psd1`:

```powershell
# Edit the manifest file
# Change: ModuleVersion = '0.26.0'
# To:     ModuleVersion = '0.27.0'  # or appropriate version
```

Also update `CHANGELOG.md` with release notes:

```markdown
## [0.27.0] - 2026-05-28

### Added
- New feature X
- New feature Y

### Fixed
- Bug fix A
- Bug fix B

### Changed
- Enhancement X
```

### Step 3: Test Local Installation

Before publishing, test installation locally:

```powershell
# Remove existing module
Remove-Module DINOForge -ErrorAction SilentlyContinue
Get-Module DINOForge | Remove-Module

# Test installation from local path
Install-Module -Path "tools/PSModule" -Scope CurrentUser -Force

# Verify all cmdlets are available
Get-Command -Module DINOForge

# Test a cmdlet
Get-DINOForgeHelp
```

### Step 4: Publish to Gallery

```powershell
# Store API key securely (one-time)
$apiKey = Read-Host "Enter PowerShell Gallery API key" -AsSecureString
$apiKey = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto(
    [System.Runtime.InteropServices.Marshal]::SecureStringToCoTaskMemUnicode($apiKey)
)

# Or read from environment variable
$apiKey = $env:PSGALLERY_API_KEY

# Publish module
$publishParams = @{
    Path            = "tools/PSModule"
    NuGetApiKey     = $apiKey
    Repository      = "PSGallery"
    Verbose         = $true
    WhatIf          = $false  # Remove after confirming output
}

Publish-Module @publishParams
```

Expected output:
```
Publishing module 'DINOForge' to repository 'https://www.powershellgallery.com/'...
Successfully published module 'DINOForge' to the PowerShell Gallery.
```

### Step 5: Verify Publication

After publication (may take a few minutes), verify on the gallery:

```powershell
# Search gallery
Find-Module -Name DINOForge

# Install from gallery (in new PowerShell session)
Install-Module -Name DINOForge -Scope CurrentUser

# Verify installation
Get-Command -Module DINOForge
```

Check gallery page: https://www.powershellgallery.com/packages/DINOForge

## Automated Publication (Future)

When ready, implement CI/CD publication via GitHub Actions:

```yaml
name: Publish to PowerShell Gallery

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Publish to PowerShell Gallery
        run: |
          $module = 'tools/PSModule'
          $apiKey = '${{ secrets.PSGALLERY_API_KEY }}'
          Publish-Module -Path $module -NuGetApiKey $apiKey
        shell: pwsh
```

**Setup:**
1. Create secret `PSGALLERY_API_KEY` in GitHub repo settings
2. Commit workflow file to `.github/workflows/publish-powershell-gallery.yml`
3. Trigger publication by creating a GitHub Release

## Update Process

### For Bug Fixes

```powershell
# 1. Fix code in DINOForge.psm1
# 2. Update version (patch): 0.26.0 -> 0.26.1
# 3. Update release notes in psd1
# 4. Test locally
# 5. Publish

Publish-Module -Path "tools/PSModule" -NuGetApiKey $apiKey
```

### For New Features

```powershell
# 1. Add cmdlet to DINOForge.psm1
# 2. Update FunctionsToExport in psd1
# 3. Update version (minor): 0.26.0 -> 0.27.0
# 4. Update release notes in psd1
# 5. Test locally
# 6. Publish

Publish-Module -Path "tools/PSModule" -NuGetApiKey $apiKey
```

### For Breaking Changes

```powershell
# 1. Document breaking changes in release notes
# 2. Update version (major): 0.26.0 -> 1.0.0
# 3. Test thoroughly
# 4. Publish with clear messaging
```

## User Installation Instructions

Once published, users can install via:

```powershell
# Install for current user only
Install-Module -Name DINOForge -Scope CurrentUser

# Install for all users (requires admin)
Install-Module -Name DINOForge -Scope AllUsers

# Install specific version
Install-Module -Name DINOForge -RequiredVersion 0.27.0 -Scope CurrentUser

# Update to latest
Update-Module -Name DINOForge

# Uninstall
Uninstall-Module -Name DINOForge
```

## Troubleshooting

### Module not found on gallery

- Wait 10-15 minutes after publication
- Clear cache: `Find-Module -Name DINOForge -Repository PSGallery -Force`
- Check publication status on gallery website

### Publication fails with "API key invalid"

```powershell
# Verify API key
Test-Path $env:PSGALLERY_API_KEY
$apiKey = $env:PSGALLERY_API_KEY
$apiKey.Length  # Should be non-zero

# Regenerate key at https://www.powershellgallery.com/users/account/ApiKeys
```

### Module manifest errors

```powershell
# Test manifest
Test-ModuleManifest "tools/PSModule/DINOForge.psd1" -Verbose

# Common issues:
# - FunctionsToExport lists non-existent functions
# - RequiredModules not available
# - Invalid version format (must be X.Y.Z)
```

### Unintended publication

Currently no way to delete published versions. Solutions:

1. **Create new version** with corrections and publish newer version
2. **Mark as "deprecated"** in release notes
3. **Request removal** by contacting PowerShell Gallery maintainers

Always test thoroughly before publishing!

## Best Practices

1. **Semantic Versioning**
   - MAJOR.MINOR.PATCH (e.g., 1.2.3)
   - MAJOR: Breaking changes
   - MINOR: New features (backward compatible)
   - PATCH: Bug fixes

2. **Release Notes**
   - Clear, user-friendly language
   - Highlight new features and fixes
   - Note any breaking changes

3. **Testing**
   - Test local installation before publishing
   - Test all cmdlets in fresh PowerShell session
   - Test on both Windows PowerShell 5.1 and PowerShell 7+

4. **Documentation**
   - Keep README.md updated
   - Document all parameters
   - Provide usage examples

5. **Versioning**
   - Increment version BEFORE publishing
   - Never re-publish same version
   - Keep CHANGELOG.md in sync

## References

- [PowerShell Gallery](https://www.powershellgallery.com/)
- [Publish-Module Documentation](https://learn.microsoft.com/en-us/powershell/module/powershellget/publish-module)
- [PowerShell Module Manifest](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/new-modulemanifest)
- [Semantic Versioning](https://semver.org/)

## Support

For issues with PowerShell Gallery:
- Gallery Issues: https://github.com/PowerShell/PowerShellGallery/issues
- DINOForge Issues: https://github.com/KooshaPari/Dino/issues
