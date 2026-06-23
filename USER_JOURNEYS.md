# CivLab User Journeys

**Version:** 1.0
**Date:** 2026-02-21
**Status:** APPROVED
**Scope:** Headless Rust civilization simulation engine for researchers, game developers, and players

---

## Overview

This document defines five archetypal user journeys through CivLab, spanning research workflows, game integration, emergent gameplay discovery, RTS command execution, and policy experimentation. Each journey captures actor goals, system interactions, expected outcomes, and failure modes.

---

## UJ-1: Researcher Launching Scenario, Tweaking Parameters, Exporting Replay

**Persona:** Economic researcher, policy analyst, or computational social scientist
**Primary Goal:** Configure and run a civ-sim scenario with controlled parameters, capture complete replay data for offline analysis

### Actor Profile
- Educational or research background in economics, political science, or sociology
- Comfortable with JSON/YAML configuration and command-line tooling
- Uses Python/R/Jupyter for post-processing analysis
- Needs bit-reproducible results and audit trails

### Journey Steps

#### Phase 1: Scenario Discovery & Configuration

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 1.1 | Researcher clones CivLab and reads `examples/scenarios/README.md` | CLI displays: list of 5 canonical scenarios (temperate city, island empire, climate crisis, economic collapse, joule-first utopia) | Scenarios exist; descriptions match research questions |
| 1.2 | Selects scenario: `civ-sim scenario apply --scenario temperate-city` | CLI loads `scenarios/temperate-city/spec.yaml` and displays: initial state summary (population, resources, institutions) | YAML parses; state matches schema validation |
| 1.3 | Inspects base spec with `civ-sim scenario inspect --scenario temperate-city --format json` | Outputs full serialized spec: geography (district count, climate zone), initial policies, actor roster | JSON schema validates; all policies reference valid institutions |
| 1.4 | Opens `temperate-city/spec.yaml` locally, edits: population growth rate +10%, food production cap +20% | File edits applied to local copy | YAML lint passes; constraints within documented bounds (0.5x–2.0x) |
| 1.5 | Adds policy experiment: joule economy baseline allocation rate to 50 joules/tick (from 40) | Modified spec saved to `temperate-city-variant.yaml` | Schema validates; policy parameters reference valid JouleAllocator config |

#### Phase 2: Run Configuration & Execution

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 2.1 | Creates run config: `civ-sim run create --spec temperate-city-variant.yaml --ticks 10000 --seed 42 --output replay.civdata` | CLI returns: `run_id: exp-001-temperate-variant` and displays estimated compute time (45 sec on CPU) | Run ID is unique; seed is recorded in metadata; output path is writable |
| 2.2 | Executes run: `civ-sim run exec --run-id exp-001-temperate-variant --progress --log-level info` | Progress bar shows tick count (0–10000), wall-clock time elapsed, estimated time remaining. Logs: each policy phase completed, economic market clearings per 100 ticks | Ticks advance at ~200 ticks/sec; no hang > 5 sec |
| 2.3 | (Interrupts run at tick 5000 and resumes) Queries run state: `civ-sim run checkpoint --run-id exp-001-temperate-variant --tick 5000 --export state.json` | Exports state snapshot (population cohort ledgers, institution states, region resource stocks) | Checkpoint file validates against state schema; resume works from same tick |
| 2.4 | Completes full 10k tick run | System writes: `replay.civdata` (binary deterministic trace), `metrics.jsonl` (one JSON object per tick) | File sizes: replay ~50 MB, metrics ~200 MB (tunable). Both files byte-stable on replay |

#### Phase 3: Replay Export & Analysis

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 3.1 | Lists available exports: `civ-sim run export --run-id exp-001-temperate-variant --format list` | CLI displays: `replay`, `metrics`, `ledger-trace`, `institution-timeline`, `actor-genealogy` | Exports match spec domains |
| 3.2 | Exports full ledger trace: `civ-sim run export --run-id exp-001-temperate-variant --format ledger-trace --output ledger.csv` | Generates CSV: `tick,from_actor_id,to_actor_id,amount,currency,transfer_type,policy_phase` (1M+ rows for 10k ticks) | CSV validates; all actor IDs reference actor genealogy; amounts sum to zero per policy phase |
| 3.3 | Exports institution timeline: `civ-sim run export --run-id exp-001-temperate-variant --format institution-timeline --output institutions.parquet` | Writes Apache Parquet: `tick, institution_id, state, legitimacy, capture_score` | Parquet reads in pandas; legitimacy values in [0, 1]; state matches FSM enum |
| 3.4 | Loads replay in Jupyter: `import civlab; replay = civlab.Replay.load('replay.civdata')` | Returns `Replay` object with methods: `.tick(n)` (jump to tick n), `.metrics(metric_name)` (time series), `.ledger_summary()` | `.tick(0)` matches exported state snapshot; time series monotonic where expected |
| 3.5 | Performs post-hoc analysis: plots population vs welfare inequality (Gini) over time | Jupyter cell outputs matplotlib figure: Gini spike at tick ~3500 (food shortage event) | Spike correlates with logged `economy.supply_shock.v1` event; Gini ≤ 1.0 |

