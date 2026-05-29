# DINOForge — `warfare-starwars` Brand Identity

> **Mod:** Star Wars: Clone Wars total conversion for *Diplomacy is Not an Option*
> **Factions:** Galactic Republic vs. Confederacy of Independent Systems (CIS / Separatists)
> **Status:** Art-direction spec. All assets are **prompt-derived / original-geometry** under fair-use parody. **No copyrighted Lucasfilm/Disney assets, fonts, or audio are to be copied, ripped, or redistributed.** See [Legal & Licensing](#8-legal--licensing).

---

## 0. Design Thesis

The mod must read instantly as "Clone Wars era Star Wars RTS" from the title screen alone, while staying legally clean. We borrow the **grammar** of the franchise (gold-on-black wordmarks, perspective crawl, faction dichotomy of pristine-Republic vs. industrial-Separatist) rather than any specific protected mark.

### Reference total-conversions — what makes their branding cohesive

These are the gold standard for SW TCs; we emulate their *discipline*, not their assets:

- **Republic at War** (Empire at War mod) — Cohesion comes from a **single restrained palette per faction** and a consistent UI chrome that never fights the unit art. Loading screens use cinematic stills with a fixed lower-third caption bar. Lesson: *one chrome, applied everywhere.*
- **Thrawn's Revenge** (Empire at War mod) — Famous for a **faction-coded full UI reskin**: every panel, button, and cursor changes color/material between factions. Strong, legible faction iconography (crest in a fixed corner slot). Lesson: *faction identity is a system, not a logo.*
- **Galaxy at War / Star Wars: A Galaxy Divided** — Cohesion from **typographic restraint** (one display face for titles, one humanist sans for body) and heavy use of the gold-on-black "crawl" motif as connective tissue across menus and loading screens. Lesson: *the crawl aesthetic is the brand glue.*

**Our cohesion contract:** (1) one display face + one body face, used everywhere; (2) gold `#FFE81F` on near-black is the universal "title layer"; (3) every screen carries the active faction's crest in a fixed slot; (4) Republic = clean/curved/bright, CIS = industrial/angular/rusted. If an asset violates one of these four rules, it is off-brand.

---

## 1. Logo Concept — Main Menu Title

The main-menu title is the highest-stakes asset. It must evoke the iconic gold-on-black **opening-crawl / title-card** feel without using the trademarked Star Wars logotype.

**Universal rules for all options:**
- Primary fill: **Republic Gold `#FFE81F`** (the canonical bright "Star Wars yellow").
- Background: near-black `#05060A` with a subtle starfield + faint nebula (deep indigo `#0B1B3A`).
- The wordmark is the title of the mod, e.g. **"CLONE WARS"** as the dominant line with a smaller supertitle **"DINOFORGE PRESENTS"** and subtitle **"GALACTIC CONFLICT"**.
- Perspective: the wordmark recedes toward a vanishing point near top-center (the "crawl" tilt), ~18–25° pitch, with a soft gradient from full gold (bottom/near) to `#7A6E12` muted gold (top/far) to sell depth.
- Edge treatment: 2px dark bevel (`#1A1A05`) + faint outer glow (gold at 30% opacity, 12px blur).

### Option A — "Crawl Plate" (recommended default)
A horizontally-locked wordmark set in a heavy geometric face, tracked tight, all-caps, sitting on the perspective tilt. A thin gold rule (2px) underlines the main line and terminates in two small geometric end-caps: a **Republic cog** (left) and a **CIS hex** (right) — the dual-faction motif embedded directly in the lockup. Vanishing-point starfield behind.
- **Geometry for the art agent:** Canvas 1920×1080. Main line cap-height ≈ 220px at the bottom edge of text, scaling to ≈ 90px at the far (top) edge to fake perspective (apply a trapezoidal transform: bottom width 1400px, top width 1040px, vertical offset 360px). Supertitle 36px centered 60px above main line, letter-spacing 0.35em. Underline rule at y = main-line baseline + 28px. Cog endcap radius 26px (8 teeth), hex endcap circumradius 26px.

### Option B — "Split Crest Emblem"
A centered circular/octagonal emblem rather than a pure wordmark. Top half = Republic (gold cog ring on white-grey), bottom half = CIS (bronze hex lattice on rust), split by a hard diagonal lightning-seam in gold. Wordmark sits beneath the emblem on a flat (non-perspective) baseline. Better for square/app-icon reuse.
- **Geometry:** Emblem 512×512 centered at top-third. Outer ring: 12-tooth cog, OD 480px, ring thickness 40px, gold `#FFE81F`. Inner field split 50/50 on a 35° diagonal. CIS half overlaid with a 6px hex-grid lattice (`#C97B3C` at 40%). Wordmark below, 96px cap-height, flat baseline.

### Option C — "Holo-Projection"
The wordmark rendered as a flickering **hologram**: cyan-shifted gold (`#FFE81F` core, `#7FE0FF` edge fringe), with horizontal scanlines (1px, `#7FE0FF` at 15%, every 4px) and a faint projection cone fading from a point at the bottom. Most "in-universe" / hardware-diegetic; pairs with the holographic menu skin (§5).
- **Geometry:** Same trapezoidal perspective as A. Add scanline overlay layer, chromatic-aberration offset of ±2px on R/B channels, and a 20%-opacity gold projection-cone gradient triangle from (960,1080) fanning to the text bounds.

**Recommendation:** Ship **Option A** as the title card and **Option B** as the app/folder/installer icon and social card. Keep **Option C** as an alternate "boot/intro" splash.

---

## 2. Color System

All values sRGB hex. The two faction palettes are deliberately opposed: Republic is **high-key, cool, clean**; CIS is **low-key, warm, oxidized**.

### 2.1 Republic (Galactic Republic / Grand Army)
| Token | Hex | Use |
|---|---|---|
| `rep-white` | `#F5F7FA` | Clone-armor base white, panel fills |
| `rep-armor-grey` | `#C5CCD6` | Plastoid armor mid-grey, secondary surfaces |
| `rep-steel` | `#8A94A3` | Durasteel shadow, dividers |
| `rep-jedi-blue` | `#2D7DD2` | Primary accent, hyperdrive blue, links |
| `rep-saber-blue` | `#5FB3FF` | Glow/hover, lightsaber bloom |
| `rep-deep-navy` | `#0B1B3A` | Background nebula, dark panels |
| `rep-gold` | `#FFE81F` | **Brand gold** — titles, victory, crest |
| `rep-gold-muted` | `#B8A415` | Gold in shadow / far-perspective |

### 2.2 CIS (Confederacy of Independent Systems / Separatist droids)
| Token | Hex | Use |
|---|---|---|
| `cis-droid-tan` | `#C8A87A` | B1 battle-droid tan, base armor |
| `cis-bronze` | `#9A6B2F` | Bronze trim, droid joints |
| `cis-rust` | `#7A3B1E` | Oxidized hull, weathering, panels |
| `cis-rust-dark` | `#3E1E10` | Deep shadow, dark panels |
| `cis-sep-red` | `#C0392B` | **Separatist red** — primary accent, alerts, crest |
| `cis-ember` | `#E8642F` | Glow/hover, reactor ember |
| `cis-gunmetal` | `#33302C` | Buzz-droid gunmetal, dividers |
| `cis-hex-amber` | `#D99A3C` | Hex-lattice motif, holo-amber |

### 2.3 Neutral / UI Chrome (faction-agnostic shell)
| Token | Hex | Use |
|---|---|---|
| `ui-void` | `#05060A` | App background, behind starfield |
| `ui-panel` | `#10131A` | Default panel fill (90% opacity over void) |
| `ui-panel-edge` | `#2A3140` | Panel borders / bevels |
| `ui-text` | `#E6EAF0` | Primary body text |
| `ui-text-dim` | `#8B93A1` | Secondary/disabled text, tip captions |
| `ui-gold` | `#FFE81F` | Universal title/CTA accent (= rep-gold) |
| `ui-success` | `#3FB950` | Confirm/build-complete |
| `ui-warn` | `#D9A227` | Caution |
| `ui-danger` | `#C0392B` | Destroy/error (= cis-sep-red) |
| `ui-scanline` | `#7FE0FF` | Holographic scanline tint (15% opacity) |

**Contrast/accessibility:** `ui-text` on `ui-panel` ≈ 12:1 (AAA). Gold `#FFE81F` on `ui-void` ≈ 16:1. Never set gold text on `rep-white` (fails contrast) — gold is for dark backgrounds only.

---

## 3. Typography

**Do NOT use the trademarked "Star Wars" / "News Gothic"-derived franchise logotype font.** The following are real, freely-licensable faces that evoke the aesthetic.

| Role | Font | License | Why |
|---|---|---|---|
| **Display / Title** | **Orbitron** (Matt McInerney) | SIL OFL 1.1 (Google Fonts) | Geometric, futuristic, wide caps — reads as sci-fi title without infringing. Use for the logo wordmark fallback and big headers. |
| **Display alt** | **Saira / Saira Condensed** | SIL OFL 1.1 | Condensed industrial sans; good for the perspective-crawl plate and CIS panels (technical feel). |
| **Body / UI** | **Exo 2** | SIL OFL 1.1 | Humanist-geometric sans, excellent legibility at small sizes; the workhorse for buttons, tooltips, stats. |
| **Body alt** | **Titillium Web** | SIL OFL 1.1 | Clean, slightly technical; good Republic-side body face if a second voice is wanted. |
| **Crawl / lore caption** | **Saira Semi Condensed** + uppercase + 0.15em tracking | SIL OFL 1.1 | Mimics the justified-block crawl typesetting for tip/lore text. |
| **Monospace (debug/console/version)** | **JetBrains Mono** | SIL OFL 1.1 | For build version, console overlay, F9 telemetry. |

**Type scale (UI):** Title 96/72px • H1 48px • H2 32px • Body 18px • Caption 14px • Mono 13px. Line-height 1.35 body, 1.0 display. Letter-spacing: display 0.06em, all-caps captions 0.15em.

**Faction voice:** Republic headers in Orbitron (curved, optimistic). CIS headers in Saira Condensed (angular, industrial). Body stays Exo 2 for both to preserve the single-body-face cohesion rule.

**Licensing note:** All recommended fonts are **SIL OFL 1.1** — free for commercial and mod use, embeddable, redistributable *with their license file*. Bundle each font's `OFL.txt` in `packs/warfare-starwars/assets/fonts/<font>/`. Do not rename the fonts in a way that drops attribution. **Fair-use posture:** evoking a genre aesthetic with original fonts is non-infringing; replicating the actual Star Wars logotype is not — so we never ship a glyph-for-glyph clone of it.

---

## 4. Loading Screen

**Concept — "Hyperspace Transit Card":**
- Background: a hyperspace-streak field — radial gold/blue streaks emanating from a center vanishing point, motion-blurred (think: jump-to-lightspeed). Streaks tinted to the **active faction** (Republic = blue→white `#5FB3FF`→`#F5F7FA`; CIS = ember→rust `#E8642F`→`#7A3B1E`).
- Faction splash: the active faction's crest (cog or hex) large and centered-upper, at 25% opacity as a watermark, with the faction name in display face beneath.
- **Lower-third caption bar:** a fixed 1920×140 dark bar (`ui-panel` at 88%) pinned to the bottom, gold 2px top rule. Left = rotating **tip text**; right = a thin progress bar (gold fill on `ui-panel-edge` track) + percentage in JetBrains Mono.
- **Tip text style:** prefixed with a small cog/hex glyph and the label `// TRANSMISSION` in `ui-text-dim` 14px all-caps tracked, then the tip in Exo 2 18px `ui-text`. Tone: in-universe ("Republic gunships deploy clone squads faster than droids can reorganize — rush early." / "Separatist factories are cheap but fragile; mass production beats quality.").
- Variants: ship **3 base loading backgrounds** (Republic, CIS, Neutral/title) so the screen matches faction selection.

---

## 5. Menu Skin

Two themeable chrome variants driven by active faction; shared geometry, swapped material + accent.

**Shared structure:**
- Panels: rounded-2px corners, 1px `ui-panel-edge` border, inner top-edge highlight (1px at 8% white) for a beveled "durasteel plate" feel, drop shadow (0 4px 16px black 40%).
- Headers: a gold (or faction-accent) 2px underline rule with a notched corner cut (clip the top-right corner at 45° for the angular sci-fi look).

**Republic chrome — "Polished Durasteel + Holo":**
- Buttons: light brushed-metal gradient (`rep-armor-grey`→`rep-white`), `rep-jedi-blue` 1px border, **holographic hover** — fill shifts to `rep-saber-blue` at 18% with a 1px scanline overlay (`ui-scanline`). Pressed: inset shadow.
- Frames: clean curved corners, subtle blue inner glow on focus.
- Icon slots and dividers in `rep-jedi-blue`.

**CIS chrome — "Beveled Bronze + Rust":**
- Buttons: dark gunmetal base (`cis-gunmetal`) with a bronze (`cis-bronze`) beveled 2px frame, riveted corner dots (4 small `#000` circles), hover glows `cis-ember`. Weathered/scuffed texture overlay at 8% opacity.
- Frames: hard angular corners (no rounding), hex-lattice watermark in `cis-hex-amber` at 6%.
- Accent and danger states in `cis-sep-red`.

**Cursor:** custom 32×32 — Republic = blue holo-arrow, CIS = amber-bronze arrow. Hover/target state = faction crest reticle.

**Holographic treatment (both):** active selections and modal overlays get a 4px scanline pattern (`ui-scanline` 12%) + 2px chromatic-aberration fringe to read as a hologram (ties to logo Option C).

---

## 6. Iconography & Motifs

Recurring shapes used as connective tissue. All original geometry.

- **Republic Cog** — an 8- or 12-tooth circular gear ring, gold `#FFE81F` on transparent. The Republic's signature mark; appears as faction crest, button bullets, loading watermark, and the left endcap of the logo. Variants: solid (crest), outline (UI bullet), with an inner clone-helmet silhouette (hero/elite marker).
- **CIS Hex** — a hexagon with an inner hex-lattice (honeycomb of 7 small hexes), bronze/amber `#9A6B2F`/`#D99A3C`. The Separatist mark; faction crest, panel watermark, right endcap of the logo. Variants: solid, lattice, with an inner B1-droid-head silhouette.
- **Clone Helmet Silhouette** — a stylized T-visor helmet profile, used for Republic unit/portrait frames, veterancy pips, and the "player = Republic" indicator. Strictly an *original* silhouette (rounded dome, vertical T-slot visor), not a traced render.
- **Droid Head Silhouette** — elongated B1-style head (original geometry), CIS counterpart to the clone helmet.
- **Lightning Seam** — a jagged gold diagonal used to split Republic/CIS in VS screens and the Option B emblem.
- **Hyperspace Streaks** — radial motion lines; the universal "loading/transition" motif.
- **Saber Glow** — a soft additive bloom (blue Republic / red CIS) for selection highlights and ability cues.

**Placement rule (faction crest):** always the **top-left** corner of any faction-owned panel at 48×48, 100% opacity; as a 256×256 watermark at 8–25% on faction backgrounds. Consistent slot = instant faction recognition (Thrawn's Revenge lesson).

---

## 7. Asset Manifest

Everything needed for the mod to feel complete. Format defaults: **PNG-24 with alpha** for sprites/UI; **PNG/JPG** for full-bleed backgrounds; **SVG** for crests/icons (source) exported to PNG at listed sizes. Target dir: `packs/warfare-starwars/assets/branding/`.

### Logos & Title
- `logo-title-crawl.png` — main menu title (Option A). **3840×2160** + **1920×1080** PNG (transparent variant + on-starfield variant).
- `logo-emblem.svg` + `logo-emblem-512.png`, `-256.png`, `-128.png` — split-crest emblem (Option B), for icon/installer/social.
- `logo-holo-splash.png` — boot/intro splash (Option C), **1920×1080**.
- `app-icon.ico` (16/32/48/256) + `app-icon-512.png` — derived from emblem.
- `social-card.png` — **1200×630** (GitHub/share card).

### Backgrounds
- `bg-mainmenu.png` — starfield + nebula, **3840×2160** + **1920×1080**.
- `bg-loading-republic.png`, `bg-loading-cis.png`, `bg-loading-neutral.png` — **1920×1080** each (hyperspace transit).
- `bg-panel-tile-republic.png`, `bg-panel-tile-cis.png` — **512×512** seamless tile (durasteel / rusted hull).

### Faction Crests & Icons
- `crest-republic.svg` + PNG **512/256/128/48/24**.
- `crest-cis.svg` + PNG **512/256/128/48/24**.
- `icon-clone-helmet.svg` + PNG **128/48/24**.
- `icon-droid-head.svg` + PNG **128/48/24**.
- `icon-set-ui.svg` — sprite sheet of UI glyphs (build, attack, defend, gather, ability, veterancy pips), each **48×48**, exported as **`ui-icons-512.png`** atlas + JSON map.

### Buttons & UI Chrome (9-slice ready)
- `btn-republic-{normal,hover,pressed,disabled}.png` — **256×64**, designed for 9-slice (corner insets 16px).
- `btn-cis-{normal,hover,pressed,disabled}.png` — **256×64**, 9-slice.
- `panel-frame-republic.png`, `panel-frame-cis.png` — **512×512** 9-slice frame (notched corners).
- `tooltip-bg.png` — **256×128** 9-slice.
- `progressbar-track.png` + `progressbar-fill-{rep,cis}.png` — **256×16**.
- `cursor-republic.png`, `cursor-cis.png`, `cursor-reticle-{rep,cis}.png` — **32×32**.
- `divider-rule-gold.png` — **512×4**.

### Loading-Screen Furniture
- `loadbar-lowerthird.png` — **1920×140** caption bar (9-slice horizontally).
- `tips-republic.json`, `tips-cis.json` — tip-text content arrays (≥ 20 tips each), Exo 2 style per §4.

### Unit / Hero Portraits (frame + placeholders)
- `portrait-frame-republic.png`, `portrait-frame-cis.png` — **160×160** 9-slice (helmet/droid-head crest in corner).
- `portrait-clone-trooper.png`, `portrait-arc-trooper.png`, `portrait-jedi.png` — **256×256** (Republic).
- `portrait-b1-droid.png`, `portrait-b2-droid.png`, `portrait-droideka.png` — **256×256** (CIS).
- `portrait-placeholder-{rep,cis}.png` — **256×256** fallback.

### Fonts (bundled, OFL)
- `fonts/Orbitron/` (+ `OFL.txt`), `fonts/Saira/`, `fonts/Exo2/`, `fonts/JetBrainsMono/` — each with license file. WOFF2/TTF.

### Optional polish
- `vs-splash.png` — **1920×1080** Republic-vs-CIS pre-match splash with lightning-seam.
- `victory-republic.png`, `victory-cis.png`, `defeat.png` — **1920×1080** end screens.

---

## 8. Legal & Licensing

- **No copyrighted assets.** Do not extract, trace, or redistribute any Lucasfilm/Disney/EA art, models, fonts, audio, or the Star Wars logotype. All art here is **original geometry / prompt-derived** evoking the *genre*, produced under fair-use parody/transformative posture for a non-commercial fan mod.
- **Fonts:** all recommended faces are **SIL OFL 1.1** — bundle each `OFL.txt`, keep attribution, do not sell the fonts standalone.
- **Color `#FFE81F`:** a hex value is not protectable; using "Star Wars yellow" gold on black is genre convention, not a copied asset.
- **Trademark distance:** the mod must not imply official endorsement. Title-screen footnote: *"Unofficial fan project. Not affiliated with or endorsed by Lucasfilm Ltd. or The Walt Disney Company. Star Wars and related marks are trademarks of their respective owners."*
- **Art-agent prompt rule:** generation prompts MUST describe *original* shapes ("8-tooth gold gear ring", "hexagonal honeycomb crest", "T-visor helmet silhouette") and MUST NOT name or request reproductions of specific copyrighted characters, ships, or the official logo.

---

## 9. Quick Reference (cohesion checklist)

- [ ] One display face (Orbitron/Saira) + one body face (Exo 2) everywhere.
- [ ] Gold `#FFE81F` on near-black for all title-layer type.
- [ ] Active faction crest in the fixed top-left slot on every faction panel.
- [ ] Republic = clean/curved/bright/blue; CIS = angular/rusted/warm/red.
- [ ] Holographic scanline treatment on selections & modals.
- [ ] Every shipped art asset traces to an entry in §7 with correct dims/format.
- [ ] No copyrighted source; fonts ship with OFL.txt; trademark disclaimer present.
