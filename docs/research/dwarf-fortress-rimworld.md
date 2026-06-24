# Dwarf Fortress + RimWorld Teardown for Civis Spec Map

## Overview

Dwarf Fortress and RimWorld are the two strongest reference points for Civis because they both turn small-scope agent simulation into readable, high-drama history. They differ in presentation and sim philosophy, but they converge on the same essential promise: individual agents have persistent needs, moods, injuries, social memory, and task preferences, and those local pressures compound into stories the player can inspect after the fact.

| Game | Core identity | Why it matters for Civis |
|---|---|---|
| Dwarf Fortress | World-simulation-first fortress/legend generator | Best reference for deep emergent history, long-horizon memory, layered social causality, and “the world remembers.” |
| RimWorld | Story generator directed by an AI storyteller | Best reference for readable colony drama, mood clarity, work assignment ergonomics, body-part injury expression, and event pacing. |

For Civis, the key split is not “which game is deeper,” but “which parts should become simulation and which parts should become presentation.” Dwarf Fortress is the stronger reference for emergent historical substrate. RimWorld is the stronger reference for surfacing sim state cleanly enough that the player can actually operate on it.

## Feature & Systems Teardown

### 1) Agent psyche: needs, moods, thoughts, memory, temperament

#### Dwarf Fortress

DF’s dwarf is not a job token. It is a persistent psychological object with:
- needs that can be satisfied or starved,
- thoughts that recur from prior experiences,
- personality facets and likes/dislikes,
- relationships that matter socially and emotionally,
- stress/focus states that influence performance,
- and a long memory of events that can haunt the dwarf repeatedly.

The important mechanical point is that thoughts are not a one-frame UI effect. A past event can keep generating new thoughts when remembered, which means the sim stores emotional residue, not just a score. Needs also interact with facet-driven behavior: a dwarf may drift from “optimal fortress worker” toward “self-directed fulfillment” if the fortress ignores their internal state.

This is the most Civis-relevant lesson from DF psyche: emotional life should be a stateful emergent process, not a list of authored mood enums. The player should not be told “this agent is angry because `ANGRY_AT_BOSS` is set.” The system should expose the causal chain: unmet drive -> interpretation -> memory reinforcement -> behavior drift.

#### RimWorld

RimWorld makes psyche legible. Pawns have needs, mood, thoughts, and breaks that are directly operable by the player. The design strength is that the internal state is decomposed into a readable mood ledger: hunger, rest, recreation, pain, beliefs, social interactions, beauty, expectations, and room quality all contribute to a clear breakdown.

RimWorld’s AI Storyteller matters here too: the game does not just simulate agent psychology; it also paces external pressure so that psychology matters. Mood failures become readable story beats because the director keeps tension in a range the player can understand.

For Civis, this suggests two separate layers:
- psyche should emerge from drives, temperament, social attachments, memory, and interpretation;
- UI should summarize the currently relevant causal factors in a way closer to RimWorld than DF.

### 2) Social networks: relationships, family, friends, enemies, grudges

#### Dwarf Fortress

DF social simulation is best understood as a long-memory network where kinship, friendship, personal history, and grudges accumulate into persistent social topology. The player can inspect that topology, but the game does not flatten it into a single “affinity” number. The legend/history layer also means that social acts can outlive the active fortress: a relationship or murder is not just local, it becomes part of world memory.

This is important for Civis because social network structure should be emergent from repeated interaction and shared context, not hardcoded caste/faction membership. A relationship graph should form naturally from co-presence, kinship, reciprocity, conflict, labor coordination, and memory.

#### RimWorld

RimWorld sociality is more immediately operational. Pawns develop family, lovers, spouses, and social opinions; those relationships feed mood and break risk. Ideology expands this further by turning shared belief systems into explicit social roles and normative pressure.

