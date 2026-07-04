# Sound & Music Emergence — Design Spec

> **Status:** Design / planner spec. No code. Emergence contracts, scale/rhythm/timbre
> derivation rules, integration points, acceptance criteria, and a phased WBS only.
>
> **Scope:** This document *narrows and binds* the §2.5 (E6) decision in
> [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) — "Sound/music EMERGES
> from culture, materials, and agent emotional state." It is the missing connective layer
> between [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) (culture vector),
> [`docs/design/LANGUAGE_EMERGENCE.md`](LANGUAGE_EMERGENCE.md) (drift dynamics model),
> [`docs/design/SENTIENCE_PSYCHE.md`](SENTIENCE_PSYCHE.md) (`Mood { valence, arousal }`), the
> voxel [`MaterialDef`](../../crates/voxel/src/material.rs) (density, viscosity, etc.), and the
> existing audio scaffold at [`crates/audio`](../../crates/audio/) + the binding visual /
> acoustic guide [`docs/design/audio-direction.md`](audio-direction.md).
>
> **Charter anchor:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) —
> *No `Music` object, no `Scale` table, no `Genre` enum authored anywhere. Conventions are
> measured patterns over the substrate.*
>
> **FR namespace:** `FR-CIV-MUSIC-*` (this doc), complementary to the already-bound
> `FR-CIV-AUDIO-*` (mixing/asset rules in `audio-direction.md`).
>
> **Companions (must read in this order):**
> 1. [`emergence-charter.md`](../guides/emergence-charter.md) — discipline + boundaries
> 2. [`civ-culture-emergent.md`](civ-culture-emergent.md) — the culture vector that feeds scale & motif
> 3. [`SENTIENCE_PSYCHE.md`](SENTIENCE_PSYCHE.md) — the `Mood` vector that drives expressivity
> 4. [`audio-direction.md`](audio-direction.md) — the four-tier mix tree this music rides on
> 5. [`RND-007-adaptive-music-kira.md`](../research/RND-007-adaptive-music-kira.md) — kira + fundsp substrate notes

---

## 0. Governing principle: music is an emergent measure, not an authored mode

Per the Emergence Charter §"Default = emerge," there is **no authored `Scale`, `Genre`,
`Instrument`, `Tempo`, `Key`, or `Tradition` enum** anywhere in Civis. What plays is a
**measured, derived pattern** over the substrate — culture drift × material availability ×
collective mood — the same way polity, law, language, and architecture styles emerge.

Concretely, the system has three layers and **each is a readout**, never a setter:

| Layer | What it is | Reads from | Writes to |
|-------|-----------|-----------|-----------|
| **`MusicalTradition`** (idiom-level) | A derived `{scale, temperament, motif_skeleton, timbre_palette}` tuple for one population cluster | culture trait vector (4 axes), contact graph, era | musical event stream (notes/intervals/densities) consumed by the synth |
| **Instrument/Material gates** (timbre-level) | Which physical *kinds* of sounds a population can produce, gated by the materials present in their territory | `MaterialDef { density, viscosity, porosity, hardness proxies }` aggregated by tile presence | which synth voice archetypes the idiom can choose from |
| **Emotional modulation** (expressivity-level) | Continuous mood vector shaping tempo, dynamics, ornamentation, dissonance | `Mood { valence, arousal }` per-civ aggregate, recent event pressure | per-note envelope, accent pattern, swing, dissonance injection |

The synth (kira + fundsp + procedural audio) is a **dumb speaker** — it has no knowledge of
culture, materials, or mood. It receives a stream of `{voice, pitch, velocity, ornament}`
events and plays them. All intelligence is in the derivation chain above. This is the
discipline rule from the charter, applied to music: **no music code reads "genre" or "scale";
it reads `CultureProfile`, `MaterialDef`, and `Mood`.**

The sound *itself* — physics, mixing, ducking, asset pipeline — is already bound by
`audio-direction.md` and the `crates/audio` substrate. This document is exclusively about
**what music is generated** and **how it is derived**.

---

## 1. The derivation chain (three cascades, one feed)

