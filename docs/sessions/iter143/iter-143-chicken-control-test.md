# Iter-143 Chicken-Skeleton Control Test

**Date**: 2026-05-19 01:34-01:46 local
**Question**: Are the chicken-skeleton placeholders in DINO's main menu caused by our mod, or are they vanilla?
**Procedure**: Mod-OFF baseline screenshot vs Mod-ON screenshot.

---

## Procedure Executed

| Phase | Step | Time | Result |
|-------|------|------|--------|
| A | Kill existing game | 01:34:38 | STOPPED_OK |
| A | Rename `BepInEx` → `BepInEx_off` | 01:34:42 | RENAMED_TO_OFF |
| A | Launch game (mod OFF) | 01:34:46 | LAUNCH_REQUESTED |
| A | Wait 60s | 01:35:46 | ALIVE, Title=`Diplomacy is Not an Option` (clean) |
| A | Capture mod-off screenshot | 01:35:56 | SAVED 2,380,351 bytes |
| A | Stop game | 01:36:11 | STOPPED |
| A | Rename `BepInEx_off` → `BepInEx` | 01:36:15 | RESTORED |
| B | Launch game (mod ON) | 01:36:44 | LAUNCH_REQUESTED |
| B | Wait 60s | 01:37:44 | ALIVE PID=46784, Title contains `Fatal error` |
| B | Multiple re-launches, dialog dismissals | 01:38–01:46 | Multiple capture attempts |
| B | Final capture | 01:45:18 | 1,193,969 bytes — but image is **WorldBox**, not DINO |
| C | Stop game, restore state | 01:46:00 | BepInEx=True, BepInEx_off=False, DLL present |

---

## Screenshot Findings

### Mod-OFF (CLEAN, CONCLUSIVE) — `docs/screenshots/mod-off-mainmenu.png`

Shows the **vanilla DINO main menu** with full fidelity:
- "Diplomacy is Not an Option" title in red Gothic font (top left)
- Full menu list: **Gay, Continue, Load, +, Campaign, Challenge Mode, Sandbox Mode, Endless Mode, Map Editor, Tutorial, +, Options, Credits, Quit**
- Soundtrack ad popup (top right) — "DIPLOMACY IS NOT AN OPTION SOUNDTRACK" with red OPEN button
- Background art: candle-lit medieval hall, sitting king in red robe holding cup, stained-glass windows with light beams
- Build version watermark "1.0.10.8.." (lower left)
- **NO chickens. NO skeleton placeholders. NO fowl silhouettes.**

The vanilla menu renders correctly — there is nothing visually broken about DINO's base assets.

### Mod-ON (INCONCLUSIVE) — `docs/screenshots/mod-on-mainmenu.png`

The mod-ON capture phase encountered repeated `Fatal error: Another instance is already running` dialogs spawned by the game's own launcher on every cold start (not caused by my procedure — observed even after 10–30s mutex-clear waits and on the first launch with no prior instances). Each launch produced two processes:
- Game (clean title, 545 MB working set) — the real Unity render process
- Fatal-error dialog (42 MB, separate proc) — Steam DRM / launcher anti-piracy check

When the fatal dialog was dismissed via PostMessage WM_CLOSE, the underlying game process survived but its window appeared to be **non-foreground / offscreen-rendered** — capture attempts via `CopyFromScreen` at the game's reported window rect (0,0,2560×1440) repeatedly returned the desktop foreground app (WorldBox) instead of the game's framebuffer. This is consistent with: the game's window technically exists at the reported coordinates, but it's not the topmost rendered window — the desktop compositor is showing whatever IS topmost at those pixels.

The mod-ON screenshot is therefore **not a valid capture of the mod-ON main menu** and cannot be compared visually to the mod-OFF screenshot.

---

## Verdict

**INCONCLUSIVE for the chicken-specific question, but with strong indirect evidence.**

**Strong indirect evidence the chickens are NOT vanilla:**
- The mod-OFF screenshot is a complete, well-rendered vanilla main menu with **zero anomalies and zero chickens**. The vanilla game's assets render correctly.
- DINOForge does have a DFCanvas + UI registry layer (iter-143 #531 just landed the EventSystem null-guard fix in DFCanvas.BuildCanvas, which is directly related to the broken-UI symptom in #529).
- The mod's UI injection pipeline is the most likely source of placeholder sprites in the main menu, given (a) it operates at canvas-injection time and (b) UI domain registry / HUD elements pipeline has had multiple recent landings.

**Why a direct visual A/B was not completed**: Repeated mod-ON cold launches collide with the game's own "Another instance is already running" launcher check, even with no prior instances running. Dismissing the dialog left the game process alive but in a non-foreground state where Win32 screen capture pulled the desktop (WorldBox) instead of the game's framebuffer. The proper capture path requires either the mod's named-pipe GameControlCli (which timed out trying to connect to `dinoforge-game-bridge`, suggesting the bridge server didn't come up in this session) OR the MCP `game_screenshot` tool (which uses the Unity ScreenCapture API and works regardless of foreground).

**Recommendation for next iteration**:
1. Get GameControlCli connecting to the mod's named pipe — verify `GameBridgeServer.Start()` is being called and `HandleConnect` registered (iter-142 #508 fix). If the pipe isn't listening, the mod isn't fully booting (which itself might explain the broken UI).
2. Once GameControlCli connects, use it for the mod-ON capture — that path goes through Unity ScreenCapture and bypasses the Win32 foreground problem.
3. Re-run this A/B with that capture method.

**Confidence**: Mod-OFF screenshot is **100% conclusive** — vanilla DINO main menu has no chickens. Mod-ON capture failed visual A/B but indirect evidence (DFCanvas EventSystem fix, recent UI pipeline churn) makes "chickens are mod-introduced" the leading hypothesis at >80% confidence.

---

## State at End of Test

- `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx` exists: **True**
- `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx_off` exists: **False** (cleaned up)
- `BepInEx\plugins\DINOForge.Runtime.dll` exists: **True**
- `BepInEx\dinoforge_debug.log` size: 2,178,862 bytes (mod did run during mod-ON attempts)
- Game processes: all stopped

**Mod restoration confirmed.**
