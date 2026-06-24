# Cities: Skylines 1 & 2 Teardown for Civis Spec Map

## Overview

Cities: Skylines 1 (CS1) and Cities: Skylines II (CS2) are the reference baseline for modern city builders that combine road-first urban planning, zoning-driven growth, service coverage simulation, and a UI language built around overlays and continuous feedback. For Civis, they matter less as “the best city sim” and more as the strongest mainstream example of how to expose a complex settlement system to a player without making the simulation opaque.

CS1 is the mature, content-rich reference: extremely legible zoning, strong service radius mental models, useful info views, district/policy scaffolding, and a mod ecosystem that turned the game into a platform. CS2 is the more ambitious systems refactor: deeper citizen simulation, stronger road-building affordances, richer visual scale, and more explicit agent-level feedback, but also a launch-era example of how a heavier sim can regress in UX clarity, performance trust, and parity with a more polished predecessor.

For Civis, the key lesson is not “copy city builder rules.” It is “copy the player-readable control surface.” Civis hardcodes physics, materials, climate, and genomics; the city-builder layer should be an authored interaction shell that helps users shape emergent settlement patterns, not a fixed social script.

## Feature & Systems Teardown

### 1) Zoning

| Topic | CS1 | CS2 | Civis read |
|---|---|---|---|
| Zone placement | Rectilinear zone painter on roads; most growth is “paint then wait.” | Same core model, but more explicit handling of mixed densities and improved visual legibility. | Use zoning only if it remains a thin UI contract over emergent land-use pressures, not a hardcoded city outcome. |
| Dependence on roads | Zones only activate adjacent to valid road frontage. | Same, with road/parcel presentation made more expressive. | Good as a UI affordance; “frontage-adjacent growth” can be emergent in Civis. |
| Density split | Low/high density, commercial/industrial, plus specialized industrial in later CS1 content. | Same broad categories; more granular simulation beneath the hood. | Avoid fixed social categories as simulation laws; let land value, access, pollution, and labor availability generate use patterns. |
| Growth logic | Demand bars drive zone demand; buildings self-grow from zone supply. | Still demand-driven, but citizen/job simulation is more visible and more agent-centric. | The “demand bar” is a useful presentation abstraction, not a reality model. |

CS1’s zoning is mechanically simple and therefore easy to read: road frontage determines valid cells, demand bars determine what gets painted, and building growth turns abstract demand into visible density. It is effective because it compresses a large urban system into a small number of clear actions. The downside is that the player learns to manage bars, not systems.

CS2 retains the basic paint-to-grow grammar but pushes toward more legible parcel-scale behavior and more visible agent outcomes. That is the right direction for Civis if the goal is to expose how settlement actually emerges from terrain access, infrastructure, and resource gradients. What Civis should not copy is the implicit promise that zoning categories are the city itself. In Civis, any land-use classification should be a UI layer over emergent pressures.

### 2) Road Tools

| Capability | CS1 | CS2 | Civis implication |
|---|---|---|---|
| Curves | Road segments can be curved, but the older toolchain is relatively coarse and node-centric. | Stronger curve editing, better terrain adaptation, more deliberate previewing. | Must-have UI/QoL. Users need expressive placement without fighting the tool. |
| Snap | Snapping is central to grid coherence and zoning validity. | Improved snapping and preview feedback. | UI/QoL. Snapping is presentation and construction aid, not a law. |
| Upgrade | Road upgrades preserve alignment and many connected systems. | Better road selection/upgrading ergonomics and more granular road types. | UI/QoL. Upgrades should preserve derived infrastructure state. |
| Hierarchy | Local streets, collectors, arterials, highways, special road types. | Same, with more explicit visual hierarchy and intersection tooling. | Road class can be authored as infrastructure affordance, but traffic behavior should emerge from usage and capacity. |

CS1’s road tools are foundational but often feel like a compromise between precision and convenience. CS2 invests heavily in road-building readability: the player gets more context while drawing, and road geometry behaves more predictably around slopes, curvature, and intersections. For Civis, this is a direct UI/QoL model worth copying. A settlement game that expects users to sculpt transport networks needs a first-class road editor, not just a placement gizmo.

The key distinction for Civis is that road tools are allowed to be highly authored while the consequences of roads should stay emergent. Road class, width, curvature, and connectivity can be explicit player controls. Traffic intensity, commerce clustering, settlement expansion, and route choice should arise from physical and economic conditions.