```
                       ┌───────────────────────────────────────────┐
                       │  Substrate (Layer-0 — hardcoded laws)       │
                       │  • CultureProfile { traits, language }      │
                       │  • MaterialDef registry (voxel/material.rs)  │
                       │  • Psyche { Mood{ valence, arousal } }       │
                       │  • EventFeedMessage3d (Birth/Death/...)      │
                       └───────────────┬───────────────────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              ▼                        ▼                        ▼
   ┌────────────────────┐  ┌────────────────────────┐  ┌─────────────────────┐
   │ A. MusicalTradition│  │ B. Material → Timbre   │  │ C. Mood → Expressivity│
   │ (idiom)            │  │ (voice availability)   │  │ (real-time shaping)   │
   │  scale             │  │  metal → idiophone     │  │  valence → mode lean  │
   │  temperament       │  │  wood  → marimba/lamell │  │  arousal  → tempo    │
   │  motif_skeleton    │  │  hide  → membrane      │  │  event_pressure      │
   │  ornament_density  │  │  bone  → flute/whistle │  │  → accent + tension  │
   └─────────┬──────────┘  └──────────┬─────────────┘  └──────────┬──────────────┘
             │                        │                            │
             └────────────────────────┼────────────────────────────┘
                                      ▼
                          ┌───────────────────────────┐
                          │  Note / Event Stream       │
                          │  {voice, pitch, vel, orn}  │
                          │  ──────► kira + fundsp     │
                          └───────────────────────────┘
```

Three derivation cascades (A, B, C) converge into a single event stream that the synth
plays. Each cascade has a **recompute policy** keyed to how slowly the underlying signal
moves:

| Cascade | Recompute cadence | Why this cadence |
|---------|------------------|------------------|
| **A — Tradition (idiom)** | Once per **era × culture-cluster** event (population split, contact-fusion, era boundary). State cached on `TraditionCache` keyed by `(culture_id_hash, era)`. | Culture drifts slowly; recomputing every tick would make idioms twitch. Era transitions are when materials/culture shift visibly. |
| **B — Timbre (voices)** | Once per **territorial material census** (every N ticks, e.g. 256). Cheap because the material set is bounded by the registry. | Material availability changes with terrain/exploration/exploitation, not every tick. |
| **C — Expressivity (mood)** | **Every tick** (slow-cadence sampler at ~1 Hz, like the MoodVector in `audio-direction.md` §2). Cheap because it's a scalar projection. | Mood is the audio's pulse — needs to feel live, but jitter-free (1 Hz sample, smoothed). |

---

## 2. Cascade A — `MusicalTradition` from culture vector

### 2.1 Inputs

- `CultureProfile.traits: [f32; 4]` (already-emergent cultural axes, 0..1)
- `Era` (prehistoric / bronze / iron / industrial / ...)
- `ContactEdge` graph (for cross-pollination rules)

### 2.2 Derived outputs

```rust
pub struct MusicalTradition {
    /// Scale degree vector (which of 12 chromatic degrees are *stable*, 0..1).
    /// e.g. [1.0, 0.0, 0.8, 0.0, 0.9, 0.0, 1.0, 0.0, 0.7, 0.0, 0.6, 0.0] = major-ish
    pub scale_weights: [f32; 12],
    /// Beat-pattern density in steps per cycle, mapped to Euclidean rhythm family.
    pub rhythm_template: RhythmTemplate,
    /// Markov chain degree for motif generation (0=unison, 1=pentatonic, 2=diatonic, 3=chromatic).
    pub motif_markov_degree: u8,
    /// How ornamented notes tend to be (trill / mordent / grace-note density 0..1).
    pub ornament_density: f32,
    /// Stable hash for cache identity; recompute only when this changes.
    pub identity_hash: u64,
}

pub struct RhythmTemplate {
    /// Number of pulses per cycle (e.g. 4, 7, 12).
    pub pulses: u8,
    /// Euclidean distribution (Bjorklund's algorithm) of hits across pulses.
    pub hits: Vec<bool>,
    /// Cycle length in ticks.
    pub cycle_ticks: u32,
}
```

### 2.3 Derivation rules (no tables — projection from axes)

| Tradition axis | Source | Projection |
|----------------|--------|-----------|
| **Scale** | `traits[0]` (axis A — interpreted as *hierarchy / stability orientation*) + `traits[1]` (axis B — *complexity*) | High A + low B → pentatonic/major-leaning weights; low A + high B → chromatic/dissonant-leaning; mid → diatonic modal. See §2.3.1. |
| **Rhythm density** | `traits[2]` (axis C — *collective/individual tension*) + era | High C → dense, syncopated Euclidean templates (12/8, 5/16); low C → sparse, regular (2/4, 3/4); industrial era → 4-on-the-floor-like templates regardless. |
| **Motif complexity** | `traits[3]` (axis D — *novelty*) + cultural distance to neighbors | High novelty + isolated cluster → higher Markov degree (more leaps); low novelty + busy contact zone → lower Markov degree (more stepwise, diatonic-conformant). |
| **Ornament density** | `culture_vec.contact: f32` (more contact → more ornaments, like trade-route flourish) + era | ornaments = sigmoid(contact × era_bonus). Prehistoric ≈ bare; industrial ≈ sustained-vibrato lean. |

