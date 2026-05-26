# Security Infrastructure

DINOForge uses a layered security approach across CI/CD pipelines, dependency management, and runtime monitoring.

## Active Security Tools

| Tool | Purpose | Frequency | Status |
|------|---------|-----------|--------|
| **Dependabot** | Automated NuGet/GHA updates | Weekly + daily security | ✅ Active |
| **CodeQL** (SAST) | Static analysis for C# code | Every PR | ✅ Active |
| **Socket.dev** | Real-time dependency analysis | Every PR + continuous | ⚠️ Setup required |
| **Sentry** | Runtime error tracking | Runtime (in-game) | ✅ Configured |
| **SonarCloud** | Code quality + coverage gates | Every PR | ⚠️ Setup required |
| **OSV/Scorecard** | Security posture scoring | Daily | ✅ Active |
| **TruffleHog** | Secret detection in commits | Every PR | ✅ Active |
| **NuGet audit** | Vulnerability scanning | Every build | ✅ Active |
| **CycloneDX SBOM** | Software bill of materials | Release | ✅ Active |

## Setup Instructions

### Socket.dev (Required)

1. Install: https://socket.dev/github
2. Grant access to `KooshaPari/Dino`
3. Done — automatic analysis starts immediately

### SonarCloud (Required)

1. Create account at https://sonarcloud.io
2. Create organization: `dinoforge`
3. Import project: `KooshaPari/Dino`
4. Add `SONAR_TOKEN` as GitHub Actions secret (already configured)
5. CI will run SonarCloud analysis on every PR automatically

### Sentry (Runtime Monitoring)

Sentry is configured in the SDK and Runtime. Set `SENTRY_DSN` as an environment variable (or GitHub Actions secret) to enable in-game error tracking.

```bash
# Local development
export SENTRY_DSN="sktsec_MI_QRGrF9UE0xp7dqoIJb4g8XezbPHEagZOo4lUf-ZbQ_api"
```

## GitHub Secrets Required

| Secret | Purpose | Status |
|--------|---------|--------|
| `SONAR_TOKEN` | SonarCloud analysis | ✅ Configured |
| `SENTRY_DSN` | Runtime error tracking | ⚠️ Set locally |
| `SENTRY_AUTH_TOKEN` | Sentry release management | Available |

## Dependabot Security Updates

Dependabot is configured to:
- Update NuGet packages weekly (grouped by Microsoft, testing, Avalonia, Stryker)
- Update GitHub Actions weekly
- Apply security-only updates daily for CVEs

Configure branch protection to require Dependabot PRs for production dependencies.
