# Aerial Completeness Audit

Scope:
- Runtime aerial code under `src/Runtime/Aviation`
- Pack content in `packs/warfare-aerial`
- Related Star Wars aerial content in `packs/warfare-starwars`
- Archived modern aerial content in `packs/_archived/warfare-airforce` and `packs/_archived/warfare-modern`

## Executive Summary

The aerial subsystem is partially implemented and is enough to:
- mark units as aerial,
- place them at a cruise altitude on spawn,
- keep them hovering at altitude,
- toggle an `IsAttacking` flag based on enemy ground proximity,
- attach `AntiAirComponent` to some building entities.

It is not yet a complete aerial combat system.

The biggest gaps are:
- no true aerial-vs-aerial combat,
- no actual attack execution in the aerial runtime,
- no target selection pipeline that distinguishes ground attack, air superiority, and anti-air behavior,
- no reliable per-building anti-air mapping,
- no flight/pathing/formation behavior beyond straight-line altitude changes,
- no doctrine/runtime integration for the many aerial roles described in the packs.

For the Star Wars units:
- `rep_v19_torrent` and `cis_tri_fighter` are represented as aerial units in data.
- They can spawn and hover at altitude if the normal unit spawn path reaches `AerialUnitMapper`.
- They do not yet behave like full fighter units with dogfighting, interception, or air-to-air threat response.

For the modern units in the archived airforce pack:
- jets, helicopters, drones, bombers, and EW aircraft are defined in content.
- They are still limited by the same runtime gaps, and the content overstates current functionality.

## What Is Actually Implemented

### 1. Aerial unit tagging and altitude

Implemented in:
- [`src/Runtime/Aviation/AerialUnitMapper.cs`](../../src/Runtime/Aviation/AerialUnitMapper.cs)
- [`src/Runtime/Aviation/AerialUnitComponent.cs`](../../src/Runtime/Aviation/AerialUnitComponent.cs)
- [`src/Runtime/Aviation/AerialSpawnSystem.cs`](../../src/Runtime/Aviation/AerialSpawnSystem.cs)
- [`src/Runtime/Aviation/AerialMovementSystem.cs`](../../src/Runtime/Aviation/AerialMovementSystem.cs)

Behavior:
- Units with behavior tag `Aerial` get `AerialUnitComponent`.
- `CruiseAltitude`, `AscendSpeed`, and `DescendSpeed` are read from the unit definition if present.
- `SpawnAtAltitude` snaps newly detected aerial units to cruise altitude.
- `AerialMovementSystem` nudges `Translation.y` toward cruise altitude, or toward ground when `IsAttacking` is true.

What this means in practice:
- aerial units can exist above ground,
- they can visually hover in place,
- they can descend and re-ascend.

### 2. Anti-air tagging

Implemented in:
- [`src/Runtime/Aviation/AntiAirComponent.cs`](../../src/Runtime/Aviation/AntiAirComponent.cs)
- [`src/Runtime/Aviation/AerialBuildingMapper.cs`](../../src/Runtime/Aviation/AerialBuildingMapper.cs)
- [`src/Runtime/Aviation/AerialSpawnSystem.cs`](../../src/Runtime/Aviation/AerialSpawnSystem.cs)
- [`src/Runtime/Aviation/AerialUnitMapper.cs`](../../src/Runtime/Aviation/AerialUnitMapper.cs)

Behavior:
- Units or buildings with `AntiAir` get an `AntiAirComponent`.
- Building anti-air values can come from `anti_air:` data or defaults.
- `AerialSpawnSystem` sweeps baked building entities and applies anti-air components after a delay.

Important limitation:
- the building sweep applies anti-air broadly because runtime building entities do not carry a per-pack identity.
- that is explicitly called out in code as a future refinement.

### 3. Spawn routing

Implemented in:
- [`src/Runtime/Bridge/PackStatMappings.cs`](../../src/Runtime/Bridge/PackStatMappings.cs)
- [`src/Runtime/Bridge/PackStatInjector.cs`](../../src/Runtime/Bridge/PackStatInjector.cs)
- [`src/Runtime/Bridge/VanillaArchetypeMapper.cs`](../../src/Runtime/Bridge/VanillaArchetypeMapper.cs)

Behavior:
- `aerial_fighter` is intentionally skipped by the generic stat injector.
- `AirstrikeProxy` is recognized as a valid unit class.
- Aerial units are expected to be handled by dedicated aviation code instead of the normal vanilla mapping path.

This is a real design choice, not a no-op:
- aerial content is intentionally routed away from the generic stat override flow.

## What Is Stubbed Or Only Partially Implemented

### 1. Combat resolution is incomplete

Implemented:
- `AerialTargetingSystem` finds nearby enemy ground units and sets `AerialUnitComponent.IsAttacking`.

Not implemented:
- actual weapon firing,
- actual damage application,
- attack cooldown execution,
- projectile spawning,
- hit resolution,
- death/retreat logic,
- attack-run selection,
- weapon-specific behavior,
- air-to-air engagement.

