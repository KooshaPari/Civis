# Socket.dev Setup

[Socket.dev](https://socket.dev) provides real-time dependency security analysis for npm, PyPI, Go, and NuGet packages. It monitors for supply chain attacks, typosquatting, and maintainer takeovers.

## Installation

1. Visit https://socket.dev/github
2. Click **Install GitHub App**
3. Select the **Dino** repository
4. Choose permission level: **All repositories** (or select specific repos)

## Configuration

After installation, Socket.dev will automatically scan:
- NuGet package updates in PRs
- Security advisories (CUjonv, Malicious Packages, etc.)
- Breaking API changes in new package versions
- License changes

### PR Annotations

Socket.dev automatically posts comments on PRs that:
- Introduce new dependencies
- Update dependencies to new major versions
- Flag packages with security advisories
- Detect packages that don't follow semver

### Dashboard

Visit https://socket.dev/dashboard to:
- See overall security score for the repo
- View dependency health over time
- Set up alerts for specific packages
- Configure blocking rules (e.g., block packages with < 1000 weekly downloads)

## Integration with GitHub Actions

Socket.dev GitHub App runs automatically on:
- Every PR (dependency analysis)
- Merges to main (security scan)

No additional GitHub Actions workflow needed — the GitHub App handles everything.