Mechanically, RimWorld turns social life into a visible source of tactical risk and colony stability. A death, breakup, insult, or betrayal is not just narrative flavor; it is a direct input to mood and future decisions. That makes it ideal as a reference for “social state must matter in decision-making.”

For Civis, the social graph should be emergent and dynamic, but the UI should surface:
- who matters to whom,
- why a relationship changed,
- where conflict clusters are forming,
- and what each agent currently remembers.

### 3) Narrative emergence: RimWorld storyteller vs Dwarf Fortress history engine

#### RimWorld AI Storyteller

RimWorld’s strongest systemic idea is the storyteller director. The game is not trying to be a pure stochastic sim; it is trying to generate a paced story. The storyteller evaluates colony state and chooses incident pressure to create a shape: rising tension, downtime, or chaos. This is not authored plot, but it is authored pacing.

That distinction matters. A raw simulation can be interesting but unreadable. RimWorld inserts a director layer that modulates inputs so the colony’s narrative remains legible.

#### Dwarf Fortress legends/history

DF does the opposite. It builds a deep history machine and lets meaning emerge retrospectively. Legends mode is a historical query surface over a world that has already accumulated centuries of causality, figures, wars, artifacts, and deeds. The drama is not curated in the moment; it is excavated afterward.

The DF lesson for Civis is that emergent history should not be a side log. It should be a first-class world artifact with searchable causality, not just ephemeral notifications.

The RimWorld lesson is that even if the simulation is emergent, the player still needs pacing control and incident readability.

### 4) Health, body parts, wounds, injuries, and surgery

#### RimWorld

RimWorld’s health model is one of the clearest in the genre:
- agents have body parts,
- body parts can be damaged or destroyed,
- injuries affect function and pain,
- injuries interact with treatment, prosthetics, and surgery,
- pain affects mood and breakdown risk,
- health is visible through an explicit medical tab.

The value here is not just anatomical granularity. It is that injury becomes both mechanical and emotional. A damaged arm changes work ability; pain changes mood; mood changes behavior. That closes the loop.

#### Dwarf Fortress

DF also models body parts, wounds, bones, nerves, and long-term impairment, but it is much harder to read. Its simulation depth is often superior in sheer weirdness and consequence, yet the player has to fight the interface to understand it. A dwarf may be permanently disabled by damage that is mechanically obvious only after digging through screens and status text.

For Civis, the takeaway is to separate:
- body-part physics and injury persistence as simulation,
- readable medical summaries and causal breakdowns as UI.

### 5) Job systems, labor permissions, work priorities, and task assignment

#### Dwarf Fortress

DF labor is a permission-based system: jobs exist, labors gate who may take them, and work details can specialize or open duties across the fortress. The strength is that labor assignment is not a single “job priority” list. It is a labor-policy layer, with semi-autonomous dwarves then choosing among permissible tasks.

This gives DF a distinctive social texture:
- the fortress has policy,
- the dwarf has autonomy,
- job completion emerges from both.

#### RimWorld

RimWorld’s work priorities are more player-readable and more immediately tactical. The player can decide who tends to what work, and the pawn’s traits, skills, and needs influence how that workload actually plays out. RimWorld is generally clearer, easier to correct, and less likely to collapse into hidden labor chaos.

For Civis, this argues for:
- emergent assignment behavior driven by task access, skills, location, urgency, trust, and autonomy,
- but a player-facing work priority UI that is much closer to RimWorld clarity than DF opacity.

### 6) Mood breakdowns and player readability

RimWorld is the better mood UX reference by a wide margin. It explicitly decomposes mood into the reasons behind it, so the player can answer:
- what is hurting this pawn,
- what would help next,
- what is chronic versus transient,
- and what will cause a break soon.

DF has deeper hidden psychology, but it is harder to operationalize. That is powerful for legend archaeology, but frustrating for active management.

For Civis, the emotional sim can be dense as long as the UI breaks it into:
- core drives,
- recent events,
- active social pressure,
- bodily stress,
- and medium-term memory influence.

