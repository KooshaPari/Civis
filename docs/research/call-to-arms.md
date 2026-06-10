# Call to Arms / Men of War Teardown for Civis Spec Map

## Overview

`Call to Arms` sits on top of the `Men of War` lineage as a hybrid RTS / direct-control battlefield simulator: a commander layer for issuing broad orders, and a hands-on layer where you can jump into a single soldier, gun, or vehicle and manually fight, aim, haul ammo, or operate equipment. That duality is the reason it matters for Civis. Civis wants a tactical layer that is not “RTS only” and not “third-person shooter grafted onto strategy,” but a coherent stack where operational intent and low-level execution can coexist.

The lineage matters more than the single SKU:

- `Men of War` established the series grammar: direct control, inventory micromanagement, destructible environments, ammo-level logistics, and vehicle crew simulation.
- `Men of War: Assault Squad` and `Assault Squad 2` refined the battlefield readability and multiplayer pacing while retaining the same unit-level mechanical identity.
- `Call to Arms` modernized presentation and expanded the same core loop into a more contemporary RTS command surface, while still preserving the direct-control inheritance and battlefield improvisation.

For Civis, this is not a model for the whole game. It is a reference for the **tactical sublayer**: how a player can remain a strategic commander while still taking over a unit, how squads can be ordered at high level while one soldier is manually managed, and how physics, terrain damage, inventory, and crew roles become the connective tissue between command and embodiment.

## Feature & Systems Teardown

### 1) RTS command layer and direct-control layer are not separate games

The core trick is that the game does not treat direct control as a minigame. It is a mode of intervention in the same battlefield state.

- The RTS layer is for intent: move here, attack there, hold that position, use this vehicle, occupy that point.
- The direct-control layer is for execution: aim, fire, peek, swap items, drag ammo, rotate a turret, repair, or drive.
- Switching between them does not reset the simulation. The unit continues to occupy the same physical world, obeying the same damage, cover, ammo, and crew constraints.

This matters because it creates a **continuity of agency**. The player can issue macro orders to preserve strategic tempo, then drop into micro control to solve a local problem the AI cannot handle well enough. That is exactly the gap Civis should exploit in its tactical layer: strategic intent should persist while direct control only temporarily overrides one agent’s local decisions.

### 2) Squad orders are broad, local, and fragile by design

The command model is not elegant in the sanitized modern RTS sense. It is field-practical.

- Orders are typically positional, directional, or target-based.
- Squads can be told to move, attack, defend, enter vehicles, or use equipment.
- Fire discipline is not abstracted away; the player often has to manually align the squad’s behavior with terrain and threat geometry.
- Units do not behave like a perfect RTS blob. They split, get pinned, lose line of sight, run out of ammo, and respond unevenly to terrain and destruction.

The tactical consequence is important: the game rewards **explicit order sequencing** under uncertainty. You do not just “attack move” and assume resolution. You choose approach vectors, line up supporting fire, manage exposure, and fix failures with direct control.

For Civis, that argues for:

- orders that are coarse but composable,
- visible order state and failure reasons,
- and a strong separation between what the player commanded and what the unit could actually execute.

### 3) Soldier inventory is a logistics system, not flavor

One of the most important Men of War-era ideas is that a soldier’s inventory is functionally part of the battlefield economy.

- Soldiers carry weapons, magazines, grenades, tools, and spare ammunition.
- Inventory is spatial and physical enough to matter during combat.
- Ammunition depletion is not an end-state; it creates local resupply behavior, scavenging, dropped-kit reuse, and triage decisions.
- Weapon swaps, captured gear, and item transfers are tactical actions, not UI dressing.

This pushes the game away from the “unit HP bar only” model. A soldier can still be alive, but effectively neutralized by empty magazines, lack of anti-armor capability, or bad kit composition.

For Civis, this is the strongest reference in the entire lineage for a tactical layer where **supply scarcity creates emergent battlefield shape**. It suggests:

- ammo as a first-class finite resource,
- transferable equipment,
- dropped inventory as persistent world state,
- and unit effectiveness as a function of both body condition and carried assets.

### 4) Logistics and ammo are local, visible, and improvisational

The series does not hide logistics behind a generic supply network. At battlefield scale, logistics are experienced directly:

- ammo is expended in real time,
- weapons are fed by carried or nearby stock,
- vehicles need ammunition and maintenance attention,
- and infantry can scavenge or rearm from dead units and containers depending on context.

That makes logistics legible in the moment. It is not just a macro production number. It is an on-map friction system that forces the player to think about sustainment during the fight, not after it.

For Civis, this is valuable because it bridges operational and tactical command:

- operational command establishes stockpiles, depots, transport, and provisioning,
- tactical command consumes and redistributes those stocks at the unit level,
- and the player can see the consequence of logistics failures immediately in combat readiness.

