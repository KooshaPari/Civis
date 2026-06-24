# Civis Competitive Benchmark — Civis vs AAA/Indie Peers

> **Status:** Research artifact (2026-05-30). Owned by Research & Spec Lead. Companion to
> [`docs/specs/feature-matrix.md`](../specs/feature-matrix.md),
> [`docs/research/art-direction.md`](./art-direction.md), and
> [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md).
>
> **What Civis is:** a voxel material-fluid CA simulation + everything-emergent civ
> god-game spanning city-builder → RTS → 4X, targeting ~20 mi × 20 mi maps on a Bevy
> desktop client. Per the charter, **only physical/environmental/genomic laws are
> authored; life, society, economy, culture, polity, language, and architecture EMERGE.**
>
> **Stance:** This is an honest, clear-eyed benchmark of *current* Civis — not the
> aspiration. Where the feature matrix marks a capability BLIND/INCOMPLETE, this document
> scores it as such. The goal is to know exactly where Civis wins, where it must reach
> parity, and the single biggest credibility gap to close first.

---

## 1. The competitive set & tiers

Civis is unusual in that it touches five genres at once, so it competes — on *one axis at a
time* — with the leader of each. We group rivals by the axis where they set the bar, and
flag whether they are a **visual benchmark** (AAA-grade-indie polish) or a **depth
benchmark** (simulation/emergence the field measures itself against).

### Tier A — Voxel / falling-sand / material sim
| Game | Why it's here | Benchmark type |
|---|---|---|
| **Noita** | Every pixel simulated via falling-sand CA; the gold standard for *emergent material interaction* (lava+water→rock+steam→rain). 64×64 dirty-rect chunks — the same architecture Civis uses. | **Depth (material-sim)** |
| **The Powder Toy** | Deepest 2D element/reaction DB; the reference for "how many materials interact how richly." | **Depth (material-sim)** |
| **Vintage Story** | Voxel survival with serious geology/strata + cloth/heat sim; "Minecraft for adults." | Mixed |
| **Minecraft** | The voxel-world baseline for scale, moddability, and cultural reach (not depth). | Breadth/moddability |
| **Eco** | Player-run economy + scientific ecosystem sim on a voxel planet; "Minecraft on steroids" with an *actual* simulated economy and pollution. | **Depth (economy/eco-sim)** |

### Tier B — Emergent god-game / sandbox
| Game | Why it's here | Benchmark type |
|---|---|---|
| **WorldBox** | ~374 god powers across 8 tabs; the **god-tool palette** gold standard. Civilizations develop autonomously (farming/war/trade), with hereditary trait inheritance. Civis's nearest *genre* sibling. | **Depth (god-sandbox) + UX (god tools)** |
| **Spore** | Creature-creator DNA → phenotype; the consumer reference for "design a species." (Stage-gates are *scripted* — the anti-pattern Civis explicitly avoids.) | Breadth (creator UX) |
| **Black & White** | Creature *learning* + the god-hand interaction metaphor; lineage ancestor of the genre. | UX (interaction metaphor) |
| **Galimulator** | Pure abstract emergent-galaxy sim; proof that "watch emergence unfold" is a viable loop. | Depth (emergence-as-spectacle) |

### Tier C — Deep society / colony sim
| Game | Why it's here | Benchmark type |
|---|---|---|
| **Dwarf Fortress** | The depth gold standard. 500+ interlocking needs/skills/memories per dwarf; **Legends mode** records emergent history/myth — nothing else comes close. | **Depth (society + emergent history)** |
| **RimWorld** | Best-in-class *individual* psyche: moods, traits, mental breaks, relationships → emergent stories via apophenia. | **Depth (psyche/story)** |
| **Songs of Syx** | Scale leader: 40k–50k+ individually-simulated agents + Total-War-grade battles. Trades individual detail for empire scale. | **Depth (scale)** |
| **Sapiens** | Hunter-gatherer → civilization emergence with a clean, readable presentation. | Mixed |

### Tier D — City-builder
| Game | Why it's here | Benchmark type |
|---|---|---|
| **Manor Lords** | The AAA-grade-indie **visual** benchmark: grounded medieval realism, organic burgage plots, desire-path roads, bespoke models. | **Visual (AAA-indie)** |
| **Cities: Skylines 2** | Tile-streaming scale + **~33 info-view overlays** (the gold standard for legibility); supply chains, budget/policy panels. | **UX/UI (info-views) + visual** |
| **Frostpunk (1 & 2)** | Bespoke art direction + society-wide morale/law system; striking, cohesive visual identity. | **Visual (AAA-indie) + narrative** |
| **Foundation / Timberborn / Against the Storm / Timber** | Organic-grid building, biome puzzles, run-based depth — strong indie polish references. | Visual/UX |