**2.3.1 Scale derivation (worked example):**

The 12 scale weights are derived by **distance-from-stable-pitch**, not by enum lookup.
Pick the 3–5 "stable pitches" of the tradition by:

1. Hash the culture ID into a small bias vector `bias: [f32; 12]` (reproducible, deterministic).
2. `stable[i] = sigmoid(α · (bias[i] − traits[0] · k_major − traits[1] · k_chromatic))`
3. `scale_weights[i] = stable[i].max(stable[(i + 7) % 12] · 0.6)` (the dominant gets weight too)

This produces **major, minor, modal, pentatonic, bluesy, and chromatic** idioms depending
on the *interpolated* trait values — *no `enum Mode { Major, Minor, ... }` exists*. Two
cultures with traits `[0.9, 0.2, ...]` and `[0.85, 0.25, ...]` produce audibly-similar but
non-identical scales (cultural drift is **smooth**, not discrete).

### 2.4 Contact-zone rule (cross-pollination)

When two `CultureProfile`s share a `ContactEdge` with weight `w > 0.3`:

- Their `TraditionCache` keys **collide**: the idiom is a **weighted blend** of both
  parents' scale/rhythm/motif. Blend weight = `w / (w + 1.0)` for the donor. This is the
  music-equivalent of **creolization** in `LANGUAGE_EMERGENCE.md` §3.4 — same principle,
  same substrate.
- Conflict case (two traditions at high cultural distance): blend with **dissonance
  injection** (raised Markov degree, wider intervals) — sonically a "trade-row exoticism."

### 2.5 Era shift rule

At era boundary, **invalidate the cache** but preserve a 30% bleed from the previous era's
tradition (so Bronze-Age culture doesn't sound like Industrial-Age culture after one tick —
the substrate transitions are continuous). This mirrors the **building-style** era
transition rule in `civ-culture-emergent.md`.

### 2.6 Determinism

- All hashing uses a fixed seeded `xxhash` of `(culture_id, era)` — same tradition per
  civilization across save/load.
- No `SystemTime`, no wall-clock; the `tick` is the clock.
- Mods can override via RON tradition profiles under `assets/music/traditions/*.ron` — the
  override replaces §2.3 derivation but keeps the §2.4 contact-blend rule.

---

## 3. Cascade B — Material → Timbre (voice availability)

### 3.1 The four timbre archetypes

The spec commits to **four physical archetypes** (matching E6 in
`emergent-systems-spec.md` §2.5). They are **not** "Instrument { Drum, Flute, ... }"
enums — they are **material-kind classifiers** the registry learns from `MaterialDef`:

| Archetype | Material properties that enable it | Physical model | Example materials (current registry) |
|-----------|-----------------------------------|----------------|--------------------------------------|
| **Idiophone** (struck rigid) | `density ≥ 1500`, `phase == Solid`, present in inventory in workable form | Modal bank (1+ partials), fast attack, exponential decay | stone, ore, copper, tin, bronze, iron |
| **Lamellophone / marimba** (struck bar/wood) | `density 400–1500`, `phase == Solid`, wood-grain `porosity > 50` | Modal + slight inharmonicity, medium decay | wood (oak, ash, pine, etc. — gated by `MaterialKind::Wood`) |
| **Membrane** (struck hide) | A *hollow vessel* material + a *flexible membrane* material co-located | Bandpass noise burst + low-freq sine, short decay | hide + hollow-log / gourd / clay-pot (multi-material rule, see §3.3) |
| **Wind** (air-column) | `Phase::Gas` source material (air/wind in a tube) + a *reed* or *edge* material | Karplus-Strong variants, breath-noise modulator | bone-flute, reed-pipe, bamboo-edge |

### 3.2 Derivation rule: which voices are *available*

```
territorial_census(): for each MaterialDef present in this civ's known/exploited tiles:
    for each archetype:
        if archetype.gates(material):
            voice_count[archetype] += material.density / 1000  // proxy for abundance
voice_pool = archetypes where voice_count >= 1.0
```