### 5) Destructibility is part of tactics, not a cosmetic effect

The Men of War line is famous for destructible terrain and damageable battlefield objects. The key point is not that things can be blown up. It is that destruction changes tactical geometry.

- Cover can be removed.
- Entrenchments can be breached.
- Paths can be opened or blocked.
- Vehicles, emplacements, and buildings can be disabled or reduced to cover fragments.
- The battlefield becomes a stateful, deformable planning surface.

This is one of the main reasons the lineage feels more “simulationist” than mainstream RTS. Cover and line of fire are not static map facts. They are contingent and can be rewritten by shells, explosives, and vehicle fire.

For Civis, destructibility should be read as a **law-level tactical substrate** only where it is physically justified. It is a strong fit for terrain, structures, fortifications, and vehicles, but not an excuse for arbitrary scripted map invalidation.

### 6) Vehicle crews are multi-role systems, not single pooled HP bars

Vehicles are not just armored tokens. Crew composition and crew survival matter.

- A vehicle can have differentiated crew roles.
- Crew damage affects vehicle operation, firepower, repairability, and survivability.
- The player can often enter or reassign roles, making vehicles partly systems of human labor rather than opaque machine shells.
- When the crew is compromised, the vehicle is not merely “damaged”; it can become partially or fully nonfunctional in role-specific ways.

This is a major differentiator from arcade RTS armor. It makes vehicles feel like compact ecosystems of people, ammunition, and machine subsystems.

For Civis, vehicle crew modeling is a clean bridge between direct control and simulated labor:

- crews can be real agents with roles,
- those roles can be expressed in vehicle capability,
- and loss of crew should degrade capability in specific, understandable ways.

### 7) Direct control coexists with RTS command through temporary ownership of a local problem

This is the central design lesson.

The game does not ask the player to choose between commander and embodied combatant. Instead, it lets the player **borrow one unit’s perspective** to resolve a local problem, then return to command without losing battlefield continuity.

That co-existence works because:

- the RTS layer remains authoritative for larger intent,
- the direct-control layer only overrides one unit at a time,
- the unit remains inside the same command hierarchy,
- and battlefield consequences remain shared.

In practice, this means a player can:

1. order a squad to occupy a flank,
2. jump into a rifleman to clear a trench,
3. hop to a machine-gunner to suppress a road,
4. then return to the map to redirect armor.

That loop is exactly the tactical fantasy Civis needs if it wants commanders to feel present without collapsing into avatar-led play.

### 8) Operational vs tactical command is a real split, not just camera zoom

The lineage implicitly distinguishes operational and tactical planning:

- Operational command: where forces go, what they should achieve, what resources they should carry, and how much sustainment they need.
- Tactical command: how an individual squad or vehicle actually survives the next 30 seconds.

This split is not presented as two interfaces with a neat separation line. It is lived through friction:

- operational mistakes become tactical shortages,
- tactical losses feed back into operational readiness,
- and the player oscillates between levels as the situation demands.

For Civis, this is a strong reference for a multi-scale command stack:

- strategic allocation of people, vehicles, and supplies,
- local battlefield execution,
- and direct intervention when AI or terrain makes the local problem too costly.

## UX / QoL / Bells-and-Whistles

The lineage’s UX quality is uneven, but the useful bits are important because they reduce blindness in a system this dense.

### What helps the player read the battlefield

- Clear unit selection and per-unit state visibility.
- Direct control camera that makes aiming, moving, and weapon interaction legible.
- Immediate feedback when a unit is wounded, pinned, low on ammo, or otherwise compromised.
- On-map readability of destruction, cover loss, and changed lines of fire.
- Ability to jump between overview and embodiment without losing state.

### QoL patterns worth noting

- Hover/tool feedback that exposes what a unit is carrying or capable of.
- Fast issue/execute loops for order correction.
- Strong hotkey dependence for power users.
- Tactical pauses or pauses by implication through slow planning in single-player.
- Context-sensitive interaction with vehicles and equipment.

### UX pain points that still matter as anti-lessons

- The systems are dense enough that poor readability quickly becomes failure.
- Micro can become exhausting if the player is forced to compensate for weak AI too often.
- If inventory, crew, and damage are not surfaced cleanly, the game feels unfair rather than deep.
- Direct-control games of this type often need better explanation of what AI will do when released back to RTS control.

For Civis, the takeaway is not “copy the old UI.” It is to preserve the informational density while modernizing the information architecture:

- stronger state overlays,
- explicit failure explanations,
- inventory and ammo visibility by default,
- better order provenance,
- and obvious handoff semantics between commander mode and direct-control mode.

## What it NAILS

