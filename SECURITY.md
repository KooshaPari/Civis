# Security Policy

## Reporting a Vulnerability

Do not open a public issue for a suspected vulnerability.

**Preferred reporting path:**

1. **GitHub Private Security Advisory** — Use GitHub's built-in vulnerability reporting feature if available for the repository
2. **Email Contact** — If GitHub reporting is not available, contact the maintainer privately at `kooshapari@gmail.com` before any disclosure

**Please include in your report:**

- Affected version(s) or tags
- Vulnerability description and impact summary
- Reproduction steps or proof of concept (if applicable)
- Suggested mitigation or fix (if you have one)
- Your contact information for follow-up

**Response timeline:**

- **Acknowledgement** — Within 3 business days
- **Initial Triage** — Within 7 business days
- **Status Update** — After triage if remediation requires additional time
- **Fix Release** — Target within 30 days for high-severity issues, 60 days for medium

**Coordinated disclosure:** Public disclosure should wait until a fix or mitigation is available. We ask for reasonable time to develop and release a patch.

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest stable release | ✅ |
| Current pre-release under active validation | Best effort |
| Older releases | ❌ |

## Security Scanning

### Credential & Secret Detection

**Gitleaks** is configured to prevent accidental commits of secrets:

- API keys (OpenAI, Anthropic, GitHub, Slack, AWS, etc.)
- Private keys and certificates (PEM format)
- JWT tokens and bearer tokens
- Database connection strings
- .NET user secrets

**Run locally before committing:**
```bash
gitleaks detect --config .gitleaks.toml
```

This check is also enforced via pre-commit hooks. If a secret is accidentally detected in history, the repository maintainer will be notified immediately.

### Dependency Scanning

**Dependabot** automatically scans for vulnerable dependencies:

- NuGet packages (C# dependencies)
- GitHub Actions (workflow security)
- Updates are proposed as pull requests
- Critical issues trigger automatic alerts

See `.github/dependabot.yml` for configuration.

### Supply Chain Security

**SBOM (Software Bill of Materials)** is generated on every release via GitHub Actions to track all dependencies and transitive dependencies. See `sbom.spdx.json` in release artifacts.

**GitHub Scorecard** tracks general repository security posture (dependency updates, branch protection, code review, etc.).

## Security Baseline

The repo baseline includes:

- Pinned GitHub Actions where practical (with Dependabot keeping them updated)
- Dependabot for dependency and action updates with automatic PR creation
- Codecov coverage reporting (detects coverage regressions)
- SBOM and Scorecard workflows on every release
- Changelog and release-governance review on all pull requests
- Gitleaks pre-commit hook and CI check (prevents secrets in commits)
- Pre-commit hooks for linting and formatting (catches common issues)
