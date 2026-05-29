# DINOForge — Themed Native Main Menu Takeover

**Status:** Design spec for full total-conversion main-menu override  
**Date:** 2026-05-28  
**Applies to:** DINOForge v0.27.0+ (implementation phase to follow `ui-ux-reskin-system.md` Phase 1)  
**Prereq reading:** `ui-ux-reskin-system.md` (ThemeEngine architecture), `identity-starwars.md`, `identity-modern.md`

---

## 0. Problem Statement

`MainMenuThemer` (iter-146) demonstrates the core mechanic: walk the main-menu Canvas via
reflection, rewrite TMP_Text, tint Colors, rewrite button labels. But the result is
cosmetic — it still *reads* as DINO with a tint. From the first frame after the splash screen,
a total-conversion mod should read as an entirely different game. That requires replacing:

1. The background art (not just tinting it)
2. The DINO wordmark / logo with the mod logo
3. The button-frame sprites (visual chrome around each button)
4. The audio track playing behind the menu
5. The button labels (partially done) and potentially adding/reordering menu items

This document specifies all five changes in enough detail for an implementation subagent, plus
two concrete before/after descriptions (Star Wars and Modern Warfare) and a complete asset
manifest per mod.

---

## 1. Main-Menu Element Inventory

The information below is synthesised from existing `ui-tree` session captures, the
`NativeMenuInjector` canvas-scan logs, and `MainMenuThemer.FindMainMenuCanvas()` behavior.
Any implementation MUST confirm element names against a live `dinoforge ui-tree` dump before
hardcoding paths.

### 1.1 Canvas identification

DINO's main menu lives on a Canvas whose name contains `MainMenu` and does NOT contain
`PrimeCanvas`. This is stable across versions seen in iter-143/146.

### 1.2 Element map

| Element | UGUI type | Vanilla object name (approx) | Notes |
|---|---|---|---|
| **Title / game name** | `TMP_Text` (TextMeshPro) | child of the main Canvas root | Contains "Diplomacy" or "Not an Option" — detected by `ReplaceTitle` regex |
| **Background image** | `UnityEngine.UI.Image` | `Window_adv` (from session logs) | Full-canvas-area Image; the largest `Image` by RectTransform area |
| **Button row container** | `RectTransform` with `VerticalLayoutGroup` | Unknown — confirm via ui-tree | Parent of all menu buttons |
| **New Game button** | `Selectable` (custom `MainMenuButton : Selectable`, NOT `UnityEngine.UI.Button`) | name contains "NewGame" or label "New Game" | DINO custom class — inject via reflection path |
| **Continue button** | same | "Continue" | Only active when a save exists |
| **Load Game button** | same | "LoadGame" | |
| **Special Missions button** | same | "SpecialMissions" | |
| **Options button** | same | "Options" | Repurposed as "Mods" by `NativeMenuInjector` |
| **Quit button** | same | "Quit" | |
| **Version text** | `TMP_Text` or legacy `Text` | bottom-corner element | Small, shows game build version |
| **Mod-injected "Mods" button** | `UnityEngine.UI.Button` (cloned) | `DINOForge_ModsButton*` | Injected by `NativeMenuInjector` — do NOT restyle as DINO vanilla |

### 1.3 What each element becomes under a theme

| Element | Vanilla state | Under total-conversion theme |
|---|---|---|
| Title TMP_Text | "Diplomacy is Not an Option" (DINO font) | Replaced with `ui_theme.surfaces.main_menu.title` text + primary color; font swapped to `font_heading` if supplied |
| Background Image (`Window_adv`) | DINO's medieval-castle / fantasy background art | Sprite replaced with `ui_theme.surfaces.main_menu.background_image` asset (full swap, not tint; see §3) |
| DINO logo / subtitle text | DINOForge/studio branding | Replaced with mod logo image or hidden; subtitle shows `ui_theme.surfaces.main_menu.subtitle` |
| Button Selectables | DINO dark-blue/red ColorBlock | ColorBlock (`normalColor`, `highlightedColor`, `pressedColor`) set to theme palette; button-frame Image.sprite swapped to `button_normal/hover/pressed` assets |
| Button labels | "New Game", "Continue", "Load Game", "Special Missions", "Options", "Quit" | Rewritten per `ui_theme.rewrites` map (already partially wired in `RewriteLabels`) |
| Version text | "v1.x.x" game version string | Prepend mod name/version: `"STAR WARS: Clone Wars v0.1.0"` — or hide with `alpha = 0` |
| Audio source (see §5) | DINO main-menu music track | AudioClip swapped to mod-supplied music |

