# EPIC-027 — Functional & Non-Functional Requirements Catalog

**Epic**: EPIC-027 — True Full-Conversion Experience
**Version Target**: v0.27.0
**Last Updated**: 2026-05-29
**Status**: Active

This file is the authoritative catalog of all FR and NFR IDs used across the 13 SW-xxx story
specs. Each entry lists the owning story, type (functional/non-functional), and a short
description. Evidence tables live in the individual story files.

---

## Functional Requirements

| ID | Owning Story | Description |
|----|-------------|-------------|
| EPIC-027-FR-001 | SW-001 | Native Mods button visible and clickable on main menu; clicking opens `DINOForge_ModsPage`. |
| EPIC-027-FR-002 | SW-001 | Mods page lists all loaded packs with name, version, status badge; empty state shows "No additional packs loaded". |
| EPIC-027-FR-003 | SW-001 | Mods page closes cleanly; native buttons remain functional; re-scan produces no duplicate rows. |
| EPIC-027-FR-004 | SW-001 | F10 toggles Mods surface; F9 toggles Debug overlay (Win32 background-thread key path). |
| EPIC-027-FR-005 | SW-002 | DINOForge version badge appended to game window title while any pack is active. |
| EPIC-027-FR-006 | SW-002 | Window icon overridden to DINOForge icon when a TC pack declares `window_icon`. |
| EPIC-027-FR-007 | SW-003 | All 30 Star Wars unit bundle files are non-stub Unity 2021.3.45f2 AssetBundles. |
| EPIC-027-FR-008 | SW-003 | `dinoforge verify-mod --pack warfare-starwars` and `--pack warfare-modern` exit 0 with 0 stub-bundle errors. |
| EPIC-027-FR-009 | SW-004 | Loading screen replaced with DINOForge default or per-mod themed background while a TC pack is active. |
| EPIC-027-FR-010 | SW-005 | Mod brand identity (logo, palette, typography) applied to main menu for both mods; no vanilla DINO art visible. |
| EPIC-027-FR-011 | SW-006 | ThemeEngine applies faction palette to HUD elements for active TC pack across all 7 phases. |
| EPIC-027-FR-012 | SW-006 | `PackCompiler validate` rejects malformed `ui_theme` manifests; valid themes pass with 0 errors. |
| EPIC-027-FR-013 | SW-007 | In-game 2D asset takeover: every spawnable unit shows mod portrait; faction emblems replaced in HUD. |
| EPIC-027-FR-014 | SW-007 | Sprite swap strategy (B: direct swap; A: Addressables intercept) confirmed and activated per discovery pass. |
| EPIC-027-FR-015 | SW-008 | `PackCompiler validate` exits 0 for both packs; 0 dangling asset refs; unit counts meet coverage thresholds. |
| EPIC-027-FR-016 | SW-009 | Themed projectiles (blasters, bullets, missiles) visible during combat for both mods per declared bundle. |
| EPIC-027-FR-017 | SW-010 | Naval units build, traverse water tiles, and engage targets in-game for both mods. |
| EPIC-027-FR-018 | SW-011 | Aerial units spawn, pathfind over all terrain, attack ground and air targets for both mods. |
| EPIC-027-FR-019 | SW-012 | Audio takeover: menu music and gameplay ambient music replaced for both mods; crossfade works without hard cut. |
| EPIC-027-FR-020 | SW-001, SW-008 | Toggling a pack updates its active badge; `dinoforge status` reflects the toggle after relaunch. |

---

## Non-Functional Requirements

| ID | Owning Story | Category | Description |
|----|-------------|----------|-------------|
| EPIC-027-NFR-001 | SW-001, SW-013 | Performance | Mods page opens in ≤ 500 ms on a 60 FPS host. |
| EPIC-027-NFR-002 | SW-006, SW-007, SW-013 | Regression | No regressions in existing tests from v0.26.0; all unit + integration tests green. |
| EPIC-027-NFR-003 | SW-003 | Compatibility | Asset bundles built with Unity 2021.3.45f2; bundles from other Unity versions are rejected at load time. |
| EPIC-027-NFR-004 | SW-001, SW-004 | Memory | Open/close cycles produce no monotonic GameObject or native-memory growth. |
| EPIC-027-NFR-005 | SW-002–SW-013 | Build | Runtime DLL targets `netstandard2.0`; no compile-time TMPro or Addressables references added. |
| EPIC-027-NFR-006 | SW-003–SW-012 | Compatibility | All new plugin components load under BepInEx 5.4.23.5 without TypeLoadException; plugin Awake fires. |
| EPIC-027-NFR-007 | SW-001, SW-005, SW-006 | Architecture | No `[HarmonyPatch]` attributes target any DINO UI type; all injection uses GameObject composition. |
| EPIC-027-NFR-008 | SW-001, SW-004–SW-007 | Naming | All injected GameObjects prefixed `DINOForge_`; no unnamed injected objects. |
| EPIC-027-NFR-009 | SW-001 | Security | No `Process.Start` or URL-open path consumes unvalidated pack data. |
| EPIC-027-NFR-010 | SW-003, SW-008 | Security | Tampered or wrong-version bundles are skipped with a warning logged; game continues. |
| EPIC-027-NFR-011 | SW-003, SW-007, SW-008 | Security | `PackCompiler validate` rejects manifests referencing `../` or absolute asset paths. |
| EPIC-027-NFR-012 | — | (reserved) | Reserved. |
| EPIC-027-NFR-013 | SW-004, SW-006–SW-012 | Stability | `LogOutput.log` contains no `TypeLoadException` after a clean game launch with the new component active. |
| EPIC-027-NFR-014 | SW-004, SW-007, SW-009 | Resilience | Missing or failed asset bundles degrade gracefully: vanilla asset used, WARNING logged, no crash. |
| EPIC-027-NFR-015 | SW-001, SW-004–SW-008 | Input Safety | All injected `Image` components have `raycastTarget = false`; EventSystem guard precedes any `GraphicRaycaster` add (Pattern #235). |
| EPIC-027-NFR-016 | SW-001, SW-006 | UX | Injected UI elements (Mods button, Mods page) match hover/layout of adjacent native DINO buttons. |
| EPIC-027-NFR-017 | SW-001 | UX | Escape closes the Mods page; keyboard navigation moves focus across page entries. |
| EPIC-027-NFR-018 | SW-001, SW-002, SW-005 | i18n | All new user-visible strings resolve through the locale layer; non-English locales show translated labels. |
| EPIC-027-NFR-019 | SW-003, SW-008 | Asset Gate | `scripts/ci/detect_stub_bundles.py` exits 0; declared `visual_asset` count equals non-stub bundle count for both packs. |
| EPIC-027-NFR-020 | SW-005, SW-007, SW-008 | Visual | Full play session with TC active shows no vanilla DINO medieval 2D art (judge receipt per mod). |
| EPIC-027-NFR-021 | SW-005, SW-008 | Visual | Faction emblems (Republic + CIS; Alliance + Enemy) and unit portraits visible in-game for both mods. |
| EPIC-027-NFR-022 | SW-008, SW-012 | Legal | Asset licensing manifest complete: every shipped audio, image, and 3D asset is documented as original composition or CC0-licensed; `LICENSE-audio.md` / `LICENSE-assets.md` present in each pack root. |