In effect:
- the system decides whether an aerial unit should be attacking,
- but it does not fully perform the attack.

### 2. Air-to-air combat is missing

The current targeting system explicitly excludes aerial targets:
- it queries enemy ground units,
- it filters out `AerialUnitComponent` entities,
- it does not define a dogfight or interception path.

That means:
- fighters cannot currently dogfight,
- escorts cannot protect bombers from enemy aircraft,
- anti-air aircraft cannot prioritize air targets,
- air superiority doctrine is not enforced by runtime.

### 3. Flight behavior is too simple

Implemented:
- vertical adjustment to a cruise altitude,
- straight descent during attack state.

Missing:
- pathfinding that understands airspace,
- collision avoidance,
- waypointing,
- patrol patterns,
- strafing logic,
- loiter/turn/return behaviors,
- formation spacing,
- altitude bands by role,
- terrain-aware attack runs,
- terrain masking or line-of-sight based flight behavior.

### 4. Anti-air defenses are incomplete as a tactical system

Implemented:
- buildings can receive `AntiAirComponent`.

Missing:
- a dedicated anti-air targeting loop that reacts to nearby aerial units,
- target prioritization by altitude, speed, or threat class,
- range, fire rate, and accuracy actually driving damage or interception,
- coordination with radar or early warning systems,
- altitude-based effective envelopes,
- low-altitude vs high-altitude differentiation.

### 5. Content/runtime mismatch

The packs describe many features that are not yet backed by runtime logic:
- reconnaissance behavior,
- charge/skirmish/kite/flank/sieger/terror/swarm roles,
- air superiority doctrine,
- support/debuff EW aircraft,
- bomber and helicopter role differentiation,
- persistent production chains for airfields and airbases.

That is content-complete only in YAML, not in gameplay.

## Star Wars Audit

Relevant units:
- [`packs/warfare-starwars/units/republic_units.yaml`](../../packs/warfare-starwars/units/republic_units.yaml)
- [`packs/warfare-starwars/units/cis_units.yaml`](../../packs/warfare-starwars/units/cis_units.yaml)

### V-19 Torrent

Current data:
- `unit_class: AirstrikeProxy`
- `vanilla_mapping: aerial_fighter`
- `behavior_tags: [Aerial, AdvanceFire, Charge]`
- `aerial:` block present with cruise altitude and ascent/descent speeds

What works:
- can be identified as aerial,
- can be assigned aerial altitude behavior,
- can be routed away from the generic stat injector.

What is missing:
- actual air-to-air combat,
- fighter interception behavior,
- bomber/escort doctrine support if intended,
- proper attack execution and cooldown,
- target selection over hostile aircraft,
- flight behavior beyond altitude and whatever the base archetype provides.

### Tri-Fighter

Current data:
- `unit_class: AirstrikeProxy`
- `vanilla_mapping: aerial_fighter`
- `behavior_tags: [Aerial, AdvanceFire, Kite]`
- `aerial:` block present

What works:
- same as above, plus it is clearly intended as a kite/swarmer-style air unit.

What is missing:
- swarm tactics,
- disengage/re-engage behavior,
- air superiority behavior,
- any meaningful dogfight logic,
- any explicit anti-air evasion model.

### Star Wars anti-air structures

The pack includes `AntiAir` buildings in data, but the runtime still needs:
- correct per-building mapping,
- actual attack logic,
- air-target prioritization,
- visible in-game validation that the towers and generators are really engaging aircraft.

## Modern Pack Audit

Relevant content:
- [`packs/_archived/warfare-airforce/pack.yaml`](../../packs/_archived/warfare-airforce/pack.yaml)
- [`packs/_archived/warfare-airforce/units/airforce_units.yaml`](../../packs/_archived/warfare-airforce/units/airforce_units.yaml)
- [`packs/_archived/warfare-airforce/buildings/airbase_buildings.yaml`](../../packs/_archived/warfare-airforce/buildings/airbase_buildings.yaml)
- [`packs/_archived/warfare-modern/pack.yaml`](../../packs/_archived/warfare-modern/pack.yaml)

### Modern aircraft represented

The archived airforce pack describes:
- fighter jets,
- attack helicopters,
- strategic bombers,
- recon drones,
- EW aircraft,
- anti-air batteries,
- airstrips and radar support.

### What is actually functional

Only the shared aerial skeleton is functional:
- aerial tags,
- altitude,
- basic ground-target selection,
- anti-air component attachment.

### What is missing for modern-style warfare

Modern air warfare needs runtime support for:
- air superiority fighters that can intercept other aircraft,
- helicopters that fly low and attack while avoiding heavy AA,
- bombers that strike buildings and avoid prolonged exposure,
- drones that scout and persist,
- EW platforms that debuff enemy targeting or detection,
- radar-linked AA batteries that react differently to aircraft classes.

None of that is presently implemented in the runtime.

