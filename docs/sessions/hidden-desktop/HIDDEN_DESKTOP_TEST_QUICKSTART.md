# Hidden Desktop Test — Quick Start

## Run the Test (Right Now)

```powershell
cd C:\Users\koosh\Dino
pwsh -File scripts/game/hidden_desktop_test.ps1
```

Expected runtime: 20-30 seconds

## What Happens

1. Creates invisible desktop "DINOForge_Test_XXXX"
2. Launches game on that desktop
3. Waits 15s for window to appear
4. Captures screenshot from game window
5. Analyzes: is it black or rendered?
6. Kills game, destroys desktop
7. **Prints final result: ✓ SUCCESS or ✗ FAILURE**

## Success Looks Like

```
Status: ✓ SUCCESS

Finding:
  Unity D3D11 rendering WORKS on hidden desktops

Implications:
  • Agents can launch game completely invisibly
  • Zero window focus interference with user's session
  • Safe for concurrent automation + user work
```

**What it means**: Hidden desktop approach is viable. Implement in MCP launcher.

## Failure Looks Like

```
Status: ✗ FAILURE

Finding:
  Unity D3D11 rendering DOES NOT work on hidden desktops

Implications:
  • Hidden desktop approach is NOT viable
  • Must fall back to separate Windows user account
```

**What it means**: GPU/driver doesn't support rendering to invisible surface. Use user account isolation instead.

## Output Files

- **Screenshot**: `$env:TEMP\DINOForge\hidden_desktop_test.png` (proof of render)
- **Log**: Console output (copy to file if debugging)

## Troubleshooting

**"Failed to create desktop: error code 1314"**
→ Run as Administrator

**"Game window not found after 15s"**
→ Increase wait time: `-WaitSeconds 30`

**"Screenshot is mostly black"**
→ D3D11 rendering didn't work (fallback to user account)

## Key Files

| File | Purpose |
|------|---------|
| `scripts/game/hidden_desktop_test.ps1` | Main test script (~500 lines, full P/Invoke) |
| `docs/sessions/HIDDEN_DESKTOP_TEST_PLAN.md` | Complete test plan (architecture, debugging, ADR) |
| `docs/sessions/HIDDEN_DESKTOP_TEST_QUICKSTART.md` | This file (quick reference) |

## Next Steps

1. **Run the test** (see above)
2. **Document result** in this file (SUCCESS or FAILURE)
3. **If SUCCESS**: Update MCP launcher to use hidden desktops by default
4. **If FAILURE**: Implement user account fallback in launcher
5. **Create slash command**: `/launch-game-agent` or `/launch-game-isolated` based on result

---

## Technical Snapshot

**Test validates**: Can Unity 2021.3.45f2 + D3D11 render frames when launched on a hidden Win32 desktop?

**P/Invoke APIs used**:
- `CreateDesktop()` - create invisible desktop
- `CreateProcess()` - launch game with lpDesktop parameter
- `FindWindow()` - locate game window
- `GetDC()` / `BitBlt()` - capture screenshot via GDI
- `GetPixel()` - analyze pixel content

**Critical decision gates**:
- ✓ Desktop created successfully? (CreateDesktop returns valid handle)
- ✓ Game launched on desktop? (CreateProcess succeeds, PID is valid)
- ✓ Window appeared? (FindWindow finds "Diplomacy is Not an Option")
- ✓ Screenshot captured? (BitBlt succeeds, PNG file created)
- ✓ Content is non-black? (sampling detects rendered pixels)

If all pass → **SUCCESS** → hidden desktop viable
If any fail → **FAILURE** → fallback to user account

---

**Status**: Prototype ready for execution
**Created**: 2026-03-25
**Owner**: DINOForge Development
