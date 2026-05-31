# Civis Audio Direction — soundscape + reactive SFX + adaptive score spec

> **Status:** Binding audio-direction guide (2026-05-30). Authority for the Audio Lead.
> Companion to [`docs/research/art-direction.md`](../research/art-direction.md) (visual
> identity this audio must match) and the kira/`bevy_kira_audio` plugin at
> [`clients/bevy-ref/src/audio.rs`](../../clients/bevy-ref/src/audio.rs).
>
> **Why this exists:** the existing `CivisAudioPlugin` is a correct but minimal skeleton —
> a single looping wind bed + 5 one-shot SFX on two channels. This guide specifies the full
> audio direction (biome-driven ambient layers, reactive event SFX, an adaptive emergent
> score, a UI sound language matching the cyan/gold identity, a CC0 sourcing plan, and a
> mixing/ducking architecture) so the implementer extends that plugin without re-architecting.
>
> **Planner stance:** this is design only. No code. File paths, layer models, hook points,
> acceptance criteria, and a sourcing checklist — nothing an engineer can copy-paste as
> implementation.

---

## 0. Design pillars (the bar to match)

The visual identity is **disciplined, layered, restrained** (art-direction §1: one hot accent
per composition, depth via layering, glow reserved for energy). Audio must read the same way:

1. **Layered, not loud.** Like the 3-stop steel gradient, the soundscape is built from
   thin cross-faded beds, never one flat loop. Nothing is a single "ambient.ogg."
2. **One sonic accent at a time.** The audio equivalent of "exactly one ember glow" —
   reactive stings and the score's lead voice must not pile up; ducking enforces this (§6).
3. **Emergent, not scripted.** Per the Emergence Charter, the score is *driven by sim state*
   (population trend, war, prosperity), not a fixed playlist. Music is a readout of the world,
   the way the event feed is.
4. **Cyan = structure/UI, Gold = positive/confirm** carries into sound: UI uses cool,
   crystalline clicks (cyan); positive sim outcomes (birth, build complete, tech, prosperity)
   resolve warm/major (gold). Hazard/alert uses the toxic-acid-green role — reserved, rare.
5. **Silence is green.** Existing behavior (missing file → warn + silent, app stays playable)
   is a locked invariant. Every layer must degrade to silence without breaking the app.
6. **CC0-only, local, no paid services.** All audio is CC0 / public-domain, sourced and
   committed locally. No streaming, no licensed libraries, no paid SaaS (repo policy).

---

## 1. The layer model (four-tier mix tree)

Audio is organized as **four tiers**, each a kira channel group so it can be ducked/muted
independently. This generalizes the current 2-channel design (`AmbientChannel` + `SfxChannel`)
into the full tree. Channels nest: tier volume × bus volume × clip volume.

```
MasterBus
├── AmbientBus        (tier 1 — environmental beds, always looping, cross-faded)
│   ├── WindBed
│   ├── WaterBed
│   ├── ForestBed
│   ├── WildlifeBed
│   └── WeatherBed    (rain / storm / snow-hush overlay)
├── ScoreBus          (tier 2 — adaptive emergent music, mood-driven stems)
│   ├── BaseStem      (drone / pad — always present, keyed to prosperity)
│   ├── RhythmStem    (pulse — population growth / building activity)
│   ├── TensionStem   (war / disaster — dissonant overlay)
│   └── LeadStem      (melodic voice — milestone / golden-age moments)
├── SfxBus            (tier 3 — reactive one-shot world events)
│   ├── LifeSfx       (birth / death)
│   ├── BuildSfx      (construction / demolition)
│   ├── CombatSfx     (battle clash / volley / siege)
│   ├── TechSfx       (research-complete chime)
│   └── DisasterSfx   (meteor / flood / quake / wildfire / storm / plague)
└── UiBus             (tier 4 — interface sound language)
    ├── ClickSfx      (cyan click / hover / select)
    ├── ConfirmSfx    (gold positive confirm)
    ├── CancelSfx     (cool dismiss)
    └── AlertSfx      (acid-green hazard notification)
```