### Tier E — 4X / grand strategy
| Game | Why it's here | Benchmark type |
|---|---|---|
| **Civilization VI** | The 4X baseline (tech tree, agendas). Civis explicitly **avoids** its hardcoded tech-tree/faction model. | Breadth (4X systems) |
| **Old World** | Dynasties, character-driven diplomacy — emergent-ish leadership. | Depth (character/diplomacy) |
| **Victoria 3** | Deep emergent *economy* + pops + political blocs at nation scale. | **Depth (economy/polity)** |

**Visual benchmarks (the "looks AAA" bar):** Manor Lords, CS2, Frostpunk 1/2.
**Depth benchmarks (the "is deep" bar):** Dwarf Fortress, RimWorld, Songs of Syx, Noita, The Powder Toy, Eco, Victoria 3.

---

## 2. Benchmark matrix

Scale: **1 = absent/BLIND, 2 = INCOMPLETE, 3 = at-genre-floor, 4 = strong, 5 = best-in-class.**
Civis scores are honest *current-state* (cross-referenced to `feature-matrix.md`), not target.
"Leader" = who sets the bar on that axis; "Gap" = behind / par / ahead of that leader.

| Axis | Leader (the bar) | Leader score | **Civis now** | Gap vs leader |
|---|---|:---:|:---:|---|
| **Visual fidelity** | Manor Lords / CS2 / Frostpunk | 5 | **2** | **far behind** — flat single-roughness RGB world; emissive inert; no real models/grading (art-direction.md §4) |
| **World-sim / physics depth (voxel-fluid)** | Noita / Powder Toy | 5 | **2–3** | behind on element/reaction breadth, but the *3D voxel-fluid + thermal/pressure + 20mi* ambition is beyond any of them in scope; substrate built (SVO+dirty-queue), surfaced partially |
| **Emergence (life/society/economy/culture)** | Dwarf Fortress | 5 | **2** | behind — genetics/species/needs SOLID, but psyche/social-graph/histories/culture BLIND-to-INCOMPLETE |
| **Agent AI depth** | RimWorld / DF | 5 | **2** | behind — utility-AI needs exist; no psyche, memory, relationships, grudges |
| **Scale** | Songs of Syx (50k agents) / CS2 (tiles) | 5 | **2–3** | streaming/LOD substrate built; not yet proven at 20mi/100k agents @ 60fps |
| **Content breadth** | Minecraft / Civ / WorldBox | 5 | **2** | behind — few finished verbs; thin god-tool palette vs WorldBox's 374 powers |
| **UX / UI polish** | CS2 (~33 overlays) | 5 | **1–2** | **far behind** — info-view suite BLIND; inspect-anything BLIND; minimal HUD. *Exception:* 2D/HUD SVG layer is already DINOForge-grade |
| **Performance** | Songs of Syx / CS2 | 4–5 | **2** | unproven at target scale; LOD tiers designed, not validated |
| **Moddability** | CS Workshop / RimWorld XML | 5 | **3** | RON law/material DB is genuinely mod-friendly; sandbox mod-host exists; no Workshop/asset pipeline yet |

**Reading the matrix:** Civis is at **2** on most axes today. It is *not* behind because the
design is shallow — it is behind because the ambitious substrate is only *partially
surfaced*. Two cells are notably worse (visual fidelity, UX/UI) and are the credibility
liabilities. One cell (moddability) is already near-par. The voxel-fluid + emergence
*combination* is where the latent score is highest.

---

## 3. Positioning

### The unique combination (the moat)
**No competitor fuses all four of:** (1) a true voxel **material-fluid** CA sim (Noita/Powder
Toy territory) in **3D**, (2) **everything-emergent** civilization (DF/WorldBox territory) with
*nothing* hardcoded above physical/genomic law, (3) **20 mi × 20 mi** continuous scale (SoS/CS2
territory), and (4) a **single continuum from city-builder → RTS → 4X** (no peer spans all
three layers in one world).

Each rival owns *one* of these:
- **Noita / Powder Toy** own material-sim depth — but in 2D, with no civilization, no scale.
- **WorldBox** owns emergent god-sandbox civ — but on a shallow, non-physical, 2D tile world.
- **Dwarf Fortress** owns emergent depth + histories — but ASCII/tile, single fortress, no fluid CA.
- **Songs of Syx** owns agent scale — but with no material sim and largely authored systems.
- **Cities: Skylines 2** owns legibility + city verbs — but a scripted, non-emergent society.
- **Eco** owns a simulated economy on a voxel world — but tiny scale, no emergent polities/culture.

**Civis's genuine differentiator:** *a living 3D world where the ground itself is simulated
matter and the civilization on top of it is fully emergent — at landscape scale.* That
intersection is empty in the market.