---

## 2. Background Art Replacement

### 2.1 Current behavior

`TintBackground` in `MainMenuThemer` finds the largest `Image` by RectTransform area (which
corresponds to `Window_adv`) and sets `image.color` to a tinted semi-transparent Color. The
original sprite remains; only the tint changes. This produces a "DINO-with-a-color-filter"
look.

### 2.2 Full sprite replacement (recommended)

Instead of (or after) tinting, replace `Image.sprite` entirely:

```csharp
// Inside MainMenuReskinner (Phase 1 of ui-ux-reskin-system.md)
Image? bg = FindLargestImage(canvas);  // existing TintBackground logic
if (bg != null)
{
    Sprite? modBg = _assets.ResolveBackground(
        theme.Surfaces["main_menu"].BackgroundImage);   // "ui/menu_bg_starfield"
    if (modBg != null)
    {
        bg.sprite = modBg;
        bg.color  = Color.white;          // remove any prior tint — let the art speak
        bg.type   = Image.Type.Simple;
        bg.preserveAspect = false;        // stretch to fill canvas (desired for BG)
    }
    else if (theme.Palette.BackgroundTint != null)
    {
        // Graceful degrade: no sprite provided, fall back to tint
        ColorUtility.TryParseHtmlString(theme.Palette.BackgroundTint, out Color tint);
        bg.color = new Color(tint.r, tint.g, tint.b, 0.85f);
    }
}
```

### 2.3 Sprite source resolution

The `background_image` field in `ui_theme.surfaces.main_menu` is a standard `IThemeAssetResolver`
source reference, resolved in priority order (as specified in `ui-ux-reskin-system.md §3`):

1. Addressables key
2. `bundle:asset`
3. Bare bundle filename
4. **Raw PNG** — `packs/<id>/assets/ui/<name>.png` → `Texture2D.LoadImage` → `Sprite.Create(tex, new Rect(0,0,w,h), new Vector2(0.5f,0.5f))`

Raw PNG is the **lowest-friction path for menu backgrounds** because they are flat 2D images
without 9-slice requirements. No Unity 2021.3 AssetBundle build is needed. The raw-PNG path
should be the primary recommendation in the modder guide for backgrounds.

### 2.4 Overlay layer (alternative / complement)

An alternative implementation places a new full-canvas `Image` GameObject (DINOForge-owned,
`DontDestroyOnLoad`) at sort order -1 behind the main canvas. This avoids mutating DINO's
own GameObject and survives scene-change issues. Downside: requires a second canvas. Recommended
only if the in-place swap proves brittle across DINO updates.

---

## 3. Mod Logo Placement

### 3.1 The DINO logo element

DINO renders its game title as a `TMP_Text` element (the text "Diplomacy is Not an Option").
There is no separate SVG/sprite logo asset in the UI — the title IS the "logo" from a uGUI
perspective.

### 3.2 Strategy A — Text replacement (already wired, no new assets needed)

`ReplaceTitle` already replaces the TMP_Text content. Setting the `title` to the mod's name
(e.g. `"STAR WARS"`) with the primary color (gold `#FFE81F`) gives a text-only logo. This is
fully functional today.

To improve fidelity: also swap `TMP_Text.font` to the mod's heading font (`font_heading`
TMP_FontAsset, loaded from an AssetBundle). The heading font carries the display character of
the mod ("Orbitron" for Star Wars, "Bebas Neue" for Modern Warfare). Font swap via reflection:

