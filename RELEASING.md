# Releasing DINOForge

This document is the operational contract for releases.

## Release signals

These are the release sources of truth, in order:

1. Git tag: `vX.Y.Z` or `vX.Y.Z-rc.N`
2. `VERSION`
3. `CHANGELOG.md`

`package.json` is the documentation toolchain package manifest. It is not the public product version authority for DINOForge releases.

## Semantic Versioning policy

DINOForge publishes SemVer tags and release assets.

While the project remains in `0.x`, use this stricter interpretation:

- patch: fixes, refactors without public contract change, operational hardening
- minor: new features, new extension points, or breaking public contract changes
- prerelease: `alpha.N`, `beta.N`, or `rc.N`

After `1.0.0`:

- patch: backward-compatible fixes
- minor: backward-compatible features
- major: breaking public contract changes

Breaking changes must always be called out in `CHANGELOG.md`, even during `0.x`.

## Keep a Changelog contract

`CHANGELOG.md` must:

- keep `## [Unreleased]` at the top
- use standard headings:
  - `Added`
  - `Changed`
  - `Deprecated`
  - `Removed`
  - `Fixed`
  - `Security`
- include a released section matching `VERSION`
- move `Unreleased` notes into the released section before tagging

## Release checklist

1. Confirm `VERSION` matches the intended release.
2. Finalize `CHANGELOG.md` for that version.
3. Confirm CI, lint, and coverage are green.
4. Confirm release assets, checksums, and provenance outputs are expected.
5. Tag with `vX.Y.Z` or `vX.Y.Z-rc.N`.
6. Verify GitHub release notes match the finalized changelog intent.

## Rollback expectations

If a release is invalid:

1. stop promotion of the release artifact
2. document the defect in `CHANGELOG.md` and the release notes
3. ship a follow-up patch release instead of mutating history

## NuGet Setup

Before cutting a release, ensure `NUGET_API_KEY` secret is set in GitHub repository settings:

1. Go to repository **Settings → Secrets and variables → Actions**
2. Create a new repository secret named `NUGET_API_KEY`
3. Get your API key from [NuGet.org Account → API Keys](https://www.nuget.org/account/apikeys)
4. Paste the key and save

The `release.yml` workflow uses this secret to publish SDK and Templates packages to nuget.org automatically on tag push.

## Cross-project alignment

The KooshaPari cross-project semantics that should remain consistent with Dino are documented in `docs/reference/kooshapari-project-semantics.md`.