## SOTA RTS Air-Unit Benchmark

To be considered complete by RTS standards, aerial units typically need all of the following:

1. Distinct role model
- interceptor/fighter,
- ground-attack,
- bomber,
- helicopter/low-altitude CAS,
- scout/drone,
- support/EW,
- transport or special utility.

2. Proper air combat rules
- air-to-air targeting,
- anti-air threat evaluation,
- altitude bands,
- vulnerability scaling by altitude and unit class,
- interception and retreat behavior.

3. Tactical movement
- patrol,
- intercept,
- attack run,
- orbit/loiter,
- disengage,
- return-to-base or landing behavior.

4. Visibility and counterplay
- radar / detection ranges,
- stealth or concealment for some aircraft,
- clear AA envelopes,
- readable attack telegraphing.

5. Economy and logistics
- dedicated production buildings,
- repair/maintenance,
- optional refuel/ammo or cooldown pressure,
- role-appropriate cost curves.

6. Integration with ground combat
- air support should affect ground frontlines without trivializing them,
- AA should be meaningful but not universal,
- air superiority should matter strategically.

Current DINOForge aerial code covers only a small subset of this benchmark:
- altitude representation,
- some spawn wiring,
- a partial anti-air marker,
- a ground-target toggle.

## Completion Checklist

### Runtime foundation
- [ ] Replace the current `IsAttacking` toggle with a real aerial combat state machine.
- [ ] Add actual attack execution for aerial units, including cooldown and damage application.
- [ ] Add target acquisition for aerial targets, not just ground targets.
- [ ] Separate fighter/interceptor logic from ground-attack logic.
- [ ] Add a proper anti-air attack loop for `AntiAirComponent`.
- [ ] Add per-building identity mapping so anti-air components are attached selectively, not broadly.

### Flight and movement
- [ ] Add patrol, intercept, loiter, and attack-run behaviors.
- [ ] Add airspace-aware movement or at least waypoint flight that bypasses ground pathing correctly.
- [ ] Add altitude bands and role-based altitude selection.
- [ ] Add retreat / disengage / re-approach behavior.
- [ ] Add formation spacing or at minimum collision-safe separation for multiple aircraft.

### Targeting and combat
- [ ] Support air-to-air engagement between fighters.
- [ ] Support air-to-ground attack priorities for bombers and CAS units.
- [ ] Support low-altitude vulnerability to anti-air and ground fire.
- [ ] Support altitude-sensitive detection or target selection.
- [ ] Implement role-specific target preferences:
  - fighters prioritize enemy aircraft,
  - bombers prioritize structures or clusters,
  - helicopters prioritize armor and soft ground targets,
  - drones prioritize recon and survival.

### Pack contract alignment
- [ ] Validate every aerial unit in `warfare-starwars` and `warfare-airforce` against the runtime contract.
- [ ] Ensure each aerial unit’s `behavior_tags` correspond to actual runtime behavior, not just flavor text.
- [ ] Ensure `aerial:` values are used in gameplay, not only stored.
- [ ] Ensure `AntiAir` buildings in the packs actually produce hostile fire or interception.
- [ ] Ensure production buildings for aircraft can spawn aerial units in-game and not just exist as YAML definitions.

### Star Wars specific
- [ ] Make `rep_v19_torrent` function as a real interceptor/strike fighter.
- [ ] Make `cis_tri_fighter` function as a real swarm interceptor/skirmisher.
- [ ] Add air-to-air combat rules that make the V-19 and Tri-Fighter interact differently from ground units.
- [ ] Add clear anti-air counterplay for Republic and CIS ground defenses.
- [ ] Verify the aerial doctrine and weapon entries actually influence runtime stats or behavior.

### Modern specific
- [ ] Make jets function as interceptors with speed and altitude advantage.
- [ ] Make helicopters function as low-altitude CAS units that are vulnerable to AA.
- [ ] Make bombers strike buildings and retreat without hovering in AA range forever.
- [ ] Make drones behave like scouts with lower combat commitment and better survivability.
- [ ] Make EW aircraft apply a real debuff or detection effect.
- [ ] Add radar-linked AA interactions if the pack intends them.

### Validation
- [ ] Add unit tests for aerial component mapping and target selection.
- [ ] Add runtime tests or integration coverage for air-to-air targeting.
- [ ] Add tests for anti-air building attachment logic.
- [ ] Add pack fixtures that prove `rep_v19_torrent`, `cis_tri_fighter`, and modern aircraft are all valid aerial definitions.
- [ ] Verify in game that at least one aerial unit can spawn, hover, attack, and be countered by anti-air.

## Bottom Line

Current state:
- aerial units can exist and hover,
- anti-air tags can be attached,
- aerial unit data is present in the packs,
- but aerial combat is not complete.

For a 100% functional aerial system, the missing pieces are combat execution, air-to-air logic, role-specific behavior, and reliable anti-air interaction. The pack data is ahead of the runtime.
