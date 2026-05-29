# Branch Consolidation Audit

Read-only inventory of `origin/*` branches against `origin/main`.

Counts:
- ALREADY-MERGED: 3
- STALE: 20
- ACTIVE-UNMERGED: 13
- NEEDS-INVESTIGATION: 1

## ALREADY-MERGED

| Branch | Last commit date | Author | Ahead of `origin/main` | Behind `origin/main` | Fully merged into `main` | Purpose guess |
|---|---:|---|---:|---:|---|---|
| `origin/docs/sonar-pr188-hotspots` | 2026-05-25 | KooshaPari | 0 | 54 | yes | Sonar PR188 hotspot checklist and exclusion guidance |
| `origin/fix/governance-pr188` | 2026-05-25 | KooshaPari | 0 | 54 | yes | Governance cleanup for changelog headers and allowlists |
| `origin/fix/sonar-pr188-blockers` | 2026-05-25 | KooshaPari | 0 | 54 | yes | Sonar workflow/security pin and blocker fixes |

## STALE

| Branch | Last commit date | Author | Ahead of `origin/main` | Behind `origin/main` | Fully merged into `main` | Purpose guess |
|---|---:|---|---:|---:|---|---|
| `origin/backup/20260426-reconcile-05cd0168` | 2026-04-24 | Forge | 8 | 62 | no | Backup/reconciliation snapshot for AGENTS.md harmonization |
| `origin/chore/add-agents-2026-05-02` | 2026-05-02 | KooshaPari | 2 | 56 | no | Repo policy/docs bootstrap work around SECURITY.md |
| `origin/chore/add-gitignore` | 2026-05-02 | Phenotype Agent | 1 | 55 | no | Add or refine `.gitignore` |
| `origin/chore/changelog-stub` | 2026-04-30 | KooshaPari | 1 | 55 | no | Introduce `CHANGELOG.md` stub and Unreleased section |
| `origin/chore/deps-high-sweep` | 2026-04-23 | Forge | 1 | 55 | no | Dependency sweep for high-severity fixes |
| `origin/chore/dino-governance-docs-20260425` | 2026-04-25 | Forge | 1 | 55 | no | Governance/spec documentation for Dino |
| `origin/cursor/gitignore-pattern-refinement-e743` | 2026-04-26 | Cursor Agent | 2 | 56 | no | Refine duplicate `.gitignore` patterns |
| `origin/dependabot/bootstrap` | 2026-04-30 | KooshaPari | 1 | 55 | no | Bootstrap Dependabot configuration |
| `origin/fix/deps-npm-2026-04-27` | 2026-04-27 | Forge | 4 | 58 | no | Fix an external asset intake link; likely superseded |
| `origin/gh-pages` | 2026-03-30 | KooshaPari | 9 | 67 | no | Deployment/docs publishing branch |
| `origin/gt/polecat-35/83fd9412` | 2026-04-24 | Polecat-35 (gastown) | 1 | 59 | no | Gastown methodology/spec artifact |
| `origin/gt/polecat-44/40f140e5` | 2026-04-24 | Polecat-44 (gastown) | 1 | 59 | no | Gastown GEMINI.md methodology guide |
| `origin/pr-template/bootstrap` | 2026-04-30 | KooshaPari | 1 | 59 | no | PR template bootstrap work |
| `origin/safety/iter140-snapshot-2026-05-18` | 2026-05-18 | KooshaPari | 1 | 59 | no | Safety snapshot for iter-140 session state |
| `origin/safety/iter145-recovery-20260523-0432` | 2026-05-26 | KooshaPari | 3 | 61 | no | Recovery branch for gamelaunch attach-mode failures |
| `origin/snyk-fix-7389a3c591b5d9eb5726479c717e9955` | 2026-05-26 | snyk-bot | 1 | 59 | no | Snyk remediation for `scripts/video/package.json` |
| `origin/snyk-fix-80168d908625f6d971bf41969dd61351` | 2026-05-26 | snyk-bot | 1 | 59 | no | Snyk remediation for e2e Python requirements |
| `origin/snyk-fix-9edfd94ce8b34257abbe93fcae9e822c` | 2026-05-26 | snyk-bot | 1 | 59 | no | Snyk remediation for docs package dependencies |
| `origin/stash/recovered-2026-05-19-1` | 2026-05-19 | KooshaPari | 1 | 59 | no | Recovered stash contents for iter-143 WIP |
| `origin/stash/recovered-2026-05-19-2` | 2026-05-19 | KooshaPari | 1 | 59 | no | Recovered stash contents for iter-143 WIP |
| `origin/stash/recovered-2026-05-19-3` | 2026-05-19 | KooshaPari | 1 | 59 | no | Recovered stash contents for iter-143 WIP |

