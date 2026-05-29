# Star Wars Space Combat R&D

## Problem Statement

The requested feature is an optional space-combat plane for the Star Wars mod: capital ships, starfighters, and orbital battles that sit alongside the existing ground campaign. The core question is not whether this is desirable, but what is actually feasible inside DINOForge as a BepInEx mod on top of Unity 2021.3 ECS without engine source.

This note compares three implementation strategies:

1. A separate ECS `World` / scene for space battles toggled from ground.
2. An overlay battle layer reusing the existing RTS systems with space skins and zero-gravity movement.
3. A minigame-style abstraction.

It then recommends the most achievable path to a first playable space skirmish.

## Constraints

The mod must work inside a game built on Unity 2021.3 ECS, injected through BepInEx, with no engine source and no assumption that the base game can be recompiled. That implies:

- You can patch gameplay logic, spawn entities, and intercept state transitions.
- You cannot rely on deep engine changes, custom render pipelines, or a clean second game process.
- Scene loading and ECS world management are possible in principle, but high-risk because the base game already owns the player loop, world bootstrap, and asset lifecycle.
- UI, camera, selection, movement, and combat feel must be built from existing mod-accessible surfaces or from layers you can add without replacing the engine.

For a first playable slice, the success criterion should be narrow: one tactical space map, a handful of ships, player selection, move/attack orders, and a victory condition.

## Option 1: Separate ECS World / Scene

### Idea

Spin up a dedicated ECS world for space combat, potentially paired with a dedicated scene or subscene. Ground play would transition into space mode when the player enters an orbital battle, and back again when the battle resolves.

### How it would integrate

- A second `World` would need its own systems, entities, physics/time assumptions, and bootstrap path.
- Scene management would need to isolate space assets, cameras, UI roots, and input routing.
- A bridge layer would translate campaign state into space-battle state and back.

### What is realistic in DINOForge

Technically possible, but the highest-risk path. The hard part is not ECS itself; it is attaching the new world cleanly to the game’s existing lifecycle without destabilizing the original world.

In a BepInEx mod, you can often create or manage a world, but the integration cost is high:

- You need to keep the original game world alive or deliberately suspend it.
- You need to avoid collisions with the game’s own singletons, update order, and rendering assumptions.
- If the base game expects only one active world, a second world can become a maintenance trap.

### Risks

- Scene bootstrap may be brittle if the game hardcodes initialization assumptions.
- Input and camera systems may be tightly coupled to the main RTS scene.
- Save/load and mission state bridging becomes non-trivial.
- Debugging is difficult because failures can look like ordinary ECS/system issues while actually being lifecycle issues.

### Assessment

This is the cleanest conceptual architecture, but the least attractive as a first mod implementation. It is more likely to become an engine-integration project than a content mod.

## Option 2: Overlay Battle Layer Reusing Existing RTS Systems

### Idea

Keep the existing RTS layer and reuse as much as possible: selection, issuing orders, AI loops, health/damage, and UI patterns. Replace or reinterpret the battlefield as an orbital plane:

- Units move in 3D or pseudo-3D on a bounded space map.
- “Ground” becomes a flat or layered space arena.
- Weapons, acceleration, and turning are retuned for zero gravity.
- Visuals are swapped for space skins, but the tactical loop stays RTS-like.

### How it would integrate

- Use the current ECS world.
- Add a space-combat mode flag and route systems accordingly.
- Add space-specific movement, targeting, and formation logic as variants of existing systems rather than replacing them outright.
- Reuse the current UI shell for selection, commands, and status panels.
- Use a mode-specific camera and scene dressing, but keep the same core game loop.

### What is realistic in DINOForge

This is the most plausible “real game” path for a BepInEx mod because it minimizes lifecycle risk and keeps you inside the game’s native update flow.

The key advantage is that you are not trying to add a second game. You are adding a tactical ruleset on top of the existing one. That means:

- Less scene-management complexity.
- Less risk of breaking the bootstrap path.
- More reuse of pathfinding/selection/UI hooks where they already exist.

The main challenge is movement and targeting. If the existing RTS systems are strongly ground-centric, you may need to layer in space movement rules and ignore or bypass terrain constraints. That is still likely easier than building a second world.

### Risks

- Existing nav/pathing may assume planar terrain, cover, or occupancy.
- Formation and collision behavior may fight zero-gravity motion.
- Visual fidelity can look “fake space” if the underlying assumptions are too ground-like.
- Some systems may need deep patching if they encode ground-specific behavior too early.

### Assessment

Best balance of feasibility and authenticity. If the goal is a believable tactical space battle inside the current game, this is the strongest candidate.