**Why four tiers, not one big channel list:** ducking (§6) operates on *buses*, not clips —
e.g. a disaster ducks AmbientBus + ScoreBus together while DisasterSfx + TensionStem rise.
Mute toggles (accessibility / settings) map one slider per tier (Master, Ambient, Music, SFX,
UI), which is the conventional and expected player-facing control set.

### Tier 1 — Ambient soundscape (biome-driven, cross-faded by camera)

The single `ambient_wind.ogg` loop becomes a **blend of biome beds selected by what the camera
is looking at**. The sim already classifies terrain into 7 `Biome` variants
(`crates/watch/src/terrain.rs`: `DeepWater, Water, Sand, Grass, Forest, Stone, Snow`) and
weather into 4 `WeatherKind` (`crates/planet/src/weather.rs`: `Clear, Rain, Snow, Storm`).

**Selection model:** sample the biome distribution within the camera's ground footprint each
frame (or on a throttled cadence, e.g. 4 Hz), produce a normalized weight vector over the bed
set, and set each bed's target volume to its weight. Cross-fade toward targets with a short
time-constant (≈ 0.75–1.5 s) so panning the camera over a coastline glides wind→water rather
than cutting. Beds always loop; only their gain moves.

| Bed | Drives | Maps from |
|-----|--------|-----------|
| WindBed | open/high terrain, default floor | `Grass`, `Stone`, `Snow`, fallback (always ≥ small floor so silence never feels dead) |
| WaterBed | shoreline lap / open-water wash | `DeepWater`, `Water`, `Sand` (coast) |
| ForestBed | leaf rustle, canopy | `Forest` |
| WildlifeBed | birds (day) / insects (night) | `Grass` + `Forest` presence, gated by day/night + `SeasonKind` (silenced in `Winter`) |
| WeatherBed | rain / storm / snow-hush overlay | `WeatherKind` over the camera region; additive on top of biome beds |

**Diurnal/seasonal modulation:** WildlifeBed swaps day(birdsong)→night(crickets/owl) by a
time-of-day signal (already present via atmosphere/sun); silenced in `Winter`. This is the
audio counterpart of the art doc's golden-hour mood coupling.

### Tier 2 — Adaptive emergent score (sim-mood stems)

No fixed soundtrack. A **stem-layering / vertical-remix** model: 4 stems share a tempo/key and
are mixed in/out by continuous sim-mood signals so "music" *emerges* from the world's trend —
fully aligned with the Emergence Charter (music is not hardcoded; it is a readout).

| Stem | Rises with | Sim signal source |
|------|-----------|-------------------|
| BaseStem | always on; warmth tracks prosperity | aggregate economy health (`crates/economy`), total population |
| RhythmStem | population growth + build activity | birth-rate trend, `BuildSfx` density (recent construction) |
| TensionStem | war + active disasters | active battles (`Battle` events), live `DisasterKind` |
| LeadStem | milestone / golden-age | tech milestones (`Tech` events), sustained prosperity peak |

**Mood → mix mapping (continuous, not discrete states):** compute a small **MoodVector**
each evaluation tick — `{ prosperity, growth, tension, wonder }` in 0..1, each an
exponentially-smoothed readout of sim aggregates. Map each component to its stem's target gain;
cross-fade like the ambient beds. **Key/mode lean:** high prosperity/wonder → major-keyed
stems dominate (gold); high tension → TensionStem's dissonant/minor stem dominates (war). The
transition is gain-only between pre-rendered stems sharing key & tempo — no real-time DSP
pitch-shifting required (keeps it CC0-clip-friendly and cheap).

**Cadence:** evaluate mood on a slow timer (e.g. every 2–4 s of wall-clock), not per frame —
music should drift, not twitch. This timer is independent of the 4 Hz ambient sampler.

### Tier 3 — Reactive event SFX (sim → sound)