```csharp
Type? tmpTextType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro");
PropertyInfo? fontProp = tmpTextType?.GetProperty("font");
object? modFont = _assets.ResolveFont(theme.Fonts.Heading);
if (fontProp != null && modFont != null)
    fontProp.SetValue(tmpTextComponent, modFont);
```

### 3.3 Strategy B — Sprite logo overlay (highest visual impact)

For a true logo image (the "Option A Crawl Plate" from `identity-starwars.md`), inject a new
`Image` GameObject as a child of the main-menu Canvas after all existing elements, parented to
the canvas root:

```csharp
// DINOForge-owned, created once per scene
GameObject logoGo = new GameObject("DINOForge_ModLogo", typeof(RectTransform));
logoGo.transform.SetParent(canvas.transform, false);
Image logoImg = logoGo.AddComponent<Image>();
logoImg.sprite = _assets.ResolveSprite(theme.Surfaces["main_menu"].Sprites["logo"]);
logoImg.preserveAspect = true;
RectTransform rt = logoGo.GetComponent<RectTransform>();
// Position: upper-center, 40% canvas width, anchored to top-center
rt.anchorMin = new Vector2(0.5f, 0.8f);
rt.anchorMax = new Vector2(0.5f, 0.97f);
rt.offsetMin = new Vector2(-384f, 0f);   // -/+ half of ~768px wide logo
rt.offsetMax = new Vector2( 384f, 0f);
logoImg.raycastTarget = false;           // must NOT block menu button clicks
```

Hide (or remove) the original TMP_Text title after placing the logo sprite:

```csharp
// Alpha to 0 rather than SetActive(false) to preserve layout flow
c.GetType().GetProperty("color")?.SetValue(tmpTitleComp, Color.clear);
```

**Named sprite slot:** `main_menu.sprites.logo` — resolves via `IThemeAssetResolver`.

### 3.4 Positioning notes

- Star Wars: logo upper-center, gold perspective-crawl plate, takes ~40% canvas width.
- Modern Warfare: logo upper-right or upper-left depending on the "SATELLITE PASS" background
  composition; the classified-stamp motif positioned alongside.
