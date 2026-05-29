# SW-005: Mod Brand Identity Applied (SW + Modern)

**Status**: Proposed
**AgilePlus WP State**: planned
**Sequence**: 5
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**AgilePlus Feature Slug**: epic-027-full-conversion
**Sprint**: 2 — Identity
**Story Points**: 8
**Priority**: P1
**File Scope**:
  - `packs/warfare-starwars/pack.yaml`
  - `packs/warfare-modern/pack.yaml`
  - `packs/warfare-starwars/assets/ui/`
  - `packs/warfare-starwars/assets/fonts/`
  - `packs/warfare-modern/assets/ui/`
  - `packs/warfare-modern/assets/fonts/`
  - `src/Runtime/UI/Theme/MainMenuReskinner.cs`
**Depends On**: [SW-006-P1]
**Requirements**: EPIC-027-FR-010, EPIC-027-NFR-005, EPIC-027-NFR-006, EPIC-027-NFR-007, EPIC-027-NFR-008, EPIC-027-NFR-015, EPIC-027-NFR-018, EPIC-027-NFR-020, EPIC-027-NFR-021

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

## Evidence Requirements

| Requirement ID | Evidence Type | Artifact Path Pattern | Transition Gate |
|----------------|---------------|-----------------------|-----------------|
| EPIC-027-FR-010 | ManualAttestation | `docs/proof/judge-receipts/SW-005-brand-sw.md` + `SW-005-brand-modern.md` (SW menu: deep-space bg, gold title, Republic palette; Modern: satellite bg, stencil title, amber palette — no vanilla DINO art visible) | Implementing → Validated |
| EPIC-027-NFR-005 | CiOutput | CI build log (Runtime csproj `netstandard2.0`; no direct TMPro compile refs in identity wiring) | Implementing → Validated |
| EPIC-027-NFR-006 | ManualAttestation | TMP font bundles built with Unity 2021.3.45f2 load without silent failure under BepInEx 5.4.x (log confirmation) | Implementing → Validated |
| EPIC-027-NFR-007 | CodeReview | No Harmony patch targets DINO ECS/UI types in SW-005 scope (grep of `[HarmonyPatch` in identity files) | Implementing → Validated |
| EPIC-027-NFR-008 | CodeReview | Logo/overlay GameObject named `DINOForge_ModLogo`; no unnamed injected objects (grep `new GameObject` in MainMenuReskinner for the logo path) | Implementing → Validated |
| EPIC-027-NFR-015 | CodeReview | `DINOForge_ModLogo` Image has `raycastTarget = false`; EventSystem guard precedes any GraphicRaycaster add (Pattern #235) | Implementing → Validated |
| EPIC-027-NFR-018 | CiOutput | New user-visible strings in SW-005 resolve through locale layer; non-English locale shows translated labels (i18n CI check) | Implementing → Validated |
| EPIC-027-NFR-020 | ManualAttestation | Full play session judge receipt per mod shows no native medieval 2D art on main menu (cross-references SW-005-brand receipts above) | Implementing → Validated |
| EPIC-027-NFR-021 | ManualAttestation | Faction emblems (Republic + CIS; Alliance + Enemy) visible in-game per mod; unit portraits present on spawnable units (screenshot per mod) | Implementing → Validated |
| SW-005 | SchemaValidation | `PackCompiler validate packs/warfare-starwars` and `packs/warfare-modern` exits 0; `ui_theme` block validates against `schemas/ui_theme.schema.json` with 0 errors | Implementing → Validated |
| SW-005 | ManualAttestation | `OFL.txt` present in every shipped font directory for both packs (code review + directory listing artifact) | Implementing → Validated |
| SW-005 | ReviewApproval | PR URL (auto-detected from WorkPackage.pr_url) | Validated → Shipped |
| SW-005 | CiOutput | GitHub Actions run URL (dotnet test green) | Implementing → Validated |

## Related

- `docs/design/identity-starwars.md`
- `docs/design/identity-modern.md`
- `docs/design/main-menu-takeover.md`
- SW-006 (ThemeEngine prerequisite)
- SW-003 (real asset bundles for font bundles)