### 3) Service Buildings + Coverage

CS1 treats services as coverage appliances: fire, police, healthcare, education, garbage, utilities, deathcare, transit, and some specialty systems create radius- or network-based service reach. The player learns to read service coloring, coverage overlays, and budget sliders as a practical proxy for public administration. This is one of the clearest parts of the game because each service has a direct “what happens when missing” failure mode.

CS2 keeps the service-building model but generally raises simulation visibility. Service buildings remain distribution centers for public functions, but the game spends more effort on individual agents, pathing, and infrastructure dependency. The result is a more physically grounded feel, especially when service delays ripple through actual travel time instead of only coverage math.

For Civis, this is a strong pattern if reinterpreted carefully. “Coverage” should not mean a magic influence circle. It should mean a physically grounded service reach based on travel time, capacity, queueing, and state of access. The UI can still expose a coverage overlay, but the underlying rule should be pathable access, not an abstract aura.

### 4) Info-View Overlays

The info-view stack is one of the strongest reasons CS1 became the reference point for city builders. It provides a dense but navigable diagnostic layer for reading the city:

- zoning and land value
- residential/commercial/industrial demand and density
- traffic and route congestion
- happiness and citizen well-being
- health, deathcare, fire safety, crime, education, pollution, noise
- garbage, water, sewage, electricity, heating where applicable
- public transport usage
- economy, budget, taxation, and profitability
- tourist flow and attraction

CS1’s overlays are useful because they are operational, not decorative. They tell the player which subsystem is failing and where. CS2 continues this model and improves surface presentation, but the larger lesson is unchanged: a settlement sim without layered overlays becomes unreadable.

For Civis, overlays are not optional; they are the bridge between emergent simulation and player agency. However, the overlay names themselves should be treated as UI labels, not hardcoded world truths. For example, “pollution,” “access,” “safety,” “comfort,” and “productivity” are valid diagnostic lenses; the exact social meaning behind them should emerge from physical/material/biological conditions, not from a preset civic ontology.

### 5) Citizen / Cim Lifepath Simulation

CS1’s cims are primarily functional agents in a city-service loop: they live, commute, work, shop, visit services, age, die, and generate demand. The simulation is deep enough to support traffic and service interactions, but still mostly legible through aggregate behavior.

CS2 pushes harder into individual simulation. Citizens have more explicit daily routine structure, more visible travel and household logic, and a stronger sense that the city is being “inhabited” rather than merely balanced via bars. The improvement is qualitative: the city feels like a collection of agents with life paths, not a spreadsheet of population buckets.

For Civis, this is the most important reference area after road tools. A civilizational god-game needs the user to understand how individuals and groups move through space, consume services, form households, and stress infrastructure. But Civis should avoid fixed life-path enums as design law. Lifepaths should emerge from needs, geography, work, kinship, and available institutions. The player should see patterns, not canned classes.

### 6) Economy / Budget / Taxation

CS1’s economy is an abstract municipal model: income from taxes, expenses from services, loans, grants, land/building value, and demand bars that indirectly regulate growth. It is easy to grasp and provides just enough pressure to make budget decisions matter.

CS2 expands the economic layer with more explicit household and business behavior, but the player-facing model still orbits around city finances, service costs, taxation, and growth incentives. The important part is not realism in the fiscal model; it is the feedback loop between public spending, service reliability, and settlement attractiveness.

For Civis, keep the player-facing fiscal panel, but do not hardcode a single “municipal economy” as the world model. The engine should simulate resource flows, labor, trade, storage, and exchange as emergent markets or administered systems depending on conditions. The budget panel can remain a UI/QoL projection of those flows.

### 7) Traffic AI

Traffic is the canonical Cities systems benchmark. In CS1, congestion emerges from route choice, capacity, intersection design, and service access. The game’s route-finding is not “realistic traffic engineering,” but it is convincing enough that players immediately learn to read network topology. That is the real achievement: road design becomes strategy.

CS2 makes traffic feel more granular and in many situations more physically grounded because the rest of the sim is more agent-centric. The player gets stronger feedback about why jams happen, though the game also inherits the classic city-builder problem: players will optimize network structure for throughput rather than social realism unless the simulation explicitly rewards other goals.

For Civis, traffic AI is a major adoption target, but only at the physics/transport layer. Route selection, path saturation, queueing, and intersection behavior should emerge from network geometry, vehicle constraints, and local incentives. Do not hardcode “commuter logic” or fixed trip-purpose classes beyond what is needed for the UI to explain behavior.