One-shots fired by sim events. Extends the existing `SfxKind` (`UiClick, Birth, Death,
Disaster, Build`). The sim already emits the exact events to hook (`EventFeedMessage3d` in
`crates/protocol-3d/src/lib.rs`: `Birth, Death, Tech, Battle, Disaster`), plus
`DisasterKind` (6 variants) for disaster-specific stings.

| New/kept SFX kind | Trigger (hook) | Sound character (palette role) |
|-------------------|----------------|-------------------------------|
| `Birth` (kept) | `EventFeedMessage3d::Birth` | soft warm gold rise |
| `Death` (kept) | `EventFeedMessage3d::Death` | low, brief, respectful — no jump-scare |
| `Build` (kept) | construction-complete (spectator `BuildingKind`) | wood/stone thunk + small gold confirm tail |
| `Tech` (new) | `EventFeedMessage3d::Tech` | bright crystalline cyan chime (discovery) |
| `Battle` (new) | `EventFeedMessage3d::Battle` | metal clash / volley; intensity-scaled volume |
| `Disaster` (kept, now per-kind) | `EventFeedMessage3d::Disaster` + `DisasterKind` | distinct per kind (below) |

**Per-disaster variants** (replace the single `disaster.ogg`, key off `DisasterKind`):
`Meteor` = high whistle→deep impact boom; `Flood` = surging water roar; `Quake` = sub-bass
rumble + debris; `Wildfire` = crackle + whoosh; `Storm` = wind gust + thunder; `Plague` =
low dread drone + sparse bell (least terrain-y, most ominous).

**Spatialization (optional, phase 2):** SFX with a world position may pan/attenuate by distance
from camera. Phase 1 ships 2D (non-positional) to match the current plugin; the event struct
should carry an optional world coord so positional audio is a later, additive upgrade.

**Throttling:** at scale, births/deaths/battles fire in bursts. The SFX drain must
**coalesce** — cap simultaneous instances per kind per frame (e.g. ≤ 3) and sum/clamp volume
rather than playing 200 birth chimes. This protects the "one accent at a time" pillar and the
mix headroom.

### Tier 4 — UI sound language (cyan/gold identity)

Maps the locked visual palette (art-direction §2) into a sound vocabulary. UI audio is the
most-heard layer, so it must be small, crisp, and consistent.

| UI sound | When | Palette role → character |
|----------|------|--------------------------|
| `Click` | button press, tool select | **cyan** — short, cool, glassy tick (matches the glass-HUD recipe) |
| `Hover` | focus / mouseover | cyan, quieter sibling of Click (≤ 50% gain) |
| `Confirm` | positive action (place, buy, accept) | **gold** — warm major two-note resolve |
| `Cancel` | dismiss, close, undo | neutral cool down-tick (cyan-deep) |
| `Alert` | hazard warning, invalid action, disaster onset notify | **acid-green** (`#5ECC38` role) — reserved, attention-grabbing, used sparingly like the toxic accent |

**Rule (from DINOForge discipline):** UI sounds never overlap the ScoreBus's LeadStem
emotionally — UI is functional/cool by default; only `Confirm` and milestone `Tech` get the
warm gold treatment, mirroring "one hot accent."

---

## 2. Sim hook map (where each layer wires in)

Single source of truth for the implementer — every layer's trigger and its existing code anchor.

| Layer | Hook point (existing) | Mechanism |
|-------|----------------------|-----------|
| Ambient beds | camera transform + `Biome` grid (`crates/watch/src/terrain.rs`) | 4 Hz footprint sampler → bed weight vector → cross-fade |
| WeatherBed | `WeatherKind` for camera region (`crates/planet/src/weather.rs`) | additive overlay gain |
| WildlifeBed gating | day/night (atmosphere/sun) + `SeasonKind` | gain gate |
| Score stems | economy/population aggregates + `Tech`/`Battle`/`Disaster` events | MoodVector on slow timer → stem gains |
| Birth/Death SFX | `EventFeedMessage3d::{Birth,Death}` (`crates/protocol-3d`) | `SfxEvent` write (coalesced) |
| Build SFX | construction-complete (spectator `BuildingKind`) | `SfxEvent` write |
| Tech SFX | `EventFeedMessage3d::Tech` | `SfxEvent` write |
| Combat SFX | `EventFeedMessage3d::Battle` | `SfxEvent` write, volume ∝ intensity |
| Disaster SFX | `EventFeedMessage3d::Disaster` + `DisasterKind` | per-kind `SfxEvent` write |
| UI SFX | egui/HUD interaction callbacks (`clients/bevy-ref` UI systems) | `SfxEvent` write |