- Logo `raycastTarget = false` is mandatory — the canvas already has a GraphicRaycaster and
  any raycast-blocking image over the button area kills clicks (see Pattern #235 lesson).

---

## 4. Menu Music Takeover

### 4.1 How DINO plays menu music

DINO uses Unity's `AudioSource` component. In the main menu scene, at least one active
`AudioSource` plays a background music clip. The most reliable way to locate it is to scan all
active `AudioSource` components for those with `isPlaying = true` and `loop = true` and whose
clip's name does not match short SFX patterns (duration > 30s).

```csharp
AudioSource? FindMenuMusicSource()
{
    foreach (var src in UnityEngine.Object.FindObjectsOfType<AudioSource>())
    {
        if (src == null || !src.isPlaying) continue;
        if (src.loop && src.clip != null && src.clip.length > 30f)
            return src;
    }
    return null;
}
```

`FindObjectsOfType<AudioSource>()` is called from the **main thread** (scene-change callback /
RuntimeDriver pump). Never call it from the background Win32 thread.

### 4.2 Swap strategy

Once the `AudioSource` is found, swap its `clip` and restart playback:

```csharp
AudioClip? modClip = LoadMenuMusicClip(theme.Audio?.MenuMusicRef);
if (modClip != null && musicSource != null)
{
    musicSource.Stop();
    musicSource.clip  = modClip;
    musicSource.volume = theme.Audio?.MenuMusicVolume ?? 1.0f;
    musicSource.loop  = true;
    musicSource.Play();
}
```

### 4.3 Music asset loading

`AudioClip` must come from a Unity AssetBundle (built with Unity 2021.3.45f2). An `.ogg` or
`.wav` file cannot be decoded into an `AudioClip` at runtime on Mono without a third-party
library — AssetBundle is required for high-quality audio. Low-effort alternative: a short
`AudioClip` baked into the DINOForge Runtime's own embedded resources via `Resources.Load` from
a sub-bundle.

New `ui_theme` fields to add:

```yaml
audio:
  menu_music:   "audio/theme_mainmenu"   # AssetBundle key OR bundle:clip
  volume:       0.75                     # 0.0–1.0, defaults to 1.0
  fade_in_ms:   1500                     # optional crossfade duration
```

### 4.4 Fade crossfade (optional)

To avoid an abrupt cut, fade DINO's track out while fading the mod track in. Because
`Update()` never fires, use a coroutine-free approach: a `System.Threading.Timer` on the main
thread that reduces `musicSource.volume` stepwise over `fade_in_ms` milliseconds.
Alternatively (simpler): use `DOTween` if it ships with DINO (check assemblies), or accept the
hard cut as acceptable for a BepInEx mod.

---

## 5. Animated / Parallax Options

These are optional flair. Each adds implementation cost; order by impact/effort:

### 5.1 Slow background pan (low effort, high impact)

After replacing the background sprite, attach a simple panning component to the background
Image RectTransform. Because `Update()` never fires, implement via a background thread loop
that calls `UnityMainThreadDispatcher.Enqueue(() => { bg.rectTransform.anchoredPosition += panStep; })`.
A slow leftward drift (1–3 px/sec) on a background image 20% wider than the canvas is the
standard "living background" technique.

Implementation note: use the existing `Win32 bg thread` pattern from `NativeMenuInjector`
(the background watcher thread). Keep the drift step tiny to avoid visual jitter.

### 5.2 Starfield particle layer (Star Wars, medium effort)

Inject a `ParticleSystem` child on the main-menu Canvas parented at z=0, behind other elements,
emitting slow-drifting white dots (star simulation). Fully runtime-constructible:

```csharp
GameObject sfGo = new GameObject("DINOForge_Starfield");
sfGo.transform.SetParent(canvas.transform, false);
ParticleSystem ps = sfGo.AddComponent<ParticleSystem>();
var main = ps.main;
main.startColor = new Color(1, 1, 1, 0.7f);
main.startSize = 0.002f;
main.startLifetime = 8f;
main.maxParticles = 200;
// ... emission + shape configured via reflection-safe Component API
```

### 5.3 Scanline / hologram overlay (Star Wars — Option C logo feel)

A full-canvas Image (size = canvas) with a scanline texture (1920×4 tileable PNG with alternating
transparent and 5%-opacity white lines) set to `Image.Type.Tiled`. Alpha ~ 0.08. Parented above
the background, below all interactive elements. Pure visual without any per-frame update.

### 5.4 CSS-style CSS-free animated button hover (both mods)

For button hover feedback, rely on Unity's native `ColorBlock.highlightedColor` (already set
by `RestyleSelectables`). If the theme supplies `button_hover` sprite, the `Image.sprite` swap
on the button background Image via an `EventTrigger.PointerEnter` listener gives frame-perfect
feedback. No coroutine needed — `EventTrigger` fires synchronously on the main thread when
the pointer enters the element.

---

## 6. Label Rewrite Map Per Mod

Label rewrites live in `ui_theme.rewrites` (the `RewriteLabels` method already implements a
partial map). The complete maps per mod, declared in `pack.yaml`:

### 6.1 Star Wars: Clone Wars (`warfare-starwars`)

```yaml
ui_theme:
  rewrites:
    "New Game":           "New Campaign"
    "Continue":           "Resume Campaign"
    "Load Game":          "Load Campaign"
    "Special Missions":   "Clone Wars Missions"
    "Options":            "Options"          # keep — "Mods" button sits alongside it
    "Quit":               "Quit"             # keep — universal
    "Food":               "Rations"
    "Wood":               "Durasteel"
    "Stone":              "Tibanna Crystal"
    "Gold":               "Credits"
    "Population":         "Clone Cadets"
```