### Success State
- Complete replay file saved and bit-reproducible (same seed = identical metrics)
- Ledger trace balances (sum of transfers = 0)
- Institution timelines show expected state transitions
- Analysis reveals meaningful causal relationships (e.g., food shock → legitimacy drop → dissent spike)

### Failure Modes & Recovery

| Failure | Symptom | Recovery |
|---------|---------|----------|
| OOM during tick loop | Process killed after 5 sec, no checkpoint | Resume from tick 5000 checkpoint with `--resume` flag |
| Determinism violation | Tick N replay differs from first run | Check for: (a) floating-point accumulation (use fixed-point), (b) unsorted transfer list, (c) RNG seed propagation |
| Ledger imbalance | `ledger.csv` transfers don't sum to zero | Query specific policy phase logs; rerun with `--audit-ledger` flag for detailed tracing |
| Missing institution state | Timeline has gaps or state = NULL | Check institution lifecycle events; may indicate institution collapse/reform not logged |

### Acceptance Criteria
- [ ] Scenario spec loads and validates (YAML schema + value constraints)
- [ ] Run completes 10k ticks in < 2 minutes (CPU)
- [ ] Replay file is deterministic (bit-reproducible with same seed)
- [ ] All three exports (ledger, institution, genealogy) have zero validation errors
- [ ] Post-hoc analysis in Jupyter produces plots without errors
- [ ] Researcher can identify at least one causal event (supply shock → legitimacy) in metrics timeline

---

## UJ-2: Game Developer Integrating CivLab Headless Core into Godot Project

**Persona:** Indie/AA game developer, Godot engine specialist
**Primary Goal:** Embed CivLab headless core in a Godot client, send commands, receive real-time state updates over WebSocket JSON-RPC

### Actor Profile
- Experienced with Godot 4.x scripting (GDScript)
- Building strategy/simulation game with custom UI
- Needs low-latency command response and streaming updates
- Prefers library bindings over subprocess invocation

### Journey Steps

#### Phase 1: Integration Planning & Environment Setup

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 1.1 | Reads integration guide: `/docs/guides/game-integration-guide.md` | Guide shows: two integration patterns: (A) subprocess + stdio, (B) embedded library + event loop | Patterns match game architecture needs |
| 1.2 | Checks language bindings availability for Godot | Support matrix shows: Rust FFI available via `civlab-ffi` crate; Go and Python bindings deferred to P2 | FFI crate has documented C ABI contract |
| 1.3 | Clones CivLab repo and reviews FFI contract: `crates/ffi/src/lib.rs` | FFI exposes: `civ_sim_new()`, `civ_sim_step()`, `civ_sim_get_state()`, `civ_sim_send_command()` | FFI interface is stable (v1.0); no planned breaking changes in P0 |
| 1.4 | Sets up Godot project structure with CivLab dependency | Creates `addons/civlab/` with FFI bindings and runs `godot --headless --script generate_ffi_bindings.gd` | Bindings auto-generated; compilation successful |

#### Phase 2: Scenario & Protocol Configuration

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 2.1 | Selects scenario for game: `scenarios/island-empire.yaml` | Scenario defines 4 factions, 12 districts, starting policies | Scenario loads without validation errors |
| 2.2 | Configures client protocol: specifies zoom level 1 (strategic) and command vocabulary | Settings YAML lists: allowed commands (move army, construct building, diplomacy) and data refresh rate (100 ms) | All commands map to valid RTS command enums; refresh rate in [50, 500] ms |
| 2.3 | Spawns headless server in background: `civ-sim serve --spec island-empire.yaml --port 9876 --protocol websocket-jsonrpc` | Server listens on `ws://localhost:9876/` and logs: "Server listening, clients can connect" | Port is available; protocol version logged |
| 2.4 | Godot client connects: `var client = civlab.WebSocketClient.new("ws://localhost:9876/")` | Client handshakes with server; receives `server_ready` message with: scenario metadata, zoom level 1 data contract version | Handshake succeeds; server returns scenario as JSON object |