A civ with only stone has **idiophone only**. A civ with copper **and** bronze **and** iron
has idiophone with rich metallic partials (voice_count modulates partial brightness, not
just availability). A civ with wood + hide + hollow gourd unlocks **membrane**. A civ with
bone + air column unlocks **wind**. Industrial-era civs gain an extra rule: any
`flammability > 50` material near an air-column enables **explosive-blank** (a 5th voice
archetype — gunpowder / steam — that can opt-in for tension stems). This is the
*material-gates-instruments* rule from §2.5 E6.2.

### 3.3 Multi-material compound rules

Some archetypes require **two materials co-located** (membrane = hide + vessel; wind = bone
+ air). The census tracks co-location: a memory tile for the last N territorial ticks
records which material tuples appeared in the same workshop voxel. When a tuple appears
≥ K times, the compound voice unlocks. **No enum** — it's a counter threshold.

### 3.4 Voice → synth mapping

`voice_pool` becomes the **selectable timbre set** passed to the synth driver. Each
archetype maps to a `fundsp` graph (Karplus-Strong / modal / noise-burst) parameterized by
the **material properties** of the dominant material in that archetype:

- **Idiophone brightness** ∝ `density / melting_point` (denser + harder = brighter)
- **Lamellophone decay** ∝ `porosity / viscosity` (more porous = shorter, breathier)
- **Membrane pitch** ∝ `field_capacity` (how much the membrane can stretch)
- **Wind breathiness** ∝ `flow_rate` (gustier air column = noisier)

The synth **does not know** "this is a flute." It knows "voice=wind, params={breath=0.4,
edge_hardness=0.7, length=0.5}." A culture with bamboo + reed gets one timbre; bone + air
gets another. **The instrument is the material.**

---

## 4. Cascade C — Mood → Expressivity (real-time shaping)

### 4.1 The expressivity vector

The per-civ `Mood` aggregate (read from `crates/agents/src/psyche.rs:46` `Mood { valence,
arousal }`) feeds a continuous **expressivity vector**:

```rust
pub struct Expressivity {
    /// Mode lean: -1 = minor/dissonant dominant, +1 = major/consonant dominant
    pub mode_lean: f32,
    /// Tempo multiplier (0.5x to 1.5x the tradition's base cycle)
    pub tempo_mult: f32,
    /// Accent intensity (0..1) — how hard on-beat notes hit vs off-beat
    pub accent: f32,
    /// Swing/shuffle amount (0=straight, 1=full triplet swing)
    pub swing: f32,
    /// Tension injection: extra dissonance probability for TensionStem
    pub tension_injection: f32,
}
```

### 4.2 Mapping rules

| Signal | Source | Projection |
|--------|--------|-----------|
| `mode_lean` | `valence` (civ-aggregate over agents in cluster) | `mode_lean = valence` clamped [-1,1]; high valence → major-friendly scale weights get +5% each tick; low valence → minor-friendly scale weights rise |
| `tempo_mult` | `arousal` | `tempo_mult = 0.5 + arousal` (0.5x at calm, 1.5x at agitated) |
| `accent` | recent event density (`EventFeedMessage3d` per tick × decay) | `accent = tanh(events_per_tick / 4.0)`; calm civ = soft even touch; war/disaster cluster = sharp attacks |
| `swing` | `traits[1]` (complexity axis from culture) + 0.1·`valence` | Low complexity + positive mood = more swing (satisfied, dance-like); high complexity = straight |
| `tension_injection` | active `Disaster` + `Battle` events (count) × `arousal` | tension overlay probability = `1 - exp(-event_count × arousal)`; clamped to TensionStem only |

### 4.3 Why this matters

This is the **expressivity layer** — same music tradition, different night. A civ in deep
peace plays its idiom calmly; the same civ under siege plays the same idiom faster, harder,
more dissonant, with the TensionStem rising (per `audio-direction.md` Tier 2). **No mode
switch is authored** — the change is continuous in the expressivity vector, which the synth
interpolates gain-only between pre-rendered stems (per `audio-direction.md` §1 tier-2).

### 4.4 Aggregation rule (per-civ mood)

Individual agents have `Mood { valence, arousal }`. The civ aggregate is a **weighted mean**
weighted by:

- `agent.maturity` (older agents count more — they "represent" the culture's accumulated feel)
- `agent.beliefs` projection onto the culture vector (more culturally-aligned agents pull more)
- exponential smoothing with `α ≈ 0.05` (slow; mood drift is gradual, not per-tick jitter)

---

## 5. Integration: from substrate to sound

### 5.1 Where it lives in the crate graph

```
crates/agents/src/psyche.rs    ─► Mood { valence, arousal }     ─┐
crates/agents/src/culture.rs   ─► CultureProfile { traits }     ─┤
crates/voxel/src/material.rs   ─► MaterialDef registry         ─┼──► crates/audio/src/tradition.rs  (NEW)
crates/protocol-3d/src/lib.rs  ─► EventFeedMessage3d           ─┤            │
                                                                       ┌──────┴───────┐
                                                                       │ MusicalTradition│
                                                                       │ Expressivity    │
                                                                       │ VoicePool       │
                                                                       └──────┬────────┘
                                                                              │ note/event stream
                                                                              ▼
                                                                     crates/audio/src/synth.rs
                                                                              │ fundsp + kira
                                                                              ▼
                                                                     clients/bevy-ref/src/audio.rs
                                                                              │
                                                                              ▼
                                                                       MasterBus (per audio-direction.md §3)
```

### 5.2 Tick flow (one simulation tick)

1. **Engine tick** — agents' `update_mood` fires, `drift_populations` mutates culture vectors, voxel/material registry returns current census.
2. **Cascade A** — if cache key (culture_id, era) is dirty, recompute `MusicalTradition`. Hash-checked; same tradition reused across stable eras.
3. **Cascade B** — every N=256 ticks, re-census materials → recompute `VoicePool`. Cheap; the material set is bounded.
4. **Cascade C** — every tick, sample aggregate `Mood`, derive `Expressivity`, project `EventFeedMessage3d` into `accent` / `tension_injection`. Smoothed with α.
5. **Synthesis** — for each tradition-active civ cluster that is *audible* (in listening range of camera, see §6.2), emit a `NoteEvent` stream to `crates/audio/src/synth.rs` with `{voice, pitch, velocity, ornament}` per the tradition × expressivity.
6. **Spatialization** — `kira` panners place the stream at the cluster's territorial centroid; gain attenuates with distance to camera. This is the audio equivalent of the `Frame3d` LOD rule in `civis-3d-verify`.

### 5.3 What is *not* derived (the discipline boundary)

The following **must** stay hardcoded because they are substrate, not convention:

- Physics: 12-TET chromatic reference, A440 (the synth's tuning reference). A tradition may *avoid* certain pitches; it does not retune them.
- Tempo range bounds: 30–300 BPM. Traditions produce rhythms *within* this; they don't redefine the tick.
- Voice archetype count: **four** (idiophone / lamellophone / membrane / wind) — these are physical categories of how solid/liquid/gas materials produce sound. Mods can add a 5th (e.g. `flam_archetype` for gunpowder) but not multiply freely.
- Voice pool cap: a civ can have at most 8 simultaneous timbre variants (4 archetypes × 2 sub-variants). This is a complexity guard, not a stylistic choice.
- Expressivity clamps: tempo 0.5–1.5×, accent 0–1, swing 0–1. The mood is felt *within* these bounds, never outside.

This is the **Hardcode only laws** rule from the charter, applied: physics and substrate
are laws; everything else is convention.

---

## 6. Listening model — who hears what

### 6.1 Per-cluster tradition tracking

`TraditionCache` is keyed by `(culture_id, era)` — each population cluster gets its own
tradition identity. Splits (colony, conquest, isolation) spawn new cache entries; fusions
collapse them per the §2.4 blend rule. This is identical to the
**polity-cluster** model in `warfare.md` §STRATEGIC layer.

### 6.2 Audible cluster rule

A civ cluster is *audible* (contributes to the music mix) when:

- Its territorial centroid is within audio range of the camera/listener (default: 30 world-units, configurable via `AudioMix` resource).
- Its population exceeds the **silence threshold** (default: 50 agents). Tiny emergent outposts don't contribute music — only ambient + SFX. This prevents 200 tiny civs from each producing a polyphonic texture (the *emergence bloat* anti-pattern).

### 6.3 Voice allocation across audible clusters

If multiple clusters are audible and overlapping:

- The cluster closest to camera gets the **LeadStem** slot (the only foreground voice, per `audio-direction.md` "one sonic accent at a time").
- The next 2–3 clusters get **RhythmStem / TensionStem** slots (texture).
- All others fall back to a quiet **pad drone** (BaseStem) summed and ducked — they exist as harmonic atmosphere, not foreground.

Allocation is recomputed on camera movement (similar to the ambient-bed cross-fade in
`audio-direction.md` §1).

---

## 7. The 4 timbre archetypes — physical grounding (spec details for the synth author)

> This section is the hand-off to the implementer of `crates/audio/src/synth.rs`. Each
> archetype is specified as a `fundsp` parameter block, *not* as an instrument enum.

### 7.1 Idiophone (struck rigid material)

```text
model:        modal bank (3–5 partials)
attack:       < 5 ms
decay:        0.3–3.0 s  (∝ 1 / density)
partials:     brightness = density / melting_point  (clamped 0..1)
              inharmonicity = 1.0 - density / 1000  (denser = more harmonic)
gain mod:     voice_count[idiophone] (more metal = louder / more present)
trigger:      RhythmTemplate hits on beats 0 and 2 (downbeats)
```

### 7.2 Lamellophone / marimba (wooden bar)

```text
model:        modal bank with mild inharmonicity
attack:       5–15 ms  (wood is softer than stone)
decay:        0.5–1.5 s  (∝ porosity)
partials:     2–4, decaying faster than idiophone
gain mod:     voice_count[lamellophone] × ornament_density (busier texture in ornamented traditions)
trigger:      RhythmTemplate hits on all true bits
```

### 7.3 Membrane (hide-stretched over hollow)

```text
model:        bandpass-filtered noise burst + low sine (membrane fundamental)
attack:       < 2 ms  (very fast transient)
decay:        0.1–0.4 s  (∝ field_capacity)
partials:     N/A — membrane is broadband-ish, filtered
gain mod:     voice_count[membrane] × accent (excited under high-arousal mood)
trigger:      RhythmTemplate hits + occasional off-beat ghost hits (accent-driven)
```

### 7.4 Wind (air-column in tube)

```text
model:        Karplus-Strong with breath-noise modulator
attack:       30–80 ms  (breath onset)
decay:        0.4–2.0 s  (∝ flow_rate of the air-column material)
partials:     KS handles it; breathiness = 1.0 - flow_rate (low-flow = airy)
gain mod:     voice_count[wind] × mode_lean (wind voices rise in major-keyed traditions)
trigger:      pitched runs generated by Markov chain over scale_weights (degree = motif_markov_degree)
```

### 7.5 Voice mix policy

Within a single tradition, the four voices are mixed by:

- **Idiophone**: always present when unlocked; foundational pulse.
- **Lamellophone**: ornamental counter-rhythm; density modulated by `ornament_density`.
- **Membrane**: rhythmic accent layer; rises with `accent` (arousal).
- **Wind**: melodic voice (LeadStem); rises with `mode_lean` and population prosperity (per `audio-direction.md` Tier 2 LeadStem rule).

When `tension_injection` > 0.3, the **wind voice shifts down a semitone** for TensionStem
(audible "ominous" effect — same voice, different pitch, no new archetype). This is the
simplest expressive gesture that carries tension without breaking the four-archetype rule.

---

## 8. Functional requirements (FR-CIV-MUSIC-*)

| ID | Requirement | Acceptance |
|----|-------------|------------|
| **FR-CIV-MUSIC-001** | `MusicalTradition` derived from `CultureProfile.traits` via projection, no enum/table | Running the derivation on two cultures with trait delta < 0.05 produces audibly similar but non-identical idioms; trait delta > 0.5 produces audibly different idioms |
| **FR-CIV-MUSIC-002** | Tradition recomputed only on culture_id/era cache miss | Cache hit rate > 90% in a 1000-tick replay of a stable culture (deterministic replay) |
| **FR-CIV-MUSIC-003** | Material census gates voice availability | A civ with no metal has zero idiophone voices; unlocking bronze unlocks idiophone within 1 census tick (≤ 256 ticks) |
| **FR-CIV-MUSIC-004** | Four timbre archetypes only (idiophone / lamellophone / membrane / wind); explosive-blank opt-in for industrial-era flammability materials | Archetype count = 4 + optional 5th per §3.2; no other voice types defined; mods may register new archetypes via the registry, gated to ≤ 8 total variants per civ |
| **FR-CIV-MUSIC-005** | Mood aggregate drives expressivity vector smoothly | `mood.valence` change of ±0.3 in one tick produces tempo_mult change < ±0.05 (smoothed); same input over 100 ticks produces a perceptible shift |
| **FR-CIV-MUSIC-006** | Same tradition reads differently in different moods | The same civ, recorded with valence = +0.8 and valence = -0.8, produces audibly different playback (mode lean, tempo, accent) while sharing scale/rhythm identity (recognizably the same "song") |
| **FR-CIV-MUSIC-007** | Contact-zone creolization | Two cultures in contact (weight > 0.3) over 200 ticks produce a blended idiom measurably closer to both parents than to either parent alone |
| **FR-CIV-MUSIC-008** | Audible-cluster rule (silence threshold + camera range) | A civ with population 30 in audio range contributes no music; the same civ with population 100 does |
| **FR-CIV-MUSIC-009** | LeadStem allocation follows "one sonic accent" pillar | At most one tradition's wind voice holds LeadStem slot at a time; nearest cluster to camera wins; switchover uses kira gain tween (no click) |
| **FR-CIV-MUSIC-010** | Determinism — same seed/save = same music | A 10 000-tick replay with a fixed seed produces identical `NoteEvent` streams (hashable); the `tradition.identity_hash` field is the canonical identity |
| **FR-CIV-MUSIC-011** | No enum / table / authored scale, genre, key, tempo, instrument anywhere in code | `grep -rE "enum (Mode|Genre|Scale|Instrument|Tempo|Key|Tradition)\b" crates/audio crates/agents crates/voxel` returns no matches except the `MusicalTradition` *struct* (which is derived, not authored) |
| **FR-CIV-MUSIC-012** | Mods can override a tradition via RON but not invent enum axes | `assets/music/traditions/<name>.ron` replaces §2.3 derivation for that name; §2.4 contact-blend still applies; the file schema permits only `scale_weights` / `rhythm_template` / `motif_markov_degree` / `ornament_density` |
| **FR-CIV-MUSIC-013** | Tempo/range/expressivity bounds enforced | Synth refuses any `tempo_mult` outside [0.5, 1.5], `accent` outside [0, 1], `swing` outside [0, 1]; logged and clamped, not crashed |
| **FR-CIV-MUSIC-014** | Crisis overload: at extreme event density, music fades to drone + SFX | When `tension_injection > 0.8` sustained 10+ ticks, LeadStem gain ducks to ≤ 0.2; TensionStem + Disaster SFX take the foreground (handoff to `audio-direction.md` ducking rules) |
| **FR-CIV-MUSIC-015** | Graceful silence — no music is still a playable game | With zero traditions materialized (no civs above silence threshold), the audio output is the same as the pre-music baseline (ambient + SFX + UI), no errors, no missing-file warnings on music paths |

---

## 9. Phased WBS (hand-off to implementer)

| Phase | Task ID | Deliverable | Depends on | FR covered |
|-------|---------|-------------|-----------|-----------|
| P1 — Tradition core | M1 | `MusicalTradition` struct + scale/rhythm/motif/ornament derivation from `CultureProfile.traits` in `crates/audio/src/tradition.rs` | `crates/agents` Mood + CultureProfile | FR-001, 011 |
| P1 — Tradition core | M2 | `TraditionCache` keyed by `(culture_id, era)`; hash-based miss/recompute policy | M1 | FR-002, 010 |
| P1 — Tradition core | M3 | Contact-zone blend rule per §2.4 | M2, `ContactEdge` | FR-007 |
| P2 — Timbre | M4 | `VoicePool` derivation from `MaterialDef` census in `crates/audio/src/voice.rs` | voxel material registry | FR-003 |
| P2 — Timbre | M5 | Multi-material compound rules (membrane / wind co-location counter) | M4 | FR-003 |
| P2 — Timbre | M6 | Four-voice synth stubs in `fundsp` (idiophone modal, lamellophone modal, membrane noise+sine, wind KS) | kira + fundsp substrate | FR-004 |
| P3 — Expressivity | M7 | `Expressivity` struct + per-tick mood aggregation + smoothing | M1, Mood | FR-005 |
| P3 — Expressivity | M8 | Mapping rules from §4.2 wired to synth params (tempo_mult → cycle length, mode_lean → scale weight bias, accent → envelope, tension → pitch offset) | M6, M7 | FR-006 |
| P4 — Integration | M9 | Audible-cluster rule + camera-range gating | M1, M4 | FR-008 |
| P4 — Integration | M10 | LeadStem allocation policy (nearest-cluster-wins, kira tween switchover) | M9 | FR-009 |
| P4 — Integration | M11 | Tension-hand-off: at sustained high `tension_injection`, LeadStem duck to ≤ 0.2; TensionStem/Disaster SFX foreground | M8, `audio-direction.md` ducking | FR-014 |
| P5 — Modding | M12 | RON tradition override schema + loader in `crates/audio/src/tradition_ron.rs` | M1 | FR-012 |
| P5 — Modding | M13 | Optional 5th archetype registration (e.g. explosive-blank) via RON, gated to ≤ 8 variants per civ | M4 | FR-004 |
| P6 — Safety | M14 | Expressivity clamp guard at synth boundary (clamp + log, never crash) | M8 | FR-013 |
| P6 — Safety | M15 | Graceful-silence test: zero traditions → output identical to pre-music baseline | M1, M4 | FR-015 |
| P6 — Safety | M16 | `grep` CI guard: forbid `enum (Mode|Genre|Scale|Instrument|Tempo|Key|Tradition)` in audio/agents/voxel crates (allow struct `MusicalTradition`) | — | FR-011 |

---

## 10. Acceptance gates (the planner's "done" criteria)

This design is **done** when:

- [ ] All 15 FR-CIV-MUSIC-* requirements have at least one passing test.
- [ ] `crates/audio/src/tradition.rs` and `crates/audio/src/voice.rs` exist with derivation logic; no `Mode`/`Genre`/`Scale`/`Instrument`/`Tempo`/`Key` enum is present in any of `crates/audio`, `crates/agents`, `crates/voxel`.
- [ ] A 1 000-tick deterministic replay of a single stable civ produces the same `NoteEvent` stream hash on every run (modulo float precision tolerance).
- [ ] Two civilizations with trait delta 0.1 and 0.9 respectively produce audibly different idioms (manual listen test, recorded sample pair).
- [ ] Same civilization, valence +0.8 vs -0.8, produces audibly different playback of the same tradition (manual listen test).
- [ ] With zero civs above the silence threshold, music is silent and the rest of the audio (ambient + SFX + UI) is unchanged.
- [ ] Modder can drop `assets/music/traditions/mytradition.ron` and have it override the derivation for a named tradition, with contact-blend still applied.
- [ ] The four-tier mix tree from `audio-direction.md` is preserved: LeadStem is at most one tradition at a time; TensionStem + Disaster SFX rise under crisis; AmbientBus never goes above ScoreBus default gain (per `audio-direction.md` §3 mix table).

---

## 11. Reference index

**Charter + companions:**
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — discipline, hardcode-only-laws, default-emerge
- [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) §2.5 — the §E6 framing this doc narrows
- [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) — culture trait vector
- [`docs/design/LANGUAGE_EMERGENCE.md`](LANGUAGE_EMERGENCE.md) — contact-zone creolization pattern (mirrors §2.4)
- [`docs/design/SENTIENCE_PSYCHE.md`](SENTIENCE_PSYCHE.md) — `Mood { valence, arousal }`
- [`docs/design/CULTURE_IDEOLOGY.md`](CULTURE_IDEOLOGY.md) — culture cluster identity (matches §6.1 cache keying)
- [`docs/design/audio-direction.md`](audio-direction.md) — four-tier mix tree this music rides on; FR-CIV-AUDIO-*
- [`docs/research/RND-007-adaptive-music-kira.md`](../research/RND-007-adaptive-music-kira.md) — kira + fundsp substrate notes

**Sim hooks (consume, do not modify):**
- [`crates/agents/src/psyche.rs:46`](../../crates/agents/src/psyche.rs) — `Mood { valence, arousal }`
- [`crates/agents/src/culture.rs:15`](../../crates/agents/src/culture.rs) — `TraitVector = [f32; 4]`, `CultureProfile`
- [`crates/voxel/src/material.rs:27`](../../crates/voxel/src/material.rs) — `MaterialDef { density, viscosity, flow_rate, porosity, field_capacity, ... }`
- [`crates/protocol-3d/src/lib.rs:346`](../../crates/protocol-3d/src/lib.rs) — `EventFeedMessage3d` (Birth/Death/Tech/Battle/Disaster) — accent + tension injection sources

**Existing audio scaffold (extend, do not re-architect):**
- [`crates/audio/src/lib.rs`](../../crates/audio/src/lib.rs) — current `CivisAudioPlugin` + bus scaffolding
- [`crates/audio/src/mood.rs`](../../crates/audio/src/mood.rs) — current MoodVector (per-tick sim aggregates)
- [`crates/audio/src/ambient.rs`](../../crates/audio/src/ambient.rs) — bed cross-fade pattern (model for §6.3 allocation)
- [`crates/audio/src/sfx.rs`](../../crates/audio/src/sfx.rs) — coalescing/clamp pattern (model for §3.2 voice cap)
- [`clients/bevy-ref/src/audio.rs`](../../clients/bevy-ref/src/audio.rs) — kira plugin host