### 6.2 Warfare Modern (`warfare-modern`)

```yaml
ui_theme:
  rewrites:
    "New Game":           "Start Campaign"
    "Continue":           "Resume Mission"
    "Load Game":          "Load Save"
    "Special Missions":   "Special Operations"
    "Options":            "Options"
    "Quit":               "Exit"
    "Food":               "Supplies"
    "Wood":               "Materials"
    "Stone":              "Steel"
    "Gold":               "Budget"
    "Population":         "Personnel"
```

### 6.3 Implementation note on label re-enforcement

`RewriteLabels` is called once at theme-apply time. DINO's UiGrid may revert labels during
gameplay updates (the same issue addressed by `EnforceModsButtonText` for the Mods button).
The `ThemeEngine.Tick` re-scan approach (from `ui-ux-reskin-system.md §2`) handles this
generically by re-applying any label found in `rewrites` that does not match the themed value
on each Tick where a canvas appears. Cost is O(text-component-count) per scene, not per frame.

---

## 7. Concrete Before / After: Star Wars

### 7.1 Vanilla DINO main menu (baseline)

- **Background:** DINO fantasy landscape, warm medieval tones (browns, greens, castle silhouette)
- **Title text:** "Diplomacy is Not an Option" in DINO's display font, white/light color
- **Buttons:** Dark-blue rectangular buttons with red highlight on hover; labels New Game / Continue / Load Game / Special Missions / Options / Quit
- **Music:** DINO's medieval/orchestral main-menu theme
- **Logo:** The "DINO" text wordmark / no separate image logo

### 7.2 Star Wars: Clone Wars takeover

| Element | After |
|---|---|
| **Background** | Deep-space starfield: near-black `#05060A` with faint indigo nebula `#0B1B3A`; procedural star dots at various opacities |
| **Title / logo** | "STAR WARS" in Orbitron Bold, Republic Gold `#FFE81F`, perspective-tilt (Option A Crawl Plate per `identity-starwars.md §1`); subtitle "Clone Wars — Republic vs Separatists" in Saira Condensed, muted gold, below the main wordmark |
| **Button background** | Tinted near-black `#0A0A1A` (background_tint); buttons normalColor `#000000` → highlightedColor `#FFE81F 85%` → pressedColor `#C0392B`; if `btn_republic_normal.9.png` supplied: swap sprite to Republic durasteel frame |
| **Button labels** | New Campaign / Resume Campaign / Load Campaign / Clone Wars Missions / Options / Quit |
| **Music** | Republic-themed orchestral piece (original composition, not Lucasfilm); loaded from `packs/warfare-starwars/assets/audio/menu_theme.unity3d` |
| **Version text** | "Star Wars: Clone Wars v0.1.0 | DINOForge" |
| **Optional** | Slow starfield parallax drift (1 px/s rightward); scanline overlay at 8% opacity |

**Immediate read from boot:** player sees black space, gold angular title, Republic-colored button highlights. Reads as a sci-fi strategy game from frame 1.

---

## 8. Concrete Before / After: Modern Warfare

### 8.1 Vanilla DINO main menu (baseline)

(Same as §7.1)

### 8.2 Warfare Modern takeover

| Element | After |
|---|---|
| **Background** | Satellite-imagery composite: desaturated aerial texture, `#141A18` tint 70% overlay, tactical map grid lines at 5% opacity (1px `#EDE8DC` every 128×96px) — the "SATELLITE PASS" look per `identity-modern.md §4` |
| **Title / logo** | "WARFARE: MODERN" in Bebas Neue (stencil military caps), `#EDE8DC` — or Oswald Bold; a small "CLASSIFIED" red stamp image (`classified-stamp.png`) overlapping lower-right of the wordmark |
| **Button background** | `#141A18` base; buttons normalColor `#1C2B3A` → highlightedColor `#243347`/`#F5A623` border → pressedColor `#0D1720`; chamfered button frame sprites (45° corner cuts) if `menu-btn-default.9.png` supplied |
| **Button labels** | Start Campaign / Resume Mission / Load Save / Special Operations / Options / Exit |
| **Music** | Tension-atmosphere military ambient (original — no copyrighted material); fade-in over 1.5s from silence |
| **Version text** | "WARFARE: MODERN v0.2.0 | DINOForge" in Share Tech Mono styling |
| **Optional** | Very slow background pan on the satellite texture (1 px/s) to imply live satellite feed; `>>` glyph prefix on button hover |