### Nearest single-axis rivals
- **Material sim:** Noita (closest architecture — dirty-rect chunked CA), The Powder Toy (DB depth).
- **Emergent civ:** WorldBox (closest *genre*), Dwarf Fortress (closest *depth ceiling*).
- **Scale:** Songs of Syx.
- **Visual bar to clear:** Manor Lords (and the project's own sibling DINOForge for art discipline).

### Credibility gaps (why a player might dismiss Civis today)
1. **"It looks flat / unfinished."** A single-roughness, non-emissive RGB voxel world reads
   as a prototype next to Manor Lords/Frostpunk. *This is the first impression and the
   biggest dismissal risk.*
2. **"I can't see what's happening."** No info-view overlays, no inspect-anything, minimal
   tooltips — the emergence is invisible, so players cannot *perceive* the depth that is the
   whole pitch. (DF survives ugliness because Legends/unit screens let you *read* the depth;
   Civis currently has neither the looks nor the legibility.)
3. **"The emergence is a promise, not a payoff."** Genetics/species substrate is real, but
   psyche, social graphs, emergent histories, culture/language drift, and emergent polities
   are BLIND/INCOMPLETE — so the headline claim isn't yet demonstrable in play.
4. **"Does it run?"** 20mi/100k-agent/60fps is unproven; Bevy's PBR + many-light + voxel
   performance is a known pressure point ([Bevy Solari / voxel rendering discussion]).

---

## 4. Targets to compete (per behind-axis, the specific bar)

| Axis | Named leader | Concrete bar to be *credible* against it |
|---|---|---|
| **Visual fidelity** | Manor Lords / Frostpunk | Per-material PBR (roughness/metallic/**emissive**/reflectance) so lava glows + water reads wet; warm-key/cool-fill lighting; tuned bloom/ACES grade + vignette; per-biome surfaces; eventually real models + GI. (All mapped in `art-direction.md §4–7`.) |
| **World-sim depth** | Noita / Powder Toy | A material/reaction DB with *dozens* of interacting elements + visible phase changes (melt/freeze/evaporate/condense/burn) in real time; thermal+pressure visibly drive behavior. Match the "pour lava on water → rock + steam → rain" legibility in 3D. |
| **Emergence** | Dwarf Fortress | **Emergent histories**: a queryable Legends-style chronicle of named agents, lineages, conflicts, and migrations generated by the sim — DF's actual moat. Plus psyche (mood/temperament/memory) and a kinship/relationship graph. |
| **Agent AI depth** | RimWorld | Per-agent traits + moods + mental breaks + relationships that produce *readable* stories (apophenia engine); surfaced in a unit inspector. |
| **Scale** | Songs of Syx | Demonstrate 50k+ LOD-tiered agents and 20mi streaming at 60fps in a captured benchmark run, with far-field statistical sim. |
| **Content breadth** | WorldBox | A god-tool palette approaching WorldBox's breadth (spawn-anything, terraform, disasters, material brush) across organized tabs — the genre's table-stakes. |
| **UX / UI polish** | Cities: Skylines 2 | The **info-view overlay suite** (pollution/land-value/happiness/wealth/services/traffic/resources/ideology/economy time-series) + **inspect-anything** click target on any voxel/agent/settlement + rich tooltips. This is what makes emergence *perceptible*. |
| **Performance** | SoS / CS2 | Validated 60fps at target scale; profile-backed, not designed-only. |
| **Moddability** | CS Workshop / RimWorld | Keep the RON law/material/asset mod path; add a sandboxed code+data mod API and a share/distribution path. (Already near-par — protect it.) |

---

## 5. Verdict

### Where Civis can realistically WIN (the moat — invest here)
1. **The voxel-fluid × emergent-civ × scale intersection.** No game occupies it. A 3D world
   where simulated *matter* is the substrate of a *fully emergent* civilization at landscape
   scale is a category of one. This is the reason to exist; everything else is in service of it.
2. **Emergent depth perceived as authored, but generated.** The DF/RimWorld lesson: depth +
   legibility beats graphics. Civis's charter (only laws authored) is a *stronger* version of
   exactly what DF/WorldBox players love. Win on **emergent histories + psyche + emergent
   polities** made *readable*.
3. **Moddability via law/material RON.** Already near-par and aligned with the emergence
   thesis (mod the *laws*, not the content). A genuine, defensible early strength.

### Where Civis must reach PARITY (table stakes — cannot ship below this)
- **Legibility/UX:** the CS2 info-view + inspect-anything bar. Without it the depth is
  invisible and the whole pitch collapses. *This is parity, not a moat — but it is non-negotiable.*
- **Baseline visual richness:** the `art-direction.md` PBR/lighting/post-FX closeout. Not
  AAA — just "not a prototype." Emissive lava + wet water + warm/cool lighting is cheap and
  clears the dismissal bar.
- **Demonstrable emergence:** at least psyche + social graph + a Legends-style chronicle so
  the headline claim is playable, not promised.

### What to NOT try to beat (don't fight on the wrong axis)
- **Do NOT out-AAA Manor Lords / Frostpunk on bespoke art.** Hand-authored medieval realism
  and a full art department are not winnable or worth it for an emergent voxel sim. Hit
  "rich and cohesive" (DINOForge-grade discipline), then stop and spend the budget on depth.
- **Do NOT out-breadth Civ/Minecraft on hardcoded content.** That contradicts the charter and
  is a losing content-arms-race. Breadth should *emerge*, not be authored.
- **Do NOT chase Noita's bespoke 2D pixel artistry** — Civis's advantage is 3D + scale + civ,
  not pixel-perfect 2D spectacle.

### The single biggest credibility gap to close FIRST
**Legibility — the CS2-style info-view overlay suite + inspect-anything.** It outranks even
the visual gap, because: (a) it is the *force-multiplier* on the moat — emergent depth that
can't be seen is worth zero, and Civis's entire differentiator is depth; (b) DF proves a game
can be visually crude and still win *if you can read the depth*, but the reverse is not true;
(c) it is mostly [UI/QoL] work (egui + egui_plot already available) with no charter risk,
unlike the open-ended emergence systems. Close legibility first, the PBR/lighting closeout
second (cheap, high-wow), and emergent histories/psyche third. Visual parity with Manor Lords
is explicitly **not** the first gap to close — and is never the gap to *win* on.

---

## 6. Sources
- [Noita — Wikipedia](https://en.wikipedia.org/wiki/Noita_(video_game)) (every-pixel CA, 64×64 dirty-rect chunks)
- [Noita: a Game Based on Falling Sand Simulation — 80.lv](https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation)
- [WorldBox — Wikipedia](https://en.wikipedia.org/wiki/WorldBox) and [Powers — Official WorldBox Wiki](https://the-official-worldbox-wiki.fandom.com/wiki/Powers) (~374 powers / 8 tabs, hereditary traits)
- [Dwarf Fortress: The Nexus of Emergent Complexity — Genezi research](https://research.genezi.io/p/dwarf-fortress-the-nexus-of-emergent) (500+ needs/skills/memories per dwarf)
- [Designing Pillars of State: Scale — GeoTechGames (Songs of Syx dev)](https://geotechgames.com/designing-pillars-of-state-scale) and [Songs of Syx — PC Gamer](https://www.pcgamer.com/songs-of-syx-is-a-base-building-game-with-massive-scale-battles/) (40k–50k+ agents, Total-War battles)
- [DF vs Songs of Syx — Steam discussion](https://steamcommunity.com/app/975370/discussions/0/3709307511569934803/) (individual detail vs scale trade-off)
- [Mood — RimWorld Wiki](https://rimworldwiki.com/wiki/Mood) and [RimWorld Psychology — neurolaunch](https://neurolaunch.com/rimworld-psychology/) (moods/traits/mental breaks)
- [RimWorld vs Frostpunk — Steam discussion](https://steamcommunity.com/app/294100/discussions/0/2952595757895242724/) (individual vs society-wide morale)
- [Eco (2018) — Wikipedia](https://en.wikipedia.org/wiki/Eco_(2018_video_game)) and [Why I Play: Eco — Massively Overpowered](https://massivelyop.com/2021/04/07/why-i-play-multiplayer-sandbox-eco-is-so-much-more-than-just-an-ecology-simulator/) (player-run economy, ecosystem sim)
- [How Frostpunk 2's Gameplay Compares to Manor Lords — Game Rant](https://gamerant.com/frostpunk-2-gameplay-manor-lords-compared-city-builders/) (city-builder visual/design contrast)
- [I played 23 city builders in 2024 — Yahoo/TechRadar](https://tech.yahoo.com/gaming/articles/played-whopping-23-city-builders-190000752.html) (2024 city-builder field)
- [Realtime Raytracing in Bevy 0.17 (Solari) — jms55](https://jms55.github.io/posts/2025-09-20-solari-bevy-0-17/) and [Bevy ray tracing support — issue #639](https://github.com/bevyengine/bevy/issues/639) (Bevy PBR/GI/voxel rendering state vs AAA)
- [Best Voxel Game Engines 2026 — Rosebud](https://lab.rosebud.ai/blog/best-voxel-game-engines-2026) (voxel engine landscape)
- Internal: [`docs/specs/feature-matrix.md`](../specs/feature-matrix.md), [`docs/research/art-direction.md`](./art-direction.md), [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md)
