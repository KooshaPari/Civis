# CIV-0800: Audio System Specification

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Audio & Engine Team
**Spec References:** CIV-0001 (Core Simulation Loop), CIV-0200 (Multi-Client Protocol)

---

## Executive Summary

CivLab's audio system is an **event-driven, multi-client audio layer** that reacts to simulation events emitted by the deterministic tick engine. Audio is never polled — the audio subsystem subscribes to the simulation event bus and responds to events as they arrive. This spec defines the complete audio architecture for both the native Bevy (Rust/Kira) client and the web browser (Howler.js/Web Audio API) client.

The audio layer spans four domains:
1. **Sound event mapping** — every simulation event that has an audio trigger
2. **Procedural music state machine** — adaptive layered music driven by simulation state
3. **SFX library** — full catalog of every sound effect, format, path, and trigger
4. **Ambient soundscape system** — camera-position-aware layered ambient audio

The audio system is **client-side only**. The headless simulation core emits events; audio is never computed server-side. All audio behavior is deterministic given the event stream.

---

## Table of Contents

1. Audio Architecture
2. Sound Event Mapping
3. Procedural Music System
4. SFX Library Taxonomy
5. Ambient Soundscape System
6. Kira (Bevy) Integration
7. Web Client (Howler.js) Integration
8. Audio Settings
9. Asset Pipeline
10. Accessibility
11. Appendix: Full Event→Audio Trigger Table

---

## Section 1: Audio Architecture

### 1.1 Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Event-driven, no polling** | Audio subscribes to simulation event bus. No per-tick audio queries. |
| **Client-side execution** | Headless core has zero audio code. All audio is a client concern. |
| **Multi-client parity** | Bevy (Rust/Kira) and Web (Howler.js) produce equivalent audio behavior from the same event stream. |
| **Non-blocking** | Audio loading and decoding are async. Audio failures are logged but never crash the client. |
| **Deterministic given event stream** | Same event sequence → same audio output (given same assets). |
| **Layered mixing** | Music, ambient, and SFX layers are mixed independently. Each layer has its own volume envelope. |

### 1.2 Component Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SIMULATION CORE (Headless Rust)                  │
│  tick.completed.v1 / war.declared.v1 / disaster.triggered.v1 / ... │
└─────────────────────────────┬───────────────────────────────────────┘
                              │  Event Bus (WebSocket / Shared Memory)
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    AUDIO EVENT DISPATCHER                           │
│  Receives simulation events → routes to audio trigger system        │
│  Filters: event type, payload fields, cooldown, deduplication       │
└──────────────┬──────────────────────────────────┬───────────────────┘
               │                                  │
    ┌──────────▼──────────┐            ┌──────────▼──────────┐
    │  MUSIC STATE MACHINE │            │  SFX TRIGGER SYSTEM  │
    │  State transitions   │            │  One-shot SFX calls  │
    │  Layer volume control│            │  Positional audio    │
    │  Cross-fade management           │  Cooldown management │
    └──────────┬───────────┘            └──────────┬───────────┘
               │                                   │
    ┌──────────▼───────────────────────────────────▼───────────┐
    │              AUDIO MANAGER (Platform-Specific)            │
    │  Kira (Bevy/Rust native) OR Howler.js (Web/TypeScript)    │
    │  Master volume, per-layer volume, output device selection │
    └───────────────────────────────────────────────────────────┘
               │
    ┌──────────▼───────────────┐
    │   AMBIENT SOUNDSCAPE     │
    │  Camera-position aware   │
    │  Layered region mixing   │
    └──────────────────────────┘
```

### 1.3 Event Bus Integration

The simulation core emits events at the end of each tick (Phase 6: Client Broadcast). The audio subsystem is a listener on this event stream. It does not query state — it only reacts to events.

**Connection model (Bevy client):**
- Audio plugin registers as an ECS system that reads from the `SimulationEventReader` resource
- Events arrive via the same protocol channel as render state updates (shared memory or WebSocket)
- Audio systems run in the `Update` schedule, after network receive systems

**Connection model (Web client):**
- `AudioManager` registers a message handler on the WebSocket connection
- Events are dispatched to audio via `audioManager.handleSimulationEvent(event)`
- No separate connection; audio piggybacks on the existing simulation WebSocket

### 1.4 Audio Subsystem Boot Sequence

```
1. Client connects to simulation core (handshake)
2. AudioManager initializes:
   a. Load audio manifest (JSON file: asset paths, metadata)
   b. Preload critical SFX (UI sounds, frequently triggered events) — async
   c. Begin loading music layers — async, streamed
   d. Register event bus listener
3. First tick broadcast received:
   a. Apply initial simulation state → set music state
   b. Begin music playback (base layer at full volume)
   c. Ambient soundscape initializes based on camera position
4. Ongoing: react to events
```

### 1.5 Audio Data Flow

```
SimulationEvent
  { event_type: "war.declared.v1",
    tick: 5000,
    payload: { aggressor: "nation_A", target: "nation_B" } }
        │
        ▼
AudioEventDispatcher.dispatch(event)
        │
        ├─── music state: MusicState → WarWinning (if player is aggressor)
        │                           or WarLosing (if player is target)
        │
        └─── SFX trigger: play("sfx/diplomacy/war_declaration_horn.ogg")
                          with spatial: false (UI-level event)
                          volume: 0.9
                          priority: HIGH
```

---

## Section 2: Sound Event Mapping

### 2.1 Mapping Table

The following table defines every simulation event type that triggers audio, what it triggers, and any conditional logic.

| Simulation Event | Audio Trigger | Variant Logic | Cooldown |
|-----------------|---------------|---------------|----------|
| `war.declared.v1` | Dramatic horn fanfare (3-5s) | Aggressor: triumphant variant; target: alarming variant | 30s per nation-pair |
| `battle.resolved.v1` | Battle resolution sound (2-3s) | `outcome == victory` → victorious fanfare; `outcome == defeat` → somber chord + drum | 5s |
| `citizen.born.v1` (batch) | Ambient crowd cheer (sampled, 1-2s) | Triggered per 100 citizens batch; volume scales with batch size; max 1x per tick | 10s |
| `disaster.triggered.v1` | Disaster-type SFX (3-8s) | `type == drought` → wind + cracking earth; `type == flood` → rushing water; `type == earthquake` → low rumble + crack; `type == pandemic` → bell toll | 60s per disaster instance |
| `tech.unlocked.v1` | Discovery chime (1.5s) | Ascending arpeggio; pitch varies by era (early: simple; modern: electronic) | 10s |
| `economy.bankruptcy.declared.v1` | Somber chord (2s) | Low register strings; optional bass drop | 30s |
| `institution.formed.v1` | Establishment fanfare (2s) | Major key, celebratory | 15s |
| `insurgency.started.v1` | Tension sting (1.5s) | Minor key, rising dissonance | 20s |
| `election.held.v1` | Crowd murmur (2s) → result reveal (1.5s) | Murmur plays on event; result reveal plays on `election.result.v1` (victory: cheer; loss: groan) | 30s |
| `tick.completed.v1` (hidden) | Ambient heartbeat (0.5s, very low volume) | Only if `stability \< 30%`; pulse rate increases linearly as stability drops toward 0% | Per-tick |
| `migration.wave.started.v1` | Distant footsteps + crowd murmur | Volume proportional to migration count | 20s |
| `famine.triggered.v1` | Mournful low horn (2s) | Sustained; overlaps with drought SFX if co-occurring | 60s |
| `trade.route.established.v1` | Market bell (1s) + light fanfare | Short and positive | 10s |
| `alliance.formed.v1` | Alliance fanfare (2s) | Two-voice harmony (one per nation) | 30s |
| `peace.declared.v1` | Relief chord + distant cheer (2s) | Major key resolution | 30s |
| `city.founded.v1` | Construction complete + crowd cheer (2s) | Upbeat, major | 15s |
| `structure.destroyed.v1` | Collapse SFX (1-2s) | `type == building` → crumble; `type == wall` → stone crash; `type == ship` → splash + crack | 3s |
| `revolt.started.v1` | Alarm bell (1.5s) + crowd roar | Urgent, repeating | 30s |
| `research.started.v1` | Study chime (0.8s) | Soft, intellectual tone | 10s |
| `citizen.died.v1` (batch) | Subdued bell toll (1s) | Only if batch >= 100 deaths; pitch varies per cause | 15s |
| `economy.market_cleared.v1` | Market ambiance tick (0.3s) | Subtle; only plays if camera on market district | 2s |
| `military.unit_moved.v1` | Footstep cadence (0.5s) | Terrain-appropriate variant (see SFX taxonomy) | 0.5s per unit |
| `military.unit_routed.v1` | Routing horn (1s) | Minor, descending | 5s |
| `military.siege.started.v1` | Siege drum pattern (3s loop entry) | Heavy, rhythmic | 60s |
| `diplomacy.treaty_signed.v1` | Quill scratch + seal stamp (1s) | Satisfying, definitive | 5s |
| `climate.carbon_threshold.crossed.v1` | Environmental alarm tone (2s) | Low-frequency drone | 120s |
| `economy.supply_shock.v1` | Market commotion (1.5s) | Urgent, discordant | 30s |
| `policy.applied.v1` | Soft stamp/approval sound (0.5s) | Very subtle, UI-level | 1s |

### 2.2 Detailed Event Trigger Specifications

#### `war.declared.v1`

```
Trigger: war.declared.v1
Payload fields used: aggressor_nation_id, target_nation_id
Audio file (aggressor): sfx/diplomacy/war_declaration_horn_aggressor.ogg
Audio file (target): sfx/diplomacy/war_declaration_horn_target.ogg
Duration: 4.2s (aggressor), 3.8s (target)
Volume: 1.0 (master-relative)
Spatial: false (UI-level; not positional)
Music state transition: → WarWinning (aggressor player) or WarLosing (target player)
Cooldown key: war_declared:{aggressor}:{target}
Cooldown duration: 30s wall-clock
```

#### `battle.resolved.v1`

```
Trigger: battle.resolved.v1
Payload fields used: outcome (victory|defeat|draw), attacker_losses, defender_losses
Audio file (victory): sfx/military/battle_resolved_victory.ogg
Audio file (defeat): sfx/military/battle_resolved_defeat.ogg
Audio file (draw): sfx/military/battle_resolved_draw.ogg
Duration: 2.5s
Volume: 0.85
Spatial: true (position = battle_location hex)
Attenuation: linear from hex 0–10; muted beyond hex 15
Cooldown key: battle_resolved:{battle_id}
Cooldown duration: 5s
```

#### `disaster.triggered.v1`

```
Trigger: disaster.triggered.v1
Payload fields used: disaster_type, affected_region_id, severity (0.0–1.0)
Audio files:
  drought:    sfx/environment/disaster_drought.ogg      (wind + cracking earth, 6s)
  flood:      sfx/environment/disaster_flood.ogg        (rushing water + thunder, 5s)
  earthquake: sfx/environment/disaster_earthquake.ogg   (low rumble + crack, 4s)
  pandemic:   sfx/environment/disaster_pandemic.ogg     (bell toll + drone, 8s)
  wildfire:   sfx/environment/disaster_wildfire.ogg     (crackling + wind roar, 5s)