**Immediate read from boot:** player sees a dark tactical map background, stencil military title, angular buttons. Reads as a modern military strategy game from frame 1.

---

## 9. Asset Manifest — Complete Menu Takeover Per Pack

### 9.1 Common structure (both mods)

All mod UI assets live under `packs/<pack-id>/assets/ui/`. Raw PNG is acceptable for all items
except fonts and 9-slice button frames (those require a Unity 2021.3 AssetBundle). The
`ui_theme` block in `pack.yaml` references these assets by the source-reference format.

The minimum set for a visually complete main-menu takeover (all others optional/graceful degrade):

| Slot key | Filename | Dims | Format | Notes |
|---|---|---|---|---|
| `main_menu.background_image` | `menu_bg.png` | 1920×1080 | PNG (RGBA) | Full-bleed background art |
| `main_menu.sprites.logo` | `menu_logo.png` | ~768×200 | PNG (RGBA) | Mod title logo; transparent BG; `raycastTarget = false` |
| `button_normal` | `btn_normal.9.png` | 256×64 | 9-slice bundle | Normal button frame |
| `button_hover` | `btn_hover.9.png` | 256×64 | 9-slice bundle | Hover/highlighted state |
| `button_pressed` | `btn_pressed.9.png` | 256×64 | 9-slice bundle | Pressed state |
| `button_disabled` | `btn_disabled.9.png` | 256×64 | 9-slice bundle | Disabled state |
| `font_heading` | `font_heading.unity3d` | — | AssetBundle | TMP_FontAsset for title/buttons |
| `audio.menu_music` | `menu_theme.unity3d` | — | AssetBundle | AudioClip, loop, >30s |

### 9.2 Star Wars: Clone Wars — full manifest addition

Add to `packs/warfare-starwars/pack.yaml` under `ui_theme`:

```yaml
ui_theme:
  title: "STAR WARS"
  subtitle: "Clone Wars — Republic vs Separatists"
  primary_color: "#FFE81F"
  secondary_color: "#000000"
  accent_color: "#C0392B"
  text_color: "#FFE81F"
  background_tint: "#0A0A1A"

  fonts:
    heading:  "font_heading"          # Orbitron Bold TMP_FontAsset
    primary:  "font_body"             # Exo 2 TMP_FontAsset
    fallback_to_default: true

  audio:
    menu_music: "audio/menu_theme"
    volume: 0.8
    fade_in_ms: 2000

  surfaces:
    main_menu:
      title: "STAR WARS"
      subtitle: "Clone Wars — Republic vs Separatists"
      background_image: "ui/menu_bg"
      sprites:
        logo:            "ui/menu_logo"
        button_normal:   "ui/btn_republic_normal"
        button_hover:    "ui/btn_republic_hover"
        button_pressed:  "ui/btn_republic_pressed"
        button_disabled: "ui/btn_republic_disabled"

  rewrites:
    "New Game":         "New Campaign"
    "Continue":         "Resume Campaign"
    "Load Game":        "Load Campaign"
    "Special Missions": "Clone Wars Missions"
```

**Asset files to create** under `packs/warfare-starwars/assets/ui/`:

| File | Description |
|---|---|
| `menu_bg.png` | Deep-space starfield + indigo nebula — 1920×1080 |
| `menu_logo.png` | "STAR WARS" gold perspective-crawl plate per Option A — ~768×200 RGBA |
| `btn_republic_normal.9.png` | Durasteel brushed-metal frame with `rep-jedi-blue` 1px border |
| `btn_republic_hover.9.png` | Holo-blue fill + scanline overlay |
| `btn_republic_pressed.9.png` | Inset shadow, `rep-gold` border |
| `btn_republic_disabled.9.png` | Low-alpha desaturated frame |
| `font_heading.unity3d` | AssetBundle: Orbitron Bold TMP_FontAsset (OFL 1.1) + OFL.txt |
| `font_body.unity3d` | AssetBundle: Exo 2 Regular TMP_FontAsset (OFL 1.1) |
| `menu_theme.unity3d` | AssetBundle: AudioClip, original Republic-orchestral composition |

### 9.3 Warfare Modern — full manifest addition

Add to `packs/warfare-modern/pack.yaml` under `ui_theme`:

```yaml
ui_theme:
  title: "WARFARE: MODERN"
  subtitle: "Western Alliance vs Classic Enemy"
  primary_color: "#F5A623"
  secondary_color: "#1C2B3A"
  accent_color: "#CC2929"
  text_color: "#EDE8DC"
  background_tint: "#141A18"

  fonts:
    heading:  "font_heading"     # Bebas Neue or Oswald Bold TMP_FontAsset
    primary:  "font_body"        # Barlow Condensed TMP_FontAsset
    monospace: "font_mono"       # Share Tech Mono TMP_FontAsset
    fallback_to_default: true

  audio:
    menu_music: "audio/menu_theme"
    volume: 0.7
    fade_in_ms: 1500

  surfaces:
    main_menu:
      title: "WARFARE: MODERN"
      subtitle: "Western Alliance vs Classic Enemy"
      background_image: "ui/menu_bg"
      sprites:
        logo:            "ui/menu_logo"
        button_normal:   "ui/btn_alliance_normal"
        button_hover:    "ui/btn_alliance_hover"
        button_pressed:  "ui/btn_alliance_pressed"
        button_disabled: "ui/btn_alliance_disabled"

  rewrites:
    "New Game":         "Start Campaign"
    "Continue":         "Resume Mission"
    "Load Game":        "Load Save"
    "Special Missions": "Special Operations"
    "Quit":             "Exit"
```

**Asset files to create** under `packs/warfare-modern/assets/ui/`:

| File | Description |
|---|---|
| `menu_bg.png` | Satellite-pass aerial composite, dark tactical grid — 1920×1080 |
| `menu_logo.png` | "WARFARE: MODERN" Bebas Neue stencil + classified stamp overlay — ~480×96 RGBA |
| `btn_alliance_normal.9.png` | Chamfered-corner dark-navy frame, `#4A5240` 1px border |
| `btn_alliance_hover.9.png` | `#243347` fill, `#F5A623` 1px border |
| `btn_alliance_pressed.9.png` | `#0D1720` fill, `#F5A623` 2px border |
| `btn_alliance_disabled.9.png` | `#141A18` fill, `#2E3528` 1px border |
| `font_heading.unity3d` | AssetBundle: Bebas Neue or Oswald Bold TMP_FontAsset (OFL 1.1) |
| `font_body.unity3d` | AssetBundle: Barlow Condensed TMP_FontAsset (OFL 1.1) |
| `font_mono.unity3d` | AssetBundle: Share Tech Mono TMP_FontAsset (OFL 1.1) |
| `menu_theme.unity3d` | AssetBundle: AudioClip, original military-ambient composition |

---

## 10. Implementation Sequence (ordered by impact / effort ratio)

This builds on `ui-ux-reskin-system.md` Phase 1 (MainMenuReskinner), which must land first.

### Step 1 — Background art swap (highest impact, lowest effort)

Extend `MainMenuReskinner.Apply()` to call sprite replacement (§2.2) before the existing tint
fallback. This requires:
- `IThemeAssetResolver.ResolveBackground(sourceRef)` for raw-PNG path (new overload, uses
  `Texture2D.LoadImage` → `Sprite.Create`).