The event-feed frame (`EventFeedFrame.events`) is the **primary sim→audio bridge**: the client
already receives it per tick; the audio system reads that same stream and emits `SfxEvent`s +
feeds the MoodVector. No new sim plumbing required — audio is a pure consumer of existing
broadcast state, keeping it additive and non-invasive.

---

## 3. Mixing / ducking architecture (kira)

`bevy_kira_audio` exposes per-channel volume/tween; the four-tier tree above maps to channel
(bus) groups. Ducking = momentarily lowering one bus's target volume with a tween while another
rises, then restoring.

**Default mix (linear, starting points — expose as a tunable `AudioMix` resource, not consts):**

| Bus | Default gain | Notes |
|-----|-------------|-------|
| Master | 1.0 | player slider |
| Ambient | 0.35 | matches current `AMBIENT_VOLUME` |
| Score | 0.30 | sits under ambient; music is atmosphere, not foreground |
| Sfx | 0.70 | events should read clearly |
| Ui | 0.55 | present but not fatiguing |

**Ducking rules (sidechain-style, tween in/out ≈ 150–400 ms):**

1. **Disaster onset:** Ambient −6 dB, Score → TensionStem-dominant; DisasterSfx plays at full;
   restore over ~2 s after the sting.
2. **Milestone / golden-age (Tech, golden age):** briefly lift LeadStem +, duck Ambient
   slightly so the moment lands; the gold "payoff" beat.
3. **UI modal / pause:** duck Ambient + Score (−6 to −9 dB), keep Ui full, so menu audio reads.
4. **Combat density:** scale TensionStem gain with active-battle count; no hard duck, a swell.
5. **Coalesce-clamp (SFX):** per §1 tier-3, cap concurrent same-kind one-shots and clamp summed
   gain to protect headroom and the one-accent pillar.

**Tweening:** all gain changes (bed cross-fades, stem remix, ducks) use kira volume tweens with
explicit time-constants — never instantaneous gain jumps (which click). Cross-fade beds with
equal-power curves where the impl supports it to avoid a mid-fade dip.

**Headroom:** target a peak budget so Ambient + Score + a burst of SFX never clips Master;
the coalesce-clamp + conservative bus defaults above are the primary guard.

**Accessibility:** five sliders (Master/Ambient/Music/SFX/UI) + a master mute. All persisted in
settings. Each tier independently mutable — a player can keep SFX + UI but silence music, etc.

---

## 4. CC0 audio sourcing plan + checklist

**Policy:** CC0 / public-domain only, committed locally as `.ogg` under
`clients/bevy-ref/assets/audio/` (the path the plugin already loads from). No paid libraries,
no streaming, no attribution-required licenses. Track provenance in a committed manifest
(`assets/audio/CREDITS.md`) listing source URL + license per file even though CC0 needs no
attribution — for auditability and reuse across the Phenotype org.

### Primary CC0 sources

| Source | Best for | License |
|--------|----------|---------|
| **Kenney.nl** (Game Assets) — "UI Audio", "Interface Sounds", "Impact Sounds", "RPG Audio" | UI clicks/confirm/cancel, build thunks, generic SFX | CC0 1.0 |
| **Freesound.org** (filter: License = Creative Commons 0) | ambient beds (wind/water/forest/birds), disaster booms/rumbles, thunder | CC0 1.0 (per-sound — verify each) |
| **OpenGameArt.org** (filter: CC0) | loopable ambient beds, music stems | CC0 (per-asset — verify) |
| **Sonniss GDC Game Audio Bundle** (the CC0-released portions) | high-quality nature/weather beds | check per-pack terms |
| **cc0-music / public-domain music archives** (e.g. CC0 on OpenGameArt, Musopen public-domain) | score stems (drone/pad/rhythm/lead sharing key+tempo) | CC0 / PD — verify |