Volume: 0.5 + (severity × 0.5)  (scales with severity)
Spatial: true (position = affected_region centroid)
Music state transition: → Crisis if stability < 20% post-disaster
Cooldown key: disaster:{disaster_type}:{affected_region_id}
Cooldown duration: 60s
```

#### `citizen.born.v1` (batch)

```
Trigger: citizen.born.v1 (accumulated batch per tick)
Batch threshold: 100 citizens
Audio file: sfx/population/crowd_cheer_birth.ogg
Sample selection: random from {cheer_a.ogg, cheer_b.ogg, cheer_c.ogg} (variety)
Volume: 0.3 + clamp((batch_size / 1000), 0.0, 0.5)  (quiet for small batches)
Spatial: false
Max per tick: 1 trigger (multiple birth batches in one tick = 1 audio trigger)
Cooldown key: citizen_born
Cooldown duration: 10s
```

#### `tick.completed.v1` (hidden heartbeat)

```
Trigger: tick.completed.v1
Condition: current_stability < 30%
Audio file: sfx/ui/heartbeat.ogg
Volume: (30 - stability) / 30 × 0.4  (louder as stability drops)
Pulse interval: lerp(2000ms, 600ms, (30 - stability) / 30)  (faster as stability drops)
Spatial: false
Priority: LOW (never preempts other SFX)
Notes: This sound is "hidden" from the user — it plays subconsciously under other audio.
       It should never be audible as a distinct click; it is a texture in the mix.
```

---

## Section 3: Procedural Music System

### 3.1 State Machine Overview

The music system is a finite state machine driven by simulation state thresholds. Music transitions happen on state changes, not on per-tick polling.

```
States:
  PEACE_PROSPEROUS    → Stable economy, no war, stability >= 60%
  PEACE_STRUGGLING    → No war, stability 30–60% OR economy declining
  WAR_WINNING         → Active war, player's military advantage > 0
  WAR_LOSING          → Active war, player's military advantage < 0
  CRISIS              → Stability < 20% (any context; overrides war states)
  VICTORY             → Victory condition met
  DEFEAT              → Defeat condition met

Transitions (trigger condition → new state):
  Any state → CRISIS          : stability drops below 20%
  CRISIS → PEACE_PROSPEROUS   : stability rises above 35% (hysteresis)
  CRISIS → PEACE_STRUGGLING   : stability 20–35%
  Any state → WAR_WINNING     : war declared + military_advantage > 0
  WAR_WINNING → WAR_LOSING    : military_advantage flips < 0
  WAR_LOSING → WAR_WINNING    : military_advantage flips > 0
  WAR_* → PEACE_*             : peace declared (economy determines sub-state)
  Any state → VICTORY         : victory condition trigger
  Any state → DEFEAT          : defeat condition trigger
```

### 3.2 Music Layers

Each state uses a fixed set of layered tracks. Layers are always playing simultaneously; their volume is automated.

| Layer ID | Description | Always Playing? | Notes |
|----------|-------------|-----------------|-------|
| `base` | Core melodic layer (main theme variations) | Yes | Volume always > 0 |
| `tension` | Dissonant strings, percussion builds | No | Volume scales with (1 - stability / 100) |
| `war` | Heavy percussion + brass | No | Volume = 1.0 when war active; 0.0 otherwise |
| `triumph` | Ascending brass and choir | No | Volume = 1.0 on WAR_WINNING; 0.0 otherwise |
| `crisis` | Low drone + distorted strings | No | Volume = 1.0 on CRISIS; 0.0 otherwise |
| `victory_sting` | Full orchestral fanfare (one-shot) | No | Triggered once on VICTORY; does not loop |
| `defeat_sting` | Somber chord decay (one-shot) | No | Triggered once on DEFEAT; does not loop |

### 3.3 State-to-Layer Mapping

```
State: PEACE_PROSPEROUS
  base layer:    volume 0.9 (full, upbeat variation)
  tension layer: volume 0.0
  war layer:     volume 0.0
  crisis layer:  volume 0.0
  music/base/peace_prosperous_loop.ogg

State: PEACE_STRUGGLING
  base layer:    volume 0.7 (subdued variation)
  tension layer: volume 0.2
  war layer:     volume 0.0
  crisis layer:  volume 0.0
  music/base/peace_struggling_loop.ogg

State: WAR_WINNING
  base layer:    volume 0.6
  tension layer: volume 0.4
  war layer:     volume 0.8
  triumph layer: volume 0.6
  crisis layer:  volume 0.0
  music/base/war_loop.ogg

State: WAR_LOSING
  base layer:    volume 0.5
  tension layer: volume 0.7
  war layer:     volume 0.8
  triumph layer: volume 0.0
  crisis layer:  volume 0.2
  music/base/war_loop.ogg

State: CRISIS
  base layer:    volume 0.3
  tension layer: volume 1.0
  war layer:     volume 0.0 (unless also at war; then 0.4)
  crisis layer:  volume 1.0
  music/base/crisis_loop.ogg

State: VICTORY
  all layers:     volume 0.0 (fade out 2s)
  victory_sting:  play once at full volume
  music/stings/victory_fanfare.ogg

State: DEFEAT
  all layers:     volume 0.0 (fade out 3s)
  defeat_sting:   play once at full volume
  music/stings/defeat_somber.ogg
```

### 3.4 Adaptive Mixing & Cross-Fades

**Rule 1: No hard cuts.** All volume changes are envelope-automated. Target volumes are reached via linear ramp unless specified.

**Rule 2: Cross-fade duration.**
- State transitions: 2.0s cross-fade (previous state out, new state in simultaneously)
- Stability-driven tension layer: 3.0s ramp (smooth, barely noticeable in real-time)
- War layer activation: 1.0s ramp (faster response — war is a dramatic event)
- Crisis layer activation: 1.5s ramp

**Rule 3: Loop continuity.** When transitioning between state variants of the base layer, align the cross-fade to the nearest loop boundary (defined in audio metadata as `loop_start` and `loop_end` sample positions). Do not restart the loop from the beginning unless transitioning from/to VICTORY/DEFEAT.

**Rule 4: Stability-driven tension volume formula.**
```
tension_volume = clamp(1.0 - (stability / 60.0), 0.0, 1.0) × user_music_volume
```
This produces:
- stability 60%: tension = 0.0 (inaudible)
- stability 30%: tension = 0.5 (clearly audible)
- stability 0%: tension = 1.0 (dominant)

**Rule 5: War layer economy.**
```
if war_active:
    war_layer_volume = 0.8 × user_music_volume
    if military_advantage > 20:
        triumph_layer_volume = 0.6 × user_music_volume
    else:
        triumph_layer_volume = 0.0
else:
    war_layer_volume = 0.0
    triumph_layer_volume = 0.0
```

### 3.5 Music Asset File Layout

```
assets/audio/music/
  base/
    peace_prosperous_loop.ogg   (4:00 loop, BPM 72, major key)
    peace_struggling_loop.ogg   (3:30 loop, BPM 64, minor key)
    war_loop.ogg                (3:00 loop, BPM 120, dramatic)
    crisis_loop.ogg             (2:30 loop, BPM 55, dissonant)
  tension/
    tension_layer.ogg           (loops, stems only, no melody)
  war/
    war_layer_drums.ogg         (loops, percussion stem)
    war_layer_brass.ogg         (loops, brass stem)
  triumph/
    triumph_layer.ogg           (loops, ascending brass/choir stem)
  crisis/
    crisis_drone.ogg            (loops, drone + distortion stem)
  stings/
    victory_fanfare.ogg         (one-shot, 12s)
    defeat_somber.ogg           (one-shot, 8s)
  metadata/
    music_manifest.json         (loop points, BPM, key, mood tags per file)