- Test: supply a solid-color 1920×1080 PNG as `menu_bg.png`; verify background is replaced,
  not just tinted. Screenshot diff as proof.

No new schema fields required — `surfaces.main_menu.background_image` is already in the
`ui-ux-reskin-system.md` schema. Only the C# resolver path needs wiring.

### Step 2 — Mod logo injection (high impact, low-medium effort)

Inject the `DINOForge_ModLogo` Image GameObject (§3.3) into the main-menu Canvas during
`MainMenuReskinner.Apply()`. After injection, zero the alpha on the original TMP_Text title
element (§3.2). This requires:
- `IThemeAssetResolver.ResolveSprite("main_menu.sprites.logo")` — already covered by existing
  sprite resolution path.
- The logo Image must NOT block raycasts (`raycastTarget = false`).
- Test: screenshot shows mod logo in upper-center; Mods button and all other buttons still
  clickable.

### Step 3 — Menu music takeover (high impact, medium effort)

Add `audio` block to schema (`schemas/ui_theme.schema.json`) and SDK model. Implement
`FindMenuMusicSource()` (§4.1) in `MainMenuReskinner`. Swap `AudioClip` on scene load (§4.2).
Requires AssetBundle with AudioClip. Modder can test with a short silent clip first.

### Step 4 — Button sprite frames (medium impact, medium effort, requires Unity 2021.3)

Swap button-frame sprites via the slot-based mechanism (§2 of `ui-ux-reskin-system.md`). Each
button's child `Image` that represents the background/frame gets its sprite set to
`button_normal/hover/pressed/disabled`. 9-slice metadata preserved only via AssetBundle import.
This is the phase that requires actual art production (the 4-state button sprite sets).

---

## 11. Schema Changes Required

Add to `schemas/ui_theme.schema.json` (new fields):

```json
"audio": {
  "type": "object",
  "properties": {
    "menu_music":  { "type": "string" },
    "volume":      { "type": "number", "minimum": 0, "maximum": 1 },
    "fade_in_ms":  { "type": "integer", "minimum": 0 }
  }
},
"surfaces": {
  "properties": {
    "main_menu": {
      "properties": {
        "background_image": { "type": "string" },
        "title":            { "type": "string" },
        "subtitle":         { "type": "string" },
        "sprites": {
          "type": "object",
          "properties": {
            "logo":            { "type": "string" },
            "button_normal":   { "type": "string" },
            "button_hover":    { "type": "string" },
            "button_pressed":  { "type": "string" },
            "button_disabled": { "type": "string" }
          }
        }
      }
    }
  }
}
```

---

## 12. Runtime Constraints Checklist

All implementation MUST respect these constraints (from CLAUDE.md + MEMORY.md):

- `MonoBehaviour.Update()` NEVER fires — all reskin logic is event-driven (scene-change callback
  or RuntimeDriver pump). No per-frame style reconciliation.
- `Resources.FindObjectsOfTypeAll` MUST be called from the **main thread only** — never from
  the Win32 background watcher thread.
- `FindObjectsOfType<AudioSource>()` — same constraint, main thread only.
- Runtime DLL stays `netstandard2.0` — no direct TMPro / Addressables compile references.
  Everything via reflection or `Type.GetType`.
- Injected GameObjects (`DINOForge_ModLogo`, etc.) MUST be named with `DINOForge_` prefix so
  `TintBackground` and other walkers skip them (the existing guard `if (img.gameObject.name.Contains("DINOForge")) continue` already handles this).
- Logo Image `raycastTarget = false` — mandatory. A transparent Image over buttons with
  `raycastTarget = true` kills all button clicks silently (Pattern #235 lesson).
- AssetBundles containing `TMP_FontAsset` or `AudioClip` MUST be built with **Unity 2021.3.45f2**
  (CLAUDE.md "Asset Bundle Creation") — bundles from other versions fail silently.
- Pattern #232: never log to disk in the main-menu reskin hot path; use BepInEx logger only.