## Option 3: Minigame-Style Abstraction

### Idea

Treat orbital battles as a separate strategic layer with simplified mechanics. The player does not directly command a full RTS simulation. Instead, space battle resolves through:

- abstract fleets,
- limited tactical choices,
- wave-based engagements,
- or semi-automated combat with high-level commands.

### How it would integrate

- Use the existing campaign layer to launch a “space encounter.”
- Build a custom overlay or full-screen minigame UI.
- Simulate battle outcomes with a small combat model rather than full unit simulation.
- Return results to the ground campaign state.

### What is realistic in DINOForge

This is the safest path technically. It avoids deep dependence on the engine’s RTS assumptions and keeps the feature bounded.

If the modding surface is too constrained for a convincing real-time space battle, a minigame abstraction gives you a way to ship the Star Wars fantasy without needing a full second combat stack.

### Risks

- It may disappoint players expecting direct fleet control.
- It can feel detached from the rest of the mod if the UI and battle feedback are too abstract.
- It is easier to prototype, but harder to make feel like a true “battle layer” rather than a menu-driven resolver.

### Assessment

The most achievable fallback. It is the best option if the goal is to ship something reliable quickly, but it is not the best match for a premium Homeworld-style experience.

## Comparison

### Separate ECS World

- Fidelity: High
- Mod risk: Very high
- Engine/source dependence: High
- First playable speed: Slow
- Recommendation: Not first choice

### Overlay RTS Layer

- Fidelity: High enough for a convincing tactical battle
- Mod risk: Moderate
- Engine/source dependence: Moderate
- First playable speed: Medium
- Recommendation: Best balance

### Minigame Abstraction

- Fidelity: Medium
- Mod risk: Low
- Engine/source dependence: Low
- First playable speed: Fast
- Recommendation: Best fallback

## What SOTA Space RTS Suggests

### Homeworld

Homeworld’s signature lesson is that space combat works when the player can think in 3D, not just on a flat map. The series emphasizes:

- fleet formations,
- vertical positioning,
- unit roles,
- and clear tactical silhouettes.

The official Remastered Collection description highlights fleet control, formations, flight tactics, and large ship rosters. That is important because the feeling of space combat comes from commanding a true fleet, not just reskinning ground units.

### Empire at War Space Battles

Empire at War shows a more conservative and accessible model: tactical space engagements that are still readable, formation-friendly, and easy to parse. The important lesson is that space combat can remain RTS-friendly if it keeps:

- small command sets,
- clear counters,
- capital ships with distinct roles,
- and a battle loop that is easy to enter and exit from the larger campaign.

That makes it a useful template for an optional orbit layer in a modded game.

### Sins of a Solar Empire

Sins is the best reference for layered fleet combat at scale:

- capital ships as long-lived heroes,
- fighters and bombers as carrier pressure,
- starbases as strategic anchors,
- and fleet supply as a limiter that keeps battles legible.

Its design proves that large space battles stay understandable when:

- unit classes are strongly differentiated,
- capital ships matter but are not alone sufficient,
- and strategic structures shape the battlefield.

For a DINOForge implementation, Sins is especially relevant because it demonstrates how to make an orbital war feel systemic rather than cinematic.

## Recommendation

For a first playable space skirmish, the most achievable path is:

1. Stay in the existing ECS world.
2. Add a space-combat mode that reuses the current RTS stack as much as possible.
3. Limit the first slice to a single orbital map, a few ship classes, direct selection, move/attack orders, and one win condition.

That is the overlay battle layer approach.

Why this is the best first step:

- It avoids the highest-risk world/bootstrap problems.
- It keeps implementation inside the modding surface most likely to be stable under BepInEx.
- It leaves room to graduate later into a more specialized space scene or a minigame fallback if technical limits appear.

## Recommended First Slice

Implement these in order:

1. A toggle or trigger that enters `space battle` mode from the campaign layer.
2. A dedicated orbital map setup with zero-gravity movement rules.
3. Two to four ship archetypes: interceptor, bomber, corvette, capital ship.
4. Selection, move, attack, and attack-move commands.
5. Basic faction differentiation and one simple victory condition.
6. A return path to campaign state after battle resolution.

If the RTS overlay proves too constrained, the first fallback should be a minigame abstraction using the same ship roster and result pipeline.

## Bottom Line

The separate-world idea is architecturally elegant but too risky for a first modded implementation. The minigame is easiest to ship but least satisfying. The overlay RTS layer is the best compromise and the recommended path to the first playable space skirmish in a BepInEx-only environment.