```

### 3.6 Music Manifest Schema

```json
{
  "tracks": [
    {
      "id": "peace_prosperous_loop",
      "file": "music/base/peace_prosperous_loop.ogg",
      "duration_ms": 240000,
      "loop_start_ms": 0,
      "loop_end_ms": 240000,
      "bpm": 72,
      "key": "C major",
      "mood": ["peaceful", "prosperous", "hopeful"],
      "layer": "base",
      "state": "PEACE_PROSPEROUS"
    }
  ]
}
```

---

## Section 4: SFX Library Taxonomy

### 4.1 Format Standard

All SFX assets conform to the following technical standard:

| Property | Value |
|----------|-------|
| Container | OGG Vorbis |
| Sample Rate | 44,100 Hz |
| Bit Depth | 16-bit |
| Channels | Mono (spatial SFX) or Stereo (UI/stings) |
| Normalization | -14 LUFS integrated loudness |
| Peak Limit | -1.0 dBFS |
| Loop points | Embedded Vorbis comment `LOOPSTART` / `LOOPEND` for looping SFX |

### 4.2 UI Sound Effects

| Sound ID | File Path | Source | Duration | Trigger |
|----------|-----------|--------|----------|---------|
| `ui_button_click` | `sfx/ui/button_click.ogg` | Sampled (CC0) | 80ms | Any UI button press |
| `ui_panel_open` | `sfx/ui/panel_open.ogg` | Sampled (CC0) | 200ms | Panel/drawer open |
| `ui_panel_close` | `sfx/ui/panel_close.ogg` | Sampled (CC0) | 150ms | Panel/drawer close |
| `ui_alert_ping` | `sfx/ui/alert_ping.ogg` | Procedural (sine decay) | 300ms | Notification alert badge |
| `ui_alert_urgent` | `sfx/ui/alert_urgent.ogg` | Sampled | 600ms | Critical alert (red) |
| `ui_zoom_in` | `sfx/ui/zoom_in.ogg` | Procedural (freq shift up) | 250ms | Camera zoom in |
| `ui_zoom_out` | `sfx/ui/zoom_out.ogg` | Procedural (freq shift down) | 250ms | Camera zoom out |
| `ui_zoom_level_change` | `sfx/ui/zoom_level_change.ogg` | Sampled | 400ms | Crossing LOD boundary |
| `ui_tooltip_appear` | `sfx/ui/tooltip_appear.ogg` | Procedural (soft tick) | 60ms | Tooltip hover show |
| `ui_menu_open` | `sfx/ui/menu_open.ogg` | Sampled | 180ms | Main menu open |
| `ui_menu_close` | `sfx/ui/menu_close.ogg` | Sampled | 150ms | Main menu close |
| `ui_confirm` | `sfx/ui/confirm.ogg` | Sampled (positive chime) | 250ms | Confirm action |
| `ui_cancel` | `sfx/ui/cancel.ogg` | Sampled (negative tone) | 200ms | Cancel action |
| `ui_error` | `sfx/ui/error.ogg` | Procedural | 350ms | Error state |
| `ui_notification_badge` | `sfx/ui/notification_badge.ogg` | Procedural | 150ms | Badge counter increment |

### 4.3 Unit Sound Effects

| Sound ID | File Path | Source | Duration | Trigger Condition |
|----------|-----------|--------|----------|-------------------|
| `unit_footstep_grass` | `sfx/units/footstep_grass.ogg` | Sampled (CC0) | 200ms | Unit moves on grass/plains hex |
| `unit_footstep_stone` | `sfx/units/footstep_stone.ogg` | Sampled (CC0) | 180ms | Unit moves on stone/road hex |
| `unit_footstep_sand` | `sfx/units/footstep_sand.ogg` | Sampled (CC0) | 220ms | Unit moves on desert hex |
| `unit_footstep_snow` | `sfx/units/footstep_snow.ogg` | Sampled (CC0) | 250ms | Unit moves on snow/tundra hex |
| `unit_footstep_mud` | `sfx/units/footstep_mud.ogg` | Sampled (CC0) | 280ms | Unit moves on swamp/mud hex |
| `unit_footstep_water` | `sfx/units/footstep_water.ogg` | Sampled (CC0) | 300ms | Unit moves through shallow water |
| `unit_weapon_clash_sword` | `sfx/units/weapon_clash_sword.ogg` | Sampled (CC0) | 350ms | Melee combat, sword-type unit |
| `unit_weapon_clash_spear` | `sfx/units/weapon_clash_spear.ogg` | Sampled (CC0) | 280ms | Melee combat, spear-type unit |
| `unit_weapon_clash_axe` | `sfx/units/weapon_clash_axe.ogg` | Sampled (CC0) | 400ms | Melee combat, axe-type unit |
| `unit_ranged_bow_release` | `sfx/units/ranged_bow_release.ogg` | Sampled (CC0) | 200ms | Ranged attack, bow/crossbow |
| `unit_ranged_projectile_fly` | `sfx/units/ranged_projectile_fly.ogg` | Procedural | 400ms | Projectile in flight (spatial) |
| `unit_ranged_projectile_hit` | `sfx/units/ranged_projectile_hit.ogg` | Sampled (CC0) | 250ms | Projectile impact |
| `unit_death_infantry` | `sfx/units/death_infantry.ogg` | Sampled (CC0) | 800ms | Infantry unit destroyed |
| `unit_death_cavalry` | `sfx/units/death_cavalry.ogg` | Sampled (CC0) | 1000ms | Cavalry unit destroyed |
| `unit_death_siege` | `sfx/units/death_siege.ogg` | Sampled | 1200ms | Siege engine destroyed |
| `unit_formation_change` | `sfx/units/formation_change.ogg` | Procedural | 500ms | Unit formation command |
| `unit_morale_high` | `sfx/units/morale_high.ogg` | Sampled | 600ms | Morale crosses high threshold |
| `unit_morale_routing` | `sfx/units/morale_routing.ogg` | Sampled | 1000ms | Unit starts routing |
| `unit_siege_catapult_launch` | `sfx/units/siege_catapult_launch.ogg` | Sampled | 800ms | Catapult/trebuchet fires |
| `unit_siege_ram_impact` | `sfx/units/siege_ram_impact.ogg` | Sampled | 1200ms | Battering ram hits wall |

### 4.4 Building & Structure Sound Effects

| Sound ID | File Path | Source | Duration | Trigger Condition |
|----------|-----------|--------|----------|-------------------|
| `building_construction_start` | `sfx/buildings/construction_start.ogg` | Sampled | 600ms | `build_structure` command queued |
| `building_construction_complete` | `sfx/buildings/construction_complete.ogg` | Sampled | 1200ms | Structure fully built |
| `building_production_tick` | `sfx/buildings/production_tick.ogg` | Procedural (subtle hum) | 500ms loop | Production building active; loops while working |
| `building_production_idle` | `sfx/buildings/production_idle.ogg` | Procedural (silence/decay) | 300ms | Production building goes idle |
| `building_destruction_small` | `sfx/buildings/destruction_small.ogg` | Sampled | 1500ms | Small structure destroyed |
| `building_destruction_large` | `sfx/buildings/destruction_large.ogg` | Sampled | 2500ms | Large structure destroyed |
| `building_destruction_wall` | `sfx/buildings/destruction_wall.ogg` | Sampled | 2000ms | Defensive wall section destroyed |
| `building_fire_start` | `sfx/buildings/fire_start.ogg` | Sampled | 800ms | Building catches fire |
| `building_fire_loop` | `sfx/buildings/fire_loop.ogg` | Sampled (looping) | — | Building burning (loops until extinguished) |
| `building_repair_start` | `sfx/buildings/repair_start.ogg` | Sampled | 400ms | Repair command begins |
| `building_repair_complete` | `sfx/buildings/repair_complete.ogg` | Sampled | 800ms | Repair complete |
| `building_garrison_enter` | `sfx/buildings/garrison_enter.ogg` | Sampled | 400ms | Unit enters garrison |
| `building_upgrade_complete` | `sfx/buildings/upgrade_complete.ogg` | Sampled | 1000ms | Structure upgraded to next tier |
| `building_market_open` | `sfx/buildings/market_open.ogg` | Sampled | 600ms | Market district opens for trading |

### 4.5 Environment Sound Effects

| Sound ID | File Path | Source | Duration | Region Type |
|----------|-----------|--------|----------|-------------|
| `env_wind_plains` | `sfx/environment/wind_plains.ogg` | Sampled (looping) | — | Plains/grassland hex |
| `env_wind_desert` | `sfx/environment/wind_desert.ogg` | Sampled (looping) | — | Desert/arid hex |
| `env_wind_mountain` | `sfx/environment/wind_mountain.ogg` | Sampled (looping) | — | Mountain/highland hex |
| `env_rain_light` | `sfx/environment/rain_light.ogg` | Sampled (looping) | — | Rain weather event (light) |
| `env_rain_heavy` | `sfx/environment/rain_heavy.ogg` | Sampled (looping) | — | Rain weather event (heavy) |
| `env_thunder` | `sfx/environment/thunder.ogg` | Sampled | 3000ms | Thunderstorm event (one-shot) |
| `env_ocean_waves` | `sfx/environment/ocean_waves.ogg` | Sampled (looping) | — | Coastal hex / sea region |
| `env_ocean_waves_calm` | `sfx/environment/ocean_waves_calm.ogg` | Sampled (looping) | — | Calm sea (no weather event) |
| `env_forest_ambiance` | `sfx/environment/forest_ambiance.ogg` | Sampled (looping) | — | Forest/jungle hex |
| `env_forest_birds` | `sfx/environment/forest_birds.ogg` | Sampled (looping) | — | Forest hex, daytime |
| `env_cave_drip` | `sfx/environment/cave_drip.ogg` | Sampled (looping) | — | Underground/cave hex |
| `env_volcano_rumble` | `sfx/environment/volcano_rumble.ogg` | Sampled (looping) | — | Volcanic region (active) |
| `env_ice_crack` | `sfx/environment/ice_crack.ogg` | Sampled | 1500ms | Ice hex cracking (spring thaw) |
| `env_river_flow` | `sfx/environment/river_flow.ogg` | Sampled (looping) | — | River/delta hex |

### 4.6 Economy Sound Effects

| Sound ID | File Path | Source | Duration | Trigger |
|----------|-----------|--------|----------|---------|
| `economy_market_bell` | `sfx/economy/market_bell.ogg` | Sampled | 800ms | `economy.market_cleared.v1` (visible in viewport) |
| `economy_currency_clink` | `sfx/economy/currency_clink.ogg` | Sampled | 400ms | Tax collection tick (subtle) |
| `economy_currency_large` | `sfx/economy/currency_large.ogg` | Sampled | 600ms | Large trade deal cleared |
| `economy_bankruptcy` | `sfx/economy/bankruptcy.ogg` | Sampled | 2000ms | `economy.bankruptcy.declared.v1` |
| `economy_trade_route_bell` | `sfx/economy/trade_route_bell.ogg` | Sampled | 1000ms | `trade.route.established.v1` |
| `economy_supply_shock` | `sfx/economy/supply_shock.ogg` | Sampled | 1500ms | `economy.supply_shock.v1` |
| `economy_boom` | `sfx/economy/boom.ogg` | Sampled | 1200ms | GDP crosses high threshold |
| `economy_recession_tone` | `sfx/economy/recession_tone.ogg` | Sampled | 1800ms | GDP declines > 20% in 10 ticks |

### 4.7 Diplomacy Sound Effects

| Sound ID | File Path | Source | Duration | Trigger |
|----------|-----------|--------|----------|---------|
| `diplomacy_treaty_signing` | `sfx/diplomacy/treaty_signing.ogg` | Sampled (quill + seal) | 1200ms | `diplomacy.treaty_signed.v1` |
| `diplomacy_war_horn_aggressor` | `sfx/diplomacy/war_declaration_horn_aggressor.ogg` | Sampled | 4200ms | `war.declared.v1` (player aggressor) |
| `diplomacy_war_horn_target` | `sfx/diplomacy/war_declaration_horn_target.ogg` | Sampled | 3800ms | `war.declared.v1` (player target) |
| `diplomacy_alliance_fanfare` | `sfx/diplomacy/alliance_formed.ogg` | Sampled | 2000ms | `alliance.formed.v1` |
| `diplomacy_peace_relief` | `sfx/diplomacy/peace_declared.ogg` | Sampled | 2200ms | `peace.declared.v1` |
| `diplomacy_embargo` | `sfx/diplomacy/embargo.ogg` | Sampled | 800ms | `diplomacy.embargo.declared.v1` |
| `diplomacy_proposal_sent` | `sfx/diplomacy/proposal_sent.ogg` | Sampled | 500ms | Any diplomatic proposal |
| `diplomacy_proposal_accepted` | `sfx/diplomacy/proposal_accepted.ogg` | Sampled | 700ms | Proposal accepted |
| `diplomacy_proposal_rejected` | `sfx/diplomacy/proposal_rejected.ogg` | Sampled | 600ms | Proposal rejected |

### 4.8 SFX Cooldown and Deduplication

The SFX trigger system enforces per-sound cooldowns to prevent audio spam. Cooldowns are keyed on `(sound_id, entity_id)` or `(sound_id, region_id)` as appropriate.

```
AudioCooldownTracker:
  cooldowns: HashMap<(SoundId, Option<EntityId>), Instant>