- The commander/direct-control handoff feels mechanically meaningful rather than cosmetic.
- Soldier inventory has true tactical consequences.
- Ammo scarcity changes decisions in the middle of combat.
- Vehicle crews feel like systems, not skins.
- Destructibility changes battlefield geometry in real time.
- Small-unit play remains embedded in a larger RTS frame.
- The player can correct AI failure locally without abandoning strategic command.
- The battle space feels physical, improvisational, and brittle.

## What to ADOPT for Civis

| Item | Tag | Why it matters for Civis | Charter tension |
|---|---|---|---|
| Player can jump into an individual unit to solve a local tactical problem while keeping the rest of the battlefield under RTS command | `[UI/QoL]` | This is the cleanest way to make Civis tactical play feel intimate without hardcoding a different game mode. | Low; this is presentation and interaction, not a social or political enum. |
| Order hierarchy with visible local failure states, not just success/failure | `[UI/QoL]` | Civis needs the player to understand whether a unit failed because of pathing, cover, suppression, ammo, or crew loss. | Low. |
| Per-soldier inventory with transferable ammo, tools, and weapons | `[LAW]` | Inventory as physical state is foundational to tactical logistics and can emerge from the agent/material substrate. | Low to medium; keep it as physical simulation, not abstract class identity. |
| Battlefield ammo and sustainment as first-class constraints | `[LAW]` | Logistic scarcity should be simulated as resource transport and depletion, not as a hidden UI meter. | Low. |
| Vehicle crew roles with role-specific degradation when crew are injured or absent | `[LAW]` | Crew is labor-in-machine, which maps well to a physical/agent simulation. | Low. |
| Destructible terrain and structures that alter line of sight, cover, access, and movement | `[LAW]` | This is exactly the kind of physical consequence Civis should author at Layer 0. | Low. |
| A clean operational/tactical split where supply, deployment, and broad goals live above local execution | `[UI/QoL]` | The split is an interaction model that helps the player reason at two scales without adding non-emergent social structure. | Low. |
| Reusable dropped gear and salvage on the battlefield | `[EMERGENT]` | This should arise naturally from inventory, injury, and death. | Low. |
| Unit effectiveness that degrades from wound state, ammo state, fatigue, and crew loss rather than a single HP bar | `[EMERGENT]` | The richer emergent state makes combat less gamey and more substrate-driven. | Low. |
| Contextual prompts for ammo, crew, and vehicle interaction | `[UI/QoL]` | Keeps the dense system readable without changing simulation law. | Low. |

## What to AVOID

- Do not hardcode “RTS mode” versus “FPS mode” as separate simulation laws. Treat direct control as a temporary control policy on the same agent.
- Do not flatten inventory into a cosmetic loadout screen. If the item exists, it should matter physically.
- Do not model vehicles as opaque hitpoint containers. Crew roles and internal capability loss are part of the point.
- Do not fake destructibility with scripted map events if the design goal is physical consequence.
- Do not build a clean abstract supply system that never appears on the battlefield. If Civis has logistics, the player should feel it tactically.
- Do not turn factions, doctrine, or battlefield identity into hardcoded enums in the simulation layer. Those belong to emergence, not authored social structure.
- Do not rely on unreadable micro. If the AI cannot hold up under scale, improve the command surfaces rather than making the player babysit every unit.

## Bevy / Rust Ecosystem Notes

There is no single Rust crate that reproduces the entire Men of War-style stack. The practical route is to compose or wrap-over-handroll the pieces:

- Use Bevy ECS for unit state, command routing, and camera/control handoff.
- Use physics/terrain destruction systems appropriate to the project’s substrate instead of trying to graft in an RTS-only abstraction.
- Treat inventory and crew as ECS components with clear authority boundaries.
- For tactical UX, prefer explicit overlays and selection state systems over opaque plugin magic.

The useful ecosystem lesson is not a specific crate recommendation here. It is architectural: the lineage works because simulation, command, and presentation stay coupled enough to be legible, but not so coupled that direct control breaks the battlefield model.

## Sources

Real URLs consulted:

1. https://calltoarmsgame.com/
2. https://store.steampowered.com/app/302670/Call_to_Arms/
3. https://store.steampowered.com/app/2043140/Call_to_Arms__Gates_of_Hell_Ostfront/
4. https://store.steampowered.com/app/400/Call_to_Arms_Men_of_War_II/
5. https://store.steampowered.com/app/244450/Men_of_War_Assault_Squad_2/
6. https://menofwargame.com/
7. https://menofwar.fandom.com/wiki/Call_to_Arms
8. https://menofwar.fandom.com/wiki/Men_of_War
9. https://en.wikipedia.org/wiki/Call_to_Arms_(video_game)
10. https://en.wikipedia.org/wiki/Men_of_War_(video_game)
