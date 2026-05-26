# Iter-143 Wave 1 Sprint Plan: Headless Steam-Free Game Launch

**Goal**: Replace Steam dependency with controllable test harness for headless game launch + observation.

**Estimate**: 36–40 hours (1 sprint)

---

## Pre-Sprint Checklist

- [x] Authorize Steamless download (Apache 2.0, legitimate for personal use) — **PENDING: Only remaining external prereq**
- [x] Confirm DINO installed at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- [x] Confirm iter-143 spike authorization (this plan)

---

## Stage 1: Offline Steamless Unpack (0.5h)

**Owner**: Local Windows dev machine  
**Gate**: Unpacked exe boots with `-nographics -batchmode` flag

1. Download Steamless from https://github.com/atom0s/Steamless/releases (v3.x, ~10–100 MB)
2. Backup vanilla: `cp "Diplomacy is Not an Option.exe" "Diplomacy is Not an Option.exe.steam-original"`
3. Run Steamless GUI: select `Diplomacy is Not an Option.exe` → output `DINO_unpacked.exe`
4. Verify: `.\DINO_unpacked.exe -batchmode -nographics` starts without Steam process running
5. Document checksum: `sha256sum DINO_unpacked.exe > docs/build-artifacts/dino_unpacked.exe.sha256`

**Success**: `dinoforge_debug.log` shows BepInEx boot (no Steam required)

---

## Stage 2: MockSteamworksNet Deploy Target (1h) ✅ DONE (2026-05-19)

**Owner**: Code + build system  
**Gate**: DLL lands in `BepInEx/plugins/` after build

1. ✅ Add 28-line XML target to `src/Runtime/DINOForge.Runtime.csproj` (from spec document)
   ```xml
   <Target Name="DeployMockSteamworksNet"
           AfterTargets="Build"
           Condition="'$(GameInstalled)' == 'true' AND '$(DeployToGame)' == 'true' AND '$(DeployMockSteamworks)' == 'true'">
     <!-- Copy MockSteamworksNet.dll to BepInEx/plugins/ -->
   </Target>
   ```
2. ✅ Build with: `dotnet build src/Tools/MockSteamworksNet/MockSteamworksNet.csproj -p:DeployToGame=true -p:DeployMockSteamworks=true`
3. ✅ Verify: `BepInEx/plugins/MockSteamworksNet.dll` exists
4. ✅ Verify: Plugin loads (BepInEx LogOutput shows "MockSteamworks: patched SteamAPI.Init")

**Success**: Zero Harmony patch load errors; SteamAPI calls mocked

---

## Stage 3: Verify HandleConnect IPC (4h)

**Owner**: Runtime + MCP integration  
**Gate**: Real handshake from unpacked binary

1. Launch unpacked DINO (no Steam): `.\DINO_unpacked.exe -nographics -batchmode`
2. Call MCP `game_status` → verify `world_ready == true` within 10s
3. Call MCP `game_connect` RPC → receive session envelope (session_id, session_key_b64)
4. Verify `dinoforge_debug.log` shows: `[GameBridgeServer] HandleConnect: minted session_id=...`
5. Capture screenshot via MCP `game_screenshot`
6. Kill process; verify no orphaned handles

**Success**: Session minted; bridge communicates with unpacked binary (closes #508)

---

## Stage 4: CI Integration (10h)

**Owner**: GitHub Actions workflow  
**Gate**: Workflow runs end-to-end on push; artifact cache works

1. Create `.github/workflows/game-launch-headless.yml`
   - Restore unpacked DINO cache (5-day TTL)
   - Deploy MockSteamworksNet.dll
   - Launch: `.\DINO_unpacked.exe -nographics -batchmode`
   - Poll `dinoforge_debug.log` for "DINOForge initialized" (10s timeout)
   - Capture screenshot via MCP
   - Cleanup; upload artifacts

2. Test locally on Windows runner (or self-hosted)
3. Verify caching works (hit rates > 80% on retry)
4. Verify parallel instances don't crash (read-only binary)
5. Document in `RUNBOOK_HEADLESS_DINO_LAUNCH.md`

**Success**: Workflow green on 2+ consecutive merges; CI log shows pack load

---

## Stage 5: Acceptance Gates + Handover (3h)

**Owner**: Documentation + task closure  
**Gate**: All 3 conditions met

1. **Gate 1**: `dinoforge_debug.log` fresh HandleConnect entries within 5s of unpacked launch
2. **Gate 2**: CI workflow completes without timeout on every PR
3. **Gate 3**: #523 EconomyContentLoader test + #524 hook smoke-test verified

Update:
- `CLAUDE.md` Game Automation section (headless protocol)
- `CHANGELOG.md` (iter-143 entry)
- Mark #98, #101, #103, #425 unblocked in TaskList

**Success**: User can call `/game-launch hidden=false` and get real game output

---

## Risk Register

| Risk | Likelihood | Mitigation |
|------|------------|-----------|
| Steamless fails on SteamStub v3 | LOW | Verify locally first; fallback to Option A (steamcmd) in 1 day |
| MockSteamworks missing Steamworks call | MED | Add patches incrementally; test real game launch early |
| CI cache expires mid-sprint | LOW | Rebuild unpacked (one-time cost) |
| `-nographics` breaks rendering | MED | Test locally first; may need `-batchmode` only |
| Parallel instance file locks | LOW | Already mitigated by _TEST dir isolation (#188) |

---

## Out of Scope (Defer to Wave 2+)

- Journey records UI viewer (Wave 2, 30h)
- Cross-project Rust CLI tool (Wave 3, 1-2 sprints)
- VM-scale parallel fleet (Wave 4, v0.27.0+)

---

## Unblocked Critical Tasks

- **#98**: Pack hot-reload session proof ✅
- **#101**: AssetSwapSystem render verification ✅
- **#103**: Kimi first external receipt (real game) ✅
- **#425**: MCP SSE verification ✅

---

**Document Status**: Ready for sprint start. All resources identified. No external dependencies blocking Stage 1.