fn can_play(sound_id, entity_id, cooldown_ms) -> bool:
    key = (sound_id, entity_id)
    match cooldowns.get(key):
        Some(last_played) if last_played.elapsed() < cooldown_ms → false
        _ → true (update cooldowns[key] = now)
```

**Deduplication rules:**
- Same SFX triggered from the same entity within cooldown window: deduplicated (play once)
- Same SFX triggered from different entities within cooldown window: play once (spatial SFX uses first entity's position; non-spatial deduplicates globally)
- High-priority events (war declaration, disaster) bypass cooldown with `force: true` flag

---

## Section 5: Ambient Soundscape System

### 5.1 Overview

The ambient soundscape is a continuously playing layered audio system that evolves based on:
1. **Camera position** — what region/district is the viewport centered on
2. **Camera zoom level** — LOD zoom level (1, 2, or 3)
3. **Local simulation state** — weather, activity level, war proximity

The ambient system is separate from music and SFX. It plays continuously in a loop mix, and its layer volumes are smoothly automated as the camera moves.

### 5.2 Zoom-Level Ambient Profiles

#### Zoom 1: Strategic / Nation View
```
Layers active:
  - env_wind_plains (or region-appropriate wind): 0.6 volume
  - distant_crowd_murmur: 0.3 volume (represents the nation below)
  - orchestral_breathe (soft low-pass filtered version of music base): 0.2 volume
  - distant_battle_rumble: 0.0–0.4 volume (scales with active_battles_in_viewport)

Update trigger: camera hex position changes by >= 5 hexes
Crossfade duration: 3.0s
```

#### Zoom 2: City / District View
```
Layers active:
  - city_market_ambiance: volume weighted by (market_activity_level / 100)
  - construction_ambient: volume weighted by (active_constructions / 10)
  - crowd_base: 0.4 volume (standard city background)
  - env_wind_region: 0.3 volume (region-specific wind)
  - env_weather: volume scaled by current weather event severity
  - distant_military_march: 0.0–0.5 volume (if military units in viewport)

Update trigger: camera hex position changes by >= 2 hexes OR district viewport changes
Crossfade duration: 2.0s
```

#### Zoom 3: Citizen / Simulation View
```
Layers active:
  - conversation_murmur: 0.5 volume (human-scale interaction sounds)
  - personal_space_ambient: 0.4 volume (indoor: hearth crackle, paper; outdoor: birds, breeze)
  - work_sounds: volume weighted by density of working citizens in viewport
    - farmer_ambient if dominant job = farmer
    - forge_ambient if dominant job = smith/craftsman
    - scholar_ambient if dominant job = scholar/researcher
  - street_noise: 0.2 volume (distant cart, footsteps)

Update trigger: any camera movement
Crossfade duration: 1.5s
```

### 5.3 Region-Type Ambient Layers

When the camera is positioned over a region, the ambient system loads the region-appropriate base:

| Region Type | Base Ambient Layer | Secondary Layer |
|-------------|-------------------|-----------------|
| Plains/Grassland | `env_wind_plains` | `env_forest_birds` (if forested) |
| Desert/Arid | `env_wind_desert` | none |
| Mountain | `env_wind_mountain` | `env_ice_crack` (if frozen season) |
| Coastal | `env_ocean_waves_calm` or `env_ocean_waves` | `env_wind_plains` |
| Forest | `env_forest_ambiance` | `env_forest_birds` (daytime) |
| Tundra/Arctic | `env_wind_mountain` | `env_ice_crack` |
| Volcanic | `env_volcano_rumble` | `env_wind_mountain` |
| River Delta | `env_river_flow` | `env_forest_birds` |

### 5.4 Camera Position Tracking (Bevy)

```rust
#[derive(Resource)]
struct AmbientAudioState {
    current_region_type: RegionType,
    current_zoom: ZoomLevel,
    camera_hex: HexCoord,
    layer_volumes: HashMap<AmbientLayerId, f32>,
    layer_handles: HashMap<AmbientLayerId, TrackHandle>,
}

fn update_ambient_soundscape(
    camera_query: Query<(&Transform, &Camera), With<GameCamera>>,
    map_state: Res<MapState>,
    mut ambient: ResMut<AmbientAudioState>,
    mut audio_manager: ResMut<AudioManagerState>,
) {
    let (transform, camera) = camera_query.single();
    let hex = world_to_hex(transform.translation.truncate());
    let zoom = camera.zoom_level();

    if hex == ambient.camera_hex && zoom == ambient.current_zoom {
        return; // No change
    }

    let region_type = map_state.region_type_at(hex);
    let target_volumes = compute_target_volumes(region_type, zoom, &map_state, hex);

    for (layer_id, target_vol) in &target_volumes {
        if let Some(handle) = ambient.layer_handles.get_mut(layer_id) {
            handle.set_volume(
                *target_vol as f64,
                Tween {
                    duration: Duration::from_secs_f32(2.0),
                    easing: Easing::Linear,
                    ..default()
                },
            );
        }
    }

    ambient.camera_hex = hex;
    ambient.current_zoom = zoom;
    ambient.current_region_type = region_type;
    ambient.layer_volumes = target_volumes;
}
```

### 5.5 Activity-Based Ambient Scaling

Ambient volume is not just region-based — it responds to local simulation activity:

```
city_market_volume = clamp(active_trade_orders_in_viewport / 50, 0.0, 0.8)
construction_volume = clamp(active_constructions_in_viewport / 5, 0.0, 0.5)
military_volume = clamp(military_units_in_viewport / 100, 0.0, 0.6)
crowd_volume = clamp(citizens_in_viewport / 1000, 0.2, 0.9)
```

These values are recomputed each time the viewport changes (camera moved or zoomed) and smoothly interpolated to their new targets.

---

## Section 6: Kira (Bevy) Integration

### 6.1 Dependencies

```toml
# Cargo.toml
[dependencies]
bevy = "0.15"
kira = { version = "0.9", features = ["ogg"] }
bevy_kira_audio = "0.21"
```

### 6.2 Plugin Structure

```rust
use bevy::prelude::*;
use kira::manager::{AudioManager, AudioManagerSettings, backend::DefaultBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::track::{TrackBuilder, TrackHandle};
use kira::tween::Tween;
use std::time::Duration;

// --- Resources ---

#[derive(Resource)]
pub struct AudioManagerState {
    pub manager: AudioManager<DefaultBackend>,
    pub music_tracks: HashMap<MusicLayerId, TrackHandle>,
    pub sfx_cooldowns: AudioCooldownTracker,
    pub current_music_state: MusicState,
}

#[derive(Resource)]
pub struct SoundAssetRegistry {
    pub sounds: HashMap<SoundId, Handle<AudioSource>>,
    pub loaded: HashSet<SoundId>,
}

#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub ambient_volume: f32,
    pub muted: bool,
}

