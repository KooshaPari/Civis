# DINOForge Wave 1 acceptance runbook (2026-04-25)

This runbook validates the iteration-37 Wave-1 infra fixes (#188, #189, #190) defined in
`docs/sessions/2026-04-25-infra-pivot-plan.md`. Run each step top-to-bottom on Windows
(PowerShell + Git Bash). Each step has an exact replayable command and a clear PASS/FAIL signal.

## Prerequisites

- DINOForge solution built: `dotnet build src/DINOForge.sln -c Release`
- `DINOForge.Runtime.dll` deployed to main install: `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true`
- Steam authenticated; DINO closed (`Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue`)
- `_TEST` install dir exists at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\`
- playCUA binary exists at `C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe`
- DINOForge MCP server running: `pwsh -File scripts/start-mcp.ps1 -Action start -Detached`

---

## Step 1 — DINOBox plugin deployment (#188 Fix 2)

**What you're verifying**: `New-DINOBoxPool.ps1` now copies `DINOForge.Runtime.dll` into each box.
Previously, boxes were vanilla DINO with no mod loaded.

```powershell
# Recreate box_1 from scratch (Force replaces any existing box_1)
pwsh -File scripts/game/New-DINOBoxPool.ps1 -BoxCount 1 -Force

# Verify the DLL landed in the box's BepInEx plugins dir
Test-Path "G:\dino_boxes\box_1\BepInEx\plugins\DINOForge.Runtime.dll"
# Expected: True

# Verify the box's boot.config has single-instance=0 (NOT 'false')
Get-Content "G:\dino_boxes\box_1\Diplomacy is Not an Option_Data\boot.config" | Select-String "single-instance"
# Expected: single-instance=0
```

**PASS**: Both `True` and `single-instance=0`. **FAIL**: Either missing.

---

## Step 2 — _TEST boot.config fix (#188 Fix 3)

```powershell
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option_Data\boot.config" | Select-String "single-instance"
# Expected: single-instance=0   (was 'single-instance=false' before this fix)
```

**PASS**: literal `single-instance=0`. **FAIL**: `single-instance=false` or missing line.

---

## Step 3 — Launch-DINOBoxInstance default flip (#188 Fix 4)

```powershell
Get-Content "scripts/game/Launch-DINOBoxInstance.ps1" | Select-String 'switch.*Hidden'
# Expected: [switch]$Hidden = $false   (was $true; hidden-desktop is broken)
```

**PASS**: `$Hidden = $false` shown. **FAIL**: `$Hidden = $true` or no match.

---

## Step 4 — playCUA path drift (#188 Fix 1)

```bash
# Use Git Bash here. The previous code referenced 'native/target/release/...'; that path doesn't exist.
grep -n "native/target/release\|native\\\\target\\\\release" src/Tools/DinoforgeMcp/dinoforge_mcp/isolation_layer.py
# Expected: no matches
```

**PASS**: zero matches (grep exits 1). **FAIL**: any line printed.

---

## Step 5 — End-to-end DINOBox launch (the actual proof)

Without this step, all of Wave 1 is just static fixes. This is where the foundation either works or doesn't.

```powershell
# Close any running DINO
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Start-Sleep 3

# Launch box_1 with default visible-mode (Hidden=$false)
pwsh -File scripts/game/Launch-DINOBoxInstance.ps1 -BoxPath "G:\dino_boxes\box_1"

# Wait ~15s for DINO to boot + BepInEx to load DINOForge.Runtime.dll
Start-Sleep 15

# Verify the box log got written
Test-Path "G:\dino_boxes\box_1\BepInEx\dinoforge_debug.log"
Get-Content "G:\dino_boxes\box_1\BepInEx\dinoforge_debug.log" -Tail 30
```

**PASS**: log file exists AND tail contains both `ModPlatform OnWorldReady` AND `GameBridgeServer started`.
**FAIL**: log empty, missing, or no `ModPlatform`/`GameBridgeServer` lines.

---

## Step 6 — Bridge bypass surface fixes (#189)

**What you're verifying**: `GameBridgeServer.HandleStatus` no longer returns literal `Running=true`
when the world is null/not-ready, and `applyOverride` reports failure when no entity matches.

```powershell
# With box_1 still running from Step 5, hit MCP game_status
$resp = Invoke-RestMethod "http://127.0.0.1:8765/tools/game_status" -Method Post
$resp | ConvertTo-Json -Depth 4
# Expected fields: Running=true, EntityCount > 0, WorldReady=true, ModPlatformReady=true

# Now kill the game and re-query (bypass surface should NOT lie)
Stop-Process -Name 'Diplomacy is Not an Option' -Force
Start-Sleep 3
$resp2 = Invoke-RestMethod "http://127.0.0.1:8765/tools/game_status" -Method Post
$resp2 | ConvertTo-Json -Depth 4
# Expected: success=false (CLI failure) OR Running=false with explicit error
```

**PASS**: live-game query reports real entity count; dead-game query reports `success=false` or `Running=false`.
**FAIL**: dead-game query returns `success=true` AND `Running=true` (bypass surface still leaks).

---

## Step 7 — Trait-fraud guard (#190)

```powershell
python scripts/analysis/check_trait_fraud.py
# Expected: "Trait-fraud check: CLEAN (199 files scanned)"
```

**PASS**: `CLEAN`. **FAIL**: any violation listed (test file + line + offending trait).

---

## Step 8 — End-to-end summary

After all 7 steps pass, Wave 1 is done. Tick each box only after the actual command above succeeded.

| Step | Verifies | Pass? |
|------|----------|-------|
| 1 | DINOBox DLL deployment + box boot.config | &#9744; |
| 2 | `_TEST` boot.config single-instance=0 | &#9744; |
| 3 | Launch script default `Hidden=$false` | &#9744; |
| 4 | playCUA path drift removed | &#9744; |
| 5 | Real DINOBox launch with mod loaded | &#9744; |
| 6 | Bridge bypass surface eliminated | &#9744; |
| 7 | Trait-fraud guard CLEAN | &#9744; |

---

## If a step fails

File a GitHub issue with:
- The step number
- The exact command that failed
- Full stderr + stdout (or PowerShell error record)
- The file:line being verified (visible in each step above)

Wave-1 fixes are localized; failures are debuggable. Do **not** mark Wave 1 done with any &#9744; remaining.

## If all 7 pass

You have:
1. An honest multi-instance DINOForge sandbox (DLL deploys, no hidden-desktop default, no path drift).
2. A bridge surface that does not lie when the game is not running.
3. A trait-fraud guard that prevents `Category=E2E`/`UserStory` tests from secretly using `FakeGameBridge`.

Wave 2 (smart-contract proof system, signed receipts, merkle bundle) builds on this foundation.
See `docs/sessions/2026-04-25-infra-pivot-plan.md` Wave 2 section and the in-flight
`docs/design/2026-04-25-smart-contract-proof-system.md`.

## Cross-references

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — pivot plan (defines Wave 1 acceptance)
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md` — audit for #188
- `docs/sessions/2026-04-25-bridge-bypass-audit.md` — audit for #189
- `docs/TRUTH_TABLE.md` — hidden-desktop / judge-receipts / CI-game-launch rows update after Wave 1 lands
