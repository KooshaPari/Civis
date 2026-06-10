# Civilization VI & Old World (4X) Teardown for Civis Spec Map

## Overview

Civilization VI and Old World are both high-polish historical 4X games, but they solve the genre in opposite ways.

- Civilization VI is the more authored, legible, and boardgame-like system. It turns civilization progress into discrete tracks: techs, civics, districts, governors, great people, agendas, religions, and victory conditions. It is excellent at presenting complex strategic state through clean icons, panel-first UI, and highly readable unlock chains. It is also highly dependent on hardcoded abstractions that flatten society into enums and scripted bonuses.
- Old World is the more simulation-flavored, character-driven, and event-driven system. It makes the player rule through families, characters, orders, legitimacy, ambitions, laws, and stochastic events. It is less visually clean than Civ VI, but more structurally aligned with succession, institutional drift, and emergent narrative.

For Civis, these two games together define the current 4X bar: Civ VI nails discoverability and strategic clarity; Old World nails multi-generational politics and event-shaped narrative. The key design lesson for Civis is not "copy their trees," but split authored presentation from underlying emergence: preserve the UI legibility of 4X, while ensuring the simulated world is not reduced to hardcoded social enums.

## Feature & Systems Teardown

### 1) Tech / civic trees: the core tension between readability and emergence

#### Civilization VI

Civ VI uses a pair of visible progress trees: Technology and Civics. The official framing is explicit: technology handles scientific development while civics handles social/political development, and both are paid forward by yield accumulation each turn. This structure is strong for player comprehension because every node has a known unlock, cost, and branching dependency. It is also the main source of Civ VI's simulation flattening: "science" becomes a universal currency for all material discovery, and "culture" becomes a universal currency for all social development, regardless of actual local conditions or institutional pathways.

Mechanical consequences:

- The tree is authored knowledge, not discovered knowledge. The player is selecting from a menu of predefined steps in history.
- The system is globally synchronized and teleological: every civilization is moving through the same canonical progress lattice.
- The tree encodes a historical inevitability model. Even when leaders differ, the underlying path is still a fixed tech ladder.
- Eurekas and inspirations soften the hard linearity, but they do not change the core fact that the destination graph is predetermined.
- Babylon in Civ VI is a revealing exception: the official design explicitly gives Eurekas full tech completion. This is the game acknowledging that discrete unlock-by-discovery can be more interesting than raw yield accumulation, but it still sits inside a fixed node graph.

Why this tensions Civis emergence:

- A hardcoded tech tree is a hardcoded ontology of civilization. It assumes a fixed set of inventions, fixed dependency order, and fixed social packaging.
- Under the emergence charter, technologies should not be treated as predefined culture-free enums. They should arise from the interaction of materials, energy, labor organization, observation, memory, and institutional transmission.
- A tech tree makes the sim answer "what can exist?" before the world has produced the conditions that would make it exist.

#### Old World

Old World is materially closer to a discovery model even though it still has authored card-like techs. Its technology system is not a classical static tree; it is a deck. You are dealt options from a draw pile, you pick one, unused cards go to the discard pile, and the deck reshuffles. That means the game still has authored nodes, but the path through them is constrained by draw, redraw, and partial information rather than pure linear planning.

Mechanical consequences:

- Research is not a perfect deterministic queue; it is semi-stochastic and offers tactical improvisation.
- Scholar archetypes and redraws matter because the system is about opportunity management, not idealized build ordering.
- The system allows the player to react to immediate state, not only to a long-planned tech script.
- Laws are separate from technology, which is important: social structure is not merged into a science tree. Laws are a political axis with mutual exclusion and strategic tradeoffs.

This is still authored content, but it is a better approximation of historical contingency than Civ VI's rigid lattice.

#### Civis implication: tech as discovered laws, not a tree

For Civis, the right model is not "no research UI". The right model is:

- Underlying simulation tracks observed phenomena, constraints, and institutional capacity.
- The UI surfaces candidate formalizations, traditions, techniques, and doctrines as discovered laws.
- A "technology" is a codified, transmissible law-like pattern the society has actually discovered, stabilized, and reproduced under local conditions.
- Unlocks should be contingent on evidence, materials, labor organization, and diffusion networks, not a universal canonical tree.

Concretely, Civis should treat technologies as a discovered-law catalog with provenance:

- a phenomenon is observed in the world,
- a model is proposed by agents,
- the model is tested and iterated,
- institutional adoption spreads the law through memory, schooling, guilds, and trade,
- the UI records the law as a named capability.

That keeps the player-facing legibility of a tech tree without hardcoding social progress as a fixed ladder.