// --- Plugin Definition ---

pub struct CivAudioPlugin;

impl Plugin for CivAudioPlugin {
    fn build(&self, app: &mut App) {
        let manager = AudioManager::<DefaultBackend>::new(
            AudioManagerSettings::default()
        ).expect("Failed to initialize Kira AudioManager");

        app
            .insert_resource(AudioManagerState {
                manager,
                music_tracks: HashMap::new(),
                sfx_cooldowns: AudioCooldownTracker::new(),
                current_music_state: MusicState::PeaceProsperous,
            })
            .insert_resource(SoundAssetRegistry {
                sounds: HashMap::new(),
                loaded: HashSet::new(),
            })
            .insert_resource(AudioSettings::default())
            .insert_resource(AmbientAudioState::default())
            .add_systems(Startup, (
                preload_critical_sfx,
                initialize_music_layers,
            ))
            .add_systems(Update, (
                handle_simulation_events,
                update_music_state,
                update_ambient_soundscape,
                apply_audio_settings,
            ).chain());
    }
}
```

### 6.3 Simulation Event Handler

```rust
fn handle_simulation_events(
    mut sim_events: EventReader<SimulationEvent>,
    mut audio: ResMut<AudioManagerState>,
    audio_settings: Res<AudioSettings>,
    registry: Res<SoundAssetRegistry>,
) {
    for event in sim_events.read() {
        if audio_settings.muted { continue; }

        match event.event_type.as_str() {
            "war.declared.v1" => {
                let is_aggressor = event.payload["aggressor_nation_id"] == local_player_nation();
                let sound_id = if is_aggressor {
                    "diplomacy_war_horn_aggressor"
                } else {
                    "diplomacy_war_horn_target"
                };
                play_sfx_oneshot(
                    sound_id,
                    None,
                    audio_settings.sfx_volume,
                    &mut audio,
                    &registry,
                );
                audio.current_music_state = if is_aggressor {
                    MusicState::WarWinning
                } else {
                    MusicState::WarLosing
                };
            }

            "battle.resolved.v1" => {
                let outcome = event.payload["outcome"].as_str().unwrap_or("draw");
                let sound_id = match outcome {
                    "victory" => "battle_resolved_victory",
                    "defeat"  => "battle_resolved_defeat",
                    _         => "battle_resolved_draw",
                };
                let position = event.payload["battle_hex"].as_hex_coord();
                play_sfx_spatial(sound_id, position, audio_settings.sfx_volume, &mut audio, &registry);
            }

            "disaster.triggered.v1" => {
                let disaster_type = event.payload["disaster_type"].as_str().unwrap_or("unknown");
                let sound_id = format!("env_disaster_{}", disaster_type);
                let severity = event.payload["severity"].as_f32().unwrap_or(0.5);
                let volume = 0.5 + severity * 0.5;
                play_sfx_oneshot_volume(
                    &sound_id,
                    volume * audio_settings.sfx_volume,
                    &mut audio,
                    &registry,
                );
            }

            "tick.completed.v1" => {
                // Hidden heartbeat — only if stability < 30%
                if let Some(stability) = event.payload["stability"].as_f32() {
                    if stability < 30.0 && !audio.sfx_cooldowns.on_cooldown("ui_heartbeat", None) {
                        let volume = (30.0 - stability) / 30.0 * 0.4 * audio_settings.sfx_volume;
                        play_sfx_oneshot_volume("ui_heartbeat", volume, &mut audio, &registry);
                        let pulse_ms = lerp(2000.0, 600.0, (30.0 - stability) / 30.0) as u64;
                        audio.sfx_cooldowns.set_cooldown("ui_heartbeat", None, pulse_ms);
                    }
                }
            }

            _ => {
                // Unmapped events: no audio trigger
            }
        }
    }
}
```

### 6.4 Music State Update System

```rust
fn update_music_state(
    mut audio: ResMut<AudioManagerState>,
    audio_settings: Res<AudioSettings>,
    sim_state: Res<SimulationStateSnapshot>,
) {
    let target_state = compute_target_music_state(&sim_state);

    if target_state == audio.current_music_state {
        return;
    }

    let crossfade = Tween {
        duration: Duration::from_secs_f32(2.0),
        easing: Easing::Linear,
        ..default()
    };

    // Fade out all layers
    for (_id, track) in &mut audio.music_tracks {
        track.set_volume(0.0, crossfade.clone());
    }

    // Compute new layer volumes for target state
    let new_volumes = music_state_layer_volumes(&target_state, &sim_state, &audio_settings);

    // Apply new volumes (they will rise as old ones fade)
    for (layer_id, target_vol) in &new_volumes {
        if let Some(track) = audio.music_tracks.get_mut(layer_id) {
            track.set_volume(*target_vol as f64, crossfade.clone());
        }
    }

    // Stability-driven tension layer (continuous, not state-locked)
    let tension_vol = compute_tension_volume(sim_state.stability, audio_settings.music_volume);
    if let Some(tension_track) = audio.music_tracks.get_mut(&MusicLayerId::Tension) {
        tension_track.set_volume(tension_vol as f64, Tween {
            duration: Duration::from_secs_f32(3.0),
            ..default()
        });
    }

    audio.current_music_state = target_state;
}
```

### 6.5 Async Sound Loading

```rust
async fn load_sound_asset(
    path: &str,
    asset_server: &AssetServer,
) -> Result<Handle<AudioSource>, AudioError> {
    let handle = asset_server.load::<AudioSource>(path);
    // Wait for load via AssetServer event or direct await in async context
    Ok(handle)
}

fn preload_critical_sfx(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<SoundAssetRegistry>,
) {
    // Critical sounds preloaded at startup — all UI sounds and frequent event sounds
    let critical_sounds = vec![
        ("ui_button_click",     "sfx/ui/button_click.ogg"),
        ("ui_alert_ping",       "sfx/ui/alert_ping.ogg"),
        ("ui_zoom_in",          "sfx/ui/zoom_in.ogg"),
        ("ui_zoom_out",         "sfx/ui/zoom_out.ogg"),
        ("ui_heartbeat",        "sfx/ui/heartbeat.ogg"),
        ("diplomacy_war_horn_aggressor", "sfx/diplomacy/war_declaration_horn_aggressor.ogg"),
        ("diplomacy_war_horn_target",    "sfx/diplomacy/war_declaration_horn_target.ogg"),
        ("battle_resolved_victory",      "sfx/military/battle_resolved_victory.ogg"),
        ("battle_resolved_defeat",       "sfx/military/battle_resolved_defeat.ogg"),
    ];

    for (id, path) in critical_sounds {
        let handle = asset_server.load(path);
        registry.sounds.insert(id.to_string(), handle);
    }
}
```

---

## Section 7: Web Client (Howler.js) Integration

### 7.1 Dependencies

```json
{
  "dependencies": {
    "howler": "^2.2.4"
  }
}
```

### 7.2 Type Definitions

```typescript
import { Howl, Howler } from 'howler';

type MusicState =
  | 'PEACE_PROSPEROUS'
  | 'PEACE_STRUGGLING'
  | 'WAR_WINNING'
  | 'WAR_LOSING'
  | 'CRISIS'
  | 'VICTORY'
  | 'DEFEAT';

type MusicLayer =
  | 'base'
  | 'tension'
  | 'war'
  | 'triumph'
  | 'crisis'
  | 'victory_sting'
  | 'defeat_sting';

interface SimulationEvent {
  event_type: string;
  tick: number;
  payload: Record<string, unknown>;
}

interface AudioSettings {
  masterVolume: number;   // 0.0–1.0
  musicVolume: number;    // 0.0–1.0
  sfxVolume: number;      // 0.0–1.0
  ambientVolume: number;  // 0.0–1.0
  muted: boolean;
}
```

### 7.3 AudioManager Class

```typescript
class AudioManager {
    private sounds: Map<string, Howl> = new Map();
    private musicLayers: Map<MusicLayer, Howl> = new Map();
    private ambientLayers: Map<string, Howl> = new Map();
    private cooldowns: Map<string, number> = new Map();
    private currentMusicState: MusicState = 'PEACE_PROSPEROUS';
    private settings: AudioSettings;
    private fadeIntervals: Map<MusicLayer, number> = new Map();

    constructor(settings: AudioSettings) {
        this.settings = settings;
        Howler.volume(settings.masterVolume);
        this.preloadCriticalSounds();
        this.initMusicLayers();
    }

    private preloadCriticalSounds(): void {
        const critical: [string, string][] = [
            ['ui_button_click',            'sfx/ui/button_click.ogg'],
            ['ui_alert_ping',              'sfx/ui/alert_ping.ogg'],
            ['ui_zoom_in',                 'sfx/ui/zoom_in.ogg'],
            ['ui_zoom_out',                'sfx/ui/zoom_out.ogg'],
            ['ui_heartbeat',               'sfx/ui/heartbeat.ogg'],
            ['diplomacy_war_horn_aggressor', 'sfx/diplomacy/war_declaration_horn_aggressor.ogg'],
            ['diplomacy_war_horn_target',   'sfx/diplomacy/war_declaration_horn_target.ogg'],
            ['battle_resolved_victory',     'sfx/military/battle_resolved_victory.ogg'],
            ['battle_resolved_defeat',      'sfx/military/battle_resolved_defeat.ogg'],
        ];
        for (const [id, path] of critical) {
            this.sounds.set(id, new Howl({ src: [path], preload: true }));
        }
    }

