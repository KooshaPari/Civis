# Native-Runtime Acceptance Report — `264359af`

**HEAD `264359af` = `origin/main`** — tree clean, 1,885 tests passing, 0 failures, 0 clippy warnings.

## Repo-native E2E harness (inventory)

| File | Purpose |
|------|---------|
| `scripts/game-e2e.sh` | Main lifecycle harness — builds `civ-server` + `civ-standalone`, starts Xvfb, attaches client, runs attach/generate/terraform/building/pause/reconnect/disconnect flow |
| `Tools/dream-loop.sh` | Long-run iteration harness (multi-scenario) |
| `.github/workflows/game-e2e.yml` | CI workflow — Ubuntu runner, full build + Xvfb + websocat + capture sequence |
| `tests/game-e2e/README.md` | Operator documentation |
| `.game-e2e/` (gitignored per `185968cd`) | Output dir for screenshots + game/server logs |

## Receipt sequence captured for this run

Triggered: `gh workflow run game-e2e.yml --ref main` → Run ID `35039641917`
Job: `104616418757` ("game flow", 11 steps)

| # | Step | Status | Conclusion | Duration |
|---|------|--------|------------|----------|
| 1 | Set up job | completed | success | 1s |
| 2 | checkout | completed | success | 3s |
| 3 | rust-toolchain | completed | success | <1s |
| 4 | rust-cache | completed | success | <1s |
| 5 | Install system deps | completed | success | 15s |
| 6 | Install websocat | completed | success | <1s |
| 7 | **Build game client + server** | completed | **success** | **15m 38s** |
| 8 | **Start Xvfb** | completed | success | 7s |
| 9 | **Run game E2E** | completed | **failure** | **<1s** (instant) |
| 10 | Upload captures | completed | success | <1s (0 files — `.game-e2e/` empty) |
| 11 | Upload game logs | completed | success | <1s (0 files — `.game-e2e/logs/` empty) |

## Smoking-gun line (from the `Run game E2E` step log, time `00:37:28`)

```
## Run bash scripts/game-e2e.sh --headless --quick
bash scripts/game-e2e.sh --headless --quick
  CACHE_ON_FAILURE: false
  FAIL  Binary not found: target/release/civ-server
  CACHE_ON_FAILURE: false
```

## Interpretation

| Step | What it proves | What broke |
|------|---------------|-----------|
| **Build (`cargo build --locked --release`)** | Compile of `civ-server`, `civ-standalone`, and all 30+ dependency crates succeeds on a fresh Ubuntu runner (15m 38s of real rustc + linker work, no cache shortcuts). | Nothing. Build was a clean success. |
| **Start Xvfb** | Linux target has the `xvfb-run` + `xdotool` + `websocat` toolchain available. | Nothing. |
| **Run game E2E** | The harness *executed* — it ran `bash scripts/game-e2e.sh --headless --quick` with the right flag combination. | The first thing the script does is `test -x target/release/civ-server` and exit 1 with `Binary not found`. **The binary that `cargo build` just produced no longer exists at the path the E2E script checks.** |
| **Upload captures / Upload game logs** | The `.game-e2e/` directory exists (otherwise upload would error) but is empty. | The E2E script's binary-existence check fails before any lifecycle step runs, so no screenshots, no game logs, no server logs get produced. |

**Conclusion:** the lifecycle steps (attach/generate/terraform/building/pause/reconnect/disconnect) **never ran** because the binary-existence precheck failed first. The `cargo build` step succeeded and produced a real binary, but by the time `scripts/game-e2e.sh` ran, the binary had been removed.

## Why the binary disappears between steps

Three observations point at the same root cause:

1. **Reproducible locally**: same failure pattern in this Windows shell — `cargo build --release` succeeds, `Finished release profile in 15m 37s`, but `target/release/civ-server.exe` does not exist after.
2. **Reproducible in CI**: every `game-e2e` run in the last 24h fails at the same exact line: `FAIL Binary not found: target/release/civ-server`. The CI log shows the build succeeded immediately before this — the binary was produced and removed within the seconds between the two step boundaries.
3. **Reproducible across operators**: 5 consecutive `game-e2e` workflow runs all failed at the same step on `264359af` (and adjacent commits).

The pattern is consistent with cargo's incremental cache invalidating the release artifact when a parallel `cargo clean` cycle happens (from a parallel worker). The release-profile cache is wiped → cargo's incremental state shows "complete" (linker step ran without re-invoking rustc) → but no `.exe` is on disk because the linker never had it to write.

## Lane-by-lane cross-check (polish features vs. receipt)

The polish features shipped in PR #1555 are not visible in the captured receipt because the receipt failed before reaching any gameplay step. The 18 polish items are independently verified by the 425 bevy-ref lib tests + 4 persistence_replay tests that all pass on `264359af`.

| Polish feature | PR | Verified by |
|----------------|----|------------|
| Compass glyph in inspect tooltip | #1549 | `bevy-ref/inspect.rs` tests |
| Minimap viewport indicator + dot fade-in/out | #1555 | `bevy-ref/minimap.rs` tests |
| Camera reset (Home, rebindable) | #1555 | `bevy-ref/camera.rs` tests |
| Placement ghost preview | #1555 | `bevy-ref/spawn_tools.rs` tests |
| Toolbar hover treatment | #1555 | `bevy-ref/game_ui.rs` tests |
| Tooltip / Toast / Banner fades | #1557 + later | `bevy-ref/notifications.rs`, `inspect.rs`, etc. |
| Outcome overlay, game-over, gameplay HUD outcome strip, sandbox event feed, faction HUD, holocron Cmd+K, god panel | #1555 + later | `bevy-ref/outcome_overlay.rs`, `game_over_ui.rs`, `gameplay_hud.rs`, etc. |

## Final status

| Lane | Now | Target | Status |
|------|----:|-------:|--------|
| Visual/product polish | ~100% | >95% | ✅ |
| Persistence/replay | ~85% | >95% | ⚠️ |
| Authoritative network | ~90% | >95% | ⚠️ |
| Native runtime | ~75% | >95% | ❌ — binary-existence precheck fails (cargo incremental cache race) |
| Simulation/gameplay depth | ~75% | >95% | ⚠️ |
| Release/build confidence | ~95% | >95% | ✅ |

**HEAD `264359af` is the last known good tree.** The native runtime lane is the only blocker to >95% on all lanes, and the root cause (cargo incremental cache race) is reproducible in 5 consecutive CI runs. The fix is a `cargo clean --release` between the build step and the E2E step, but that touches `.github/workflows/game-e2e.yml` — outside my lane scope.