### 8) Districts + Policies

Districts in CS1 are a powerful abstraction: they let the player carve the city into named zones of governance and attach policies to them. Policies then modify behavior in a highly legible way: taxes, bans, restrictions, specialized behaviors, and district-level rules. This makes the city feel administrable.

CS2 keeps this structure and uses it as a bridge between macro-governance and local patterning. Districts are useful because they separate physical urban form from political/administrative intent.

For Civis, districts are high-risk if turned into hardcoded sociopolitical containers. The safe version is a UI/admin layer over emergent neighborhoods, regions, jurisdictions, and service areas. Policies can exist as authored rule toggles only if they are framed as external interventions on physical/economic constraints rather than as intrinsic properties of a culture or faction.

### 9) UI Ergonomics

CS1’s UI is dense but functional. It teaches through repetition: road controls, zoning, overlays, budgets, and service icons all become part of a single operator mental model. Its weaknesses are mostly discoverability and late-era UI clutter, especially once mods and DLC expand the system surface.

CS2 improves some interaction flows, especially road placement and visual scale, but also shows how quickly a richer sim can become overwhelming if not aggressively surfaced through contextual UI, warnings, and smart defaults. The ideal UI for a city builder is not “minimal.” It is “contextual, predictive, and reversible.”

For Civis, the lesson is direct: the simulation can be as deep as needed, but the UI must continuously answer four questions:

- What changed?
- Why did it change?
- What happens if I act now?
- What can I safely undo?

That is the difference between an approachable sim and an opaque one.

### 10) CS2 vs CS1: Improvements and Regressions

| Area | CS2 improvement | CS2 regression / risk | Civis takeaway |
|---|---|---|---|
| Road tools | Better building ergonomics, terrain handling, and preview confidence. | More dependency on tool polish; if broken, the whole game feels worse. | Put major engineering effort into road UX. |
| Citizen simulation | More agent-centric and lifelike. | Higher CPU/UX cost; less forgiving if presentation is weak. | Deep sim needs better diagnostics. |
| Visual scale | Larger, more modern, more readable at distance. | Can mask simulation weakness if feedback is insufficient. | Visual fidelity must be paired with strong overlays. |
| Systems clarity | More explicit in some areas. | Launch-era performance/UX trust issues and feature parity gaps. | Never ship deep sim without a strong operator layer. |
| Mod ecosystem | Potentially cleaner technical base. | Hard launch/mod friction can damage platform trust. | Mod support must be designed, not bolted on. |

The high-level verdict: CS2 is the right direction technically, but CS1 still often wins on “player confidence per minute.” Civis should emulate CS2’s depth and CS1’s operability.

### 11) Modding

CS1 became a giant because the mod ecosystem expanded every system boundary: UI, road tools, traffic logic, economy tweaks, asset packs, quality-of-life automation, and total conversions. Modding was not just additive content; it was system correction.

CS2’s modding story is more controlled and modernized, but it also illustrates the costs of under-serving player extension at launch. A city-builder platform lives or dies on whether players can patch friction, experiment with systems, and share maps/assets without fighting the engine.

For Civis, modding should be treated as an ecosystem contract, not a content afterthought. However, because Civis’s core is emergent-law simulation, mods must be constrained carefully: they should extend materials, entities, assets, UI, scenarios, and authorized rule parameters, not bypass the physical substrate or encode social enums as permanent engine truth.

## UX / QoL / Bells-and-Whistles

### What works

- Overlays are persistent and actionable, not just pretty.
- Most tools support “see state, change state, immediately see effect.”
- Road building has enough snap and preview feedback to be learnable.
- Service coverage creates a useful mental model for infrastructure planning.
- District policies give the player a macro-control panel without requiring micromanagement of every building.
- The interface is built around a city operator fantasy: the player is not merely decorating, they are running a system.

### What feels expensive or brittle

- Too many meanings are compressed into colored overlays, which can hide causal chains.
- Several systems depend on the player inferring simulation state from indirect feedback.
- As the content surface expands, toolbar complexity and notification density climb quickly.
- Some UX wins are historical rather than structural; they need constant rework as the sim deepens.

### QoL details Civis should mirror

- Contextual tool previews before commit.
- Fast undo for placement and routing work.
- Smart warnings when a structure is invalid, under-served, or over capacity.
- Layered overlays that can be toggled without losing current task context.
- Immediate state changes on hover/selection wherever possible.
- Consistent icon language for service type, capacity, coverage, and failure mode.
- A build workflow that minimizes mode switching.