### 2) Diplomacy and AI agendas

#### Civilization VI

Civ VI's diplomacy is intentionally readable, but it is also heavily mediated by leader agendas. The official site says leaders pursue their own agendas based on historical traits, and their interactions evolve over time. In practice, the agenda system gives each leader a personality template with visible and hidden preferences.

Mechanical consequences:

- The player gets a predictable strategic read on other leaders, but the AI is still executing authored behavioral biases.
- Diplomacy is less about negotiation over concrete state and more about satisfying or violating hidden preference checks.
- The system gives flavor and replay variation, but it can feel synthetic because it reduces political behavior to weighted rules.

For Civis, agenda-like behavior is only acceptable if it emerges from actual incentives, memory, institutional affiliation, kinship, and environmental constraints. The UI may summarize motives, but the motive generator should not be an enum list of leader archetypes.

#### Old World

Old World is much closer to a political simulation. Legitimacy comes from ambitions, cognomens, and events; ambitions become legacies when leaders die; each point of legitimacy directly yields family opinion and orders. Characters sit on a council and can produce different outputs depending on role and attributes. Families, opinion networks, and leader continuity make diplomacy feel like an extension of court politics rather than a separate minigame.

Mechanical consequences:

- Foreign policy is entangled with domestic legitimacy and family management.
- Political outcomes persist across generations; this makes diplomacy narrative rather than purely transactional.
- Leaders are not abstract nation avatars. They are characters with age, role, and evolving status.

This is the more Civis-compatible pattern because it is already closer to agentic social structure. The remaining issue is that Old World still constrains social reality into authored roles and attributes rather than letting those roles emerge fully from the substrate.

### 3) Culture and religion spread

#### Civilization VI

Civ VI makes religion and culture explicit map-layer systems. Religion is founded, then spread by units and pressure; culture feeds civics and tourism; the game clearly separates these as strategic resources. This is excellent for player understanding and victory planning.

Strengths:

- Clear feedback loops: faith, religion founding, spread units, and religious victory are visible and actionable.
- Culture is a strategic throughput resource rather than only a score.
- Players can reason about pressure and conversion as an explicit map state.

Weaknesses for Civis:

- Religion is still a predefined belief container with authored choices.
- Culture is often reduced to yield accumulation rather than organically emerging norms, language, and institutions.
- Spread is modeled as a game-system contagion, not as a coupled social, logistical, and identity process.

#### Old World

Old World handles religion in a more socially entangled way. Cities and characters each follow religions; opinions matter; state religion can be adopted; theologians and doctrines modify spread and yields; dissent can emerge if opinion worsens. Culture is a city meter that advances culture level and triggers positive events.

Mechanical consequences:

- Religion is not just a map overlay. It affects opinion, discontent, legitimacy, and city yields.
- Spread is embedded in networks of city proximity, opinion, family adoption, and state structure.
- Culture is directly tied to city maturation and event generation, not only to empire-wide progress.

For Civis, the key takeaways are:

- treat religion as emergent doctrine, ritual, and identity cluster rather than as a top-level enum;
- treat cultural spread as memetic transmission across contact networks, trade, kinship, and institutions;
- expose these states clearly in UI, but do not hardcode the cultural outcomes.

### 4) City growth and districts

#### Civilization VI

Civ VI's cities physically expand onto the map, and districts are terrain-anchored specializations. This is one of the game's best systemic ideas because it externalizes city planning into geography.

Why it works:

- The map becomes a city planning puzzle rather than a generic production queue.
- Adjacency creates spatial strategy: terrain, layout, and district placement matter.
- Districts make cities visibly legible from the world map.

Why it is limiting for Civis:

- Districts are still authored building categories, not emergent settlement morphology.
- The city pattern is a fixed taxonomy of functions.
- The simulation does not actually model organic urban morphology, infrastructure gradients, or spontaneous land-use clustering.

#### Old World

Old World city growth is more layered. Cities have growth, culture, and happiness bars; culture level unlocks benefits and events; family assignment matters; improvements and specialists alter outputs; religion and laws feed back into growth. This is a more process-driven city model than Civ VI's district puzzle.

Mechanical consequences:

- City development is not just "place district on best tile".
- Growth, culture, happiness, and discontent are interacting subsystems.
- Event triggers make city progression feel stateful rather than purely optimizable.

For Civis, the lesson is to make cities emergent settlements with districts as a UI projection, not as the simulation primitive.

### 5) Old World character, dynasty, orders, and event-driven narrative

Old World is the clearest example here of a 4X game trying to be a dynasty strategy game.

Key systems:

- Characters have age, roles, strengths, weaknesses, and family ties.
- Council positions create structural benefits and costs.
- Orders are the central turn-limiting action currency.
- Legitimacy is accumulated from ambitions, cognomens, and events, and directly affects Family Opinion and Orders.
- Ambitions are short-horizon goals; once a leader dies they become legacies.
- Random and structured events shape narrative trajectories.

This is an important design bridge to Civis:

- Narrative should come from agent history, institutional continuity, and event cascades, not from canned quest scripts.
- Authority should be distributed through characters and offices, not only through a sovereign civilization object.
- Orders are a good abstraction for command bandwidth, but in Civis the equivalent should emerge from communication, hierarchy, distance, and cognitive load rather than being a universal meta-currency.

Old World proves that a 4X can become dramatically more interesting when it stops pretending the nation is a single mind.

### 6) Great people

#### Civilization VI

Great People are a pure abstraction for exceptional individuals. They are recruited in categories, used for one-shot effects or passive benefits, and occupy a strategic race space. Civ VI also adds 24 Great People in some packs, showing how expandable the model is.

Strengths:

- Easy to understand.
- Gives the player intermittent high-impact decision moments.
- Makes elite knowledge and talent into a strategic resource.

Weaknesses for Civis:

- Individuals are pretyped into great scientist / great artist / great merchant boxes.
- The model is reward-gated and gamey rather than socially generated.
- It conflates fame, capability, institutional role, and influence into one resource loop.

#### Old World

Old World's characters already do some of the work that Great People do, but in a more institutional form. Characters can become governors, generals, councilors, ambassadors, spouses, heirs, and leaders, with attributes shaping outcomes.

For Civis, the better model is not a "Great Person" spawn pool. It is:

- exceptional agents whose reputation, capability, and institutional placement emerge from the underlying sim,
- visible ranks or offices in the UI,
- no universal category of human excellence hardcoded as a civilization currency.

### 7) How 4X strategic UI presents complex state

This is where Civ VI remains the benchmark.

What Civ VI does exceptionally well:

- Clear iconography for yields, unlocks, and modifiers.
- Strong tree visualization for long-term planning.
- Highly readable city production, district placement, and leader interactions.
- Layered information density that remains navigable for new players.
- Tutorialization and guided onboarding that make an otherwise opaque strategy game approachable.

What Old World does well:

- Character portraits and role-based tabs make court politics legible.
- Explicit bars for growth/culture/happiness give city state a visible lifecycle.
- Orders, legitimacy, and opinion are surfaced as governing constraints rather than buried stats.
- Events create an understandable narrative rhythm.

Civis should combine the best of both:

- UI must be clear, compressed, and inspectable.
- Simulation should remain emergent under the hood.
- The UI may name patterns, cluster them, and recommend actions.
- The UI must not become the simulation truth source.

## UX / QoL / Bells-and-Whistles

### Civilization VI

Civ VI's UX is one reason the genre became broadly legible.

Notable strengths:

- Strong map readability with physical city expansion.
- Obvious unlock progression through tech/civics trees.
- Tooltips and cross-referenced modifiers make the game teach itself.
- Leaders, agendas, and diplomacy are easy to scan.
- Victory conditions are well signposted.

Pain points relevant to Civis:

- Tree browsing can become a cognitive crutch rather than a discovery interface.
- Too much of the game is optimized around clean menus instead of world-readable simulation.
- The player can feel like they are pushing author-defined buttons rather than inhabiting a world.

### Old World

Old World's UX is denser and more diagnostic, but less immediately elegant.

Strong points:

- Orders are a clean, global command budget that prevents turn paralysis.
- Characters are interactable and readable as units of narrative and policy.
- City resource bars expose progression and event triggers.
- The game makes legitimacy, opinion, and culture visible enough to manage.

Weak points:

- More opaque than Civ VI in some systems because the sim is richer and less tutorialized.
- Many processes are buried in layered rules and specialized terms.
- The UI is not always as instantly apprehensible as Civ VI's best panels.

For Civis, QoL should be treated as simulation accessibility infrastructure:

- inspectable causality chains,
- explainable recommendations,
- world overlays for emergent structures,
- action previews,
- timeline/event logs,
- stable hotkeys and camera workflows,
- undo where simulation integrity allows it,
- and strong onboarding for each layer of complexity.

## What it NAILS

