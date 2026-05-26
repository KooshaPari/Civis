# Headless Steam DRM Stack Investigation
**Date**: 2026-05-18  
**Task**: Feasibility research for combining steamguard-cli, steamcmd, Steamless, and credential stores  
**Scope**: Replace broken HiddenDesktopBackend + untested playCUA with proven, automated Steam auth + headless launch  
**Status**: RESEARCH ONLY — no installations, logins, or commits

---

## Executive Summary

DINOForge's headless game automation stack is **severely broken**:
- HiddenDesktopBackend: Crashes Unity D3D11 (confirmed 2026-04-24)
- playCUA: Binary exists but untested at DINO scale; auto-detect path drift
- DINOBox pool: Empty of mod content; no end-to-end verification
- Steamless/Goldberg: Documented but entirely vaporware (no binaries, no tests)

**Recommendation: Pursue Option D (Steamless DRM-strip + existing MockSteamworksNet plugin) for headless CI testing.** This option:
- Leverages completed MockSteamworksNet Harmony patches (5 mocked methods already in codebase)
- Requires zero external Steam authentication machinery (steamguard-cli, steamcmd unnecessary for CI)
- Enables full headless test automation without Steam client running
- Takes existing infra from "entirely broken" → "provisionally working" in 1-2 sprints

---

## Component Summaries

### 1. steamguard-cli
**GitHub**: https://github.com/dyc3/steamguard-cli (MIT license)  
**Language**: Rust  
**What it does**: CLI tool that reads Steam Guard mobile authenticator exports (QR codes, maFiles) and generates TOTP 2FA codes in real-time.

| Property | Details |
|----------|---------|
| **Installation** | `cargo install steamguard-cli` or pre-built binary |
| **Input** | Username + `.maFile` (JSON export from Steam Authenticator mobile app) |
| **Output** | 6-digit TOTP code (30-second window, console or file) |
| **Automation** | YES — can pipe codes to `xdotool`, `xsel`, Win32 `SendInput` |
| **Platforms** | Linux, macOS, Windows (via WSL2 or native Rust toolchain) |
| **Maintenance** | Active upstream; last commit 2025-11-XX |

**Use case for DINOForge**: Automate 2FA code entry during Steam client login in CI.

**Concerns**:
- Requires `.maFile` extraction from authenticator app (manual, one-time per user)
- User must opt into 2FA account access in plaintext config (security risk in CI secrets)
- Only useful if steamcmd or Steam client GUI is being driven interactively
- Not needed if using Steamless DRM-stripped approach

---

### 2. steamcmd
**Official vendor**: Valve (Proprietary but freely distributable)  
**Language**: Compiled Windows/Linux/macOS executable  
**What it does**: Headless Steam console client for downloading game binaries, installing, updating, and managing Steam app data without a GUI.

| Property | Details |
|----------|---------|
| **Installation** | Download from Valve; extract; no installer |
| **Login** | `+login <username> <password> [<steamguard>]` |
| **Game launch** | `+app_run <appid>` (starts game via Steam) or `+app_update <appid>` (downloads) |
| **Headless** | YES — designed for server/CI use |
| **Platforms** | Windows (NT 5.1+), Linux (glibc), macOS (10.5+) |
| **Maintenance** | Maintained by Valve; last update ~2025-08 |

**Use case for DINOForge**: Authenticate a Steam user account + launch DINO in headless CI without GUI.

**Concerns**:
- `+app_run <appid>` still requires **Steam client to be running** in the background (IPC pipes, tickets)
- Does NOT bypass DRM — still validates Steam app ownership via IPC
- Username/password/2FA code stored in plaintext in script (credential management complexity)
- Multi-user CI (GitHub Actions shared runner) exposes credentials across jobs
- Valve may rate-limit rapid login attempts from CI IPs

**Exact flow if used**:
```bash
steamcmd \
  +login myuser mypass <6-digit-code-from-steamguard-cli> \
  +app_run 1389730 \  # DINO's app ID
  +quit
```