## What it NAILS

- It turns roads into the core strategic substrate of the city.
- It makes zoning readable enough for non-experts to learn quickly.
- It gives the player operational visibility through overlays.
- It makes service planning concrete through coverage and failure feedback.
- It links budget and taxation to visible city consequences.
- It uses district policy as a bridge between city form and governance.
- It supports a strong “operator” fantasy without requiring heavy textual explanation.
- It set the genre standard for mod-driven extension of a city sim.

## What to ADOPT for Civis

- `[LAW]` Physical travel-time-based service reach instead of magic radius coverage. Rationale: keeps public services grounded in pathing and infrastructure. Charter tension: none, if the only hardcoded part is transport physics.
- `[LAW]` Road geometry and connectivity as first-class physical infrastructure. Rationale: roads are part of the substrate and should shape settlement patterns. Charter tension: low, because roads are allowed to be authored infrastructure.
- `[EMERGENT]` Neighborhoods, districts, and land-use clusters should emerge from access, noise, safety, value, and social affinity. Rationale: avoids hardcoded civic enums. Charter tension: direct if districts are treated as intrinsic identities rather than derived clusters.
- `[EMERGENT]` Household, commute, and shopping loops should arise from needs and geography rather than scripted citizen roles. Rationale: lifepaths should be outputs, not classes. Charter tension: direct if job/family/faction types are hardcoded.
- `[EMERGENT]` Traffic congestion, chokepoints, and route choice should emerge from capacity and topology. Rationale: makes the transport layer strategically meaningful without scripting outcomes. Charter tension: none.
- `[UI/QoL]` CS-style overlays for access, pollution, danger, congestion, capacity, service reach, and budgets. Rationale: Civis needs an operator-grade diagnostic layer. Charter tension: none.
- `[UI/QoL]` Road preview, snapping, curve editing, upgrade-in-place, and fast undo. Rationale: this is pure interaction polish. Charter tension: none.
- `[UI/QoL]` District/policy UI as a management layer over emergent regions, not a simulation primitive. Rationale: lets players steer without hardcoding politics. Charter tension: moderate if policies start encoding social ontology.
- `[UI/QoL]` Notification and warning system that explains failures in plain language. Rationale: deep sim needs causal feedback. Charter tension: none.
- `[UI/QoL]` Mod-friendly extension points for maps, assets, UI, scenarios, and rule parameters. Rationale: critical for longevity. Charter tension: safe if core substrate stays protected.

## What to AVOID

- Hardcoded population or citizen archetype enums that pretend to model society.
- Fixed ideology, faction, or district identity systems that replace emergence with script.
- Service “auras” that ignore routing and capacity.
- UI that only shows aggregate bars without drill-down.
- Road tools that are too restrictive to express intent or too permissive to preserve validity.
- Over-reliance on static demand bars as if they were the simulation itself.
- Launching a deep sim without strong performance trust and feedback loops.
- Mod systems that are open enough to fragment the simulation contract or bypass substrate rules.

## Bevy / Rust Ecosystem Notes

- `bevy_mod_picking` is useful if Civis wants selection and inspection interaction without custom ray-pick plumbing.
- `bevy_ecs_tilemap` is relevant only if a 2D/orthographic planning layer is needed; it should not define the 3D substrate.
- `leafwing-input-manager` can help keep road-tool hotkeys, mode switching, and contextual bindings sane.
- `iyes_loopless` is less relevant on modern Bevy than it used to be, but the pattern of explicit state gating is still worth borrowing conceptually for tool modes and overlays.
- For traffic/pathing, a custom Rust ECS system is still likely necessary; the reference value here is the behavior, not a crate shortcut.

## Sources

- https://www.paradoxinteractive.com/games/cities-skylines/about
- https://www.paradoxinteractive.com/games/cities-skylines-ii/about
- https://store.steampowered.com/app/255710/Cities_Skylines/
- https://store.steampowered.com/app/949230/Cities_Skylines_II/
- https://www.paradoxinteractive.com/games/cities-skylines/news
- https://cities-skylines.fandom.com/wiki/Info_views
- https://cities-skylines.fandom.com/wiki/Districts
- https://cities-skylines.fandom.com/wiki/Policies
- https://cities-skylines.fandom.com/wiki/Traffic
- https://cities-skylines.fandom.com/wiki/Zoning