## ACTIVE-UNMERGED

| Branch | Last commit date | Author | Ahead of `origin/main` | Behind `origin/main` | Fully merged into `main` | Purpose guess |
|---|---:|---|---:|---:|---|---|
| `origin/agent/coderabbit-main-config` | 2026-05-26 | KooshaPari | 1 | 55 | no | Main-branch CodeRabbit approval config |
| `origin/ci/pin-trufflehog` | 2026-05-20 | Phenotype Agent | 6 | 60 | no | CI hardening and security tooling pinning |
| `origin/cursor/agent-merge-workflow-issues-8376` | 2026-05-27 | Cursor Agent | 1 | 55 | no | Merge workflow/logging fixes for Actions automation |
| `origin/cursor/bridge-and-security-issues-6930` | 2026-05-27 | Cursor Agent | 1 | 55 | no | Bridge/runtime and security bug fixes |
| `origin/cursor/docs-mermaid-lockfile-a19a` | 2026-05-26 | Cursor Agent | 2 | 56 | no | Docs lockfile refresh for Mermaid dependency |
| `origin/cursor/security-bypass-and-code-duplication-9748` | 2026-05-27 | Cursor Agent | 1 | 55 | no | Security bypass guard and duplication cleanup |
| `origin/dependabot/cargo/src/Tools/AssetPipelineRust/nalgebra-0.35.0` | 2026-05-29 | dependabot[bot] | 1 | 55 | no | Rust asset pipeline dependency bump |
| `origin/dependabot/cargo/src/Tools/AssetPipelineRust/ndarray-0.17.2` | 2026-05-29 | dependabot[bot] | 1 | 55 | no | Rust asset pipeline dependency bump |
| `origin/dependabot/npm_and_yarn/npm_and_yarn-e9ce4f7be9` | 2026-05-29 | dependabot[bot] | 1 | 55 | no | NPM/Yarn dependency bump across one directory |
| `origin/dependabot/npm_and_yarn/playwright-1.60.0` | 2026-05-29 | dependabot[bot] | 1 | 55 | no | Playwright dependency upgrade |
| `origin/dependabot/npm_and_yarn/scripts/video/remotion/cli-4.0.467` | 2026-05-29 | dependabot[bot] | 1 | 55 | no | Remotion CLI dependency bump for video tooling |
| `origin/docs/ci-workflow-bootstrap` | 2026-05-24 | KooshaPari | 1 | 55 | no | CI workflow bootstrap documentation |
| `origin/feat/journey-impl` | 2026-05-01 | Phenotype Agent | 1 | 55 | no | Journey traceability and iconography implementation |

## NEEDS-INVESTIGATION

| Branch | Last commit date | Author | Ahead of `origin/main` | Behind `origin/main` | Fully merged into `main` | Purpose guess |
|---|---:|---|---:|---:|---|---|
| `origin/origin` | 2026-05-28 | dependabot[bot] | 0 | 54 | no | Pseudo-ref or malformed remote ref; not a normal branch |

## Notes

- `Fully merged into main` was derived from `git branch -r --merged origin/main`.
- `Ahead` and `Behind` were computed with `git rev-list --count origin/main..BRANCH` and `git rev-list --count BRANCH..origin/main`.
- `origin/origin` appeared in `git branch -r` output and does not look like a valid working branch name; it is separated out for manual inspection.