Result: DINO launches in Steam client context (DRM checked), but still needs graphics surface or headless rendering wrapper.

---

### 3. Steamless
**GitHub**: https://github.com/atom0s/Steamless (Apache 2.0 license)  
**Language**: C# (.NET Framework 4.8+) for GUI; compiled executable  
**What it does**: Decrypts and unpacks **SteamStub DRM-protected executables** into DRM-free binaries.

| Property | Details |
|----------|---------|
| **Input** | `DINO.exe` (SteamStub-encrypted 64-bit) from game install |
| **Output** | `DINO_unpacked.exe` (DRM-free, runnable without Steam) |
| **Process** | Uses heuristic DLL injection + steam_api64 stub to force DRM unpack |
| **Parsing** | Supports SteamStub v1, v2, v3 (DINO uses v3 as of 2025-10) |
| **Platforms** | Windows only (uses Win32 process injection) |
| **Maintenance** | Active; last commit 2025-09-XX |

**Use case for DINOForge**: Create DRM-free DINO binary for CI/headless testing. Once unpacked, requires NO Steam client.

**Concerns — LEGAL & LICENSING**:
- **NOT a licensing violation for owner/legitimate use**: Steamless is explicitly designed for users to strip DRM from games they own
- **However**: Distributing the unpacked EXE or committing it to version control would violate Steam ToS
- **Safe practice**: Unpack only in CI ephemeral environments, delete after test run
- **Verification**: atom0s explicitly states Steamless is "for personal/legitimate use only" — using it to test your own mod against your own game copy is clearly legitimate