> Verify-each rule: Freesound and OpenGameArt host mixed licenses. **Confirm CC0 per file**
> before committing; record it in `CREDITS.md`. When in doubt, drop the file.

### Sourcing checklist (per slot)

Ambient beds (5): WindBed, WaterBed, ForestBed, WildlifeBed (day + night), WeatherBed
(rain/storm/snow) — each a seamless **loop** (≥ 20 s, loop-point-clean, no obvious repeat).

Score stems (4): BaseStem, RhythmStem, TensionStem, LeadStem — **same key + tempo**, each a
seamless loop, mixable in any combination. Sourcing tip: prefer a single CC0 multitrack/stem
pack so the stems are guaranteed harmonically compatible; otherwise author/select to a fixed
key (e.g. A minor / C major pair) and BPM.

Event SFX: Birth, Death, Build, Tech (chime), Battle (clash/volley), + 6 disaster variants
(Meteor, Flood, Quake, Wildfire, Storm, Plague).

UI SFX: Click, Hover, Confirm, Cancel, Alert.

**Per-file acceptance gate:**
- [ ] License confirmed **CC0 / public-domain**, recorded in `assets/audio/CREDITS.md`
- [ ] Format `.ogg` (Vorbis), mono for SFX / stereo for beds+score, normalized to a consistent
      reference level (so the §3 mix table holds without per-file tweaking)
- [ ] Loops are loop-point-clean (no click/pop at the seam) for beds + stems
- [ ] Score stems share key + tempo (mix-compatible in any subset)
- [ ] One-shots are pre-trimmed (no leading silence) so triggers feel immediate
- [ ] Sits in the palette role (cyan-cool UI, gold-warm positive, acid-green alert)
- [ ] File present at the `AudioFiles`-mapped path; absence still warns-and-silences (invariant)

---

## 5. Functional requirements (FR-CIV-AUDIO-*)

Traceable requirements for the Audio Lead. Each maps to a layer/section above.

| ID | Requirement | Acceptance |
|----|-------------|------------|
| **FR-CIV-AUDIO-001** | Four-tier bus tree (Ambient/Score/Sfx/Ui under Master) with independent volume control | 5 player sliders + master mute; each tier independently mutable; persisted in settings |
| **FR-CIV-AUDIO-002** | Biome-driven ambient beds cross-fade by camera location | Panning across a coastline glides wind→water (no hard cut); bed gains track the `Biome` footprint weight vector at ≥ 4 Hz |
| **FR-CIV-AUDIO-003** | Weather + diurnal/seasonal modulation of ambient | WeatherBed follows `WeatherKind`; WildlifeBed swaps day/night and silences in `Winter` |
| **FR-CIV-AUDIO-004** | Adaptive emergent score from MoodVector stems | 4 stems remix by `{prosperity, growth, tension, wonder}`; war pushes TensionStem, prosperity pushes major stems; gain-only, slow cadence (no twitch) |
| **FR-CIV-AUDIO-005** | Reactive event SFX for Birth/Death/Build/Tech/Battle/Disaster | each `EventFeedMessage3d` variant triggers its mapped SFX; disasters branch per `DisasterKind` (6 variants) |
| **FR-CIV-AUDIO-006** | SFX coalescing / clamp under event bursts | ≤ N concurrent same-kind one-shots per frame; summed gain clamped; no headroom clipping during birth/battle storms |
| **FR-CIV-AUDIO-007** | UI sound language matching cyan/gold/acid identity | Click/Hover = cyan; Confirm = gold; Cancel = cool; Alert = acid-green, reserved/rare |
| **FR-CIV-AUDIO-008** | Bus ducking (disaster, milestone, UI-modal, combat-swell) | ducks use tweens (150–400 ms), restore cleanly; no audible gain-jump clicks anywhere |
| **FR-CIV-AUDIO-009** | CC0-only sourcing with committed provenance manifest | every shipped clip is CC0/PD and listed in `assets/audio/CREDITS.md` with source + license |
| **FR-CIV-AUDIO-010** | Graceful silence invariant preserved | any missing clip warns-and-silences; app stays green/playable with zero audio files present |
| **FR-CIV-AUDIO-011** | Mix + cadence tunables exposed as resources, not consts | `AudioMix` (bus gains) + sampler/mood cadences live in editable resources for tuning/testing |
| **FR-CIV-AUDIO-012** | (Phase 2) Optional positional SFX | world-positioned events carry an optional coord; pan/attenuate by camera distance as an additive upgrade |