    private initMusicLayers(): void {
        const layers: [MusicLayer, string, boolean][] = [
            ['base',          'music/base/peace_prosperous_loop.ogg', true],
            ['tension',       'music/tension/tension_layer.ogg',      true],
            ['war',           'music/war/war_layer_drums.ogg',         true],
            ['triumph',       'music/triumph/triumph_layer.ogg',       true],
            ['crisis',        'music/crisis/crisis_drone.ogg',         true],
            ['victory_sting', 'music/stings/victory_fanfare.ogg',      false],
            ['defeat_sting',  'music/stings/defeat_somber.ogg',        false],
        ];
        for (const [layer, path, loop_] of layers) {
            const howl = new Howl({
                src: [path],
                loop: loop_,
                volume: 0,
                preload: true,
            });
            if (loop_) howl.play();
            this.musicLayers.set(layer, howl);
        }
        // Set initial state volumes
        this.applyMusicStateVolumes('PEACE_PROSPEROUS', 0);
    }

    triggerEvent(event: SimulationEvent): void {
        if (this.settings.muted) return;

        switch (event.event_type) {
            case 'war.declared.v1': {
                const isAggressor = (event.payload.aggressor_nation_id as string) === localPlayerNationId();
                const soundId = isAggressor
                    ? 'diplomacy_war_horn_aggressor'
                    : 'diplomacy_war_horn_target';
                this.playSfx(soundId);
                this.setMusicState(isAggressor ? 'WAR_WINNING' : 'WAR_LOSING');
                break;
            }

            case 'battle.resolved.v1': {
                const outcome = event.payload.outcome as string;
                const soundId = outcome === 'victory'
                    ? 'battle_resolved_victory'
                    : outcome === 'defeat'
                    ? 'battle_resolved_defeat'
                    : 'battle_resolved_draw';
                this.playSfx(soundId);
                break;
            }

            case 'disaster.triggered.v1': {
                const type_ = event.payload.disaster_type as string;
                const severity = (event.payload.severity as number) ?? 0.5;
                const volume = (0.5 + severity * 0.5) * this.settings.sfxVolume;
                this.playSfxVolume(`env_disaster_${type_}`, volume);
                break;
            }

            case 'tech.unlocked.v1':
                this.playSfx('tech_discovery_chime');
                break;

            case 'tick.completed.v1': {
                const stability = event.payload.stability as number;
                if (stability < 30 && !this.onCooldown('ui_heartbeat')) {
                    const volume = ((30 - stability) / 30) * 0.4 * this.settings.sfxVolume;
                    this.playSfxVolume('ui_heartbeat', volume);
                    const pulseMs = this.lerp(2000, 600, (30 - stability) / 30);
                    this.setCooldown('ui_heartbeat', pulseMs);
                }
                break;
            }

            case 'economy.bankruptcy.declared.v1':
                this.playSfx('economy_bankruptcy');
                break;

            case 'institution.formed.v1':
                this.playSfx('institution_established_fanfare');
                break;

            default:
                // Unmapped event: no audio
                break;
        }
    }

    setMusicState(state: MusicState): void {
        if (state === this.currentMusicState) return;
        this.applyMusicStateVolumes(state, 2000);
        this.currentMusicState = state;
    }

    private applyMusicStateVolumes(state: MusicState, fadeDurationMs: number): void {
        const volumes = MUSIC_STATE_LAYER_VOLUMES[state];
        for (const [layer, targetVol] of Object.entries(volumes)) {
            const howl = this.musicLayers.get(layer as MusicLayer);
            if (!howl) continue;
            const scaledVol = (targetVol as number) * this.settings.musicVolume;
            if (fadeDurationMs > 0) {
                howl.fade(howl.volume(), scaledVol, fadeDurationMs);
            } else {
                howl.volume(scaledVol);
            }
        }

        if (state === 'VICTORY') {
            this.musicLayers.get('victory_sting')?.play();
        }
        if (state === 'DEFEAT') {
            this.musicLayers.get('defeat_sting')?.play();
        }
    }

    setStabilityDrivenTension(stability: number): void {
        const tensionVol = Math.max(0, 1.0 - stability / 60.0) * this.settings.musicVolume;
        this.musicLayers.get('tension')?.fade(
            this.musicLayers.get('tension')?.volume() ?? 0,
            tensionVol,
            3000
        );
    }

    setMasterVolume(vol: number): void {
        this.settings.masterVolume = Math.max(0, Math.min(1, vol));
        Howler.volume(this.settings.masterVolume);
        this.persistSettings();
    }

    setMusicVolume(vol: number): void {
        this.settings.musicVolume = Math.max(0, Math.min(1, vol));
        this.applyMusicStateVolumes(this.currentMusicState, 500);
        this.persistSettings();
    }

    setSfxVolume(vol: number): void {
        this.settings.sfxVolume = Math.max(0, Math.min(1, vol));
        this.persistSettings();
    }

    setAmbientVolume(vol: number): void {
        this.settings.ambientVolume = Math.max(0, Math.min(1, vol));
        for (const layer of this.ambientLayers.values()) {
            layer.volume(this.settings.ambientVolume);
        }
        this.persistSettings();
    }

    setMuted(muted: boolean): void {
        this.settings.muted = muted;
        Howler.mute(muted);
        this.persistSettings();
    }

    private playSfx(soundId: string): void {
        this.playSfxVolume(soundId, this.settings.sfxVolume);
    }

    private playSfxVolume(soundId: string, volume: number): void {
        const howl = this.sounds.get(soundId);
        if (!howl) {
            // Lazy-load if not preloaded
            const path = SFX_ASSET_PATHS[soundId];
            if (!path) return;
            const lazy = new Howl({ src: [path], volume });
            lazy.play();
            this.sounds.set(soundId, lazy);
            return;
        }
        howl.volume(volume);
        howl.play();
    }

    private onCooldown(key: string): boolean {
        const lastMs = this.cooldowns.get(key);
        if (lastMs === undefined) return false;
        return Date.now() - lastMs < (SOUND_COOLDOWNS[key] ?? 1000);
    }

    private setCooldown(key: string, durationMs: number): void {
        this.cooldowns.set(key, Date.now());
        // Store custom duration alongside timestamp
        SOUND_COOLDOWNS[key] = durationMs;
    }

    private lerp(a: number, b: number, t: number): number {
        return a + (b - a) * Math.max(0, Math.min(1, t));
    }

    private persistSettings(): void {
        localStorage.setItem('civ_audio_settings', JSON.stringify(this.settings));
    }

    static loadSettings(): AudioSettings {
        const stored = localStorage.getItem('civ_audio_settings');
        if (stored) {
            return JSON.parse(stored) as AudioSettings;
        }
        return {
            masterVolume: 1.0,
            musicVolume: 0.8,
            sfxVolume: 0.9,
            ambientVolume: 0.6,
            muted: false,
        };
    }
}

// Layer volume lookup table
const MUSIC_STATE_LAYER_VOLUMES: Record<MusicState, Partial<Record<MusicLayer, number>>> = {
    PEACE_PROSPEROUS: { base: 0.9, tension: 0.0, war: 0.0, triumph: 0.0, crisis: 0.0 },
    PEACE_STRUGGLING: { base: 0.7, tension: 0.2, war: 0.0, triumph: 0.0, crisis: 0.0 },
    WAR_WINNING:      { base: 0.6, tension: 0.4, war: 0.8, triumph: 0.6, crisis: 0.0 },
    WAR_LOSING:       { base: 0.5, tension: 0.7, war: 0.8, triumph: 0.0, crisis: 0.2 },
    CRISIS:           { base: 0.3, tension: 1.0, war: 0.0, triumph: 0.0, crisis: 1.0 },
    VICTORY:          { base: 0.0, tension: 0.0, war: 0.0, triumph: 0.0, crisis: 0.0 },
    DEFEAT:           { base: 0.0, tension: 0.0, war: 0.0, triumph: 0.0, crisis: 0.0 },
};
```

---

## Section 8: Audio Settings

### 8.1 User-Facing Controls

| Setting | Type | Range | Default | Description |
|---------|------|-------|---------|-------------|
| Master Volume | Float slider | 0–100% | 100% | Global multiplier for all audio |
| Music Volume | Float slider | 0–100% | 80% | Music layer volume (independent of SFX) |
| SFX Volume | Float slider | 0–100% | 90% | Sound effects volume |
| Ambient Volume | Float slider | 0–100% | 60% | Ambient soundscape volume |
| Mute All | Toggle | on/off | off | Mutes all audio without changing settings |
| Audio Output Device | Dropdown | system devices | Default | Select output device (desktop only) |

### 8.2 Persistence

**Web client:** Settings persisted to `localStorage` under key `civ_audio_settings` (JSON).

```javascript
// Write
localStorage.setItem('civ_audio_settings', JSON.stringify(settings));

// Read (with defaults)
const stored = localStorage.getItem('civ_audio_settings');
const settings = stored ? JSON.parse(stored) : DEFAULT_AUDIO_SETTINGS;
```

**Desktop client (Bevy):** Settings persisted to platform config file.

```rust
// Linux:  ~/.config/civlab/audio.toml
// macOS:  ~/Library/Application Support/CivLab/audio.toml
// Windows: %APPDATA%\CivLab\audio.toml

#[derive(Serialize, Deserialize, Resource)]
struct AudioSettingsFile {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    ambient_volume: f32,
    muted: bool,
    output_device: Option<String>,
}
```

### 8.3 Output Device Selection

**Desktop (Bevy/Kira):**
Kira's `DefaultBackend` uses the system default output device. To support device selection, use `kira`'s `CpalBackend` with an explicit device selector:

```rust
use kira::manager::backend::cpal::CpalBackend;

