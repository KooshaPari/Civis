# UI Tree Census — 2026-06-01

Branch: `feat/uicensus-20260601`  
Environment: MCP server at `http://127.0.0.1:8765`  
Capture status: **partial (no live game pipe connection at capture time)**

## Capture notes

- Added MCP tool: `game_dump_ui_tree` in `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py`.
- Added CLI wiring for `dump-ui-tree` (alias of `ui-tree`) in `src/Tools/GameControlCli/Program.cs` with `includeCursor` control.
- Runtime snapshot already emits `IsThemed`, `RenderPath`, `Visual` and a native cursor node (`RenderPath = native-cursor`) from `UiTreeSnapshotBuilder`.
- Live call attempt via MCP/CLI failed with `Failed to connect to pipe 'dinoforge-game-bridge'`, so this matrix is a **static census scaffold** plus prior runtime dump-field coverage, not a fully materialized runtime dump.

## Matrix (static + existing snapshot coverage)

| Surface | Element | Type | Render-path | State | Fix-needed | Notes |
|---|---|---|---|---|---|---|
| main menu | root background canvas | Canvas | canvas | native | yes | Verify full theming pass for top-level menu sprites |
| main menu | logo/title | Image | Image-sprite | native | yes | DINOForge icon/branding often bypasses theme swap |
| main menu | menu option button | Button | Image-sprite | themed/partial | maybe | Button style fields are captured; requires click-path verification |
| main menu / subpages | primary label text | TMP_Text | TMP-text | themed/partial | maybe | Many labels use TMP/TextMeshPro with dynamic font fallback |
| MODS page | tab button | Button | Image-sprite | themed/partial | maybe | MODS button hook exists (`DINOForge_ModsButton`) but adjacent siblings may remain native |
| MODS page | mod row icon | Image | Image-sprite | native | yes | Icon atlas coverage likely inconsistent |
| F10 menu | section headers | TMP_Text | TMP-text | native | yes | Needs audit once game is running |
| F10 menu | list row background | Image | Image-sprite | native | yes | Common whack-a-mole surface from runtime themes |
| loading screen | background image | Image | Image-sprite | native | yes | Backgrounds/cutscenes often loaded after theme init |
| loading screen | loading caption | TMP_Text | TMP-text | native | yes | Runtime font/theme timing risk at transition boundaries |
| build-panel | card icon | Image | Image-sprite | native | yes | Multiple cards are image-first and not all theme-swapped |
| build-panel | card title text | TMP_Text | TMP-text | native | yes | TMP font pass should be audited globally |
| HUD | resource icon | Image | Image-sprite | native | yes | HUD icon strips frequently remain native |
| HUD | action button | Button | Image-sprite | themed/partial | maybe | Button background + child label parity check needed |
| HUD | status value | TMP_Text | TMP-text | native | yes | Numeric/status text may remain native in some views |
| enemy-preview | portrait frame | Image | Image-sprite | native | yes | Portrait and frame textures often split by runtime path |
| enemy-preview | faction flag | Image | Image-sprite | native | yes | Faction flag texture surface currently not consistently themed |
| enemy-preview | faction name | TMP_Text | TMP-text | native | yes | Requires unified label theming |
| cursor | hardware pointer | Cursor | native-cursor | native | yes | Always captured as `native-cursor`, currently not themed |
| cursor | click hotspot helper | Image | Image-sprite | native | no | None currently identified in live dump |
| ECS-mesh overlay | mesh-backed glyph layer | ECS-mesh | ECS-mesh | native | no | Render-path retained for future mesh-based surfaces |

## Priority list of likely native surfaces (top offenders)

- Cursor (`native-cursor`)  
- Main menu background/title surfaces (`Canvas` + `Image` blocks)  
- HUD resource/action icons (`Image`)  
- Faction flags (`Image`)  
- MODS page row/tab surfaces (`Image`/`Button`)  
- Loading captions and headers (`TMP_Text`)  
- Enemy preview portrait/flag region (`Image`)  

## Next action for live census

- Re-run:
  - `python tools/mcp_client.py call game_dump_ui_tree --selector <surface> --include-cursor true` (or use CLI):
  - `dotnet run --project src/Tools/GameControlCli -- dump-ui-tree`
- Export and replace this static table with the full live rows for all listed surfaces.