#### Phase 3: Real-Time Gameplay Loop

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 3.1 | Game advances tick: `civ_sim_step()` called by Godot per frame (60 FPS) | Server advances simulation by 1 tick and broadcasts state delta: population change, resource deltas, new events | State delta is < 1 KB; broadcast latency < 16 ms (60 FPS frame budget) |
| 3.2 | Player issues command: "Move Army #5 from Region A to Region B" via Godot UI | Godot client calls `civ_sim_send_command({"type": "unit_move", "unit_id": 5, "target_region": "B"})` | Server enqueues command in policy phase queue; returns ACK with `command_id` |
| 3.3 | Command executes in policy phase (tick N+2) | Server emits event: `military.unit_moved.v1` with source/dest regions, army composition, logistics cost | Event arrives in client within 100 ms; Godot animates army movement on map |
| 3.4 | Player queries strategic view state: "What's the military balance in Region B?" | Godot calls `civ_sim_get_state({"filter": "region:B:military"})` | Server returns: army rosters for all factions in region B, comparative strength indices | Data loads in < 50 ms; Godot displays as UI overlay |
| 3.5 | Player pauses game and inspects ledger for Region B | Godot calls `civ_sim_export_slice({"tick": current_tick, "region": "B", "format": "ledger"})` | Server exports ledger slice as JSON; shows: intra-region transfers, external trade flows | JSON loads; player can drill down to individual actor transfers |

#### Phase 4: Extended Scenario (Multi-Hour Play Session)

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 4.1 | Player plays for 2 hours (1000 game ticks at 10 ticks/sec wall-clock) | Server maintains stable memory footprint (~200 MB) and tick rate | No memory leaks detected; tick rate stable ± 5% |
| 4.2 | A major event occurs: institution collapse in Region C | Server emits: `institution.state_changed.v1` with state=COLLAPSED, delegated command authority to region military | Event propagates to Godot client; UI highlights region and shows "Governance Crisis" alert |
| 4.3 | Player saves game state: Godot calls `civ_sim_checkpoint({"format": "game-save"})` | Server exports: full simulation state, ledger history, institution timelines as binary blob | Save file is ~500 MB; loads back in < 5 seconds |
| 4.4 | Player resumes from save | Server loads checkpoint and advances from saved tick | Resume tick count matches save point; next command executes as expected |

### Success State
- Godot client renders strategic map with real-time army/resource updates
- Player commands execute with < 100 ms latency
- Multi-hour play session shows no memory leaks or tick rate degradation
- Save/load cycle preserves complete game state

### Failure Modes & Recovery