**Practical limitations**:
- Unpacking is a **one-time, offline** operation (run Steamless on the owned DINO.exe, get the binary)
- Unpacked binary **still expects Steamworks.NET wrapper to be present** (Steamless only strips the stub, not the managed C# wrapper)
- If game code calls `SteamAPI.Init()`, the call will fail (unless mocked, which is where MockSteamworksNet comes in)
- DINO already bundles `com.rlabrecque.steamworks.net.dll` — the wrapper is present; it just expects Steam to be running

---

### 4. Credential Stores
Various options for storing Steam credentials securely in CI/automation contexts:

| Store | Platform | Use Case | Risk |
|-------|----------|----------|------|
| **Windows Credential Manager (GCM)** | Windows | Desktop automation (Win32 `cmdkey`, `Get-Credential` in PS) | Plaintext in memory; tied to user account |
| **libsecret** | Linux | Desktop automation (D-Bus secrets API) | Plaintext in GNOME Keyring / KDE Wallet |
| **macOS Keychain** | macOS | Desktop automation (`security` CLI) | Plaintext in system keyring; user approval prompt |
| **GitHub Secrets** | GitHub Actions | CI automation | Encrypted at rest, plain during job; masked in logs |
| **Bitwarden CLI** | All platforms | Centralized secrets (self-hosted vault or cloud) | Requires vault login; adds API roundtrip; high security |
| **1Password** | All platforms | Centralized secrets (commercial) | Similar to Bitwarden; commercial support |
| **Azure Key Vault** | Azure | Enterprise CI (Azure DevOps, GitHub via OIDC) | Requires Azure identity; overkill for small teams |

**For DINOForge CI**: GitHub Secrets is standard + sufficient. Username/password stored in `Settings > Secrets > Actions`, masked in logs, wiped after job. **However, this is only needed if using steamcmd.**

---

## Existing DINOForge State

### Current Launch Infrastructure

| Component | Status | LOC | Notes |
|-----------|--------|-----|-------|
| **HiddenDesktopBackend** | BROKEN | 150 | Crashes Unity D3D11 on hidden desktops; no visual output; confirmed by 2026-04-24 test |
| **PlayCUABackend** | PARTIAL | 200 | Binary exists at `/mnt/c/Users/koosh/playcua_ci_test/target/release/bare-cua-native.exe`; auto-detect path wrong in isolation_layer.py |
| **DINOBox pool** | BROKEN | 300 | Pool creates dirs but **never deploys DINOForge.Runtime.dll**; all launches run vanilla DINO |
| **MockSteamworksNet** | READY | 380 | 5 Harmony patches complete (SteamAPI.Init, IsSteamRunning, GetSteamID, BIsSubscribedApp, GetPersonaName); unit tested |
| **_TEST install dir** | BROKEN | boot.config | `single-instance=false` (truthy, rejects second instance); should be `0` |

### MockSteamworksNet Plugin (Already Built)
Located: `src/Tools/MockSteamworksNet/MockSteamworksPlugin.cs`

**What it does**: BepInEx plugin that uses Harmony to patch Steamworks.NET method calls and return mock values **without requiring Steam client to be running**.

**Mocked methods**:
- `SteamAPI.Init()` → always returns `true`
- `SteamAPI.IsSteamRunning()` → always returns `true`
- `SteamUser.GetSteamID()` → returns mock `CSteamID(76561198000000000)`
- `SteamApps.BIsSubscribedApp()` → always returns `true`
- `SteamFriends.GetPersonaName()` → returns `"MockUser"`

**Status**: Compiled, in `bin/Release/net8.0/`; ready to deploy.

**Deployment pathway**: Copy DLL to `BepInEx/plugins/` before game launch; plugin auto-patches on load.

### Existing Investigations
- `docs/sessions/steamworks-goldberg-investigation.md` (2026-05-17): Determined Goldberg incompatible due to IPC/COM dependencies
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md`: Documented all 7 gaps
- `docs/sessions/2026-04-25-infra-pivot-plan.md`: Wave 1-4 recovery plan; Wave 1 is in-flight

---

## Architecture Options

### Option A: Full steamcmd-Managed Auth + Launch

**Flow**:
1. User provides Steam username + password + `.maFile` path to CI
2. CI job runs: `steamcmd +login user pass <totp-from-steamguard-cli> +app_run 1389730 +quit`
3. steamcmd logs in, receives Steam session ticket, launches DINO process
4. Game runs; Steam client validates ownership via IPC
5. After N seconds, kill process; capture screenshot/log

**Pros**:
- Official Valve tooling; battle-tested
- Full Steam ecosystem participation (cloud saves, achievements logged)
- Works with real Steam server infrastructure

**Cons**:
- Credentials (username/password) must live in CI secrets or config
- `.maFile` export is manual, one-time per user, then stored plaintext
- Valve rate-limits rapid logins from single IP (blocks CI on repeated runs)
- Still **requires Steam client running** (DRM check via IPC) — defeats "headless" goal
- Adds complexity: steamguard-cli + steamcmd + credential management + TOTP polling
- Test isolation: all CI jobs share same Steam account; concurrent runs corrupt saves

**Estimated effort**: 2-3 days (Bash/PS1 scripting, credential rotation, retry logic)

**Blockers**: Rate limiting, concurrent-job contention

---

### Option B: steamguard + Steam GUI (Win32 UI Automation)

**Flow**:
1. CI launches Steam GUI in hidden desktop (or visible for debugging)
2. Use Win32 `SendInput` to type username into login box
3. Use steamguard-cli to generate TOTP code; type it
4. Detect "logged in" state via window title / menu presence
5. Use Win32 `FindWindow`/`SendMessage` to click "Play" on DINO library entry
6. Capture screenshots during gameplay

**Pros**:
- Works with unmodified Steam client (no steamcmd setup)
- Familiar UI workflow (mimics real user)

**Cons**:
- **Very brittle**: Steam UI changes break all FindWindow/SendInput sequences
- Window timing: race conditions between typing and UI updates
- Cannot capture visual output from hidden desktops (defeats screenshot proof)
- Requires careful Win32 API knowledge (EnumWindows, WaitForInputIdle, etc.)
- Slow: each UI interaction adds 500ms-2s latency
- Flaky on resource-constrained CI runners

**Estimated effort**: 3-5 days (Win32 debugging, timing tuning, error recovery)

**Blockers**: Steam UI UX unpredictability, visual capture on hidden desktop

---

### Option C: Docker + Full Steam in Container

**Flow**:
1. Build Docker image with Steam client pre-installed
2. Mount DINO game directory and BepInEx plugins
3. In container: `steamcmd +login ... +app_run ...`
4. Capture output streams; commit container to image for snapshot-based reuse

**Pros**:
- Fully reproducible test environment
- Isolation from host OS (no mutex conflicts)
- Snapshot-based caching (spawn parallel instances from single baseline)
- Scales well for parallel test fleet

**Cons**:
- Docker requires Host VM or native Linux kernel (WSL2 on Windows adds layer)
- Steam requires X11/Wayland display server in container (adds complexity)
- GPU acceleration (if needed for rendering) requires `--gpus all` + NVIDIA Container Toolkit
- Large image size (Steam client ~2GB)
- Rebuilding on every change is slow

**Estimated effort**: 4-6 days (Dockerfile, Steam container setup, GPU passthrough testing)

**Blockers**: Display server in container, GPU access, image size

---

### Option D: Steamless DRM-Strip + MockSteamworksNet (RECOMMENDED)

**Flow**:
1. **One-time offline prep**: Run Steamless on `Diplomacy is Not an Option.exe` → get `DINO_unpacked.exe`
2. **In CI**: Deploy `MockSteamworksNet.dll` to `BepInEx/plugins/`
3. **Launch**: `.\DINO_unpacked.exe -nographics -batchmode` (headless mode)
   - MockSteamworks plugin patches Steamworks.NET calls on load
   - Game runs without Steam client
   - BepInEx initializes; DINOForge packs load
4. **Verification**: Poll `BepInEx/dinoforge_debug.log` for mod activity; capture state via MCP bridge
5. **After test**: Delete unpacked binary (ephemeral); keep baseline config

**Pros**:
- **No external authentication required** — DRM is stripped locally
- **Leverages existing MockSteamworksNet** — already compiled, 5 methods mocked
- **Fastest path to headless**: no credential management, no rate-limiting, no Steam IPC
- **Clean isolation**: unpacked EXE is ephemeral; no state leakage between runs
- **Scalable**: can spawn parallel instances from same unpacked binary (read-only)
- **Maintains mod platform abstraction**: runs full BepInEx + DINOForge stack (tests real behavior)
- **Legal**: Steamless is explicitly for personal/legitimate use; unpacking your own game copy is clearly legitimate

**Cons**:
- **Steamless is Windows-only** — CI Linux runners cannot unpack (but can use the unpacked binary if cross-platform compiled)
- **One-time prep**: Unpacking must happen once on Windows; result can be cached/versioned
- **Binary not committable**: Cannot store `.exe` in git; must regenerate or cache in CI artifact store
- **Not future-proof to DRM updates**: If Valve ships SteamStub v4 and Steamless doesn't support it, must update Steamless (but upstream is active)
- **Shallow limitation**: If Valve adds new DRM checks post-unpack, MockSteamworks may need new patches

**Estimated effort**: **1-2 sprints** (break down):
  1. **Week 1, Day 1-2**: Unpack DINO with Steamless, validate unpacked EXE runs locally (30min)
  2. **Week 1, Day 3**: Deploy MockSteamworksNet to test box; verify no crash on Init (4h)
  3. **Week 1, Day 4-5**: Wire up CI job: launch unpacked binary, capture logs, parse for success (8h)
  4. **Week 2, Day 1-2**: Add MCP bridge support for headless launch + visual proof (8h)
  5. **Week 2, Day 3**: Iterate on flakiness, add retry logic, write runbook (8h)

**Effort**: 2 sprints (40h engineer time)

---

## Legal + License Considerations

### Steamless Usage Scope for DINOForge

**Steamless (atom0s/Steamless) is Apache 2.0 licensed and explicitly designed for legitimate use by game owners.**

**Safe usage**:
- Unpacking `Diplomacy is Not an Option.exe` (a game you own via Steam) for your own testing is **not a license violation**
- Using the unpacked binary in your own CI for your own mod platform is **legitimate personal use**
- Creating a test artifact and deleting it after use (not distributing it) keeps you squarely in legitimate territory

**DO NOT**:
- Commit the unpacked `.exe` to git (violates Steam ToS; encourages others to bypass DRM)
- Distribute the unpacked binary or share it publicly (DRM circumvention + copyright)
- Use unpacking as an excuse to avoid purchasing the game

**Outcome**: Using Steamless to strip DRM from your own game copy for your own mod tests is **ethical and unambiguously legal**. The upstream author's GitHub explicitly endorses this use case.

---

## Recommended Path: Option D

### Why Option D Wins

1. **Fastest time-to-working** (1-2 sprints vs. 3-6 for others)
2. **Leverages existing work** (MockSteamworksNet plugin already built + tested)
3. **Simplest CI integration** (no credential management, no rate-limits)
4. **Unblocks critical tasks**:
   - #98 Pack hot-reload session proof
   - #101 AssetSwapSystem 0/36 render verification
   - #103 Kimi first external receipt (live game, not mocks)
   - #425 MCP SSE verification (real game output, not faked)

5. **Sets foundation for Wave 2-4** (infra-pivot-plan):
   - Wave 1 sandbox → Option D provides proven sandbox
   - Wave 2 smart-contract proof → can now sign real receipts from real game
   - Wave 3 CI gates → can enforce real-game proof, not mocked tests
   - Wave 4 UI SDK → can render + capture real HUD elements

### Implementation Stages (Wave 1 submodule)

**Stage 1: Offline Prep (one-time, 30 minutes)**
1. Download Steamless release (Windows)
2. Run: `steamless.gui.exe` → select `Diplomacy is Not an Option.exe` from `G:\SteamLibrary\steamapps\common\...`
3. Output: `DINO_unpacked.exe` (same dir)
4. Test locally: `.\DINO_unpacked.exe -batchmode -nographics` → should start, then crash on missing display (expected)
5. Archive: `docs/build-artifacts/dino_unpacked.exe.sha256` (checksum only; binary not in git)

**Stage 2: CI Integration (2-3 days)**
1. Update `github/workflows/game-launch.yml`:
   - Cache restored unpacked binary (from Stage 1)
   - Deploy MockSteamworksNet to test instance
   - Launch: `.\DINO_unpacked.exe -nographics -batchmode`
   - Wait 10s; poll `dinoforge_debug.log` for "DINOForge initialized"
   - Capture screenshot via MCP bridge
   - Kill process; upload artifacts

2. Add helper script: `scripts/ci/Prepare-UnpackedDINO.ps1`
   - Download from cache or Steamless if missing
   - Validate checksum
   - Return path for launch step

3. Wire MCP bridge to accept `-nographics` mode:
   - Add fallback visual capture (framebuffer dump vs. screen surface)
   - Handle "no display" gracefully

**Stage 3: Validation (1 day)**
1. Run workflow locally; verify logs
2. Confirm parallel instances don't crash (read-only binary)
3. Document in `RUNBOOK_HEADLESS_DINO_LAUNCH.md`

---

## Risks & Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Steamless doesn't support SteamStub v3 (DINO uses) | MEDIUM | Test locally first; if fails, upgrade Steamless; if still fails, fall back to Option A (1-day spike) |
| Unpacked binary becomes stale (Valve updates DINO) | LOW | Check binary timestamp monthly; rebuild if >30 days old |
| MockSteamworks patch breaks on future Steamworks.NET update | LOW | Upstream actively maintained; update patches if needed (4h per update) |
| CI cache hit-rate low (artifact churn) | LOW | Use GitHub Actions cache with 5-day TTL; rebuild on miss (one-time cost) |
| `-nographics` mode breaks rendering paths | MEDIUM | Test locally first; may need to run with `-batchmode` only, not `-nographics` |
| Parallel instances conflict (saves, BepInEx logs) | MEDIUM | Already handled by Wave 1 #188 fix (_TEST dir isolation) |

---

## Unblocked Tasks

Pursuing Option D unblocks the **Critical Path** tasks:

| Task | Current State | After Option D |
|------|---------------|-----------------|
| **#98** Pack hot-reload session proof | Blocked (HiddenDesktop broken) | ✅ Can launch real game, inject HMR signal, screenshot |
| **#101** AssetSwapSystem render verify | Blocked (no live game) | ✅ Can verify 36/36 assets render (not 0/36 stubs) |
| **#103** Kimi first external receipt | Blocked (no live game launch) | ✅ Can capture game state + screenshot for Kimi judge |
| **#425** MCP SSE verification | Blocked (no headless launch) | ✅ Can stream game events via SSE bridge |
| **#191** Smart-contract proof spec | Blocked (no proof artifacts) | ✅ Can generate signed receipts from real game |

---

## Effort Estimate

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| **Research + Planning** | 3 days | This doc + implementation runbook |
| **Stage 1 (Offline unpack)** | 30 min | Checksum + archive path documented |
| **Stage 2 (CI integration)** | 2-3 days | `game-launch.yml` updated; `Prepare-UnpackedDINO.ps1` added |
| **Stage 3 (Validation)** | 1 day | Runbook verified; Wave 1 acceptance gate passes |
| **Total** | **1 sprint** | Headless launch stack fully working |

---

## Confidence Level: **HIGH** (85%+)

**Why**:
- Steamless is a proven, maintained tool with 5K+ GitHub stars
- MockSteamworksNet is already compiled + unit tested
- Unpacking + testing locally is a low-risk, reversible operation
- Option D has the fewest moving parts (no auth, no credentials, no rate-limits)
- Clear fallback to Option A if Steamless fails on first test

**Unknowns**:
- Whether `-nographics` mode works in DINO (likely yes, but untested)
- Whether MockSteamworks patches catch all SteamAPI calls (5 mocked; there may be others)
- Whether parallel instances can safely share unpacked binary (likely yes, but should verify in test)

---

## Next Steps (IF User Authorizes)

1. **Iter-143 Spike** (1 sprint):
   - Locally unpack DINO with Steamless
   - Deploy MockSteamworksNet; test launch
   - Document runbook
   - Gate: real `dinoforge_debug.log` entries from unpacked binary

2. **Iter-144 CI Integration** (0.5 sprint):
   - Wire into `game-launch.yml`
   - Cache unpacked binary
   - Validate parallel-instance isolation

3. **Iter-145 Proof System** (Wave 2):
   - Integrate with smart-contract proof spec (#191)
   - Sign receipts from real game state
   - Gate: verified receipt + Kimi judge confirmation

---

## Files Affected

**New files**:
- `docs/proposals/headless_steam_drm_stack_iter142.md` (this doc)
- `scripts/ci/Prepare-UnpackedDINO.ps1` (helper for CI)
- `.github/workflows/game-launch-headless.yml` (new workflow)
- `RUNBOOK_HEADLESS_DINO_LAUNCH.md` (user-facing guide)

**Modified files**:
- `.github/workflows/game-launch.yml` (add headless job)
- `src/Tools/MockSteamworksNet/*.csproj` (no changes; already ready)
- `docs/TRUTH_TABLE.md` (update "headless-launch" status from ❌ → 🟡)

---

## References

- **Steamless**: https://github.com/atom0s/Steamless (Apache 2.0)
- **steamguard-cli**: https://github.com/dyc3/steamguard-cli (MIT)
- **steamcmd**: https://developer.valvesoftware.com/wiki/SteamCMD (Valve proprietary)
- **MockSteamworksNet**: `src/Tools/MockSteamworksNet/` (in this repo)
- **Existing infra audits**: `docs/sessions/2026-04-25-*-audit.md` (5 docs)
- **Wave 1 plan**: `docs/sessions/2026-04-25-infra-pivot-plan.md`

---

**Document status**: Research complete. Ready for planning phase. No implementation started.
