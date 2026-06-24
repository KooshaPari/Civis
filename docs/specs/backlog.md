# Civis Spec-Map Backlog — Prioritized Waves

**Owner:** Research & Spec Lead. **Deliverable 4/4.**
**Source:** [`feature-matrix.md`](./feature-matrix.md) gaps + [`requirements/`](./requirements/) FR/NFR.
**Sequencing principle:** highest-leverage **BLIND / unpolished** fills first — the things that make Civis "feel blind, incomplete, unpolished." A capability you cannot *see* (info-views, inspect) blocks evaluating everything else, so the "perception layer" leads. Each wave is a pull-queue domain Leads draw from; dependencies are a DAG (no cycles).

Charter gate on every item: **only Layer-0 laws are authored; everything else emerges.** Tags: [LAW]/[EMERGENT]/[UI/QoL].

## DAG (high level)
```
Wave 0 (Perception)  ──► unblocks evaluation of all sim work
   INFOVIEW, INSPECT, NOTIFY-feed
        │
        ▼
Wave 1 (Sandbox feel) ──► GODTOOL palette, spawn/poke loop, time, undo
        │
        ▼
Wave 2 (Life/Mind)   ──► PSYCHE, social graph, emergent history  (needs INSPECT to read)
        │
        ▼
Wave 3 (City verbs)  ──► ROAD desire-paths, vehicles, architecture, districts
        │
        ▼
Wave 4 (Society/Econ)──► markets, polities, ideology/culture/language overlays
        │
        ▼
Wave 5 (War layers)  ──► strategic/operational/tactical integration + dual-scale camera
        │
        ▼
Wave 6 (Scale/Polish)──► 20mi streaming, 60fps, audio juice, onboarding, modding, save/replay
```
Wave 0 has no sim dependency and is the **highest leverage** — do it first.

## Wave 0 — Perception layer (HIGHEST LEVERAGE — fills the worst BLIND gaps)
| Item | FR | Tag | Owner | Depends on |
|---|---|---|---|---|
| Data-driven overlay registry + toggle system | INFOVIEW-900/901 | UI/QoL | UI | — |
| Environmental + resource overlays (read existing planet/voxel/econ data) | INFOVIEW-910/911 | UI/QoL | UI | overlay registry |
| Inspect-anything (pick → inspector) + tooltips | INSPECT-900/910 | UI/QoL | UI | bevy_picking |
| Agent/voxel/settlement inspector panels | INSPECT-901/902/903 | UI/QoL | UI | inspect core |
| Event/alert feed + camera-jump | NOTIFY-900 | UI/QoL | UI | (chronicle stub ok) |

## Wave 1 — Sandbox "poke the world" feel
| Item | FR | Tag | Owner | Depends on |
|---|---|---|---|---|
| Categorized god-tool palette (data-driven) | GODTOOL-900/901 | UI/QoL | UI | overlay registry |
| Material/terrain brushes on voxel CA | GODTOOL-910 | UI/QoL | Voxel-Render | palette |
| Life/spawn + disaster tools (emergent outcomes) | GODTOOL-911/912 | UI/QoL→[EMERGENT] | Voxel-Render+Life-Sim | palette |
| Time controls + god-hand cursor | GODTOOL-920 | UI/QoL | UI | palette |
| Undo + region blueprint | GODTOOL-921 | UI/QoL | UI | brushes |

## Wave 2 — Life & mind (the soul; currently BLIND)
| Item | FR | Tag | Owner | Depends on |
|---|---|---|---|---|
| Emergent psyche (temperament from DNA, mood) | PSYCHE-900/901 | EMERGENT | Life-Sim | genetics/species (exist) |
| Kinship + contact social graph | PSYCHE-910 | EMERGENT | Life-Sim | psyche |
| Belief/culture drift → clusters | PSYCHE-911 | EMERGENT | Life-Sim | social graph |
| Language emergence/drift/creoles | PSYCHE-912 | EMERGENT | Life-Sim | social graph |
| Emergent chronicle / legends mode | PSYCHE-920 | EMERGENT+UI | Life-Sim+UI | event hooks |
| Society overlays wired to above | INFOVIEW-913 | UI/QoL | UI | clusters/graph |

## Wave 3 — City-builder verbs (BLIND)
| Item | FR | Tag | Owner | Depends on |
|---|---|---|---|---|
| Desire-path road emergence (trail→road→highway) | ROAD-900 | EMERGENT | Infrastructure | agent traversal |
| Emergent architecture (build on need+resource) | ROAD-901/902 | EMERGENT | Infrastructure | building graph (exists) |
| Vehicles/transport on emergent roads | ROAD-910 | EMERGENT | Infrastructure | road graph |
| Manual road tools (curve/snap/upgrade) | ROAD-920 | UI/QoL | Infrastructure+UI | road graph |
| District/zone designation lens | ROAD-921 | UI/QoL | UI | — |
| Infrastructure overlays | INFOVIEW-914 | UI/QoL | UI | road/building graph |

## Wave 4 — Society & economy depth
| Item | FR/matrix | Tag | Owner |
|---|---|---|---|
| Multiple emergent market types | matrix §3 (econ exists, extend) | EMERGENT | Life-Sim+Econ |
| Emergent polities (cluster-overlap, no faction:u32) | matrix §3, CIV-0105 | EMERGENT | Life-Sim |
| Taxation/budget UI + policy levers | matrix §3 | UI/QoL | UI |
| Tech = discovered laws (not a tree) | matrix §3, ADR-006 | EMERGENT+LAW | Laws+Research |

## Wave 5 — Warfare layers + dual-scale camera
| Item | FR/matrix | Tag | Owner |
|---|---|---|---|
| Strategic↔tactical seamless camera (EAW two-layer) | matrix §7, CIV-0101 | UI/QoL | UI |
| Operational maneuver + logistics | matrix §5 | EMERGENT | Tactics |
| Tactical direct-control + RTS command coexist (CtA) | matrix §5, CIV-0300 | EMERGENT+UI | Tactics+UI |

## Wave 6 — Scale, performance & polish
| Item | NFR/FR | Tag | Owner |
|---|---|---|---|
| 20mi streaming working set + disk format | SCALE-900/901/902 | NFR | Voxel-Render |
| LOD-tiered agent sim at scale | SCALE-910, PSYCHE-921 | NFR+EMERGENT | Life-Sim+Perf |
| 60fps + GPU instancing/indirect | PERF-900/901/902 | NFR | Voxel-Render+Perf |
| Determinism across LOD transitions | SCALE-920, NFR-CIV-DET | NFR | Engine |
| Stats dashboards (egui_plot) at empire scale | NOTIFY-910/911 | UI/QoL | UI |
| Onboarding + rebindable hotkeys | NOTIFY-920/921 | UI/QoL | UI |
| Adaptive audio + UI juice | matrix §8, CIV-0800 | UI/QoL | Audio |
| Save/replay determinism + modding API hardening | matrix §8, CIV-0700/1000 | LAW+UI | Engine+Mod |

## Recommended next wave
**Wave 0 (Perception layer)** — start immediately. It is unblocked, fills the #1 and #2 BLIND gaps (info-views, inspect-anything), and is a prerequisite for visually verifying every downstream sim feature (per the vision-verify principle). Concretely, the first three pulls: INFOVIEW-900/901 (registry+toggle), INFOVIEW-910 (environmental overlays over existing `civ-planet`/`civ-voxel` data), INSPECT-900/910 (pick→inspector+tooltips). These reuse data Civis already computes, so they ship fast and immediately reduce the "blind" feeling.
