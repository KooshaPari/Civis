# civ

CIV is a deterministic simulation and policy-driven architecture workspace.

## What This Repository Includes

- Core Rust workspace and crates
- Governance and specification docs
- VitePress documentation site
- CI workflows for quality and docs/pages deployment

## Quick Start

1. Install Rust toolchain and Node.js.
2. Run `cargo test`.
3. Run `task quality`.
4. Build docs with `npm run docs:build`.

## Documentation

- Root specs: `PRD.md`, `ADR.md`, `FUNCTIONAL_REQUIREMENTS.md`, `PLAN.md`, `USER_JOURNEYS.md`
- Reference trackers: `docs/reference/`
- Guides and checklists: `docs/guides/`, `docs/checklists/`

## CI/CD

- `.github/workflows/quality.yml`
- `.github/workflows/docs-site.yml`
- `.github/workflows/pages.yml`

## Governance

This repo follows spec-first, deterministic, fail-loud practices.