---

## 6. Phased WBS (implementation handoff)

DAG of build order for the implementing engineer. Audio is a pure consumer of existing
broadcast state, so most work is client-side and parallelizable.

| Phase | Task ID | Deliverable | Depends on |
|-------|---------|-------------|-----------|
| P1 Bus tree | A1 | Expand 2 channels → 4-tier bus tree + `AudioMix` resource (FR-001/011) | — |
| P1 Bus tree | A2 | 5-slider + mute settings wiring (FR-001) | A1 |
| P2 Ambient | B1 | Biome-footprint sampler → bed weight vector (FR-002) | A1 |
| P2 Ambient | B2 | Equal-power bed cross-fade on tween (FR-002) | B1 |
| P2 Ambient | B3 | Weather + day/night/season modulation (FR-003) | B2 |
| P3 SFX | C1 | Extend `SfxKind` (Tech, Battle, per-`DisasterKind`); event-feed → SFX bridge (FR-005) | A1 |
| P3 SFX | C2 | Coalesce/clamp burst handling (FR-006) | C1 |
| P3 SFX | C3 | UI sound-language hooks (cyan/gold/acid) (FR-007) | A1 |
| P4 Score | D1 | MoodVector readout from sim aggregates + events (FR-004) | C1 |
| P4 Score | D2 | 4-stem remix on slow cadence (FR-004) | D1 |
| P5 Mix | E1 | Ducking rules: disaster / milestone / UI-modal / combat (FR-008) | B2, C1, D2 |
| P6 Assets | F1 | CC0 sourcing per §4 checklist + `CREDITS.md` (FR-009) | parallel from start |
| P6 Assets | F2 | Verify graceful-silence invariant with 0 files (FR-010) | A1 |
| P7 (later) | G1 | Positional SFX upgrade (FR-012) | C1 |

---

## 7. Reference index

**Match (this audio must mirror):**
- [`docs/research/art-direction.md`](../research/art-direction.md) §1–2, §5 — discipline,
  palette roles (cyan/gold/acid), one-accent rule, golden-hour mood.

**Extend (do not re-architect):**
- [`clients/bevy-ref/src/audio.rs`](../../clients/bevy-ref/src/audio.rs) — `CivisAudioPlugin`,
  `SfxKind`, `SfxEvent`, `AudioFiles`/`AudioHandles`, the warn-and-silence invariant, the
  two-channel base that grows into the four-tier tree.

**Sim hooks (consume, do not modify):**
- `crates/watch/src/terrain.rs` — `Biome` (7 variants) for bed selection.
- `crates/planet/src/weather.rs` — `WeatherKind` / `SeasonKind` for WeatherBed + wildlife.
- `crates/protocol-3d/src/lib.rs` — `EventFeedMessage3d` (Birth/Death/Tech/Battle/Disaster),
  `EventFeedFrame` (the per-tick sim→audio bridge).
- `crates/engine/src/disasters.rs` — `DisasterKind` (6 variants) for per-disaster stings.
- `crates/economy/*` — aggregates feeding the MoodVector prosperity/growth signals.

**Assets:**
- `clients/bevy-ref/assets/audio/` — drop CC0 `.ogg` here; add `CREDITS.md` provenance manifest.
