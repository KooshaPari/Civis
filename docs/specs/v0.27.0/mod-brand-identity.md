# SW-005: Mod Brand Identity Applied (SW + Modern)

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 2 — Identity
**Story Points**: 8
**Priority**: P1

---

## User Story

As a **mod player**, I want the `warfare-starwars` and `warfare-modern` packs to apply their
defined logo, color palette, and typography to every UI surface — so that the first frame of
the main menu reads as a completely different game, not DINO with a tint.

## Background

Design specs for both mods are complete:
- `docs/design/identity-starwars.md` — Republic Gold `#FFE81F`, Orbitron/Exo2 typography,
  clone-cog / CIS-hex iconography, legal/OFL font guidance.
- `docs/design/identity-modern.md` — Olive Drab / NATO Navy palette, Bebas Neue / Barlow
  Condensed, tactical crosshair / stencil motifs.
- `docs/design/main-menu-takeover.md` — concrete before/after per mod, full asset manifests,
  label rewrite maps, logo injection strategy.

This story produces the identity assets and wires them into both pack manifests so the
ThemeEngine (SW-006) can apply them. SW-006 must land in the same sprint.

## Acceptance Criteria

### Scenario 1 — Star Wars: Clone Wars main menu identity

**Given** `warfare-starwars` is the active total-conversion pack,
**When** the player reaches the main menu,
**Then**:
- Background is deep space (near-black `#05060A`, indigo nebula) — not DINO's castle art.
- Title reads "STAR WARS" or "CLONE WARS" in Orbitron Bold, Republic Gold `#FFE81F`.
- Button hover color is gold, not DINO's red.
- Button labels read "New Campaign", "Resume Campaign", "Clone Wars Missions".
- Version line reads "Star Wars: Clone Wars v{version} | DINOForge".
- External judge screenshot confirms the screen reads as a Star Wars RTS, not a medieval game.

### Scenario 2 — Warfare Modern main menu identity

**Given** `warfare-modern` is the active total-conversion pack,
**When** the player reaches the main menu,
**Then**:
- Background is the "SATELLITE PASS" tactical aerial-map composite.
- Title reads "WARFARE: MODERN" in Bebas Neue or Oswald Bold, buff white `#EDE8DC`.
- Button hover color is amber `#F5A623`.
- Button labels read "Start Campaign", "Resume Mission", "Special Operations", "Exit".
- External judge screenshot confirms the screen reads as a modern military strategy game.

### Scenario 3 — Asset manifests validated

**Given** the asset pipeline has run for both packs,
**When** `PackCompiler validate packs/<id>` runs,
**Then** 0 errors and 0 dangling asset references.

### Scenario 4 — No vanilla DINO identity bleeds through

**Given** either total-conversion pack is active,
**When** the player is on the main menu,
**Then** no DINO medieval-fantasy text, castle background, or default-red hover color is visible.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | Both packs declare a complete `ui_theme` block in `pack.yaml` (palette, fonts, rewrites, surfaces.main_menu). |
| F-02 | Background art is replaced via `Image.sprite` swap, not color tint. |
| F-03 | Mod logo injected at `DINOForge_ModLogo` with `raycastTarget = false`. |
| F-04 | Original DINO title TMP_Text alpha set to 0 when mod logo is active. |
| F-05 | Label rewrites applied per `docs/design/main-menu-takeover.md §6`. |
| F-06 | All shipped fonts are OFL-licensed; each font dir contains `OFL.txt`. |
| F-07 | No copyrighted Lucasfilm / Disney / EA assets. SW title includes non-endorsement disclaimer. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Menu identity application completes within one `ThemeEngine.Tick()` call (< 16 ms). |
| N-02 | Background PNG loads via `File.ReadAllBytes` + `Texture2D.LoadImage` on the main thread. |
| N-03 | All assets reside under `packs/<id>/assets/ui/` and `packs/<id>/assets/fonts/`. |

## Minimum Asset Manifest

### warfare-starwars (`packs/warfare-starwars/assets/`)

- `ui/menu_bg.png` — 1920×1080 starfield + indigo nebula
- `ui/menu_logo.png` — ~768×200 RGBA crawl-plate logo (Option A per identity-starwars.md §1)
- `ui/btn_republic_{normal,hover,pressed,disabled}.9.png` — 256×64 9-slice durasteel frames
- `fonts/Orbitron/` + `OFL.txt`, `fonts/Exo2/` + `OFL.txt`

### warfare-modern (`packs/warfare-modern/assets/`)

- `ui/menu_bg.png` — 1920×1080 satellite-pass tactical map
- `ui/menu_logo.png` — ~480×96 RGBA stencil logo + classified stamp
- `ui/btn_alliance_{normal,hover,pressed,disabled}.9.png` — 256×64 chamfered frames
- `fonts/BebasNeue/` + `OFL.txt`, `fonts/BarlowCondensed/` + `OFL.txt`, `fonts/ShareTechMono/` + `OFL.txt`

## Engine Quirks / Dependencies

- Background PNGs are raw-file loaded — no AssetBundle required for flat images.
- TMP_FontAsset for runtime use requires a Unity 2021.3.45f2-built AssetBundle (tracked in SW-003).
- Depends on ThemeEngine Phase 1 (MainMenuReskinner) from SW-006.
- `IThemeAssetResolver.ResolveBackground` raw-PNG path must be implemented first.
- Logo Image must not block raycasts (Pattern #235).

## Definition of Done

- [ ] Both packs have complete `ui_theme` blocks validated by `PackCompiler validate`.
- [ ] Background PNGs, logos, and button frames present and referenced correctly.
- [ ] External judge receipt: SW menu reads as Star Wars; Modern reads as military.
- [ ] No vanilla DINO UI visible in either screenshot.
- [ ] OFL.txt present for every shipped font.
- [ ] `dotnet test` green.

## Related

- `docs/design/identity-starwars.md`
- `docs/design/identity-modern.md`
- `docs/design/main-menu-takeover.md`
- SW-006 (ThemeEngine prerequisite)
- SW-003 (real asset bundles for font bundles)
