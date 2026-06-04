# EPIC-027: True Full-Conversion Experience

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Version Target**: v0.27.0
**AgilePlus Stories**: SW-001 through SW-013 (see `docs/specs/v0.27.0/`)

---

## Summary

From the first frame after launch, an active total-conversion pack must read as an entirely
different game. v0.27.0 delivers every layer of that experience: a visible and usable native
Mods page, brand identity injection, loading-screen takeover, whole-game UI reskin engine,
2D and 3D asset replacement, new mechanical domains (naval, aerial, themed projectiles), and
audio takeover. The program closes the last gap between "DINO with a color filter" and a game
a player would not recognize without being told.

## Driving User Story

As a **mod player**, I want to load a total-conversion pack and immediately feel like I am
playing a completely different game — from the title screen through every minute of gameplay —
so that DINOForge can support franchise-quality total conversions on par with *Republic at War*
or *Thrawn's Revenge*.

As a **mod author**, I want a declarative YAML manifest and a well-documented asset pipeline
so that I can produce a full total conversion without writing C# or patching runtime internals.

## In Scope — v0.27.0

- Mods page visible and native-styled (fixes blank-menu regression from iter-147)
- DINOForge active indicator in window title / window icon
- Loading screen takeover: default DINOForge screen + per-mod themed screens
- Mod brand identity: SW + Modern logo, palette, and typography applied everywhere
- Full UI/UX reskin engine (7-phase plan, `docs/design/ui-ux-reskin-system.md`)
- In-game 2D asset takeover (backgrounds, UI sprites, unit portraits, cursor)
- Real Unity 2021.3.45f2 asset bundles — kills #101 stub-bundle regression (0/36 in-game)
- Naval combat domain for both mods
- Aerial combat completion for both mods
- Themed projectile support (blasters, bullets, missiles)
- SW space combat as R&D / stretch goal
- 100% asset completeness for `warfare-starwars` and `warfare-modern`
- Audio takeover: music and SFX replacement per active total-conversion

## Out of Scope (deferred)

- Multiplayer pack synchronization
- Steam Workshop integration
- VDD virtual display driver (Tier 1 isolation — v0.28.0+)
- Docker/Kubernetes backend for headless testing

## Architecture Constraints (load-bearing)

All implementation must respect the DINO runtime execution model
(`docs/adr/ADR-014-runtime-execution-model.md` and CLAUDE.md):

- `MonoBehaviour.Update()` / `OnGUI()` NEVER fire — all logic must be event-driven
  (scene-change callbacks, RuntimeDriver pump, Win32 background thread).
- `Resources.FindObjectsOfTypeAll` from a background thread DEADLOCKS — main thread only.
- Runtime DLL stays `netstandard2.0` — no direct TMPro or Addressables compile references;
  all access via reflection or `Type.GetType`.
- AssetBundles (fonts, audio, 3D meshes) MUST be built with Unity 2021.3.45f2.
- Injected `GameObject`s MUST be prefixed `DINOForge_` so existing canvas walkers skip them.
- Logo / overlay `Image`s MUST have `raycastTarget = false` (Pattern #235).
- No Harmony patches on DINO's own ECS systems (ADR-016).

## Quality Gates (all must be green before v0.27.0 tag)

- `dotnet build src/DINOForge.sln -c Release` exits 0.
- `dotnet test src/DINOForge.sln` — all tests green; no regressions from v0.26.0.
- Both packs pass `PackCompiler validate` with 0 errors.
- In-game screenshot + external judge receipt in `docs/proof/judge-receipts/` for each story.
- `dinoforge status` reports all packs Active.
- CHANGELOG.md updated. All new public APIs carry XML doc comments.

## Child Stories and Sprint Order

| Story ID | Title | Sprint | Points |
|---|---|---|---|
| SW-001 | Native Mods Page Visible | 1 — Foundation | 8 |
| SW-002 | DINOForge Active Indicator | 1 — Foundation | 3 |
| SW-003 | Real Asset Bundles (kill #101) | 1 — Foundation | 13 |
| SW-004 | Loading Screen Takeover | 2 — Identity | 13 |
| SW-005 | Mod Brand Identity (SW + Modern) | 2 — Identity | 8 |
| SW-006 | Full UI/UX Reskin Engine | 2 — Identity | 21 |
| SW-007 | In-Game 2D Asset Takeover | 3 — Assets | 13 |
| SW-008 | Mod Asset Completeness (SW + Modern) | 3 — Assets | 13 |
| SW-009 | Blaster / Projectile Support | 4 — Mechanics | 8 |
| SW-010 | Naval Combat | 4 — Mechanics | 13 |
| SW-011 | Aerial Combat Complete | 4 — Mechanics | 13 |
| SW-012 | Audio Takeover | 4 — Mechanics | 8 |
| SW-013 | SW Space Combat (stretch) | 5 — Stretch | 13 |

**Total (excluding stretch):** 131 points  
**Total (including stretch):** 144 points

## Related Design Documents

- `docs/design/ui-ux-reskin-system.md`
- `docs/design/identity-starwars.md`
- `docs/design/identity-modern.md`
- `docs/design/loading-screen-system.md`
- `docs/design/main-menu-takeover.md`
- `docs/design/ingame-asset-takeover-spec.md`
- `docs/milestones/MILESTONE-M5-example-packs.md`