fn create_audio_manager_with_device(device_name: &str) -> AudioManager<CpalBackend> {
    let backend_settings = CpalBackendSettings {
        device: Some(device_name.to_string()),
    };
    AudioManager::new(AudioManagerSettings {
        backend_settings,
        ..default()
    }).expect("Failed to create audio manager")
}
```

Device enumeration:
```rust
use cpal::traits::{DeviceTrait, HostTrait};

fn enumerate_output_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices()
        .expect("Failed to enumerate devices")
        .filter_map(|d| d.name().ok())
        .collect()
}
```

**Web (Howler.js):**
Web Audio API sink selection is available via `AudioContext.setSinkId()` (Chrome 110+):

```typescript
async function setOutputDevice(deviceId: string): Promise<void> {
    const ctx = Howler.ctx as AudioContext & { setSinkId?: (id: string) => Promise<void> };
    if (ctx.setSinkId) {
        await ctx.setSinkId(deviceId);
    }
    // setSinkId not supported → log warning, continue with default device
}

async function enumerateOutputDevices(): Promise<MediaDeviceInfo[]> {
    const devices = await navigator.mediaDevices.enumerateDevices();
    return devices.filter(d => d.kind === 'audiooutput');
}
```

---

## Section 9: Asset Pipeline

### 9.1 Asset Source Strategy

| Asset Type | Source Strategy | License Requirement |
|------------|----------------|---------------------|
| Music (background loops) | Human composer export OR AI-generated (MusicGen/Suno) | CC0 or custom/owned |
| SFX (one-shots) | Freesound.org CC0 OR procedural generation | CC0 only for Freesound |
| SFX (procedural) | Generated via `sox` / Python `pedalboard` | N/A (generated) |
| Ambient loops | Freesound.org CC0 OR field recordings | CC0 only |

### 9.2 Audio Specification IR (AudioSpec)

For AI-generated or commissioned audio, an `AudioSpec` IR document describes the desired asset. This is analogous to ArtSpec for visual assets.

```json
{
  "audio_spec_version": "1.0",
  "asset_id": "music_peace_prosperous_loop",
  "type": "music_loop",
  "duration_seconds": 240,
  "mood": ["peaceful", "prosperous", "hopeful", "pastoral"],
  "instrumentation": ["strings", "woodwinds", "light_percussion"],
  "tempo_bpm": 72,
  "key": "C major",
  "reference_tracks": [
    "Civilization VI: Medieval Era theme (peaceful variant)",
    "Age of Empires IV: village ambiance"
  ],
  "loop_requirements": {
    "seamless": true,
    "loop_start_beats": 0,
    "loop_end_beats": 128
  },
  "output_format": "ogg_vorbis_44100_16bit",
  "target_loudness_lufs": -14.0,
  "peak_limit_dbfs": -1.0
}
```

### 9.3 Procedural SFX Generation

Simple SFX are generated procedurally using `sox` (shell) or Python `pedalboard`:

```bash
# Generate heartbeat sound (80ms sine burst at 60Hz, with decay)
sox -n -r 44100 -b 16 sfx/ui/heartbeat.ogg \
    synth 0.08 sine 60 \
    fade 0 0.08 0.04 \
    norm -14

# Generate discovery chime (ascending arpeggion: C4, E4, G4, C5)
sox -n -r 44100 -b 16 sfx/tech/discovery_chime.ogg \
    synth 0.15 sine 261.63 : \
    synth 0.15 sine 329.63 : \
    synth 0.15 sine 392.00 : \
    synth 0.3 sine 523.25 \
    fade 0 0.75 0.3 \
    norm -14
```

```python
# Python pedalboard: generate market bell
from pedalboard import Reverb, LowpassFilter
from pedalboard.io import AudioFile
import numpy as np

sample_rate = 44100
duration = 0.8
t = np.linspace(0, duration, int(sample_rate * duration))
freq = 880.0
signal = np.sin(2 * np.pi * freq * t) * np.exp(-t * 8)
signal = signal.astype(np.float32).reshape(1, -1)

board = pedalboard.Pedalboard([Reverb(room_size=0.3), LowpassFilter(cutoff_frequency_hz=4000)])
processed = board(signal, sample_rate)

with AudioFile("sfx/economy/market_bell.ogg", "w", sample_rate, 1) as f:
    f.write(processed)
```

### 9.4 Audio Processing Pipeline

All audio assets pass through this processing pipeline before inclusion in the game:

```
Step 1: Source acquisition
  - Download from Freesound.org (CC0 license confirmed)
  - OR: export from DAW (human composer)
  - OR: generate via MusicGen API / Suno API
  - OR: generate procedurally via sox/pedalboard

Step 2: Normalization
  - Target: -14 LUFS integrated (EBU R128)
  - Tool: ffmpeg-normalize or loudnorm filter
  - Command: ffmpeg -i input.wav -af loudnorm=I=-14:TP=-1:LRA=7 output_normalized.wav

Step 3: Peak limiting
  - Target: -1.0 dBFS true peak
  - Prevents clipping during mixing
  - Tool: FFmpeg alimiter or replaygain

Step 4: Loop point detection (for looping assets)
  - Find zero-crossing near intended loop point
  - Embed loop markers in OGG Vorbis LOOPSTART/LOOPEND comments
  - Tool: sox or custom Python script

Step 5: Format conversion to OGG Vorbis
  - Sample rate: 44,100 Hz
  - Bit depth: 16-bit (via Vorbis encoder quality q6)
  - Channels: Mono for spatial SFX; Stereo for music/UI
  - Command: ffmpeg -i input_normalized.wav -c:a libvorbis -q:a 6 -ar 44100 output.ogg

Step 6: Validation
  - Verify loudness: ffmpeg -i output.ogg -af ebur128 -f null -
  - Verify format: ffprobe output.ogg
  - Verify loop points: parse OGG Vorbis comments
  - Add to audio_manifest.json registry

Step 7: Asset registration
  - Add entry to assets/audio/audio_manifest.json
  - Commit asset to VCS (Git LFS for binary files)
```

### 9.5 MusicGen API Integration (Agentic Audio)

For automated music generation, the audio spec IR drives a MusicGen API call:

```python
import requests

def generate_music_from_spec(spec: dict) -> bytes:
    """
    Submit AudioSpec to MusicGen API and return OGG audio bytes.
    Fails loudly if generation fails — no silent fallback.
    """
    prompt = build_musicgen_prompt(spec)
    response = requests.post(
        "https://api.musicgen.example/v1/generate",
        json={
            "prompt": prompt,
            "duration": spec["duration_seconds"],
            "format": "ogg",
        },
        timeout=120,
    )
    response.raise_for_status()  # Fail loud on HTTP error
    return response.content

def build_musicgen_prompt(spec: dict) -> str:
    mood = ", ".join(spec["mood"])
    instruments = ", ".join(spec["instrumentation"])
    refs = "; ".join(spec.get("reference_tracks", []))
    return (
        f"{mood} background music for a civilization strategy game. "
        f"Instrumentation: {instruments}. "
        f"Tempo: {spec['tempo_bpm']} BPM. Key: {spec['key']}. "
        f"Reference: {refs}."
    )
```

---

## Section 10: Accessibility

### 10.1 Visual Indicators for Audio Cues

Every audio cue that communicates important game information MUST have a corresponding visual indicator. Users who play with audio muted (deaf players, noisy environments) must receive equivalent information.

| Audio Cue | Visual Equivalent |
|-----------|------------------|
| War declaration horn | Red alert banner + screen edge flash (2s, red) |
| Battle result sound | Battle report notification badge + victory/defeat icon |
| Disaster SFX | Disaster zone hex highlight + alert in event log |
| Tech discovery chime | Tech notification badge + animated research completion indicator |
| Bankruptcy chord | Economy alert badge + treasury status indicator (red) |
| Heartbeat (stability \< 30%) | Screen edge pulse effect (red, subtle, matches heartbeat rhythm) |
| Insurgency sting | Insurgency alert badge + affected district hex highlight |
| Crisis music state | Crisis indicator overlay (subtle red vignette at screen edges) |

### 10.2 Subtitles for Critical Events

Critical events display text captions in a dedicated caption zone (bottom-left of screen, above UI bar):

```
Event: war.declared.v1
Caption: "[Nation B] has declared war on [Nation A]."
Duration: 5s
Priority: HIGH (interrupts lower-priority captions)

Event: disaster.triggered.v1
Caption: "Disaster: [Disaster Type] strikes [Region Name]. Severity: [X]%"
Duration: 8s
Priority: HIGH

Event: economy.bankruptcy.declared.v1
Caption: "[Nation] has declared economic bankruptcy."
Duration: 6s
Priority: HIGH
```

Captions are:
- Enabled by default (can be disabled in accessibility settings)
- Font size configurable (small / medium / large)
- Contrast ratio >= 4.5:1 (WCAG AA) against background
- Not obscured by other UI elements

### 10.3 Mute Without Visual Impact

Muting audio has zero impact on visual gameplay:
- All audio indicators described in 10.1 are always active regardless of mute state
- Mute button is clearly labeled and accessible via keyboard shortcut (default: `M`)
- Partial mutes (music only, SFX only, ambient only) do not affect other visual indicators

### 10.4 Screen Reader Friendly UI Controls

Audio settings panel controls are ARIA-labeled for screen reader compatibility:

```html
<div role="group" aria-label="Audio Settings">
  <label for="master-vol">Master Volume</label>
  <input
    id="master-vol"
    type="range"
    min="0" max="100" step="1"
    aria-valuenow="100"
    aria-valuemin="0"
    aria-valuemax="100"
  />

  <button
    id="mute-toggle"
    aria-label="Mute all audio"
    aria-pressed="false"
  >
    Mute
  </button>

  <label for="output-device">Audio Output Device</label>
  <select id="output-device" aria-label="Select audio output device">
    <option value="default">Default</option>
    <!-- Dynamically populated -->
  </select>
