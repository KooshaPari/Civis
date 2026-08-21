# Civis regression and recovery audit

Date: 2026-07-22

## Executive finding

The current `main` lineage does not contain the June 1–5 feature commits recovered from GitHub. The commits remain available as Git objects and are anchored locally under `refs/heads/recovery/june-2026/*`. No recovered work has been merged.

## Preserved June anchors

| Local recovery ref | Commit | Recovered feature |
|---|---|---|
| `recovery/june-2026/pbr2-triplanar` | `92fd460d` | smooth mesher, font-path hardening, terrain cull-at-load |
| `recovery/june-2026/theme-fix` | `ea20dbe5` | Keycap UI/theme tokens |
| `recovery/june-2026/map-diagnostics` | `f98959db` | 2D map and minimap diagnostics |
| `recovery/june-2026/terrain-camera` | `416d124b` | terrain material/lighting and Cities: Skylines camera |
| `recovery/june-2026/seed-mix` | `a5faca8a` | seed-mixed FBM world generation |
| `recovery/june-2026/chunk-seam` | `974e7eaa` | APRON 2->3 seam continuity |
| `recovery/june-2026/texture-gating` | `625d98a3` | JPEG textures and voxel test gating |
| `recovery/june-2026/anim-guard` | `b1b91002` | animation graph re-attachment guard |

All eight were verified against the GitHub repository and are not ancestors of the current branch/main lineage.

## Current validation

- `cargo test -p civ-bevy-ref --test shell_attest --features bevy,egui -- --nocapture`: **14 passed, 0 failed**.
- A prior test invocation without required features collected zero tests and was rejected as insufficient evidence.
- `cargo tree --workspace -i rustls-webpki@0.101.7`: no reachable package.
- `cargo tree --workspace -i quick-xml`: no matching package in the current workspace graph.
- Current dependency remediation tip: `62e7680ad` (`fix(infra): drop AWS legacy rustls 0.21 / webpki 0.101`).

The active shell branch is 48 commits ahead of `main` and differs by 61 files (`3,517` insertions and `801` deletions) at the current tip. Representative unique work includes shell controls/launch hardening, camera bindings, HUD/audio polish, crash-path hardening, and the dependency remediation commit.

The June PBR2 anchor was checked at blob level: all four relevant files differ from the current tip (`clients/bevy-ref/src/lib.rs`, `ui_theme.rs`, `voxel_sim.rs`, and `voxel_smooth_mesher.rs`). The recovered commit files therefore cannot be treated as already present merely because similarly named current files exist.

### Follow-up terrain assessment

The current active branch still contains `clients/bevy-ref/src/voxel_smooth_mesher.rs` and `voxel_sim.rs`; the mesher resolves the smooth path and defines `APRON: usize = 3`. The June terrain/mesher work is therefore classified as **evolved/present**, not lost. No restoration patch is warranted for this lane.

## Native smoke status

The release standalone binary is present at `target/release/civ-standalone.exe`. A pre-existing responsive `civ-standalone.exe` instance was observed and not terminated. A uniquely named copy was launched in an isolated job; after 8 seconds the process was alive (`binary_alive_after_8s=yes`) with no captured stderr, then the audit process was stopped. This is a successful bounded native launch/liveness smoke, though it does not certify full GPU rendering or gameplay behavior.

## Working-tree safety

The active checkout contains pre-existing user modifications in Bevy UI/readme/tool-category files and an untracked `settings.ron`. These were preserved. Recovery refs are additive only; `main` and the active branch were not reset or merged.

## Remaining work

1. Build the standalone binary after the shared Cargo lock is released.
2. Run a bounded native launch smoke and capture exit/window initialization evidence.
3. Complete the file-level June-to-current feature matrix across all eight anchors and active branches.

## 2026-07-23 branch divergence checkpoint

The checked-out feature branch is not at the reported `bba2411` tip. It is
`5406e5a6b`, while `origin/main` is `1ab3dd39c`; the branches diverge 75
commits ahead / 4 commits behind from the active branch perspective. Main now
contains a newer voxel-bridge implementation sequence (`8434bed6d` through
`03dd25cec`) and `feat(ai): implement dev Ollama provider` (`1ab3dd39c`).
Those commits overlap the local bridge activation work, so they must be
reviewed as a patch series before any cherry-pick, merge, or rebase.

The bridge and provider series were subsequently recovered as local commits:
`03f4ca6ff`, `208959541`, `e97ecd186`, `c00cf9222`, `d017a2bd5`,
`75092be36`, `4bffa34b1`, `724d9e5ea`, `af3e9eb3b`, and `2b435a6f5`.
Bridge validation passed all four adapter tests. The pre-expansion AI suite
passed 35 unit tests, 10 integration tests, one doc-test, with one known
ignored test. Post-expansion validation is now complete: `cargo test -p
civ-ai --lib` passed 35/35, and `cargo test -p civ-ai` passed 35 unit tests,
10 integration tests, and one doc-test, with one pre-existing ignored test.
The remaining release gate is `cargo deny check advisories`; it has not
returned because concurrent unrelated Cargo builds continue to hold the
shared package-cache state.

## 2026-07-23 resumed recovery checkpoint

The branch now points to `fd1d14008`, which adds the cross-platform
quality-manifest pre-push wrapper recovered from the newer main-line CI
series. The wrapper preserves the existing PowerShell emitter, supports
PowerShell or Bash fallback, and avoids Lefthook's platform-specific command
parsing. `bash -n` and `git diff --check` passed.

Read-only comparison after fetching `origin` shows the named voxel, material
CA, and AI-provider patches are present locally as equivalent recovered
commits. `origin/main` still contains a much broader, unrelated 109-file
delta (engine, Bevy, server, dashboard, workflows, and assets); that delta is
not being merged wholesale. User changes remain uncommitted and untouched.

## 2026-07-28 completion checkpoint

The shared build contention cleared on checkpoint `c9e15ad9e`. The pending
security gate completed successfully: `cargo deny check advisories` reported
`advisories ok`; the configured `RUSTSEC-2026-0204` exception matched no
crate.

Current targeted validation also passed:

- `cargo test -p civ-ai --lib`: 35 passed, 0 failed.
- `cargo test -p civ-voxel-bridge`: 4 test binaries passed, 0 failed;
  doc-tests passed.

No merge, reset, process termination, lock removal, or cleanup was performed.

## 2026-08-05 playable release checkpoint

The release build was produced with the repository's installed `1.96.0`
toolchain, offline, using one Cargo job. The resulting artifact was
`C:\temp\wt-proto-mcp-target\release\civ-standalone.exe` (built
`2026-08-04T21:07:12-07:00`). The repository `target/release` executable was
older and was not used as evidence.

Production smoke command used `CIVIS_SMOKE_FRAMES=60` and an explicit
`CIVIS_ASSET_ROOT` pointing at `clients/bevy-ref/assets`. Results:

- GPU preflight: RTX 3090 Ti / DX12 passed.
- Missing assets: 0.
- Panic lines: 0.
- Bounded smoke exit: 1 (60 frames).
- Process exit code: 0.
- Observed steady sample: 103.99 FPS / 9.63 ms frame time after startup;
  early loading frames were intentionally slower.

This proves the release artifact launches and exits cleanly when given its
asset root. Packaging the asset directory beside the executable remains the
next distribution check.
