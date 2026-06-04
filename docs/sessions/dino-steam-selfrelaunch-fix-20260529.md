# DINO Steam Self-Relaunch Root Cause + Fix (2026-05-29)

## Symptom
DINO launches WITH a BepInEx console, then crashes with "no more than one
process / another instance", and a NEW DINO launches WITHOUT BepInEx — so no
MODS button, no F9/F10. The earlier "dormant plugin" finding was actually the
BepInEx-injected process being killed by this self-relaunch.

## Root Cause (live-evidenced)
DINO's bootstrap detects it was NOT launched via Steam (no `steam_appid.txt`
beside the exe, Steam not the parent). It RELAUNCHES ITSELF THROUGH STEAM. The
original directly-launched, doorstop-injected (winhttp.dll) process exits on the
game's single-instance guard. The Steam-relaunched survivor's parent is
`steam.exe`; the doorstop is not re-applied in a way that survives the handoff →
BepInEx gone.

### STEP 1 evidence — self-relaunch confirmed (before fix)
Direct `Start-Process` of the exe, watched via `Get-CimInstance Win32_Process`:

```
t=1-7s : [PID=612388 parent=pwsh.exe   start=21:20:18]   <- our injected launch
t=8s   : NO DINO PROCS                                   <- original EXITS
t=9s+  : [PID=458572 parent=steam.exe  start=21:20:28]   <- Steam self-relaunch
```
Final survivor cmdline: `"...\Diplomacy is Not an Option.exe"` parent=steam.exe (PID 524628).

Log evidence (injected process loaded fully, then OnDestroy = single-instance kill):
```
[Message: BepInEx] Chainloader startup complete
[Info: DINOForge Runtime] DINOForge Runtime loaded successfully.
[Info: DINOForge Runtime] [Plugin] BepInEx plugin object OnDestroy (persistent root still alive).
```
Player.log: `Game Version [Steam]: dno_1.0.152_r`
Preconditions confirmed: `steam_appid.txt` ABSENT; `winhttp.dll` (26 KB doorstop) present.

## STEP 2 — Fix applied
Created `steam_appid.txt` next to the exe, content exactly `1272320`
(DINO Steam AppID 1272320), UTF-8 **no BOM**, no trailing newline:

```powershell
$enc = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText(
  "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\steam_appid.txt",
  "1272320", $enc)
```
Verified: 7 bytes = 49,50,55,50,51,50,48 ("1272320"), no BOM.

This makes Steamworks report the game already running under the correct AppID,
so DINO's bootstrap does NOT relaunch through Steam — the directly-launched
(doorstop'd) process is authoritative and survives with BepInEx.

## STEP 3 — Verification (after fix) — PASS
Killed all, launched the exe DIRECTLY, watched 80s:
```
t=1..18s : [PID=608572 parent=pwsh.exe]   (no steam.exe handoff ever appeared)
final    : PID=608572 parent=pwsh.exe alive at 80s
```
- NO self-relaunch. No steam-parented process appeared.
- Live process loaded the doorstop: `WINHTTP.dll` from the game dir present in modules.
- LogOutput.log fresh: `Chainloader startup complete`, `DINOForge Runtime loaded successfully`.
- Heartbeat ADVANCING (plugin alive, not dormant): 186 → 196 over 5s; later 472 → 482.
- **`ENGINE-UI READY: packs=10 modsButton=True f9=True f10=True (via scene-change)`**

Screenshot (MCP `game_screenshot`, WGC backend, 1680x1050):
`docs/screenshots/mods-button-FIXED-steamappid-20260529.png`
Main menu visibly shows the **MODS** button (between OPTIONS and CREDITS).

## ANSWER: Is the MODS button + F9/F10 now on screen? **YES.**

## Winning launch method
Direct launch of the exe (`Start-Process -FilePath <exe> -WorkingDirectory <dir>`)
**with `steam_appid.txt` present**. No Steam URL needed. The directly-launched
doorstop'd process survives to MainMenu with the MODS button.

## STEP 5 — Persistence / recommendations
- **Steam re-validation (Verify Integrity of Game Files) may DELETE `steam_appid.txt`.**
  Treat it as a deploy artifact, re-create on every deploy.
- **DeployToGame should drop `steam_appid.txt`** next to the exe (content `1272320`,
  UTF-8 no BOM) — add to the Runtime deploy target.
- **MCP `game_launch` / scripts/game launch tooling**: keep direct-exe launch, but
  ensure `steam_appid.txt` exists (create if missing) before launching. Do not
  switch to `steam://rungameid/` — direct + appid is the verified winning path.
- **CLAUDE.md**: add to the game section a note that `steam_appid.txt`=`1272320`
  must exist beside the exe or DINO self-relaunches through Steam and drops BepInEx.

## Files
- Fix artifact: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\steam_appid.txt`
- Proof screenshot: `docs/screenshots/mods-button-FIXED-steamappid-20260529.png`
- Deployed Runtime DLL (iter-149): MD5 `D035165932913911572F53D25F842E59`, 2026-05-29 21:15