</div>
```

### 10.5 Reduced Motion Compatibility

The visual indicators for audio cues respect the `prefers-reduced-motion` media query:

```css
@media (prefers-reduced-motion: reduce) {
  .audio-visual-indicator {
    animation: none;
    transition: none;
    opacity: 1; /* Show static indicators without animation */
  }
}
```

For the screen edge flash (war declaration, crisis pulse): replace animated pulse with a solid static tint for the duration of the event.

---

## Appendix A: Full Event → Audio Trigger Reference Table

This table is the canonical source for implementation. Both the Bevy and Web client audio systems MUST match this table exactly.

| Event Type | Sound ID | Variant Logic | Spatial? | Volume Formula | Cooldown (s) | Music State Change |
|------------|----------|---------------|----------|----------------|--------------|-------------------|
| `war.declared.v1` | `diplomacy_war_horn_aggressor` / `diplomacy_war_horn_target` | Player role | No | `1.0 × sfx_vol` | 30 | WAR_WINNING / WAR_LOSING |
| `battle.resolved.v1` | `battle_resolved_{outcome}` | outcome field | Yes | `0.85 × sfx_vol` | 5 | None |
| `citizen.born.v1` | `crowd_cheer_birth` | Batch >= 100 | No | `0.3 + batch/1000 × sfx_vol` | 10 | None |
| `disaster.triggered.v1` | `env_disaster_{type}` | disaster_type | Yes | `(0.5 + severity×0.5) × sfx_vol` | 60 | CRISIS if stability \< 20% |
| `tech.unlocked.v1` | `tech_discovery_chime` | Era variant | No | `sfx_vol` | 10 | None |
| `economy.bankruptcy.declared.v1` | `economy_bankruptcy` | None | No | `sfx_vol` | 30 | None |
| `institution.formed.v1` | `institution_established_fanfare` | None | No | `sfx_vol` | 15 | None |
| `insurgency.started.v1` | `insurgency_tension_sting` | None | Yes | `sfx_vol` | 20 | None |
| `election.held.v1` | `crowd_murmur_election` | Phase: murmur/result | No | `0.7 × sfx_vol` | 30 | None |
| `tick.completed.v1` | `ui_heartbeat` | stability \< 30% only | No | `(30-stab)/30 × 0.4 × sfx_vol` | pulse_ms | None |
| `migration.wave.started.v1` | `migration_crowd_footsteps` | count proportional | Yes | `count/1000 × sfx_vol` | 20 | None |
| `famine.triggered.v1` | `famine_horn` | None | Yes | `sfx_vol` | 60 | None |
| `trade.route.established.v1` | `economy_trade_route_bell` | None | No | `sfx_vol` | 10 | None |
| `alliance.formed.v1` | `diplomacy_alliance_fanfare` | None | No | `sfx_vol` | 30 | None |
| `peace.declared.v1` | `diplomacy_peace_relief` | None | No | `sfx_vol` | 30 | PEACE_* |
| `city.founded.v1` | `city_founded_cheer` | None | Yes | `sfx_vol` | 15 | None |
| `structure.destroyed.v1` | `building_destruction_{size}` | structure size | Yes | `sfx_vol` | 3 | None |
| `revolt.started.v1` | `revolt_alarm_bell` | None | Yes | `sfx_vol` | 30 | None |
| `research.started.v1` | `research_start_chime` | None | No | `0.6 × sfx_vol` | 10 | None |
| `citizen.died.v1` | `citizen_death_bell` | batch >= 100 | No | `0.5 × sfx_vol` | 15 | None |
| `military.unit_moved.v1` | `unit_footstep_{terrain}` | terrain type | Yes | `0.4 × sfx_vol` | 0.5 | None |
| `military.unit_routed.v1` | `unit_morale_routing` | None | Yes | `sfx_vol` | 5 | None |
| `military.siege.started.v1` | `unit_siege_catapult_launch` | None | Yes | `sfx_vol` | 60 | None |
| `diplomacy.treaty_signed.v1` | `diplomacy_treaty_signing` | None | No | `sfx_vol` | 5 | None |
| `climate.carbon_threshold.crossed.v1` | `climate_alarm_tone` | None | No | `sfx_vol` | 120 | None |
| `economy.supply_shock.v1` | `economy_supply_shock` | None | No | `sfx_vol` | 30 | None |
| `policy.applied.v1` | `ui_confirm` | None | No | `0.3 × sfx_vol` | 1 | None |

---

## Appendix B: Audio Asset Directory Structure

```
assets/audio/
  music/
    base/
      peace_prosperous_loop.ogg
      peace_struggling_loop.ogg
      war_loop.ogg
      crisis_loop.ogg
    tension/
      tension_layer.ogg
    war/
      war_layer_drums.ogg
      war_layer_brass.ogg
    triumph/
      triumph_layer.ogg
    crisis/
      crisis_drone.ogg
    stings/
      victory_fanfare.ogg
      defeat_somber.ogg
    metadata/
      music_manifest.json
  sfx/
    ui/
      button_click.ogg
      panel_open.ogg
      panel_close.ogg
      alert_ping.ogg
      alert_urgent.ogg
      zoom_in.ogg
      zoom_out.ogg
      zoom_level_change.ogg
      tooltip_appear.ogg
      menu_open.ogg
      menu_close.ogg
      confirm.ogg
      cancel.ogg
      error.ogg
      notification_badge.ogg
      heartbeat.ogg
    units/
      footstep_grass.ogg
      footstep_stone.ogg
      footstep_sand.ogg
      footstep_snow.ogg
      footstep_mud.ogg
      footstep_water.ogg
      weapon_clash_sword.ogg
      weapon_clash_spear.ogg
      weapon_clash_axe.ogg
      ranged_bow_release.ogg
      ranged_projectile_fly.ogg
      ranged_projectile_hit.ogg
      death_infantry.ogg
      death_cavalry.ogg
      death_siege.ogg
      formation_change.ogg
      morale_high.ogg
      morale_routing.ogg
      siege_catapult_launch.ogg
      siege_ram_impact.ogg
    buildings/
      construction_start.ogg
      construction_complete.ogg
      production_tick.ogg
      production_idle.ogg
      destruction_small.ogg
      destruction_large.ogg
      destruction_wall.ogg
      fire_start.ogg
      fire_loop.ogg
      repair_start.ogg
      repair_complete.ogg
      garrison_enter.ogg
      upgrade_complete.ogg
      market_open.ogg
    environment/
      wind_plains.ogg
      wind_desert.ogg
      wind_mountain.ogg
      rain_light.ogg
      rain_heavy.ogg
      thunder.ogg
      ocean_waves.ogg
      ocean_waves_calm.ogg
      forest_ambiance.ogg
      forest_birds.ogg
      cave_drip.ogg
      volcano_rumble.ogg
      ice_crack.ogg
      river_flow.ogg
      disaster_drought.ogg
      disaster_flood.ogg
      disaster_earthquake.ogg
      disaster_pandemic.ogg
      disaster_wildfire.ogg
    economy/
      market_bell.ogg
      currency_clink.ogg
      currency_large.ogg
      bankruptcy.ogg
      trade_route_bell.ogg
      supply_shock.ogg
      boom.ogg
      recession_tone.ogg
    diplomacy/
      treaty_signing.ogg
      war_declaration_horn_aggressor.ogg
      war_declaration_horn_target.ogg
      alliance_formed.ogg
      peace_declared.ogg
      embargo.ogg
      proposal_sent.ogg
      proposal_accepted.ogg
      proposal_rejected.ogg
    military/
      battle_resolved_victory.ogg
      battle_resolved_defeat.ogg
      battle_resolved_draw.ogg
    population/
      crowd_cheer_birth.ogg
      crowd_murmur_election.ogg
      citizen_death_bell.ogg
      migration_crowd_footsteps.ogg
    tech/
      discovery_chime.ogg
      research_start_chime.ogg
  ambient/
    city_market.ogg
    construction_ambient.ogg
    crowd_base.ogg
    conversation_murmur.ogg
    personal_space_indoor.ogg
    personal_space_outdoor.ogg
    street_noise.ogg
    farmer_ambient.ogg
    forge_ambient.ogg
    scholar_ambient.ogg
    distant_battle_rumble.ogg
    distant_crowd_murmur.ogg
  audio_manifest.json
```

---

## Appendix C: Implementation Checklist

- [ ] Kira `AudioManager` initializes without panic; fails loudly on device failure
- [ ] All critical SFX preloaded at startup; missing assets log error and continue
- [ ] Music layers all start playing (volume=0) at startup; no audio gap on first state transition
- [ ] All 28 simulation events in Appendix A table are handled in `handle_simulation_events`
- [ ] Music state machine transitions trigger correct layer volume changes
- [ ] Stability-driven tension layer updates on every tick that changes stability
- [ ] SFX cooldown tracker prevents spam for all cooldown-specified events
- [ ] Heartbeat plays only when stability \< 30%; pulse rate matches formula
- [ ] Ambient system updates on camera position change; crossfades at correct durations
- [ ] Web client `AudioManager` is functionally equivalent to Bevy implementation
- [ ] Audio settings persist to localStorage (web) and config file (desktop)
- [ ] Output device selection works on desktop (cpal) and web (setSinkId where supported)
- [ ] All visual indicators present for all audio cues (Appendix A table)
- [ ] Captions system shows critical event text with correct priority and duration
- [ ] ARIA labels present on all audio settings UI controls
- [ ] `prefers-reduced-motion` respected for animated visual indicators
- [ ] All OGG assets at 44.1kHz / 16-bit / -14 LUFS / -1dBFS peak
- [ ] Loop points embedded in looping assets
- [ ] Audio manifest JSON validated against all file paths on CI