### 7) Memory, grudges, trauma, and persistent relational consequence

DF and RimWorld both understand that a sim without memory is shallow. The difference is orientation:
- DF uses memory to deepen world biography and legend.
- RimWorld uses memory to drive immediate mood and social consequence.

For Civis, memory should likely be multi-layered:
- short-term emotional residue,
- long-term autobiographical memory,
- relationship-specific memory,
- and social reputation/grudge inheritance.

That memory must be emergent, not a hardcoded “grudge enum.” The sim should derive grudges from repeated betrayal, harm, humiliation, failed support, kinship violations, and observed norm-breaking.

### 8) Emergent histories, legends, and world-level after-the-fact meaning

DF is unmatched at turning simulation byproducts into mythic history. The player can inspect an entire world’s legacy, and the world remembers. That is not merely lore flavor. It is a data model for civilization-level continuity.

For Civis, this is the cleanest reference for:
- archived event graphs,
- searchable causal chains,
- historical figures,
- lineage and association graphs,
- and post hoc narrative reconstruction.

This should be treated as emergent world record-keeping, not a scripted chronicle.

## UX / QoL / Bells-and-Whistles

This is where RimWorld is the stronger direct reference and where DF is often the cautionary tale.

| UX area | Dwarf Fortress | RimWorld | Civis implication |
|---|---|---|---|
| Readability | Extremely rich but often opaque | Clean, legible, immediately actionable | Prefer RimWorld-style clarity for active play |
| Historical inspection | Best-in-class legend archaeology | More limited retrospective tooling | Civis should have a first-class history browser |
| Medical UX | Deep but demanding | Explicit and usable | Build a strong medical summary panel |
| Work management | Policy-heavy, power-user oriented | Clear priority control | Use readable work planning with automation hooks |
| Notifications | Functional but cognitively heavy | Strong alerting and tutoring | Civis needs blunt, low-friction alerts |
| Tutorialization | Community/tool-assisted | Built-in adaptive teaching | Civis should teach without hiding mechanics |

Specific QoL lessons to carry forward:
- show why an agent is choosing an action, not just the action result.
- expose the top causal factors for mood, job refusal, injury risk, and social conflict.
- provide search and filtering for relationships, memories, and historical figures.
- provide “at risk” dashboards for starvation, stress collapse, wound infection, grief, and labor shortage.
- use direct overlays for cause/effect rather than nested menu spelunking.

## What It NAILS

- DF nails deep world memory: once a thing happens, it can matter for years.
- DF nails legend-scale continuity: the world is not reset to present tense every tick.
- DF nails emergent causality: small incidents compound into civilization-level consequences.
- RimWorld nails readable colony psychology: mood is transparent enough to manage.
- RimWorld nails narrative pacing: the storyteller keeps pressure in a playable range.
- RimWorld nails injury-to-mood coupling: pain is not isolated from behavior.
- RimWorld nails work orchestration clarity: the player can actually steer labor efficiently.
- Both nail the feeling that people are not “units”; they are unstable bundles of history, body, and social pressure.

## What to ADOPT for Civis

