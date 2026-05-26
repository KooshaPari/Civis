# KooshaPari Project Semantics

This document captures the release and governance semantics that should stay consistent across KooshaPari projects, plus the semantics that are intentionally Dino-specific.

## Shared KooshaPari semantics

These are the patterns already visible across active KooshaPari repositories such as `Dino`, `cliproxyapi-plusplus`, `thegent`, `portage`, and `helios-cli`.

### 1. Canonical repo governance files

Every active public project should carry:

- `README.md`
- `CHANGELOG.md`
- `SECURITY.md`
- `.github/`
- `docs/`

When a repo is intended for sustained maintenance, also add:

- `.github/CODEOWNERS`
- `CONTRIBUTING.md`
- `RELEASING.md`

For agent-first or autonomously-maintained repositories, also add:

- `AGENTS.md` (agent governance and autonomous workflow guidelines)

### 2. Release semantics

Use one public release language across repos:

- SemVer tags in the form `vX.Y.Z`
- Keep a Changelog with `[Unreleased]`
- a machine-readable version signal when the repo is not purely a language/toolchain implementation (i.e., when it includes application or library code beyond just compiler/interpreter source)
- release artifacts with checksums when binaries or packages are published

### 3. CI and security semantics

Prefer the same formal controls across repos:

- pinned GitHub Actions
- Dependabot
- security policy and supported-version statement
- Codecov or equivalent coverage reporting with repo config
- SBOM and supply-chain attestations where artifacts are distributed
- explicit policy-gate style workflow for governance checks

### 4. Ownership semantics

Review routing should be explicit rather than tribal:

- `.github/CODEOWNERS` for ownership
- pull request template for verification
- release checklist for artifact review

### 5. Agent and automation semantics

KooshaPari projects increasingly assume autonomous or semi-autonomous engineering workflows. Shared semantics should therefore include:

- machine-parseable CLI output where practical
- documented governance for changelog/version/release behavior
- ADRs or equivalent decision records for architectural choices
- narrow policy gates rather than informal review-only enforcement

## Dino-specific semantics

These should stay explicit in Dino even if they do not apply to every KooshaPari repo.

### 1. Schema-first mod contracts

In Dino, schemas are part of the product contract, not just validation helpers.

### 2. Layered Runtime, SDK, Domains, Tools, and Packs split

The Runtime, SDK, Domains, Tools, and Packs separation is central to Dino’s architecture and should remain more explicit than in standard app repos.

### 3. Pack compatibility and framework versioning

Pack manifests, framework compatibility ranges, and runtime bridges are first-class release concerns in Dino.

### 4. Agent-first tool output

Dino’s toolchain should continue to prefer machine-readable output because the repo is intentionally agent-operated.

## What Dino should export back into other KooshaPari repos

Dino has a few strong patterns worth carrying outward:

- ADR discipline for architecture decisions
- explicit release and changelog governance
- schema/contract-centric thinking
- artifact checksum expectations for shipped binaries

## What Dino should import from the wider KooshaPari portfolio

Dino should continue adopting the stronger portfolio-wide governance posture:

- CODEOWNERS everywhere a repo is actively maintained
- standardized release semantics
- consistent coverage reporting policy
- explicit security reporting guidance
