# Topic → Chunk Map

## Economy & Ledger Mechanics
- chunk_001\.md — Core joule premise: universal energy unit replacing currency, citizen lifecycle, retirement pool
- chunk_001\.md — Item acquisition, commoditization, financialization of energy output
- chunk_001\.md — Government subsidy mechanics, "right to live" thresholds, energy debt
- chunk_003.md — Formal simulation engine design: Engine.step(), Vec&lt;Agent&gt;, staged phases (labor → production → allocation → consumption → update)
- chunk_005\.md — City-scale economy design pattern: households, firms, budget panels, resource flows

## Conservation Laws \& Invariants
- chunk_001\.md — Joule conservation: total system energy accounting, no-creation-from-nothing invariants
- chunk_001\.md — Retirement pool mechanics and cumulative energy output thresholds

## Citizen Lifecycle & Agent State
- chunk_001\.md — Citizen birth, labor entry, productivity curve, retirement, death lifecycle
- chunk_001\.md — Health, stress, and energy deficit as citizen state variables
- chunk_003.md — Agent/Model separation: agent step(), staged activation order, scheduler patterns
- chunk_007.md — Individual-level social dynamics: trust, alienation, rebellion state flows

## Market Clearing Without Price Signals
- chunk_001\.md — Allocation without price signals: joule-rationed distribution, scarcity queuing
- chunk_005\.md — Indirect controls as policy levers (taxes, zoning, subsidies) rather than direct money injection

## Policy DSL & Governance Levers
- chunk_001\.md — Policy effects on citizen happiness, energy allocation, social group reactions
- chunk_005\.md — Policy abstraction: overlay-first UI, heatmaps for scarcity / unemployment / rent / tyranny
- chunk_006.md — Democracy series-style multi-faction policy preference modeling
- chunk_007.md — Multi-scale governance layers: global → city → individual

## ABM Framework Comparisons
- chunk_002.md — Repast, NetLogo, Mesa, Swarm, AgentPy, Agents.jl — OSS ABM framework survey
- chunk_002.md — Policy Simulation Library (PSLmodels), OmniEcon Nexus, Casevo, BESSIE — research simulators
- chunk_003.md — Mesa deep dive: Model.step(), DataCollector pattern, web visualization pipeline
- chunk_003.md — Mapping Mesa patterns into CivLab Rust engine architecture

## Simulation Architecture
- chunk_003.md — CivLab Rust engine design: Engine.step(), metrics module, staged phase pipeline
- chunk_003.md — DataCollector clone pattern: every tick emits structured metrics for dashboard
- chunk_003.md — Sim → stream state → render pipeline (web visualization concept)
- chunk_005\.md — LOD guidance: big picture sector aggregates vs zoomed-in sampled agents per district
- chunk_007.md — Hidden network layer: influence graph of actors, nodes/edges, patronage relationships

## Shadow/Deep State & Hidden Power Networks
- chunk_006.md — Republic: The Revolution — faction and hidden ideological power within districts
- chunk_007.md — Hidden Network Layer design pattern: influence graph, node ideology vectors, edge relationships
- chunk_007.md — Shadow government mechanics: bribery, intimidation, coercive pressure, influence gain/loss

## Social Systems & Health Dynamics
- chunk_006.md — Compartmental state flows (healthy/infected/recovered) mapped to civic trust/alienation/rebellion
- chunk_006.md — Plague Inc. / Rebel Inc. — stability as function of civilian satisfaction vs insurgent pressure
- chunk_007.md — Deep world mechanics: social systems, health dynamics, complex ideologies, multi-scale governance

## Ideology & Faction Mechanics
- chunk_001\.md — Social group ideological drift under joule scarcity and surplus
- chunk_006.md — Democracy 3/4 multi-voter faction preference aggregation — conservatives, socialists, liberals
- chunk_006.md — Global cooperation vs competition as strategic overlays; resource diplomacy tied to legitimacy
- chunk_007.md — Ideology vector modeling for actors in hidden network layer

## Gamification & UI Patterns
- chunk_002.md — OSS games with economic/city/societal simulation elements: Lincity and others
- chunk_004.md — What to steal from Cities: Skylines — overlay-first UI, causality via bottlenecks, indirect controls
- chunk_004.md — What to steal from WorldBox — faction identity, state machines, macro chaos with legible primitives
- chunk_005\.md — Design Patterns Matrix: Cities Skylines / WorldBox / Diplomacy Is Not an Option / Civ 7
- chunk_005\.md — LOD design: big-picture sector aggregates vs zoomed-in district drilldowns

## War, Diplomacy & Geopolitics
- chunk_004.md — WorldBox diplomacy: coarse faction stance states (peace, war, alliance, sanction)
- chunk_006.md — Global Change Game / World Game — competing regional goals, cooperation vs conflict
- chunk_007.md — Shadow state effects on election outcomes and foreign policy

## Multi-Scale Governance
- chunk_006.md — Three LOD scales: global/civilization → city/municipal → individual
- chunk_006.md — Sub-governance dynamics at city level; hearts-and-minds stability loops
- chunk_007.md — Layered simulation design integrating all scale levels causally

## chunk_007 Sub-Parts (Deep World Mechanics — 11k lines)
See chunk_007_parts/INDEX.md for fine-grained navigation within this chunk.