- Civ VI nails strategic readability. A player can understand empire state at a glance because almost everything is expressed as a known icon, meter, or node.
- Civ VI nails the physical-city map puzzle. Districts turn geography into strategic shape.
- Civ VI nails leader flavor. Agendas create personality and replay variation.
- Old World nails multi-generational politics. Characters, legitimacy, and ambitions make time matter.
- Old World nails event-driven narrative. Outcomes feel like history accumulating, not just number go up.
- Old World nails command bandwidth. Orders are a good throttle on action explosion.
- Old World nails the connection between culture, religion, opinion, and city outcomes.
- Both games nail the presentation of complex state through clean summary UI, even when the underlying simulation is simplified.

## What to ADOPT for Civis

- [LAW] Model material prerequisites, production constraints, and local observations as the basis for "discovered laws" rather than a fixed tech tree. This preserves discovery while avoiding a hardcoded ontology; the tree itself should not be the truth source.
- [EMERGENT] Let social institutions, norms, doctrines, and specializations arise from repeated agent behavior, diffusion, and local utility. Civ VI-style civic trees should not be hardcoded social ladders.
- [UI/QoL] Build a tree-like discovery browser as a presentation layer if needed, but make it a view over discovered laws, not a simulation structure.
- [EMERGENT] Make diplomacy depend on actual incentives, memory, kinship, logistics, fear, tribute, and reputation. Civ VI agendas are useful as UI summaries, but not as authored personality enums.
- [UI/QoL] Expose diplomatic motives, historical grievances, and expected reactions in a compact panel so the player can reason about AI behavior.
- [EMERGENT] Represent culture and religion as diffusing belief/ritual/language clusters that spread through contact networks, institutions, and prestige. Old World/Civ VI show why this needs to be visible, but not hardcoded.
- [UI/QoL] Provide clear overlays for cultural influence, religious pressure, and institutional reach, with explainable spread sources.
- [EMERGENT] Model cities as settlement ecosystems with growth, infrastructure, specialization, and local path dependence. Districts should emerge as spatial patterns or be derived from those patterns, not define them.
- [UI/QoL] Keep Civ VI-style map legibility: inspectable city extents, adjacency effects, and projected specialization zones.
- [EMERGENT] Use characters, offices, legitimacy, and family networks to carry political continuity across generations. Old World is a strong reference for how this can generate narrative without scripting outcomes.
- [UI/QoL] Surface office assignment, lineage, opinion, and succession clearly so the player can manage continuity instead of guessing.
- [EMERGENT] Treat great people as exceptional agents emerging from the world rather than a separate reward pool of predefined archetypes.
- [UI/QoL] If exceptional agents are surfaced, show them as ranked people with provenance, not as abstract collectible cards.
- [UI/QoL] Invest heavily in tooltips, compare panes, filters, search, event logs, and undo affordances where safe. This is the difference between a rich sim and a frustrating one.

## What to AVOID

- [LAW] Avoid a fixed science tree that hardcodes the order and contents of civilization progress.
- [LAW] Avoid hardcoded social tech/civic enums that pretend every society shares the same canonical path.
- [EMERGENT] Avoid leader agendas as the sole explanation for diplomacy. That is personality theater, not simulation.
- [EMERGENT] Avoid pretyped "great people" as a universal elite currency detached from actual institutional formation.
- [EMERGENT] Avoid religion as a mere faction tag or yield buff list.
- [EMERGENT] Avoid districts as the base simulation object if the goal is emergent settlement morphology.
- [UI/QoL] Avoid burying state in hidden rules. If a system is emergent, the player still needs readable diagnostics.
- [UI/QoL] Avoid over-reliance on novelty UI that hides causality behind visual polish.

## Bevy / Rust ecosystem notes

- A Civis equivalent of the Civ VI/Old World strategic UI should likely be built as a data-driven inspectable layer in Bevy rather than a bespoke one-off panel system.
- For emergent social and economic state, prefer ECS-friendly representations and event logs over hardcoded manager singletons.
- If a tree-like discovery browser is needed, it should be rendered from simulation data, not authored as the simulation itself.
- For narrative/event systems, a Rust event-sourcing or rule-engine layer can support explainable state transitions better than ad hoc scripts.

## Sources

- https://civilization.2k.com/en-US/civ-vi/
- https://civilization.2k.com/civ-vi/new-frontier-pass/babylon/
- https://civilization.2k.com/civ-vi/new-frontier-pass/byzantium-gaul/
- https://wiki.hoodedhorse.com/Old_World/Technology
- https://wiki.hoodedhorse.com/Old_World/Religion
- https://wiki.hoodedhorse.com/Old_World/Characters
- https://wiki.hoodedhorse.com/Old_World/Cities
- https://wiki.hoodedhorse.com/Old_World/Legitimacy
- https://wiki.hoodedhorse.com/Old_World/Resources
- https://wiki.hoodedhorse.com/Old_World/Laws