| Item | Tag | Rationale | Emergence charter tension |
|---|---|---|---|
| Agents have persistent drives, temperament, memory, and social attachment | `[EMERGENT]` | Core psyche should arise from simulation state, not authored personality classes. | Low |
| Social networks emerge from repeated contact, kinship, labor, conflict, and reciprocity | `[EMERGENT]` | Avoid fixed faction/relationship enums; let topology arise. | Low |
| Grudges and trauma persist as memory-linked behavior shifts | `[EMERGENT]` | Memory must have downstream behavioral consequence. | Low |
| World history is stored as a searchable event/causality graph | `[EMERGENT]` | This is the DF-style legend layer, but it should be a record of emergence, not a script. | Low |
| Body-part injury affects work ability, pain, and downstream mood | `[EMERGENT]` | Physical damage should propagate into psyche and labor naturally. | Low |
| A readable mood breakdown panel showing current causal contributors | `[UI/QoL]` | Presentation only; does not hardcode behavior. | None |
| A social graph inspector with filters for kinship, affinity, conflict, and memory | `[UI/QoL]` | Essential for managing emergent society at scale. | None |
| A medical/body-part panel with injury severity, functional loss, and treatment state | `[UI/QoL]` | High-value operational visibility. | None |
| A work assignment UI that can express priorities, permissions, and automation hints | `[UI/QoL]` | UI policy is allowed even when the underlying labor selection remains emergent. | None |
| A storyteller-like pressure tuner for scenario pacing in curated modes | `[UI/QoL]` | If implemented, keep it as a presentation/pacing tool, not a social law. | Medium if it starts authoring incidents |
| Historical legend browser with timeline, figure, and relationship search | `[UI/QoL]` | Lets players inspect emergence without hardcoding the outcomes. | None |

### Civis-specific reading of the tags

- `[EMERGENT]` means the engine should not contain explicit “jealousy,” “grief job,” “faction loyalty,” or “mood event” enums as design primitives unless they are directly derived from substrate and memory.
- `[UI/QoL]` means the player is allowed to see or manipulate the system in a human-friendly way, even if the underlying sim remains messy.
- `[LAW]` should be reserved for actual substrate or physical constraints only. For this topic, almost nothing from DF/RimWorld belongs there.

## What to AVOID

- Do not hardcode personality archetypes or fixed social roles as the source of behavior.
- Do not turn factions, cultures, beliefs, or relationships into static enums.
- Do not encode “story beats” as scripted quest chains if the same drama can emerge from pressure, memory, and scarcity.
- Do not bury mood/health causality behind opaque UI; that reproduces DF’s pain without its payoff.
- Do not flatten injuries into a single HP bar if the sim depends on body-part consequence.
- Do not use a storyteller system as a replacement for simulation; use it only, if at all, as pacing/pressure control.
- Do not let work automation become a black box that overrides agent autonomy without explanation.
- Do not treat legends/history as a post-processing export only; if it is not queryable in-world, it loses most of the value.

## Bevy / Rust Ecosystem Notes

Relevant implementation ideas to reuse or study in Rust rather than hand-rolling blindly:
- `bevy` for ECS orchestration, world queries, and UI integration.
- `bevy_egui` or native Bevy UI patterns for inspectable debug panels, though Civis may want a more customized shell for dense simulation readouts.
- `petgraph` for relationship and memory graphs if the social network becomes graph-heavy.
- `pathfinding` or `pathfinding`-style crates for labor routing and agent task selection when the problem is graph/search oriented.
- `serde` + durable event log formats for legend/history reconstruction.

The important note is architectural: the social, emotional, and historical layers should probably be modeled as data and event streams first, then rendered through UI lenses. That matches the emergence charter better than authoring social rules directly.

## Sources

Real URLs consulted:
- https://rimworldgame.com/
- https://rimworldgame.com/index.php?lang=en
- https://rimworldgame.com/index.php/ideology
- https://rimworldgame.com/index.php/biotech
- https://store.steampowered.com/app/294100/RimWorld/
- https://rimworldwiki.com/wiki/AI_Storytellers
- https://rimworldwiki.com/wiki/Needs
- https://rimworldwiki.com/wiki/Health
- https://rimworldwiki.com/wiki/Thoughts
- https://rimworldwiki.com/wiki/Body_Parts
- https://dwarffortresswiki.org/index.php/Legend
- https://dwarffortresswiki.org/index.php/Needs
- https://dwarffortresswiki.org/index.php/Labor
- https://dwarffortresswiki.org/index.php/Thought
- https://dwarffortresswiki.org/index.php/Thoughts_and_Preferences
- https://store.steampowered.com/app/975370/Dwarf_Fortress/