| Failure | Symptom | Recovery |
|---------|---------|----------|
| Command validation fails | Server rejects `unit_move` as invalid (unit ID doesn't exist) | Godot client catches error event and disables UI button; logs to console |
| Network lag spikes | WebSocket message arrives > 1 sec late | Client has local prediction model (next section); UI shows ghost position, snaps to correct position on update |
| Server crashes | Connection drops mid-tick | Godot detects disconnect and pauses game; offers: (a) resume from last checkpoint, (b) reconnect if server restarts |
| Zoom level mismatch | Client requests zoom level 3 (micro) but server configured for zoom 1 (macro) | Server returns error event; client downgrades to zoom 1 and re-syncs state |

### Acceptance Criteria
- [ ] Godot client connects to headless server and receives handshake
- [ ] Strategic state (unit positions, resource counts) updates every 100 ms
- [ ] Player command (`unit_move`) executes successfully and broadcasts event
- [ ] Multi-hour play session runs without memory leak (memory growth < 1 MB/hour)
- [ ] Save/load cycle preserves simulation state bit-identically
- [ ] Protocol latency < 100 ms (p95) over WiFi

---

## UJ-3: Player Experiencing Emergent City Collapse and Understanding Why

**Persona:** Casual/hardcore strategy gamer
**Primary Goal:** Play an engaging city-sim, observe an unexpected event (collapse), and understand the causal chain through in-game analytics

### Actor Profile
- Plays strategy games for engagement and surprise (Crusader Kings emergent stories)
- Wants to understand "why" without reading code
- Appreciates AI-driven emergent failures as core gameplay
- Expects clear causality visualization

### Journey Steps

#### Phase 1: Gameplay & Emerging Crisis

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 1.1 | Player starts new city in temperate region, builds initial infrastructure | Game shows: population grows from 100 to 500 (ticks 0–500); food production stable at 150/tick; citizen welfare high | Welfare metric displayed as green bar; growth curve smooth |
| 1.2 | Player constructs a new district (advanced manufacturing) | Game deducts resources: wood 50, stone 30, labor 20 from reserves | District appears on map; labor cost reflected in reduced welfare briefly |
| 1.3 | Natural event triggers: drought reduces food production by 40% | Game displays alert: "DROUGHT: Food production -40% for 200 ticks" | Alert shown with clock icon; affected resource production visibly drops |
| 1.4 | Food shortage propagates: citizens begin to migrate or dissent | Game shows: population migration out of city (100 citizens per tick for 50 ticks); welfare bar turns orange | Animation shows citizens leaving; legitimacy metric begins to drop |
| 1.5 | Crisis escalates: institution (Council) becomes "contested" | Game event: "CONTESTED COUNCIL: Factions compete for legitimacy. Policy effectiveness -30%" | Contested state shown in institution panel; policy actions have reduced impact |
| 1.6 | Player attempts to mitigate: reduces taxes to boost welfare | Policy change issued; one tick later, legitimacy rises +5 but fiscal budget becomes constrained | Fiscal tension visible: player now cannot afford to build/maintain infrastructure |
| 1.7 | Fiscal constraint triggers default: city cannot pay debts to external actors | Game displays: "DEFAULT: Outstanding debt unpaid. Trade embargo from 2 neighbors. Credit cost +50%" | Trade flows visibly disconnect on map; red lines show severed connections |
| 1.8 | Cascading failure: institution collapses due to legitimacy < 10 | Game alert: "COUNCIL COLLAPSED. Authority transferred to military. Unemployment +20%, Unrest +50%" | Institution panel shows COLLAPSED state; military icon takes over governance; welfare bar flashes red |
| 1.9 | Final failure: population exodus peaks (200 citizens/tick leaving) | Game shows: population falls from 300 → 50 over 30 ticks. City is de facto abandoned. | Population graph shows sharp cliff; remaining citizens are labeled "hardened" |

#### Phase 2: Post-Mortem Analytics (In-Game UI)

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 2.1 | Player opens "Replay Browser" in pause menu | UI shows: slider to scrub through entire game (0–tick 5000), timeline with event markers | Slider works smoothly; events (drought, migration, default, collapse) marked with color-coded icons |
| 2.2 | Player scrubs to tick 500 (drought trigger) and pauses | Game rewinds; UI shows exact state at tick 500: food production 150 → 90, welfare 0.8 → 0.6 | State matches tick 500 metrics export; welfare matches ledger balance |
| 2.3 | Player opens "Causal Analysis" panel and filters by: "food crisis" | Panel displays: causal chain graph showing drought → supply_shock → migration_decision → legitimacy_drop → institution_contest → default → collapse | Graph nodes are clickable; edges labeled with time lag (e.g., "drought → migration: 15 ticks") |
| 2.4 | Player clicks on "migration_decision" node | Panel expands showing: 50 citizen-level decisions triggered by welfare < 0.4. Each citizen row shows: name, job, final_location, decision_time_tick | Player can expand individual citizen to see decision logic: "Welfare 0.35 < threshold 0.4 → migrate" |
| 2.5 | Player clicks on "default" node | Panel shows: debt_owed=500, revenue_available=350, tick_when_overdue=3200, policy_response=none (user did not build new revenue sources) | Player sees: the fiscal constraint was predictable; no policy action taken |
| 2.6 | Player opens "Economic Ledger" for the 30 ticks preceding collapse | Displays: table of all transfers (tick, from, to, amount, reason) | Player traces: policy cuts save 20 units/tick but still insufficient to prevent budget imbalance by tick 3200 |

#### Phase 3: Learning & Replay

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 3.1 | Player clicks "Export Replay for Analysis" | Game saves: `collapse_replay.civdata` and offers to open in analytics mode | File saved; analytics mode opens in new window |
| 3.2 | Player explores causal DAG in analytics mode | UI shows directed acyclic graph: nodes are state variables/events, edges are causal dependencies with magnitude (how much does X affect Y) | Nodes colored by impact (red = high impact on collapse, blue = low). Drought node is bright red. |
| 3.3 | Player hovers over edge "drought → migration" | Tooltip shows: "When food supply drops below 100/tick, ~2% of citizens migrate per tick. 50 migration events in this game." | Magnitude matches observed 50 citizens migrating |
| 3.4 | Player explores "what if" scenario: re-runs with drought -20% instead of -40% | Analytics UI offers: "Simulate from tick 500 with drought = -20%". Player accepts. | New branch created; shows alternate timeline where city survives (population = 280 at tick 5000). Original timeline shown in parallel. |
| 3.5 | Player compares two timelines | Side-by-side view: original (collapse at tick 3500) vs alternate (stable at tick 5000) | Graphs overlay; player sees: -20% drought avoids crisis threshold |

### Success State
- Player observes an emergent failure and understands the causal chain
- Post-mortem analytics reveals: drought → migration → legitimacy → default → collapse (5-step chain, 3000 tick span)
- Player recognizes policy intervention opportunity (didn't take it) and learns for next playthrough
- "What-if" exploration shows: -20% less drought would have prevented collapse

### Failure Modes & Recovery

| Failure | Symptom | Recovery |
|---------|---------|----------|
| Causal chain unclear | Collapse happens but analytics show no clear reason | Check event log for gaps; may indicate missing event emission or causal edge not recorded |
| Ledger doesn't match observed behavior | Analytics show migration but player doesn't see population drop in timeline | Validate: population metrics should reflect migration events; check for off-by-one tick errors |
| Replay browser crashes at high tick counts | Scrubbing to tick 4000+ causes UI lag or freeze | Optimize: load only local window of events; stream ledger incrementally |
| "What-if" branch doesn't match expected outcome | Re-simulation with drought=-20% still shows collapse | Check: RNG seed propagation; determinism violated if branch differs from re-run |

### Acceptance Criteria
- [ ] Game displays clear population/welfare/legitimacy graphs over time
- [ ] Event alert shown when major state transition occurs (drought, migration, institution state change, default, collapse)
- [ ] Causal analysis panel shows 5+ node causal chain with edges labeled by time lag
- [ ] Ledger export matches timeline metrics (migrations = population delta)
- [ ] "What-if" replay branch produces measurably different outcome based on scenario tweak
- [ ] Player can articulate the causal chain by reading analytics UI (no code reading required)

---

## UJ-4: RTS Player Issuing Military Commands Against AI-Governed Faction

**Persona:** Real-time strategy enthusiast (StarCraft, Company of Heroes player)
**Primary Goal:** Command armies in real-time, engage in tactics against an AI-controlled faction, experience responsive UI

### Actor Profile
- Expects < 100 ms command latency and fluid unit movement
- Wants AI opponents to behave logically (not stupid)
- Values emergent squad tactics (AI formations, coordinated attacks)
- Familiar with conventional RTS UI (right-click move, drag-select, hot keys)

### Journey Steps

#### Phase 1: Scenario Setup & Unit Control

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 1.1 | Player selects RTS scenario: "Island Skirmish" (2 factions, 40 game ticks per second wall-clock) | Game loads: player controls red faction (500 soldiers, 2 cities); AI controls blue faction (400 soldiers, 2 cities) | Unit counts match; both factions have stable initial policies |
| 1.2 | Player builds a Scout unit in their city | Game deducts: 50 wood, 30 metal, 5 worker-ticks from city reserves. Scout unit spawned on map. | Deduction visible in city resource panel; unit appears within 1 tick |
| 1.3 | Player right-clicks to move scout toward enemy city | Client sends: `{"type": "unit_move", "unit_id": 1, "target": {"x": 150, "y": 200}}` | Server ACKs within 10 ms; client shows green movement indicator; unit begins movement animation |
| 1.4 | Scout reaches enemy city and reveals enemy unit positions | Scout has 2-hex vision radius; enemy units pop into fog-of-war as scout approaches | Vision range matches documented scout spec (2 hex); enemy positions accurate |
| 1.5 | Player issues complex command: select 30 soldiers, hold formation, move to point (200, 300) | Client sends: `{"type": "unit_group_move", "unit_ids": [2..31], "formation": "wedge", "target": {"x": 200, "y": 300}}` | Server validates: all 30 units exist and player owns them. Enqueues formation command. | Command ACKs in < 50 ms; units begin moving in wedge formation |

#### Phase 2: Tactical Engagement & AI Response

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 2.1 | Player's army (30 soldiers + 2 knights) approaches enemy position | Player army reaches distance 3 hexes from enemy camp (5 soldiers) | Enemy presence confirmed by player scouts |
| 2.2 | Enemy AI (autonomous faction policy) detects threat and issues counter-response | Server: AI faction evaluates military balance, decides to defend. Spawns 10 soldiers from barracks, moves them to intercept player army. | AI policy response takes 1 tick; units move per military engagement rules |
| 2.3 | Player anticipates AI response and splits army: 15 soldiers + 2 knights attack head-on; 15 soldiers flank left | Player issues two commands: (a) `unit_group_attack(ids=[2..16], target_unit_ids=[AI_soldiers])`, (b) `unit_group_move(ids=[17..31], target=(190, 250))` | Both commands ACK within 50 ms. Client predicts unit positions locally; server authorizes attack outcome. |
| 2.4 | Combat resolves: combat phase applies armor, HP, morale damage per unit pair-wise | Server combat simulator: 2 knights vs 3 AI soldiers → 2 AI soldiers die, 1 knight takes 20 HP damage; 15 player soldiers vs 5 AI soldiers → 3 AI die, 2 player die (1 survivor wounded). | Combat animations show unit deaths and damage; casualties match server ledger. |
| 2.5 | Flanking army engages from left; AI soldiers on right are surprised (morale -30%) | Flanking attack resolves: 4 AI soldiers killed, player loses 1. Remaining AI soldiers route (morale too low). | Enemy routing visible on map; player soldiers pursue. |
| 2.6 | Player's main army has 20 soldiers remaining; AI army is routed and fleeing toward AI city | Player has achieved local victory; AI begins repair/recovery phase. | AI issues: retreat command to rally remaining soldiers; barracks resume spawn training; policy shifts to "defensive" posture |

#### Phase 3: Supply Lines & Logistics

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 3.1 | Player's army is now 5 hexes from own city, deep in enemy territory | Army supply status: food 150 ticks, ammunition 200 ticks (sufficient) | Supply bar shows % remaining; estimated time before resupply needed displayed |
| 3.2 | Player builds a forward supply depot 2 hexes from main army | Game deducts: 100 wood, 50 metal from player city. Depot placed on map and begins construction (takes 20 ticks). | Depot structure appears; progress bar shows 20/20 ticks to completion. Resource deduction visible in city panel. |
| 3.3 | Player routes army to supply depot when food < 50 ticks remaining | Army moves to depot, resupply occurs automatically. Food ticks jump from 40 → 150. | Animation shows "resupply" event; food meter refills. |
| 3.4 | AI faction raids supply depot with 8 soldiers | Server: AI combat policy detects undefended structure and targets it. Combat resolves: 8 AI soldiers attack depot (which has no defense). Depot takes 80 damage (100 HP total, now 20 HP remaining). | Depot visibly damaged; threat alert shown to player. |
| 3.5 | Player diverts 5 soldiers to defend depot and repair it | Repair command: `{"type": "structure_repair", "structure_id": 42, "unit_ids": [35..39]}` | Repair begins (5 HP/tick per soldier). Soldiers garrison depot. |

#### Phase 4: Extended Engagement (20 game minutes = 24000 ticks)

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 4.1 | Game progresses; player wages defensive war against AI | Player army losses: 50 soldiers over 20 minutes. AI losses: 80 soldiers. Player city grows to 550 soldiers via training; AI city shrinks to 320. | Unit count updates visible; both factions' military graphs shown in top-right UI |
| 4.2 | Player achieves territory dominance: controls 60% of map | Player has 2 AI cities under occupation + 1 forward base. AI has 1 city remaining (low morale, population declining). | Map control % visible in victory condition panel |
| 4.3 | Player presses final assault on AI capital | Siege begins; AI makes final stand with 50 soldiers + barracks-spawned reinforcements | Siege UI shown; garrison strength vs attacking force compared |
| 4.4 | AI surrenders after defending for 100 ticks | Server: AI faction legitimacy < 5%, cohesion < 2. Surrender event emitted: `military.faction_surrender.v1`. Player awarded: 200 resources, 50 prestige. | AI units stop fighting; white flag shown above AI city. Victory screen displays. |

### Success State
- Player commands 30+ units with < 50 ms latency per command
- Tactical decisions (flank vs. head-on) produce measurably different outcomes
- AI responds intelligently to player threats (builds defenses, counters attacks)
- Supply line management adds strategic depth
- 20-minute engagement with no lag spikes or desync

### Failure Modes & Recovery

| Failure | Symptom | Recovery |
|---------|---------|----------|
| Command lag spike | Player clicks move; unit moves 1 sec later instead of 50 ms | Client has local prediction model; unit moves locally while waiting for server ACK; snaps to correct position when server confirms |
| Unit desynced (client shows different position than server) | Visual mismatch between client animation and server authoritative position | Server periodically broadcasts full unit state; client hard-syncs if delta > 1 hex from expected position |
| AI doesn't respond to threat | Player army approaches but enemy AI doesn't defend | Check: AI faction policy evaluation is triggered per tick; military threat assessment may need tuning (sensitivity threshold) |
| Supply line attack logic broken | Depot raided but no damage registered | Verify: structure HP model is enforced per tick; combat damage is applied to structure and logged in ledger |

### Acceptance Criteria
- [ ] Player command latency < 50 ms (p95) from click to unit start moving
- [ ] Tactical formations (wedge, line) execute correctly
- [ ] AI faction spawns counter-units within 2 ticks of threat detection
- [ ] Combat outcomes are deterministic (same state + RNG seed = same casualties)
- [ ] Supply depot can be built, attacked, and repaired
- [ ] 20-minute skirmish runs with zero desync or lag spikes (< 16 ms frame time)

---

## UJ-5: Policy Experimenter Testing Joule Economy vs Market Economy on Identical Scenarios

**Persona:** Economic modeler, complexity researcher
**Primary Goal:** Run the same scenario with two different allocators (joule economy vs market), compare outcomes, publish findings

### Actor Profile
- Academic or think-tank researcher
- Designs computational experiments for causal analysis
- Uses A/B testing and statistical methods
- Needs deterministic, reproducible runs with full parameter exposure

### Journey Steps

#### Phase 1: Scenario Preparation & Parameter Specification

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 1.1 | Researcher selects baseline scenario: "temperate-city.yaml" | Scenario loads; researcher notes: 500 initial population, 10 districts, mixed policy initial state | Scenario spec validates; parameters within documented ranges |
| 1.2 | Creates experiment plan document: "Joule vs Market: Resource Allocation Efficiency" | Outlines: hypothesis, methods, metrics, baseline and variant configurations | Document cites CIV-0107 (Joule Economy spec) and CIV-0100 (Market model) |
| 1.3 | Configures variant A (market economy): opens scenario spec, sets `economy.allocator = "market"` | Config specifies: price discovery method = "clearinghouse", bid/ask matching = "FIFO", inflation target = 0.02 | YAML validates; allocator is recognized; inflation target within [0, 0.1] |
| 1.4 | Configures variant B (joule economy): duplicates spec, sets `economy.allocator = "joule"` | Config specifies: joule accumulation rate = 40 joules/tick, baseline allocation = 50 joules/tick, retirement threshold = 5000 joules | YAML validates; joule parameters within bounds; retirement threshold is achievable in 200+ ticks |
| 1.5 | Specifies identical random seed for both: `seed = 12345` | Both configs include: `seed: 12345`, `initial_population_seed: 12345`, `climate_event_seed: 12345` | Researcher documents: "Same seed ensures identical exogenous shocks (climate, births/deaths)" |

#### Phase 2: Parallel Experiment Execution

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 2.1 | Launches both runs in parallel with script: `civ-sim run batch --configs market-variant.yaml joule-variant.yaml --ticks 5000 --parallel 2` | CLI spawns two independent processes (CPU-isolated). Reports: `run_id_A: exp-market-001`, `run_id_B: exp-joule-001`. Progress shown for both. | Both runs start; CPU load shows 2 cores busy; memory footprint < 1 GB total |
| 2.2 | Market run completes at tick 5000 | Server writes: `exp-market-001/metrics.jsonl`, `exp-market-001/replay.civdata`, `exp-market-001/ledger.csv` | All three files written within 1 sec of tick 5000 |
| 2.3 | Joule run completes at tick 5000 | Server writes: `exp-joule-001/metrics.jsonl`, `exp-joule-001/replay.civdata`, `exp-joule-001/ledger.csv` | All three files written within 1 sec of tick 5000 |
| 2.4 | Researcher verifies both runs started from identical state (tick 0) | Queries: `civ-sim run inspect --run-id exp-market-001 --tick 0 --export state_A.json` and similar for joule | Both JSON states are byte-identical; population, policies, initial resources match |

#### Phase 3: Comparative Analysis

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 3.1 | Loads both replays in Python: `import civlab; market = civlab.Replay.load('exp-market-001/replay.civdata'); joule = civlab.Replay.load('exp-joule-001/replay.civdata')` | Both replays load successfully; researcher can query `.metrics('wealth_gini')`, `.ledger_summary()`, etc. | Replay objects have identical interface regardless of allocator |
| 3.2 | Extracts primary outcome metric: Gini coefficient (wealth inequality) over time | Computes: market_gini = market.metrics('wealth_gini'), joule_gini = joule.metrics('wealth_gini') | Time series returned as numpy arrays; both have length 5000 |
| 3.3 | Plots comparative Gini trajectories | Matplotlib figure shows: market Gini rises monotonically from 0.3 → 0.65 (inequality grows); joule Gini stays flat at 0.25–0.35 (controlled) | Visual shows dramatic difference; market inequality significantly higher |
| 3.4 | Computes secondary metrics for both runs | Extracts: total production (food + metal), citizen welfare distribution, institution stability, GDP per capita, transfer volume | Market: production +15% higher than joule; welfare distribution more skewed; transfers 2x higher volume |
| 3.5 | Exports ledger summaries and performs statistical test | T-test on Gini coefficient time series: market > joule, p < 0.001. Effect size (Cohen's d) = 2.1 (very large) | Statistical output confirms: allocator choice has significant causal effect on inequality |
| 3.6 | Analyzes mechanism: which transfers differ between allocators | Queries: market ledger for price-based transfers; joule ledger for quota-based allocations | Market shows: voluntary exchange transfers (buyer willing to pay > production cost); joule shows: administrative allocations at fixed energy cost |
| 3.7 | Creates visualization: allocation mechanism comparison | Sankey diagram of good flows: market (price-driven) vs joule (quota-driven) | Diagram shows: market has more cross-cohort transfers (voluntary trade); joule has planned flows from producers to consumers |

#### Phase 4: Sensitivity Analysis & Publication Prep

| Step | Action | System Response | Validation |
|------|--------|-----------------|-----------|
| 4.1 | Runs sensitivity test: vary joule accumulation rate (40 → 50 → 60) and re-run joule scenario 3x | Spawns 3 new runs with different joule configs: `civ-sim run batch --configs joule-40.yaml joule-50.yaml joule-60.yaml` | All 3 runs complete; Gini metrics extracted for all variants |
| 4.2 | Plots Gini sensitivity to joule rate: shows Gini increases from 0.25 (rate=40) → 0.35 (rate=60) | Figure shows: higher joule accumulation → slightly higher inequality (workers earn more discretionary joules) | Sensitivity pattern matches theoretical prediction (labor scarcity drives joule inflation) |
| 4.3 | Documents reproducibility: includes all config files, seeds, and git commit hash | README in results directory includes: scenario version, allocator params, random seeds, git hash of CivLab code | Researcher can provide to peer reviewers; anyone can `git checkout <hash>` and reproduce |
| 4.4 | Exports publication-ready tables | Generates LaTeX tables: allocation type, Gini (mean ± SD), welfare, production, transfers | Tables formatted for journal submission; includes row for p-value and confidence interval |
| 4.5 | Submits to preprint server with reproducibility appendix | Uploads paper + supplementary materials (configs, metrics CSV, R analysis script) to arXiv | Paper includes link to reproducible data; others can download and verify |

### Success State
- Market and joule runs start from identical state (verified by tick-0 state comparison)
- Both runs complete deterministically (same seed = identical exogenous events)
- Primary outcome (Gini) differs significantly: market 0.65 vs joule 0.35 (p < 0.001)
- Mechanism analysis reveals: market enables voluntary exchange; joule enforces planned allocation
- Sensitivity test shows effect magnitude is robust to parameter variation
- Publication includes reproducible configs and statistical summary

### Failure Modes & Recovery

| Failure | Symptom | Recovery |
|---------|---------|----------|
| Runs diverge despite identical seed | Tick N metrics differ between market and joule (unexpected, since only allocator changed) | Check: RNG seed propagation is isolated per allocator; verify both runs use identical climate events and birth/death rolls |
| Ledger imbalance detected | Transfer totals in market ledger = -10 (should be 0) | Regenerate ledger with `--audit-ledger` flag; trace imbalance to specific transfer; may indicate allocator bug |
| Gini metric null for some ticks | Some ticks have Gini = NaN or missing | Check: wealth data exists for all cohorts; may indicate population collapse in variant; filter data to ticks where population > 0 |
| Joule accumulation doesn't converge | Joule balances grow unbounded instead of stabilizing | Verify: joule consumption rates match production rates; may need to rebalance joule cost of living (CIV-0107 param) |

### Acceptance Criteria
- [ ] Both scenarios start from identical state (tick 0 states byte-identical)
- [ ] Both runs use same random seed and produce identical exogenous events (climate, demographics)
- [ ] Primary metric (Gini) differs significantly: market > joule (p < 0.001)
- [ ] Ledger balances are zero for both allocators (conservation invariant holds)
- [ ] Sensitivity test shows effect magnitude is stable across allocator parameter ranges
- [ ] Full reproducibility package can be shared with peer reviewers (configs + seed + git hash)

---

## Cross-Journey Patterns & Common Requirements

### Data & Infrastructure
| Feature | Required By | Spec Reference |
|---------|-----------|-----------------|
| Deterministic replay | UJ-1, UJ-3, UJ-5 | CIV-0101, ADR-003 |
| Ledger balance verification | UJ-1, UJ-3, UJ-5 | CIV-0100 |
| Event log with causal edges | UJ-3 | CIV-0100, CIV-0103 |
| Zoom level 1 (strategic) LOD | UJ-2, UJ-4 | CIV-0101 |
| WebSocket JSON-RPC protocol | UJ-2, UJ-4 | Design doc (P0) |
| Multi-run batch execution | UJ-5 | Design doc (P0) |

### User Experience
| Feature | Required By | Spec Reference |
|---------|-----------|-----------------|
| Causal analysis UI (post-mortem) | UJ-3 | Design doc (P1) |
| Command latency < 100 ms | UJ-2, UJ-4 | Design doc (P0) |
| Replay browser with timeline | UJ-3 | Design doc (P0) |
| What-if scenario branching | UJ-3 | Design doc (P1) |
| Formation/squad management | UJ-4 | Design doc (P0) |
| Supply line logistics | UJ-4 | Design doc (P1) |

---

## Metrics for Success

Each journey should achieve:
1. **Functional Completeness:** All steps execute without errors
2. **Performance SLAs:** Command latency < 100 ms, tick rate stable, memory stable
3. **Determinism:** Replays are byte-reproducible; same seed = same metrics
4. **Causality Clarity:** Player/researcher can articulate causal chain from UI/analytics alone
5. **Reproducibility:** Full config + seed + git hash enables third-party reproduction

