# CivLab Functional Requirements (Master Document)

**Version:** 1.0
**Date:** 2026-02-21
**Status:** APPROVED
**Scope:** Complete functional requirements for headless Rust civilization simulation engine

---

## Overview

This document specifies all functional requirements (FR) for CivLab organized by domain. Each requirement is identified by unique ID (FR-CIV-{DOMAIN}-{NNN}), includes SHALL statements, priority (P0/P1/P2), acceptance criteria, and traceability to specification documents and journeys.

**Structure:**
- FR-CIV-ECON: Economy (market, joule, ledger)
- FR-CIV-RTS: RTS command interface
- FR-CIV-GEO: Geography, terrain, LOD zoom
- FR-CIV-ACT: Actor/citizen lifecycle
- FR-CIV-WAR: War, diplomacy, shadow networks
- FR-CIV-RES: Research/sandbox API (scenario, metrics, export, replay)

---

## DOMAIN: Economy (FR-CIV-ECON-*)

### FR-CIV-ECON-001: Ledger Double-Entry Accounting
**ID:** FR-CIV-ECON-001
**Shall:** The economy module SHALL implement double-entry ledger accounting such that all transfers between actors are recorded as (from_actor, to_actor, amount, currency, reason) and the sum of all transfers per currency per tick SHALL equal zero (conservation invariant).
**Priority:** P0
**Spec Reference:** CIV-0100 (Economy Spec v1), Section "Data Model Additions"
**User Journey:** UJ-1 (Researcher validation), UJ-3 (Ledger matching analysis), UJ-5 (A/B test ledger balance check)
**Acceptance Criteria:**
- [ ] All transfers recorded in `ledger_transfers` table with 8 fields (run_id, tick, from_actor, to_actor, amount, currency, transfer_type, created_at)
- [ ] Sum of all transfers per currency per tick = 0 (verified by property test)
- [ ] Ledger export (CSV) validates without errors; row count = total transfers in run
- [ ] Replay determinism: identical state + policy = identical ledger transfers in identical order
- [ ] No negative balances in any actor ledger (invariant enforced pre-transfer)

---

### FR-CIV-ECON-002: Market Clearing Algorithm
**ID:** FR-CIV-ECON-002
**Shall:** The market allocator SHALL implement price discovery and clearing such that, for each good per tick, bid and ask orders are matched by clearing price, and the price is calculated to minimize unmet demand subject to available supply.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Interfaces"; ADR-002 (Joule Economy as Pluggable Allocator)
**User Journey:** UJ-5 (Market economy variant runs)
**Acceptance Criteria:**
- [ ] For each good per tick: collect all bid and ask orders from actors
- [ ] Compute clearing price: P_clear such that quantity_demanded(P_clear) ≈ quantity_supplied(P_clear)
- [ ] Emit `economy.market_cleared.v1` event with: tick, good, bid_volume, ask_volume, clearing_price, unmet_demand
- [ ] Unmet demand = max(0, quantity_demanded - quantity_supplied) (never negative)
- [ ] Stress test: supply shock (production -40%) does not produce negative inventories or silent failures

---

### FR-CIV-ECON-003: Joule Economy Allocator Implementation
**ID:** FR-CIV-ECON-003
**Shall:** The joule allocator SHALL implement energy-based work accounting where citizens accumulate joules (J) via work output, and consumption debits joules from personal balance. Allocation decision SHALL respect conservation invariant: sum of allocated joules ≤ total joules available.
**Priority:** P0
**Spec Reference:** CIV-0107 (Joule Economy System Spec v1)
**User Journey:** UJ-5 (Joule economy variant runs)
**Acceptance Criteria:**
- [ ] Each citizen maintains EnergyLedger with fields: lifetime_work_accumulated, current_balance, discretionary_allocation, baseline_fulfillment, quota_remaining, retirement_status
- [ ] Work output: JoulesEarned_i(t) = output_i(t) × energy_intensity(work_type) + baseline_discretionary_bonus
- [ ] Consumption: debits joules from citizen balance. Deficit allocation blocked (allocation fails if insufficient joules)
- [ ] Retirement rule: citizen can retire when lifetime_work_accumulated ≥ retirement_threshold (configurable, default 5000 J)
- [ ] Conservation check: sum of allocated joules per tick ≤ available supply (validated by JouleAllocator.validate())
- [ ] Determinism: identical state + seed → identical allocations and retirement decisions

---

### FR-CIV-ECON-004: Policy-Driven Fiscal Control
**ID:** FR-CIV-ECON-004
**Shall:** The policy evaluation engine SHALL accept a policy bundle (tax rate, subsidy amounts, transfer targets, inflation target) and emit fiscal and pricing control outputs that are deterministic for identical state + policy + seed.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Interfaces"
**User Journey:** UJ-1 (Parameter tweaking), UJ-3 (Policy intervention analysis)
**Acceptance Criteria:**
- [ ] Policy.evaluate(state, context) → control where control includes: tax_rate, subsidy_targets, transfer_amounts, inflation_target
- [ ] Tax collection: state taxes applied to actors; revenue goes to state treasury (double-entry ledger)
- [ ] Subsidies: state treasury pays subsidies to designated cohorts; recorded as double-entry transfers
- [ ] Determinism: same state + policy + seed → identical fiscal outputs
- [ ] Bounds checking: tax_rate ∈ [0, 1], inflation_target ∈ [0, 0.1], subsidy_amount ≥ 0
- [ ] Audit trail: all policy changes logged with `policy.applied.v1` event including policy_hash for tracking

---

### FR-CIV-ECON-005: Inflation & Price Index Tracking
**ID:** FR-CIV-ECON-005
**Shall:** The economy module SHALL track energy_price_index (rolling average of clearing prices) and emit inflation metrics per tick. Inflation calculation SHALL use fixed-point arithmetic to ensure determinism.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Data Model Additions" (energy_price_index)
**User Journey:** UJ-1 (Metrics export), UJ-5 (Comparative analysis)
**Acceptance Criteria:**
- [ ] Compute energy_price_index as exponential moving average (EMA) of clearing prices: EMA_t = 0.9 × EMA_{t-1} + 0.1 × price_t
- [ ] Emit `economy.price_index_updated.v1` per tick with current_index and delta from previous tick
- [ ] Inflation rate = (index_t - index_{t-1}) / index_{t-1}, recorded in metrics snapshot
- [ ] Use fixed-point arithmetic (u128 with 18 decimals) not floating-point
- [ ] Validation: price_index monotonic (inflation≥ 0 always); bounded (index < 2x initial after 10k ticks)

---

### FR-CIV-ECON-006: Supply Shock Handling
**ID:** FR-CIV-ECON-006
**Shall:** When exogenous supply shocks occur (climate event reduces food production by X%), the economy module SHALL propagate shortage through market clearing, trigger substitution/migration decisions in downstream actors, and log supply shock magnitude for audit purposes.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Stress test"
**User Journey:** UJ-3 (Drought event and cascading failure)
**Acceptance Criteria:**
- [ ] Supply shock is modeled as production_cap reduction. E.g., drought reduces food production_cap by 40%.
- [ ] Market clearing fails to meet full demand (unmet_demand > 0). Price rises per clearing algorithm.
- [ ] Emit `economy.supply_shock.v1` event with shock_type, magnitude (%), affected_goods, unmet_demand
- [ ] Downstream effects: citizens with unmet demand face consumption deficit, may trigger migration/dissent decisions
- [ ] No silent failures: deficit consumption explicitly blocked; rejected allocation logged as event
- [ ] Recovery: when shock ends, unmet_demand → 0 and price normalizes

---

### FR-CIV-ECON-007: Actor Wealth & Inequality Metrics
**ID:** FR-CIV-ECON-007
**Shall:** The metrics module SHALL compute Gini coefficient (wealth inequality) per tick based on actor wealth distribution. Gini SHALL be calculated deterministically and recorded in metrics snapshots.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Data Model Additions" (metric_snapshots includes gini)
**User Journey:** UJ-5 (Comparative Gini analysis, market vs joule)
**Acceptance Criteria:**
- [ ] Gini coefficient computed as: G = 1 - 2 * Σ(i=1..n) (n+1-i) * wealth_i / (n * Σ wealth_i), where wealth sorted ascending
- [ ] Gini ∈ [0, 1] where 0 = perfect equality, 1 = perfect inequality
- [ ] Computed per tick and recorded in metrics_snapshots table
- [ ] Determinism: identical actor wealth distribution → identical Gini
- [ ] Validation: Gini bounded; plot over time should show allocator effect (market > joule expected)

---

### FR-CIV-ECON-008: Liquidity & Solvency Monitoring
**ID:** FR-CIV-ECON-008
**Shall:** The economy module SHALL track actor liquidity (available cash/credits) and solvency (assets ≥ liabilities). When an actor's liabilities exceed assets, flag as insolvent and emit constraint breach event.
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Event Contracts" (economy.constraint_breached.v1)
**User Journey:** UJ-3 (Default event and cascading crisis)
**Acceptance Criteria:**
- [ ] Track per-actor: assets (inventory value + property), liabilities (debt owed, obligations)
- [ ] Liquidity ratio = liquid_assets / immediate_obligations
- [ ] Solvency = assets ≥ liabilities. Flag: solvency_status ∈ {SOLVENT, INSOLVENT, CRITICAL}
- [ ] Emit `economy.solvency_status_changed.v1` when status transitions
- [ ] Insolvent actor: restrict new loans, apply policy penalties (see UJ-3)
- [ ] Validation: no silent insolvency; all status changes logged

---

### FR-CIV-ECON-009: Hybrid Allocator (Market + Joule)
**ID:** FR-CIV-ECON-009
**Shall:** The economy module SHALL support a hybrid allocator that uses market clearing for some goods (food, metal) and joule-based allocation for others (care, education, research). Allocation decisions for hybrid goods SHALL respect conservation invariant and be deterministic.
**Priority:** P2
**Spec Reference:** CIV-0107, Section "Integration"; ADR-002 (Pluggable Allocator pattern)
**User Journey:** UJ-5 (Potential future comparative test)
**Acceptance Criteria:**
- [ ] Spec bundle includes allocation_map: {good_id: allocator_type} where allocator_type ∈ {market, joule}
- [ ] Market goods use clearing price; joule goods use quota allocation
- [ ] Hybrid allocator validates conservation separately for market and joule sectors
- [ ] Determinism: identical state + allocator_map + seed → identical allocations
- [ ] No cross-allocator arbitrage: price in market goods ≠ energy cost in joule goods (by design)

---

### FR-CIV-ECON-010: Transfers & Redistribution Policy
**ID:** FR-CIV-ECON-010
**Shall:** The economy module SHALL implement configurable transfer policies (basic income, emergency relief, targeted subsidies) that debit from state treasury and credit to recipient actors. All transfers SHALL be recorded in double-entry ledger with explicit transfer_type.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Data Model Additions" (ledger_transfers.transfer_type)
**User Journey:** UJ-3 (Policy mitigation during crisis)
**Acceptance Criteria:**
- [ ] Transfer types: BASIC_INCOME, EMERGENCY_RELIEF, SUBSIDY, DEBT_FORGIVENESS, REPARATIONS
- [ ] Each transfer type has associated policy parameter (e.g., basic_income_amount = 10 units/tick)
- [ ] State treasury tracks incoming (tax, resource production) and outgoing (subsidies, transfers)
- [ ] Ledger entry: (state, recipient, amount, currency, BASIC_INCOME) recorded per tick
- [ ] Determinism: identical state + transfer policy → identical transfer amounts and recipients
- [ ] Validation: total outgoing transfers ≤ state treasury balance (no deficit spending unless explicitly allowed)

---

### FR-CIV-ECON-011: Market Order Book Simulation
**ID:** FR-CIV-ECON-011
**Shall:** The market allocator SHALL maintain an order book of buyer and seller orders per good per tick. Orders SHALL be matched by FIFO within price bands, and unmatched orders SHALL carry forward or expire per order lifetime rule.
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Interfaces"; ADR-002 (Market Allocator design)
**User Journey:** UJ-1 (Ledger trace export showing market orders)
**Acceptance Criteria:**
- [ ] Order book structure: {good_id: {tick: [buy_orders], [sell_orders]}}
- [ ] Buy order: {buyer_id, quantity, max_price, lifetime_ticks, created_tick}
- [ ] Sell order: {seller_id, quantity, min_price, lifetime_ticks, created_tick}
- [ ] Matching: FIFO within price overlap; partial fills allowed
- [ ] Unmatched orders: carry forward until lifetime expires (default 100 ticks)
- [ ] Determinism: matching order is stable (sort by order_id if prices tie)
- [ ] Export: ledger trace includes order matching details (which buy/sell matched, partial fill qty)

---

### FR-CIV-ECON-012: Production & Capacity Constraints
**ID:** FR-CIV-ECON-012
**Shall:** The economy module SHALL model production with capacity constraints. A producer has max_output per good per tick, constrained by labor availability, material inputs, and infrastructure. Exceeding capacity SHALL fail and log constraint breach.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Interfaces" (supply state); CIV-0101 (LOD aggregation)
**User Journey:** UJ-1 (Resource constraints in scenario setup)
**Acceptance Criteria:**
- [ ] Producer state: {good_type, capacity_per_tick, current_inventory, input_requirements}
- [ ] Output calculation: min(capacity, available_inputs, requested_by_demand)
- [ ] Capacity constraint: production ≤ capacity_per_tick (hard limit)
- [ ] Input constraint: production ≤ available_inputs (e.g., food production requires water)
- [ ] Emit `economy.production_capped.v1` event if production < requested (capacity binding)
- [ ] Validation: no production > capacity; supply never exceeds produced + reserves

---

### FR-CIV-ECON-013: Economic Events & Audit Trail
**ID:** FR-CIV-ECON-013
**Shall:** The economy module SHALL emit structured events for all material state changes (market cleared, transfer booked, policy applied, constraint breached, solvency changed) with EventEnvelopeV1 format including timestamp, correlation_id, and full payload.
**Priority:** P0
**Spec Reference:** CIV-0100, Section "Event Contracts"; ADR-003 (Deterministic Replay)
**User Journey:** UJ-1 (Event log review), UJ-3 (Causal analysis panel), UJ-5 (Mechanism analysis)
**Acceptance Criteria:**
- [ ] Event types: policy.applied, economy.market_cleared, economy.transfer_booked, economy.constraint_breached, economy.supply_shock, economy.solvency_status_changed
- [ ] Event payload: tick, correlation_id (trace through causal chain), full context (prices, quantities, actors, reasons)
- [ ] Event ordering: stable and deterministic per tick (sorted by entity_id or event_type)
- [ ] Replay: replaying events in same order reconstructs identical economy state
- [ ] Audit: all events written to immutable event log (append-only)

---

### FR-CIV-ECON-014: Price Volatility & Market Dynamics
**ID:** FR-CIV-ECON-014
**Shall:** The market allocator SHALL model price elasticity and volatility. When supply/demand ratio deviates from equilibrium, price changes per elasticity curve. Extreme shocks (>50% supply drop) SHALL trigger price spikes (>100% increase).
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Interfaces"; Macro-economics theory
**User Journey:** UJ-3 (Food shortage → price spike → welfare crisis)
**Acceptance Criteria:**
- [ ] Elasticity model: price_change ∝ (demand - supply) / supply (simplified linear)
- [ ] Price bounds: 0 < price < ∞ (no negative prices); bounded change per tick (e.g., ±20% max)
- [ ] Supply shock elasticity: 40% supply drop → ~80% price increase (2x elasticity)
- [ ] Recovery: when shock ends, price returns to equilibrium (price path logged)
- [ ] Determinism: identical supply/demand → identical price path
- [ ] Validation: price volatility test shows expected behavior under modeled shocks

---

### FR-CIV-ECON-015: Reserve & Buffer Management
**ID:** FR-CIV-ECON-015
**Shall:** Actors SHALL maintain reserves (emergency stockpiles, cash buffers) with explicit reserve_in and reserve_out flows per tick. Reserves decayed or withdrawn per actor policy. Conservation equation includes reserve flows: supply + reserves_in - losses - consumption - reserves_out = delta_stock.
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Invariants" (conservation equation)
**User Journey:** UJ-3 (Supply shock exhausts reserves, then crisis escalates)
**Acceptance Criteria:**
- [ ] Actor state: reserves_storage[good_type], reserve_decay_rate
- [ ] Reserve_in: voluntary accumulation or mandatory emergency draw
- [ ] Reserve_out: consumption during shortage (override normal allocation)
- [ ] Conservation: supply + reserves_in - losses - consumption - reserves_out = delta_stock (verified per tick)
- [ ] Decay: reserves_storage_t+1 = reserves_storage_t × (1 - decay_rate) (modeling spoilage, maintenance cost)
- [ ] Bounds: reserves cannot exceed storage capacity; reserve_out cannot exceed reserves_storage

---

### FR-CIV-ECON-016: Exchange Rate & Currency Management
**ID:** FR-CIV-ECON-016
**Shall:** The economy SHALL support multiple currencies (energy joules, trade credits, national currency) with exchange rates that can vary over time. Conversions in double-entry ledger SHALL explicitly record from_currency and to_currency.
**Priority:** P2
**Spec Reference:** CIV-0100, Section "Data Model Additions" (ledger_transfers.currency)
**User Journey:** UJ-1 (Multi-scenario parameter exploration)
**Acceptance Criteria:**
- [ ] Currency types: {JOULES, CREDITS, NATIONAL_CURRENCY, FOREIGN_CURRENCY}
- [ ] Exchange rates: maintained in state, can vary per tick based on market conditions
- [ ] Ledger entry for currency conversion: {from_actor, to_actor, amount_from, currency_from, amount_to, currency_to, exchange_rate, reason: CURRENCY_CONVERSION}
- [ ] No hidden conversions: all exchanges logged explicitly
- [ ] Determinism: identical exchange rates + conversion amounts → identical ledger
- [ ] Bounds: exchange rates > 0; cannot create arbitrage (e.g., A→B→A conversion returns < original amount)

---

### FR-CIV-ECON-017: Multipliers & Policy Modifiers
**ID:** FR-CIV-ECON-017
**Shall:** Policy bundle SHALL include multipliers (production_multiplier, consumption_multiplier, trade_multiplier, inequality_growth_rate) that scale outputs of allocators. Multipliers SHALL be applied deterministically and logged in policy audit trail.
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Interfaces" (policy.evaluate control outputs)
**User Journey:** UJ-1 (Parameter tweaking, including multipliers)
**Acceptance Criteria:**
- [ ] Multiplier fields: production_mult ∈ [0.5, 2.0], consumption_mult ∈ [0.5, 2.0], trade_mult ∈ [0.5, 2.0]
- [ ] Applied to allocator outputs: actual_output = base_output × production_mult
- [ ] Logged: policy.applied.v1 includes all multiplier values
- [ ] Determinism: identical state + multipliers → identical scaled outputs
- [ ] Composition: multipliers are multiplicative (not additive) to avoid unbounded growth

---

### FR-CIV-ECON-018: Bankruptcy & Debt Forgiveness
**ID:** FR-CIV-ECON-018
**Shall:** When an actor's liabilities exceed assets by threshold (e.g., 50%), the system SHALL offer bankruptcy event. Bankruptcy triggers: debt restructuring, asset seizure, creditor recovery. Debt forgiveness is a policy option that must be logged and traced to policy decision.
**Priority:** P1
**Spec Reference:** CIV-0100, Section "Constraints"; ADR-002 (Conservation invariant & allocator validation)
**User Journey:** UJ-3 (Default and forced policy response)
**Acceptance Criteria:**
- [ ] Bankruptcy trigger: liabilities > assets × threshold (threshold = 1.5x by default)
- [ ] Bankruptcy event: `economy.bankruptcy_triggered.v1` with debtor_id, liabilities, assets, recovery_plan
- [ ] Recovery options: (a) creditor haircut (debt writeoff, creditor loss), (b) asset liquidation, (c) principal restructure (extend repayment timeline)
- [ ] Debt forgiveness: creditor loss is recorded as double-entry: (creditor, state_insurance_fund, forgiven_amount, reason: DEBT_FORGIVENESS)
- [ ] Policy control: policy bundle can include bankruptcy_threshold and preferred_recovery_option
- [ ] Determinism: identical bankruptcy state + policy → identical recovery plan

---

### FR-CIV-ECON-019: Economic Feedback Loops & Stability
**ID:** FR-CIV-ECON-019
**Shall:** The economy module SHALL detect positive feedback loops (e.g., inflation → wage spiral → further inflation) and negative feedback loops (e.g., unemployment → demand drop → price drop → cost reduction). Feedback loops SHALL be logged as causal edges for post-hoc analysis.
**Priority:** P2
**Spec Reference:** CIV-0100, Section "Event Contracts"; CIV-0103 (Citizen Lifecycle & Institutions)
**User Journey:** UJ-3 (Causal analysis graph shows feedback loops)
**Acceptance Criteria:**
- [ ] Feedback loop detection: track prices, wages, unemployment, demand over rolling window (e.g., 50 ticks)
- [ ] Positive loop: if dP/dt > 0 and dW/dt > 0 and dP/dW > threshold (wage-price spiral), flag
- [ ] Negative loop: if dU/dt > 0 and dD/dt < 0 and dP/dt < 0, flag (stability loop)
- [ ] Emit `economy.feedback_loop_detected.v1` with loop_type, magnitude (amplification factor), causal_chain
- [ ] Validation: feedback loops are accurately described; magnitude bounds growth (e.g., amplification ≤ 2x per 100 ticks)
- [ ] Analytics: researcher can export feedback_loop log and visualize in causal graph

---

### FR-CIV-ECON-020: Economic Scenario Presets
**ID:** FR-CIV-ECON-020
**Shall:** The system SHALL provide canonical economic scenario presets: Free Market (zero tax, market allocator, no subsidies), Planned Economy (high tax, plan allocator, full redistribution), Joule-First Utopia (joule allocator, baseline universal income, low inequality), Mixed Social Market (50% market / 50% joule, moderate tax/subsidy). Each preset SHALL be fully specified in YAML and loadable via CLI.
**Priority:** P1
**Spec Reference:** CIV-0100 (Economy Spec), ADR-002 (Allocators), CIV-0107 (Joule Economy)
**User Journey:** UJ-1 (Scenario selection), UJ-5 (Preset comparison)
**Acceptance Criteria:**
- [ ] Four preset specs exist: `free-market.yaml`, `planned-economy.yaml`, `joule-utopia.yaml`, `mixed-social-market.yaml`
- [ ] Each preset includes: allocator choice, tax_rate, subsidy_targets, transfer_policies, inflation_target, Gini_target (for comparison)
- [ ] Loadable: `civ-sim scenario apply --preset free-market` loads and validates without error
- [ ] Determinism: running same preset multiple times with same seed produces identical outputs
- [ ] Documentation: each preset has README describing rationale and expected outcomes

---

## DOMAIN: RTS Command Interface (FR-CIV-RTS-*)

### FR-CIV-RTS-001: Unit Movement Command
**ID:** FR-CIV-RTS-001
**Shall:** The system SHALL accept unit move commands specifying unit_id and target_location (x, y). Movement SHALL resolve in the movement phase per tick, with unit traveling per speed stat, encountering terrain, and triggering proximity alerts when enemies detected.
**Priority:** P0
**Spec Reference:** CIV-0101 (LOD Spec, Zoom Level 2 Tactical); Design doc (P0 RTS interface)
**User Journey:** UJ-2 (Godot game integration), UJ-4 (RTS gameplay)
**Acceptance Criteria:**
- [ ] Command format: `{"type": "unit_move", "unit_id": <int>, "target": {"x": <int>, "y": <int>}}`
- [ ] Movement resolution: per_tick_distance = unit.speed × terrain_modifier(current_tile). Unit stops at target or obstacles.
- [ ] Terrain cost: rough terrain = 0.5x speed; water = 0.25x speed; roads = 1.5x speed
- [ ] Collision detection: unit stops 1 hex before obstacle (another unit, structure, impassable terrain)
- [ ] Alerts: proximity_alert when enemy unit detected within vision_range
- [ ] Latency: command ACK < 50 ms; unit starts moving within 1 tick

---

### FR-CIV-RTS-002: Unit Combat & Attack Orders
**ID:** FR-CIV-RTS-002
**Shall:** The system SHALL accept attack orders specifying attacking_unit and target_unit (or target_location). Combat SHALL resolve per combat phase mechanics: compare armor, apply damage rolls, track morale, output casualties, and emit combat event with damage/loss detail.
**Priority:** P0
**Spec Reference:** Design doc (P0 Combat mechanics); CIV-0105 (War & Diplomacy)
**User Journey:** UJ-4 (Combat engagement with AI)
**Acceptance Criteria:**
- [ ] Command format: `{"type": "unit_attack", "attacker_unit_id": <int>, "target_unit_id": <int>}`
- [ ] Combat calculation: damage = attacker.attack_strength - target.armor + random(±10%)
- [ ] Morale mechanic: if damage > morale_threshold, unit morale drops; critical morale → routing (uncontrolled retreat)
- [ ] Casualties: damage accumulates per unit; HP < 0 → unit destroyed, removed from map
- [ ] Determinism: identical unit states + RNG seed → identical damage rolls and casualties
- [ ] Event: `military.unit_combat.v1` includes attacker_id, target_id, damage_dealt, casualties, morale_change
- [ ] Validation: no negative casualties; damaged units remain on map until HP = 0

---

### FR-CIV-RTS-003: Unit Group & Formation Control
**ID:** FR-CIV-RTS-003
**Shall:** The system SHALL accept group move commands specifying multiple unit_ids and a formation type (wedge, line, column, circle). Units in formation SHALL move together, maintain spacing, and re-form after obstacles.
**Priority:** P0
**Spec Reference:** Design doc (P0 Formations); UJ-4 (Formation tactics)
**User Journey:** UJ-4 (RTS gameplay with formations)
**Acceptance Criteria:**
- [ ] Command format: `{"type": "unit_group_move", "unit_ids": [1, 2, 3, ...], "formation": "wedge", "target": {"x": <int>, "y": <int>}}`
- [ ] Formation types: wedge (point), line (parallel), column (file), circle (defensive)
- [ ] Formation spacing: units maintain 1-hex separation (formation_offset per unit)
- [ ] Pathfinding: group finds path to target avoiding obstacles; formation holds if path is clear; re-forms after clearing obstacles
- [ ] Movement: group moves at speed of slowest unit
- [ ] Flank detection: if formation flanked (enemy detected within 90° arc on flank), emit `military.flanked.v1` event
- [ ] Determinism: identical group state + target + RNG seed → identical movement path and timings

---

### FR-CIV-RTS-004: Command Queuing & Auto-Execute
**ID:** FR-CIV-RTS-004
**Shall:** The system SHALL support command queues where player issues multiple commands (move → attack → move) and they execute sequentially when prior condition completes (unit reaches destination, combat resolves).
**Priority:** P1
**Spec Reference:** Design doc (P1 Advanced RTS)
**User Journey:** UJ-4 (Extended engagement, multi-turn tactics)
**Acceptance Criteria:**
- [ ] Queue structure: {unit_id: [cmd1, cmd2, cmd3]}
- [ ] Command conditions: (a) unit_idle (ready for next command), (b) target_reached, (c) combat_resolved
- [ ] Auto-execute: when condition met, pop next command and execute
- [ ] Cancellation: player can cancel queued commands with `{"type": "cancel_queue", "unit_id": <int>}`
- [ ] Determinism: identical queue execution → identical unit states and timings
- [ ] Validation: invalid commands (target unreachable, unit dead) are rejected and logged

---

### FR-CIV-RTS-005: Supply Line & Logistics
**ID:** FR-CIV-RTS-005
**Shall:** The system SHALL model unit supply (food, ammunition). Units consume supply per tick; when supply depleted, unit combat effectiveness drops (morale -50%, damage -20%). Supply can be restored via resupply command or proximity to supply depot.
**Priority:** P1
**Spec Reference:** Design doc (P1 Logistics); UJ-4 (Supply depot construction and resupply)
**User Journey:** UJ-4 (Supply management)
**Acceptance Criteria:**
- [ ] Unit supply state: {supply_type, remaining_ticks, consumption_per_tick, max_capacity}
- [ ] Supply depletion: supply_ticks -= consumption_per_tick × number_of_units per tick
- [ ] When supply < 25% max: emit `military.low_supply.v1` alert
- [ ] When supply = 0: unit combat_effectiveness_mult = 0.5 (damage 50% reduced, morale breaks at lower threshold)
- [ ] Resupply command: `{"type": "unit_resupply", "unit_ids": [..], "source_structure_id": <int>}` refills supply from structure inventory
- [ ] Resupply cost: deducts from structure storage; if insufficient, partial refill
- [ ] Auto-resupply: units within 1 hex of supply depot auto-resupply at 20 supply_ticks/tick

---

### FR-CIV-RTS-006: Structure Construction & Management
**ID:** FR-CIV-RTS-006
**Shall:** The system SHALL accept structure construction commands specifying structure_type and location. Construction takes N ticks, consumes resources from city storage, and outputs structure with specified HP and properties (e.g., supply depot has 500 inventory slots).
**Priority:** P0
**Spec Reference:** Design doc (P0 Structures); CIV-0101 (Zoom Level 2 Districts/Structures)
**User Journey:** UJ-2 (Structure build in game), UJ-4 (Supply depot construction)
**Acceptance Criteria:**
- [ ] Command format: `{"type": "structure_build", "structure_type": "supply_depot", "location": {"x": <int>, "y": <int>}}`
- [ ] Construction time: structure_type.build_time_ticks (e.g., supply_depot = 20 ticks)
- [ ] Resource cost: structure_type.cost_resources = {wood: 100, metal: 50, labor: 10}; deducted from city_storage
- [ ] Construction progress: structure.construction_progress increments; when = 100%, structure completed and becomes active
- [ ] Active structure: property HP (can be damaged), inventory (if applicable), and occupants (if garrison)
- [ ] Cancellation: incomplete structure can be cancelled, returning 50% of resources
- [ ] Determinism: identical construction resources + timeline → identical structure state

---

### FR-CIV-RTS-007: Structure Damage & Repair
**ID:** FR-CIV-RTS-007
**Shall:** When structures are attacked or damaged (by combat, environmental event), their HP decreases. Repair command restores structure HP using worker units or resources. Destroyed structure (HP = 0) is removed from map.
**Priority:** P1
**Spec Reference:** Design doc (P1 Structure mechanics); CIV-0105 (War & Diplomacy)
**User Journey:** UJ-4 (Depot damage and repair)
**Acceptance Criteria:**
- [ ] Structure attack: `{"type": "attack_structure", "attacker_unit_id": <int>, "target_structure_id": <int>}` calculates damage and applies to structure.HP
- [ ] Damage calculation: structure_armor reduces attack damage (similar to unit combat)
- [ ] Repair command: `{"type": "repair_structure", "structure_id": <int>, "worker_unit_ids": [..]}` restores structure HP at repair_rate per worker
- [ ] Resource repair: `{"type": "repair_structure_cost", "structure_id": <int>, "resources": {wood: 20}}` repairs using inventory (1 resource = 2 HP restored)
- [ ] Destruction: when HP = 0, structure is destroyed, removed from map, resources are partially salvaged
- [ ] Determinism: identical damage + repair → identical structure state

---

### FR-CIV-RTS-008: Vision & Fog of War
**ID:** FR-CIV-RTS-008
**Shall:** The system SHALL track vision for player and AI factions. Units have vision_range (hex distance); terrain modifies vision (forests reduce vision by 50%). Unknown territory (outside vision range) is fogged (not visible to player). When units move or perish, fog updates.
**Priority:** P0
**Spec Reference:** Design doc (P0 Vision); UJ-4 (Enemy unit visibility)
**User Journey:** UJ-4 (Scout reveals enemy positions)
**Acceptance Criteria:**
- [ ] Vision calculation: unit.vision_range based on unit_type (scout = 2 hex, soldier = 1 hex, commander = 3 hex)
- [ ] Terrain modifier: forest = 0.5x vision, hill = 1.5x vision, building = blocks vision (line-of-sight)
- [ ] Fog-of-war: cells outside vision_range show no unit/structure information (fully fogged)
- [ ] Cell in vision: shows all unit/structure positions and types
- [ ] Historical fog: cells previously visible but now fogged show stale information (last seen snapshot) with "?stale?" marker
- [ ] Update: when unit moves or dies, fog updates immediately in next tick
- [ ] Determinism: identical unit positions + vision ranges → identical fog state per player

---

### FR-CIV-RTS-009: Diplomacy & Treaties
**ID:** FR-CIV-RTS-009
**Shall:** The system SHALL allow player-controlled factions to issue diplomacy requests to AI factions: propose alliance, declare war, offer trade, request peace. Requests are accepted/rejected by AI faction policy and result in state transitions (cooperative → conflict, etc.).
**Priority:** P1
**Spec Reference:** CIV-0105 (War, Diplomacy, Shadow Networks); Design doc (P1 Diplomacy)
**User Journey:** UJ-4 (Diplomacy panel and AI response)
**Acceptance Criteria:**
- [ ] Diplomacy commands: `{"type": "propose_alliance", "target_faction_id": <int>}`, `propose_war`, `offer_trade`, `request_peace`
- [ ] AI response: determined by faction policy and relationship scoring. Response options: ACCEPT, REJECT, COUNTER_OFFER
- [ ] State transition: if alliance accepted, diplomatic_state(playerA, AIB) = COOPERATIVE. War declaration triggers ACTIVE_CONFLICT.
- [ ] Trade agreement: specifies goods and quantities per tick. Reduces if trade agreement broken.
- [ ] Peace treaty: ACTIVE_CONFLICT → DEESCALATING with 100-tick cooldown before war can resume
- [ ] Determinism: identical faction state + policy → identical diplomacy response

---

### FR-CIV-RTS-010: Siege Mechanics
**ID:** FR-CIV-RTS-010
**Shall:** When a player army occupies a hex adjacent to an enemy city, siege begins. Besieging army blocks trade in/out and drains garrison resources. Garrison must choose: defend (combat with besiegers) or surrender (if morale/supplies depleted).
**Priority:** P1
**Spec Reference:** CIV-0105 (War & Diplomacy); Design doc (P1 Siege)
**User Journey:** UJ-4 (Final siege of AI capital)
**Acceptance Criteria:**
- [ ] Siege trigger: if player army count > threshold adjacent to enemy city, siege starts
- [ ] Siege effect: (a) garrison supply depletion rate +200%, (b) incoming trade blocked, (c) garrison morale -5% per tick
- [ ] Garrison defense: can issue combat command to fight besiegers; combat occurs and may break siege
- [ ] Garrison surrender: if morale < 10% or supplies < 10%, garrison can surrender (player choice or AI auto-surrender)
- [ ] Surrender result: player occupies city, takes control of resources and citizens (population loyalty = 0 initially)
- [ ] Determinism: identical siege state + garrison policy → identical surrender decision timing

---

### FR-CIV-RTS-011: Espionage & Shadow Operations
**ID:** FR-CIV-RTS-011
**Shall:** The system SHALL allow player to issue spy/sabotage orders: infiltrate city (reveal enemy structure), sabotage structure (reduce HP), assassinate leader (remove unit). Spy actions have success_probability based on target_security and spy_skill.
**Priority:** P2
**Spec Reference:** CIV-0105 (Shadow Networks); Design doc (P2 Espionage)
**User Journey:** UJ-4 (Extended late-game espionage)
**Acceptance Criteria:**
- [ ] Spy command: `{"type": "spy_infiltrate", "target_city_id": <int>, "spy_unit_id": <int>}` sends spy to city
- [ ] Infiltration: spy_unit moves to city, enters hidden state (not visible in fog-of-war)
- [ ] Hidden spy actions: reveal_structures, gather_intelligence, sabotage_structure (chooses target structure)
- [ ] Success probability: P_success = 1 - (target_security - spy_skill) / 10, bounded in [0, 1]
- [ ] On success: sabotage deals 50 damage to structure; reveal structures outputs intelligence event
- [ ] On failure: spy exposed, can be captured/killed (removed from map)
- [ ] Determinism: identical spy/target state + RNG seed → identical success/failure

---

### FR-CIV-RTS-012: Turn-Based vs Real-Time Command Resolution
**ID:** FR-CIV-RTS-012
**Shall:** The system SHALL support both turn-based (1 tick per player input) and real-time (10 ticks/second wall-clock) command execution modes. Player can switch modes during gameplay. Command latency SHALL be < 100 ms in real-time mode.
**Priority:** P0
**Spec Reference:** Design doc (P0 Game Modes)
**User Journey:** UJ-2 (Game mode selection in Godot), UJ-4 (RTS real-time mode)
**Acceptance Criteria:**
- [ ] Mode setting: `{"game_mode": "turn_based"}` or `"real_time"`
- [ ] Turn-based: each player input advances 1 tick; AI acts per tick
- [ ] Real-time: ticks advance at 10 Hz; player issues commands asynchronously
- [ ] Mode switch: can pause game and switch modes (tick counter persists)
- [ ] Latency (real-time): command issued at time T, executed by tick T+0.05s (< 100 ms)
- [ ] Determinism: identical input sequence executed in both modes → identical final state (within tolerance for tick alignment)

---

### FR-CIV-RTS-013: Unit Experience & Leveling
**ID:** FR-CIV-RTS-013
**Shall:** Units gain experience (XP) per combat and kill enemy units. XP accumulates toward next level; leveled units gain stat bonuses (attack +10%, armor +5%, morale +10%).
**Priority:** P1
**Spec Reference:** Design doc (P1 Unit progression)
**User Journey:** UJ-4 (Veteran units become valuable assets)
**Acceptance Criteria:**
- [ ] Unit XP: unit.experience_points accumulated per tick in combat (1 XP per hit, bonus for kill)
- [ ] Leveling: experience_threshold = 100 × level. At threshold, unit.level++, reset XP = 0
- [ ] Stat bonuses: level_bonus = (level - 1) × delta per stat. Attack += 10% per level, armor += 5%, morale += 10%
- [ ] Max level: 10 (soft cap, can exceed with exceptional performance)
- [ ] Persistent: unit level persists across movements and repairs
- [ ] Determinism: identical unit combat history + seed → identical XP and level progression

---

### FR-CIV-RTS-014: Faction AI Behavior & Decision Making
**ID:** FR-CIV-RTS-014
**Shall:** AI factions issue military commands autonomously per tick based on faction policy. AI evaluates: threat (military balance vs player), opportunity (undefended enemies), resource state (can afford new units?). AI decisions SHALL be deterministic given policy and state.
**Priority:** P0
**Spec Reference:** CIV-0103 (Actor Lifecycle); CIV-0105 (Geopolitics); Design doc (P0 AI)
**User Journey:** UJ-4 (AI faction defense and counter-attack)
**Acceptance Criteria:**
- [ ] AI policy: {threat_tolerance, opportunity_threshold, resource_spending_rate, preferred_tactics}
- [ ] Threat eval: compare own_military_strength vs player_military_strength; if ratio < threat_tolerance, raise alert
- [ ] Defense: if threatened and defense_structures < threshold, spawn defensive units (if resources available)
- [ ] Attack: if opportunity (undefended enemy structure) and resources available, queue attack order
- [ ] Diplomacy: if heavily outnumbered, may propose peace/trade (not surrender); can form alliances
- [ ] Determinism: identical faction state + policy + seed → identical decisions and unit orders

---

### FR-CIV-RTS-015: Client-Side Prediction & Replay Correction
**ID:** FR-CIV-RTS-015
**Shall:** The client SHALL predict unit movements locally (extrapolate position based on speed). When server authoritative state arrives, client corrects prediction if delta > 1 hex. Prediction SHALL be transparent to player (snap corrections < 100 ms).
**Priority:** P1
**Spec Reference:** Design doc (P1 Client Prediction); UJ-2 (Godot integration)
**User Journey:** UJ-2 (Smooth movement over network lag)
**Acceptance Criteria:**
- [ ] Local prediction: client maintains predicted_unit_positions and advances per tick based on movement command
- [ ] Server update: server sends authoritative state every 100 ms (or on state change)
- [ ] Correction: if predicted_pos ≠ authoritative_pos, delta = abs(predicted_pos - authoritative_pos)
- [ ] Snap correction: if delta > 1 hex, unit snaps to authoritative position (visible but quick)
- [ ] Transition: if delta ≤ 1 hex, unit smoothly transitions over 50 ms
- [ ] Determinism: client prediction is deterministic (same movement + seed = same predicted path)

---

## DOMAIN: Geography & Terrain (FR-CIV-GEO-*)

### FR-CIV-GEO-001: Terrain Types & Properties
**ID:** FR-CIV-GEO-001
**Shall:** The system SHALL define terrain types (plains, forest, hill, mountain, water, city) with properties: movement_cost, vision_modifier, resource_richness, structure_buildability. Terrain properties affect gameplay mechanics.
**Priority:** P0
**Spec Reference:** CIV-0101 (Spatial & LOD), Design doc (P0 Terrain)
**User Journey:** UJ-2 (Terrain in game map), UJ-4 (Tactical movement)
**Acceptance Criteria:**
- [ ] Terrain type enum: PLAINS, FOREST, HILL, MOUNTAIN, WATER, CITY, SHALLOW_WATER
- [ ] Properties per terrain: {movement_cost_multiplier, vision_range_modifier, combat_defense_bonus, production_multiplier, buildability}
- [ ] Plains: 1.0x move, 1.0x vision, 0% defense, 1.0x production, buildable
- [ ] Forest: 0.5x move, 0.5x vision, +10% defense (units hide), 0.8x production (partial clear needed), buildable
- [ ] Mountain: 0.25x move, 1.5x vision (high ground), +20% defense, 0% production, not buildable
- [ ] Water: 0.1x move (must have boat), 1.0x vision, 0% defense, 0% production, not buildable (on water hex)
- [ ] Determinism: terrain properties static (version controlled in scenario), not random

---

### FR-CIV-GEO-002: Map Generation & Biome Systems
**ID:** FR-CIV-GEO-002
**Shall:** The system SHALL support map generation with biome specification. Biomes (temperate, desert, tropical, tundra) determine climate (rainfall, temperature), resource distribution, and terrain mix. Generation SHALL be deterministic given seed.
**Priority:** P0
**Spec Reference:** CIV-0102 (Climate Followup); Design doc (P0 Map Gen)
**User Journey:** UJ-1 (Scenario selection and map characteristics)
**Acceptance Criteria:**
- [ ] Biome types: TEMPERATE, DESERT, TROPICAL, TUNDRA
- [ ] Biome properties: climate (temperature range, rainfall), resource_pool (food, metal, wood mix), terrain_distribution
- [ ] Generation: Perlin noise for terrain elevation, adjusted per biome parameters
- [ ] Determinism: given seed + biome, map generation is identical (no randomness in terrain placement)
- [ ] Scenario spec: includes map_seed, map_biome, map_size (W×H in hexes)
- [ ] Validation: generated map matches expected resource distribution within ±5%

---

### FR-CIV-GEO-003: District & Region Subdivision
**ID:** FR-CIV-GEO-003
**Shall:** The map SHALL be subdivided into districts (ZoomLevel 2) and regions (ZoomLevel 1). Districts represent city areas; regions represent strategic areas (multi-district). Zoom transitions preserve identity: drilling down from region shows constituent districts with LOD aggregation.
**Priority:** P0
**Spec Reference:** CIV-0101 (Two-Zoom LOD Spec)
**User Journey:** UJ-2 (Game integration with LOD), UJ-4 (Strategic vs tactical zoom)
**Acceptance Criteria:**
- [ ] Region structure: {region_id, name, parent_nation_id, constituent_district_ids, aggregated_resources, aggregated_population}
- [ ] District structure: {district_id, region_id, hex_coordinates, resource_stocks, population_cohorts, institutions}
- [ ] Zoom level 1 (strategic): displays regions, region-level aggregates (total food, total soldiers, regional GDP)
- [ ] Zoom level 2 (tactical): displays districts within region, per-district resources and population
- [ ] Drill-down: player clicks on region → maps zooms to show 9 constituent districts in layout
- [ ] Roll-up: region.aggregated_X = sum(district.X for X in constituent_districts)
- [ ] Determinism: region aggregation is deterministic function of districts at same tick

---

### FR-CIV-GEO-004: Neighbor Queries & Pathfinding
**ID:** FR-CIV-GEO-004
**Shall:** The system SHALL provide neighbor queries (hexagonal grid) and A* pathfinding. Pathfinding avoids impassable terrain and obstacles. Pathfinding results are deterministic (stable ordering of equal-cost paths).
**Priority:** P0
**Spec Reference:** Design doc (P0 Pathfinding)
**User Journey:** UJ-2 (Unit movement), UJ-4 (Army navigation)
**Acceptance Criteria:**
- [ ] Neighbor query: given hex (x, y), return 6 neighbors (hexagonal grid)
- [ ] Passability: terrain determines passability; some units can traverse water (boats), others cannot
- [ ] Pathfinding: A* with heuristic = distance to goal. Cost = terrain_movement_cost
- [ ] Path optimality: returned path has minimal cost (or minimal steps if cost-tied)
- [ ] Determinism: identical start + goal + grid → identical path (stable tie-breaking by hex_id)
- [ ] Performance: path query < 10 ms for 10k-hex map

---

### FR-CIV-GEO-005: Resource Distribution & Renewal
**ID:** FR-CIV-GEO-005
**Shall:** Map resources (food, metal, wood) are distributed across districts. Resources renew per tick at renewal_rate (e.g., food +2/tick). Extraction of resources depletes renewable at rate > renewal, creating pressure to manage extraction.
**Priority:** P0
**Spec Reference:** CIV-0100 (Supply state), CIV-0102 (Climate); UJ-1 (Resource constraints)
**User Journey:** UJ-3 (Drought reduces food renewal, triggers shortage)
**Acceptance Criteria:**
- [ ] Resource state: {food_renewable: N/tick, metal_renewable: N/tick, wood_renewable: N/tick, current_stock: {food, metal, wood}}
- [ ] Renewal: each tick, stock += renewable_rate (subject to climate modifiers, see FR-CIV-GEO-006)
- [ ] Extraction: when citizens/workers extract, stock -= extraction_amount
- [ ] Sustainability: extraction_rate ≤ renewal_rate → stable; extraction > renewal → depletion
- [ ] Depletion: if stock → 0, extraction fails (workers assigned but produce 0)
- [ ] Climate modulation: drought can reduce food renewable by 40% (see climate events)

---

### FR-CIV-GEO-006: Climate Events & Modulation
**ID:** FR-CIV-GEO-006
**Shall:** Climate events (drought, flood, volcanic eruption, hurricane) occur stochastically per tick based on biome and climate state. Events modify resource renewal rates, cause population displacement, or trigger infrastructure damage.
**Priority:** P1
**Spec Reference:** CIV-0102 (Climate Followup); UJ-3 (Drought event)
**User Journey:** UJ-3 (Drought supply shock)
**Acceptance Criteria:**
- [ ] Event types: DROUGHT (food -40%), FLOOD (water damage, population -10%), ERUPTION (metal +20%, ash fallout -20% vision), HURRICANE (structure damage -50%)
- [ ] Event probability: per_tick_probability = base_probability × climate_severity (varies per biome, time)
- [ ] Event trigger: deterministic RNG per seed; same seed = same event timeline
- [ ] Effect duration: DROUGHT lasts 200 ticks; FLOOD 50 ticks; ERUPTION instantaneous; HURRICANE 10 ticks
- [ ] Recovery: resource renewal_rate returns to base after event ends
- [ ] Cascade: drought → supply_shock (FR-CIV-ECON-006) → migration decisions → institution stress
- [ ] Logging: all events emitted as `climate.event.v1` with type, magnitude, affected_districts

---

### FR-CIV-GEO-007: District Connectivity & Trade Routes
**ID:** FR-CIV-GEO-007
**Shall:** Districts are connected by trade routes (roads, rivers). Road quality affects trade_speed and trade_cost. Trade between districts follows cheapest routes; disconnected districts pay high trade cost (luxury goods only).
**Priority:** P1
**Spec Reference:** CIV-0100 (Markets, trade flows); CIV-0101 (District topology)
**User Journey:** UJ-2 (Map shows trade networks), UJ-3 (Default breaks trade routes)
**Acceptance Criteria:**
- [ ] Road types: DIRT (1.0x cost), STONE (0.7x cost), PAVED (0.5x cost)
- [ ] Trade route: sequence of districts connected by roads. Route cost = sum of segment costs.
- [ ] Cheapest path: system finds minimum-cost trade route between districts (weighted A*)
- [ ] Disconnected: if no road path exists, trade happens via sea (5x cost) or not at all (embargo)
- [ ] Trade modulation: good_cost_to_buyer = base_price × route_cost_multiplier
- [ ] Determinism: identical district topology + road layout → identical trade routes and costs

---

### FR-CIV-GEO-008: Population Density & Urban Growth
**ID:** FR-CIV-GEO-008
**Shall:** Districts have population_density (population / area). High density (>100 per hex) triggers urbanization: spawn additional institutions, infrastructure demand rises, disease risk increases. Low density (<10 per hex) risks depopulation.
**Priority:** P1
**Spec Reference:** CIV-0103 (Citizen Lifecycle); CIV-0101 (District properties)
**User Journey:** UJ-3 (Migration driven by density differences)
**Acceptance Criteria:**
- [ ] Density calculation: population_density = total_population / district_area_hexes
- [ ] Urbanization trigger: density > 100 → auto-spawn government institution, spawn market
- [ ] Infrastructure demand: high density increases required housing, roads, water supply per capita
- [ ] Disease risk: density > 150 → disease.outbreak_probability increases by +5% per tick
- [ ] Depopulation: density < 10 → no population growth; immigration blocked; emigration accelerated
- [ ] Growth limit: natural max density = 200 (extremely crowded); population growth fails above this

---

### FR-CIV-GEO-009: Disaster Zones & Recovery
**ID:** FR-CIV-GEO-009
**Shall:** Major disasters (earthquakes, pandemics, wars) create disaster zones. Districts in disaster zones lose production (-50%), suffer population loss, and incur recovery costs. Recovery over time (50 ticks) restores productivity.
**Priority:** P2
**Spec Reference:** CIV-0102 (Climate); CIV-0103 (Citizen Lifecycle stress)
**User Journey:** UJ-3 (Extended crisis with recovery phase)
**Acceptance Criteria:**
- [ ] Disaster zone property: {district_id, disaster_type, start_tick, recovery_progress}
- [ ] Production penalty: production_mult = 0.5 while in disaster zone
- [ ] Population loss: immediate 20% population death (disaster.v1 event)
- [ ] Recovery: recovery_progress increments 2% per tick. At 100%, disaster_zone removed and production restored
- [ ] Recovery cost: state must invest resources to accelerate recovery (optional policy)
- [ ] Determinism: identical disaster state → identical recovery timeline and output

---

### FR-CIV-GEO-010: LOD Rendering Contract & Data Schema
**ID:** FR-CIV-GEO-010
**Shall:** Zoom level transitions SHALL emit `metrics.lod_snapshot.v1` events with versioned LOD mappings. Data schema for each zoom level is explicit and frozen (schema versioning). Client receives only zoom-appropriate data to reduce bandwidth.
**Priority:** P0
**Spec Reference:** CIV-0101 (Two-Zoom LOD Spec, Sections "Interfaces" and "Data Model")
**User Journey:** UJ-2 (Godot zoom transitions), UJ-4 (Strategic ↔ Tactical zoom)
**Acceptance Criteria:**
- [ ] Zoom level 1 (strategic): {region_id, aggregated_population, aggregated_resources, military_unit_count, dominant_institution, diplomatic_status}
- [ ] Zoom level 2 (tactical): {district_id, population_cohorts, resource_stocks, structures, military_units, citizen_morale}
- [ ] Zoom level 3 (sim): {citizen_id, job, welfare, ideology, location, stress_score} (research mode only)
- [ ] Data size: zoom 1 snapshot ~0.5 KB per region; zoom 2 ~2 KB per district
- [ ] Schema versioning: `lod_snapshots.schema_version` tracks mapping version. Client rejects mismatched versions.
- [ ] Event: `metrics.lod_snapshot.v1` emitted per zoom transition or on demand (client refresh)
- [ ] Determinism: identical state → identical LOD snapshot (byte-stable if using deterministic serialization)

---

## DOMAIN: Actor & Citizen Lifecycle (FR-CIV-ACT-*)

### FR-CIV-ACT-001: Citizen Birth & Initialization
**ID:** FR-CIV-ACT-001
**Shall:** Citizens are born per tick based on population growth rate (fertility rate × population × time_in_tick). New citizens initialize with random traits (job_preference, ideology, health_status) drawn from distribution.
**Priority:** P0
**Spec Reference:** CIV-0103 (Citizen Lifecycle Model)
**User Journey:** UJ-1 (Population growth visible in metrics)
**Acceptance Criteria:**
- [ ] Birth rate: births_per_tick = population × fertility_rate / 365 (assuming 365-tick year)
- [ ] Initialization: new citizen gets {citizen_id, birth_tick, job_preference, ideology, health_status, location, age=0}
- [ ] Traits: job_preference ∈ {farmer, warrior, scholar, trader, priest} drawn from district preference distribution
- [ ] Ideology: random draw from normal distribution N(mean_ideology, stdev); allows diversity
- [ ] Health: health_status ∈ {healthy, sickly, wounded}; drawn from population health distribution
- [ ] Determinism: given population, fertility_rate, and RNG seed → identical births (same IDs, traits)
- [ ] Logging: birth events emitted as `citizen.born.v1` with all initialization data

---

### FR-CIV-ACT-002: Citizen Aging & Mortality
**ID:** FR-CIV-ACT-002
**Shall:** Each citizen has age (increments per tick). Mortality probability increases with age: baseline μ(age) = 0.001 per year; at age 65+, μ increases exponentially. Citizens die when age > max_lifespan (random draw from distribution, typically 75 years).
**Priority:** P0
**Spec Reference:** CIV-0103 (Citizen Lifecycle)
**User Journey:** UJ-1 (Population dynamics)
**Acceptance Criteria:**
- [ ] Age increment: age += 1/365 per tick (assuming 365-tick year)
- [ ] Mortality rate: μ(age) = 0.001 × (1.1)^(age-20) for age ≥ 20; μ = 0.01 for age < 5 (infant mortality)
- [ ] Lifespan: max_lifespan drawn from Gompertz distribution, typically 75 years (σ=8 years)
- [ ] Death trigger: age > max_lifespan or random death_check passes (roll < mortality_rate)
- [ ] Removal: citizen removed from simulation, death event emitted
- [ ] Population impact: total population decreases; age distribution skews younger if births > deaths
- [ ] Determinism: given age cohorts and seed → identical mortality (same individuals die in same tick)

---

### FR-CIV-ACT-003: Citizen Job Assignment & Work
**ID:** FR-CIV-ACT-003
**Shall:** Citizens are assigned jobs (farmer, warrior, scholar, trader, priest, admin). Job assignment is driven by district labor demand and citizen job_preference. Work output varies by job: farmers produce food, warriors provide military power, scholars advance tech, etc.
**Priority:** P0
**Spec Reference:** CIV-0100 (Production capacity), CIV-0107 (Work output in joule economy)
**User Journey:** UJ-1 (Job assignments affect production)
**Acceptance Criteria:**
- [ ] Job types: FARMER, WARRIOR, SCHOLAR, TRADER, PRIEST, ADMIN
- [ ] Assignment logic: priority given to citizens with matching job_preference. Excess citizens assigned to lowest-demand jobs.
- [ ] Assignment determinism: identical labor demand + citizen cohort + seed → identical job distribution
- [ ] Work output: farmer produces food_output (units/tick), warrior = military_power (1 unit per soldier), scholar = tech_advance_rate (% per tick), etc.
- [ ] Unemployment: if jobs < citizens, some remain unemployed (0 output, low welfare, may migrate)
- [ ] Job change: citizens can switch jobs (retraining cost in welfare/time), but prefer stability (high switching cost)
- [ ] Logging: job assignment changes emit `citizen.job_changed.v1` event

---

### FR-CIV-ACT-004: Citizen Welfare & Stress
**ID:** FR-CIV-ACT-004
**Shall:** Each citizen has welfare score (consumption - production) and stress score (unmet needs, coercion, inequality). Welfare in [0, 1]; stress in [0, 1]. High stress increases dissent probability; low welfare increases migration probability.
**Priority:** P0
**Spec Reference:** CIV-0103 (Lifecycle drivers); CIV-0100 (Consumption/welfare)
**User Journey:** UJ-3 (Welfare drop triggers migration)
**Acceptance Criteria:**
- [ ] Welfare calculation: welfare_i = (consumption_i - baseline_need) / max_consumption; bounded [0, 1]
- [ ] Stress calculation: stress_i = weight_unmet × unmet_fraction + weight_coercion × coercion_level + weight_inequality × local_gini
- [ ] Stress weights: default {0.4, 0.3, 0.3}, configurable via policy
- [ ] Migration probability: P_migrate = stress_i × 0.05 per tick (5% baseline, scales with stress)
- [ ] Dissent probability: P_dissent = stress_i × 0.02 per tick (2% baseline, joins dissent cohort)
- [ ] Logging: welfare changes emitted per tick as `citizen.welfare_changed.v1` if delta > threshold
- [ ] Determinism: identical consumption + stress inputs + seed → identical welfare and dissent decisions

---

### FR-CIV-ACT-005: Citizen Migration & Mobility Constraints
**ID:** FR-CIV-ACT-005
**Shall:** Citizens can migrate to other districts if welfare < threshold and destination has available housing. Migration can be blocked by policy (closed borders, curfew) or geography (no road connection). Migration cost: 1 tick travel time + welfare penalty.
**Priority:** P0
**Spec Reference:** CIV-0103 (Migration drivers); CIV-0101 (District connectivity)
**User Journey:** UJ-3 (Drought-triggered migration wave)
**Acceptance Criteria:**
- [ ] Migration trigger: welfare < 0.4 and stress > 0.5 → citizen evaluates migration (not all stressed citizens migrate)
- [ ] Destination selection: citizen evaluates all connected districts; chooses highest_welfare district within 5-hex distance
- [ ] Migration cost: travel_time = distance / movement_speed = 1-10 ticks; arrival in destination at next tick
- [ ] Housing constraint: destination must have available_housing > 0 (cannot exceed capacity)
- [ ] Blocked migration: if closed_border policy active, migration rejected; citizen stays (stress increases)
- [ ] Logistics: during travel, citizen is in "migrating" lifecycle stage (not working, not home)
- [ ] Determinism: identical welfare + destination choices + seed → identical migration decisions
- [ ] Logging: `citizen.migrated.v1` event emitted with origin, destination, travel_time

---

### FR-CIV-ACT-006: Citizen Retirement & Lifecycle Completion
**ID:** FR-CIV-ACT-006
**Shall:** Citizens retire when: (a) age > retirement_age (default 65), (b) accumulated wealth ≥ retirement_wealth_threshold (joule economy), or (c) health declines (injured/sickly). Retired citizens receive pension (basic income), stop working, and remain as population.
**Priority:** P1
**Spec Reference:** CIV-0103 (Lifecycle stages); CIV-0107 (Joule economy, retirement threshold)
**User Journey:** UJ-5 (Joule economy retirement mechanics)
**Acceptance Criteria:**
- [ ] Retirement triggers: (a) age ≥ 65, (b) lifetime_wealth ≥ retirement_threshold (5000 joules in joule economy), (c) health_status = critically_wounded
- [ ] Retirement stage: citizen transitions to RETIRED lifecycle stage (no job assignment)
- [ ] Pension: retired citizen receives 10 units/tick basic income (from state treasury)
- [ ] Production: retired citizen produces 0 (not working)
- [ ] Lifespan: retired citizens continue aging until death (typical lifespan extends to 75-80)
- [ ] Economy impact: population is supported but produces nothing (drain on resources)
- [ ] Logging: `citizen.retired.v1` event emitted; citizen visible in population pyramid as 65+ cohort

---

### FR-CIV-ACT-007: Ideology & Political Alignment
**ID:** FR-CIV-ACT-007
**Shall:** Each citizen has ideology ∈ [-1, 1] where -1 = libertarian, 0 = centrist, 1 = authoritarian. Ideology influences: job preference, migration decisions, support for policies, and institution loyalty. Policy-citizen ideology mismatch increases dissent.
**Priority:** P1
**Spec Reference:** CIV-0103 (Citizen traits); CIV-0106 (Social dynamics) [assumed]
**User Journey:** UJ-3 (Crisis stresses ideology-mismatched citizens more)
**Acceptance Criteria:**
- [ ] Ideology initialization: drawn from N(mean_ideology, stdev=0.3) per district
- [ ] Ideology evolution: can shift based on experienced policies (e.g., high taxes push libertarian)
- [ ] Policy-citizen mismatch: policy_score_i = 1 - abs(citizen.ideology - policy.ideology) × weight; low score increases dissent
- [ ] Institutional preference: citizen supports institutions matching their ideology
- [ ] Determinism: given initial ideology and policy sequence → ideology path is deterministic
- [ ] Logging: ideology changes emitted as `citizen.ideology_shifted.v1` when delta > threshold

---

### FR-CIV-ACT-008: Health & Disease
**ID:** FR-CIV-ACT-008
**Shall:** Citizens have health_status ∈ {healthy, sickly, wounded, diseased}. Health degrades from age, disease outbreaks, malnutrition. Sickly citizens produce 50% output. Disease spreads stochastically within districts (contact transmission).
**Priority:** P1
**Spec Reference:** CIV-0103 (Health-related stress); CIV-0102 (Disease outbreaks)
**User Journey:** UJ-1 (Disease outbreaks visible in metrics)
**Acceptance Criteria:**
- [ ] Health progression: healthy → sickly (random, age > 50 increases probability) → recovered (tick counter) or diseased (disease outbreak)
- [ ] Production impact: sickly citizen produces 0.5x output; diseased produces 0.1x
- [ ] Disease mechanics: outbreak probability per district = base_rate × population_density × sanitation_modifier
- [ ] Transmission: diseased citizen contacts 3 neighbors (stochastic); each contact has 20% infection probability
- [ ] Recovery: sickly citizen recovers in 50 ticks (deterministic); diseased recovers in 200 ticks (or dies)
- [ ] Mortality: diseased citizen has 5% death probability per tick (cumulative)
- [ ] Logging: `health.disease_outbreak.v1` event emitted per district when outbreak starts; `citizen.health_changed.v1` per individual

---

### FR-CIV-ACT-009: Skill & Expertise Development
**ID:** FR-CIV-ACT-009
**Shall:** Citizens develop skills (farming, combat, scholarship) over time. Skill level (0-10) increases 0.01 per tick of work in that job. Higher skill increases job output (1% bonus per skill level) and unlock advanced jobs.
**Priority:** P2
**Spec Reference:** CIV-0103 (Citizen lifecycle); Career progression
**User Journey:** UJ-1 (Long-term pop develops specialized workforce)
**Acceptance Criteria:**
- [ ] Skill types: FARMING, COMBAT, SCHOLARSHIP, TRADING, ADMINISTRATION
- [ ] Skill gain: skill_level += 0.01 per tick of relevant work
- [ ] Skill max: 10 (diminishing returns after level 8)
- [ ] Output bonus: job_output_multiplier = 1 + (skill_level × 0.01) [e.g., skill 5 = 5% bonus]
- [ ] Advanced jobs: unlocked at skill_level > 5 (e.g., general, master scholar, trade minister)
- [ ] Skill transfer: when citizen changes jobs, retains skill (transarable across related jobs)
- [ ] Determinism: identical job history → identical skill levels and output bonuses

---

### FR-CIV-ACT-010: Cohort Analytics & Demographics
**ID:** FR-CIV-ACT-010
**Shall:** Population is aggregated into cohorts (age ranges, job, ideology) for analytics. Cohort data is emitted per tick for metrics. Cohort statistics (fertility, mortality, skill distribution) feed into population dynamics.
**Priority:** P1
**Spec Reference:** CIV-0103 (Time-series); CIV-0101 (LOD aggregation at zoom 1)
**User Journey:** UJ-1 (Population pyramid in metrics), UJ-5 (Demographics comparison)
**Acceptance Criteria:**
- [ ] Cohort definition: {age_range: [min, max), job_type, ideology_range}
- [ ] Cohorts emitted per tick in `citizen_lifecycle` table: (run_id, tick, cohort_id, stage, stress_score, count)
- [ ] Aggregate statistics: fertility_rate, mortality_rate, avg_skill, avg_stress per cohort
- [ ] Population pyramid: vertical bar chart of cohorts (age on Y, count on X)
- [ ] Determinism: identical population → identical cohort counts and statistics
- [ ] Analytics: researcher can export cohort timeline and plot demographic shifts

---

### FR-CIV-ACT-011: Citizen Dissent & Rebellion
**ID:** FR-CIV-ACT-011
**Shall:** Dissenting citizens form coalitions and may rebel against state authority. Rebellion risk is driven by legitimacy (institutional + leadership) and coercion (enforcement level). High rebellion risk triggers protest events, blockades, or armed insurgency.
**Priority:** P2
**Spec Reference:** CIV-0105 (Insurgency modifiers); CIV-0103 (Dissent drivers)
**User Journey:** UJ-3 (Crisis triggers dissent, escalates to rebellion)
**Acceptance Criteria:**
- [ ] Dissent mechanics: citizens with high stress can transition to "dissenting" lifecycle stage
- [ ] Rebellion risk: R_rebellion = dissent_count / population × legitimacy_malus × coercion_malus
- [ ] Protest events: R_protest ∈ [0.05, 0.5] per tick; triggers `dissent.protest.v1` event, blocks some production
- [ ] Insurgency: R_insurgency ∈ [0.01, 0.1]; triggers armed insurgency units (attack infrastructure)
- [ ] Coercion response: enforcement troops can suppress protests (reduce R_protest at morale cost)
- [ ] Determinism: identical state + seed → identical dissent and rebellion decisions

---

### FR-CIV-ACT-012: Leadership & Power Structures
**ID:** FR-CIV-ACT-012
**Shall:** Institutions have leaders (citizens designated as administrators, generals, bishops). Leaders have personal traits (competence, corruption, loyalty) that affect institutional effectiveness and citizen trust. Leader death or removal triggers succession crisis.
**Priority:** P2
**Spec Reference:** CIV-0103 (Institutions); CIV-0105 (War & Diplomacy leaders)
**User Journey:** UJ-4 (Leader assassination as espionage option)
**Acceptance Criteria:**
- [ ] Leader struct: {leader_id, title, institution_id, competence [0-10], corruption [0-1], loyalty [0-1], age, health}
- [ ] Effectiveness: institutional_output_mult = 1 - corruption × 0.5 + competence × 0.1
- [ ] Citizen trust: trust_in_leader = competence - corruption, affects institutional legitimacy
- [ ] Succession: when leader dies or removed, institution selects new leader from pool (random draw or promotion)
- [ ] Succession crisis: new leader with low competence may trigger legitimacy drop
- [ ] Determinism: identical leader pool + selection seed → identical leader assignments

---

### FR-CIV-ACT-013: Family Units & Kinship
**ID:** FR-CIV-ACT-013
**Shall:** Citizens can form family units (optional for deep sim). Family members have kinship relationships that affect: migration decisions (family follows), inheritance of wealth, support obligations. Family units may have shared objectives.
**Priority:** P2
**Spec Reference:** CIV-0103 (Advanced citizen model)
**User Journey:** UJ-1 (Deep genealogy tracking in research mode)
**Acceptance Criteria:**
- [ ] Family struct: {family_id, patriarch_id, members: [citizen_ids], shared_assets}
- [ ] Marriage: citizens form monogamous pairs (probability increases with age 18-40); eligible if opposite ideology within 0.3
- [ ] Children: married couples produce children (fertility rate higher for married couples)
- [ ] Kinship effects: family members more likely to migrate together (family_cohesion_modifier)
- [ ] Inheritance: family assets distributed to family_id when patriarch dies
- [ ] Determinism: identical population + kinship rules + seed → identical families and marriages

---

### FR-CIV-ACT-014: Citizen Genealogy & Lineage Tracking
**ID:** FR-CIV-ACT-014
**Shall:** The system SHALL track complete genealogy: parent-child relationships, marriages, and lineage lines. Genealogy data is exported for research analysis (long-term family success, genetic traits, lineage drift).
**Priority:** P2
**Spec Reference:** CIV-0103 (Time-series, citizen lifecycle); UJ-1 (Export genealogy)
**User Journey:** UJ-1 (Genealogy export for research)
**Acceptance Criteria:**
- [ ] Genealogy table: {run_id, parent_citizen_id, child_citizen_id, relationship_type: BIOLOGICAL|ADOPTED}
- [ ] Export: `civ-sim run export --run-id <id> --format actor-genealogy` outputs complete family tree
- [ ] Analytics: researcher can query "descendants of citizen_id" and trace lineage across 100+ generations
- [ ] Determinism: identical population and reproduction rules → identical genealogy
- [ ] Performance: genealogy queries < 100 ms for trees with 10k+ individuals

---

### FR-CIV-ACT-015: Career & Specialization Paths
**ID:** FR-CIV-ACT-015
**Shall:** Citizens can specialize in career paths (e.g., farmer → master farmer → agricultural advisor → trade minister). Specialization unlocks job options and increases output. Career transitions have costs (retraining time, welfare penalty).
**Priority:** P2
**Spec Reference:** CIV-0103 (Citizen job assignment); CIV-0107 (Work output)
**User Journey:** UJ-1 (Workforce develops specialization over time)
**Acceptance Criteria:**
- [ ] Career paths: define skill prerequisites and unlocked jobs per level (e.g., farming 5 → master farmer role)
- [ ] Specialization bonus: master/specialist jobs have +20% output multiplier
- [ ] Transition cost: switching specialization requires 20-tick retraining period; welfare penalty during retraining
- [ ] Skill carryover: specialization skills are non-transferable (master farmer cannot become master scholar)
- [ ] Determinism: identical career history + seed → identical specialization outcomes

---

## DOMAIN: War, Diplomacy, Shadow Networks (FR-CIV-WAR-*)

### FR-CIV-WAR-001: Diplomatic States & Transitions
**ID:** FR-CIV-WAR-001
**Shall:** Diplomatic relationships between factions are modeled as finite-state machine: COOPERATIVE, STRAINED, SANCTIONED, ESCALATING, ACTIVE_CONFLICT, DEESCALATING. Transitions triggered by resource stress, coalition breakage, enforcement overreach, shadow financing flows.
**Priority:** P0
**Spec Reference:** CIV-0105 (War, Diplomacy, Shadow Networks Spec)
**User Journey:** UJ-4 (Diplomacy decisions)
**Acceptance Criteria:**
- [ ] State enum: COOPERATIVE, STRAINED, SANCTIONED, ESCALATING, ACTIVE_CONFLICT, DEESCALATING
- [ ] Transition triggers: resource_stress (food shortage) → STRAINED; coalition_conflict → ESCALATING; military_attack → ACTIVE_CONFLICT
- [ ] Transition rate: state_duration += 1 tick; transitions checked per tick per faction pair (sorted stable order)
- [ ] Output effects: COOPERATIVE = +50% trade, ACTIVE_CONFLICT = no trade (embargo), SANCTIONED = trade ÷ 2, -50% credit availability
- [ ] Determinism: identical faction state + policy + seed → identical diplomatic trajectory
- [ ] Logging: `diplomacy.state_changed.v1` emitted per transition with pressure_score, reason

---

### FR-CIV-WAR-002: Conflict State & Military Balance
**ID:** FR-CIV-WAR-002
**Shall:** When factions enter ACTIVE_CONFLICT, military balance is calculated: military_strength_ratio = faction_A_military / (faction_B_military + ε). Military strength includes: unit count, unit experience, morale, supply status. Imbalanced conflicts trigger escalation or surrender offers.
**Priority:** P0
**Spec Reference:** CIV-0105 (Conflict mechanics); UJ-4 (AI response to military imbalance)
**User Journey:** UJ-4 (Military balance displayed, AI responds strategically)
**Acceptance Criteria:**
- [ ] Military strength formula: strength = Σ(unit_count × experience_mult × morale_mult × supply_mult)
- [ ] Balance ratio: ratio = own_strength / enemy_strength (1.0 = parity, >1.0 = advantage)
- [ ] Escalation: if ratio > 2.0, attacker escalates (requests unconditional surrender); if ratio < 0.5, defender seeks peace
- [ ] Attrition rate: units lose morale per tick of conflict (−2% baseline); supply depletion accelerates casualties
- [ ] Determinism: identical military states → identical balance calculations and AI responses
- [ ] Logging: `military.balance_calculated.v1` emitted per tick during conflict with ratio, projections

---

### FR-CIV-WAR-003: Sanction & Pressure Mechanics
**ID:** FR-CIV-WAR-003
**Shall:** Sanctioned factions suffer: trade embargoes (-50% import/export), credit cost rises (+50%), and technology transfer blocked. Sanction pressure accumulates over time; sanctions last until diplomatic reset or terms accepted.
**Priority:** P0
**Spec Reference:** CIV-0105 (Sanctions & leakage)
**User Journey:** UJ-4 (Diplomacy panel shows sanctions)
**Acceptance Criteria:**
- [ ] Sanction trigger: faction behavior violates coalition norms (e.g., excessive military buildup, human rights abuses)
- [ ] Sanction effects: trade_availability *= 0.5, credit_cost *= 1.5, tech_transfer = blocked
- [ ] Pressure accumulation: pressure_score += 10 per tick; at pressure > 100, targeted faction must accept terms or escalate conflict
- [ ] Leakage: 10-20% of sanctioned imports leak through gray market (illicit channels, see FR-CIV-WAR-010)
- [ ] Sanction lift: faction accepts terms or pressure drops to 0 (coalition consensus lost)
- [ ] Determinism: identical sanction trigger + duration → identical pressure trajectory

---

### FR-CIV-WAR-004: Coalition Mechanics & Collective Security
**ID:** FR-CIV-WAR-004
**Shall:** Factions can form coalitions (alliances of 2+ factions). Coalition members commit to mutual defense: if one member attacked, others join the conflict. Coalition stability is affected by shared interests and external pressure.
**Priority:** P1
**Spec Reference:** CIV-0105 (Conflict resolution)
**User Journey:** UJ-4 (Coalition formation affects military balance)
**Acceptance Criteria:**
- [ ] Coalition struct: {coalition_id, members: [faction_ids], shared_goals, stability_score}
- [ ] Mutual defense: if member_A attacked by non-member, other members join conflict within 5 ticks
- [ ] Stability: stability_score = 1.0 - Σ(ideology_mismatch_ij) / (n choose 2); members must share values to stay cohesive
- [ ] Breakdown: if stability < 0.3, coalition dissolves (members can exit)
- [ ] Benefit: coalition members get +20% military coordination (combined units fight better)
- [ ] Determinism: identical coalition state + policy → identical mutual defense decisions

---

### FR-CIV-WAR-005: Peace Treaties & War Termination
**ID:** FR-CIV-WAR-005
**Shall:** Active conflicts can end via: surrender (losing faction gives 20% of resources to winner), peace treaty (mutual agreement with terms), or stalemate (neither side can achieve victory, both exhaust resources after 500 ticks). Peace treaties specify reparations, territory transfer, trade terms.
**Priority:** P1
**Spec Reference:** CIV-0105 (War termination); UJ-4 (Siege surrender)
**User Journey:** UJ-4 (War termination options)
**Acceptance Criteria:**
- [ ] Surrender condition: military_strength_ratio < 0.2 for 50 consecutive ticks → losing faction offers surrender
- [ ] Surrender terms: winner receives 20% of loser's resources + war reparations for N years (20% of income)
- [ ] Peace treaty: mutual agreement with treaty_terms = {reparations, trade_agreement, non_aggression_duration}
- [ ] Stalemate: if conflict lasts > 500 ticks and neither winning, automatic peace proposal (neutral mediator role)
- [ ] Treaty enforcement: violating treaty (resuming hostilities) triggers sanctions from other factions
- [ ] Determinism: identical war state → identical termination decision and treaty terms
- [ ] Logging: `military.treaty_signed.v1` emitted with treaty_terms and signatories

---

### FR-CIV-WAR-006: Enforcement & Overreach
**ID:** FR-CIV-WAR-006
**Shall:** Factions with high enforcement intensity (police, military) suppress dissent and insurgency but at cost: enforcement_overreach score increases. High overreach causes legitimacy_loss (−10% per tick) and accelerates dissent (dissent_probability × 2).
**Priority:** P1
**Spec Reference:** CIV-0105 (Enforcement & legitimacy); CIV-0103 (Insurgency modifiers)
**User Journey:** UJ-3 (High enforcement triggers crisis escalation)
**Acceptance Criteria:**
- [ ] Enforcement_intensity: faction policy parameter [0, 1] controlling police/military deployment
- [ ] Effect: dissent_reduction = enforcement_intensity × 0.5 (50% of dissent suppressed at max intensity)
- [ ] Overreach accumulation: overreach_score += enforcement_intensity × 0.1 per tick
- [ ] Legitimacy penalty: legitimacy_mult = 1 - (overreach_score / 100) (at overreach = 100, legitimacy = 0)
- [ ] Dissent response: at overreach > 50, dissent_probability × (1 + overreach/100) (doubles at overreach = 100)
- [ ] Overreach decay: overreach_score -= 0.05 per tick when enforcement_intensity < 0.3 (enforcer reduction)
- [ ] Determinism: identical enforcement + legitimacy baseline + seed → identical overreach trajectory

---

### FR-CIV-WAR-007: Shadow Networks & Covert Influence
**ID:** FR-CIV-WAR-007
**Shall:** Factions can fund shadow networks (covert operatives, propaganda, subversion). Shadow flows are stochastic and hard to detect; detection_score increases per tick based on flow_amount. Detected networks face counter-intelligence operations.
**Priority:** P1
**Spec Reference:** CIV-0105 (Shadow-network behavior)
**User Journey:** UJ-4 (Espionage panel, shadow operations)
**Acceptance Criteria:**
- [ ] Shadow network struct: {network_id, sponsor_faction_id, target_faction_id, activity_type, flow_amount, detection_score}
- [ ] Activity types: PROPAGANDA, SABOTAGE, INFILTRATION, BRIBERY
- [ ] Flow mechanics: flow_amount spent per tick (deducted from sponsor treasury); increases target detection_score
- [ ] Detection: detection_score = flow_amount × (1 - target_counterintelligence_capacity)
- [ ] Exposure: if detection_score > threshold (100), network exposed → `shadow.network_exposed.v1` event
- [ ] Effects: PROPAGANDA boosts ideology drift; SABOTAGE damages structures; INFILTRATION reveals intelligence; BRIBERY turns officials
- [ ] Counter-intel: target faction can spend resources on counterintelligence (reduces detection_score, disrupts networks)
- [ ] Determinism: identical shadow state + seed → identical detection and exposure timings

---

### FR-CIV-WAR-008: Intelligence & Espionage Mechanics
**ID:** FR-CIV-WAR-008
**Shall:** Factions can gather intelligence on enemies: unit strength, economic state, institutional stability, research progress. Intelligence is obtained via scouts, spies, or diplomatic channels. Intelligence quality decays over time (stale intelligence).
**Priority:** P2
**Spec Reference:** CIV-0105 (Geopolitics); UJ-4 (Scout reveals positions)
**User Journey:** UJ-4 (Scout intelligence gathering)
**Acceptance Criteria:**
- [ ] Intelligence struct: {faction_id, intel_type, target_faction_id, data, quality [0-1], age_ticks}
- [ ] Intel types: MILITARY_STRENGTH, ECONOMIC_STATE, INSTITUTIONAL, TECHNOLOGY, POPULATION
- [ ] Gathering: scouts get free intel on visible units; spies get targeted intel (risky, detection_score increases)
- [ ] Quality decay: quality -= 0.01 per tick (half-life = 100 ticks); age = current_tick - gathered_tick
- [ ] Obsolescence: intel > 200 ticks old is deleted (unreliable)
- [ ] Accuracy: actual enemy state vs intelligence state may differ; margin of error = (1 - quality) × actual_value
- [ ] Determinism: identical gathering actions + seed → identical intel quality and accuracy

---

### FR-CIV-WAR-009: Insurgency & Guerrilla Tactics
**ID:** FR-CIV-WAR-009
**Shall:** Highly dissenting populations may spontaneously generate insurgency units (ragtag guerrilla fighters). Insurgents cannot match regular military but are highly resilient (heal in place, no supply line needed). Insurgency risk driven by legitimacy and enforcement overreach.
**Priority:** P2
**Spec Reference:** CIV-0105 (Insurgency modifiers); CIV-0103 (Dissent)
**User Journey:** UJ-3 (Crisis triggers insurgency)
**Acceptance Criteria:**
- [ ] Insurgency trigger: dissent_count > population × 0.2 AND legitimacy < 0.3 → insurgency risk > 0
- [ ] Unit spawning: R_spawn = dissent_fraction × (1 - legitimacy) × enforcement_modifier; per 100 ticks, spawns insurgent units
- [ ] Unit stats: insurgent_HP = 30 (vs regular soldier 50), attack = 20 (vs 25), morale = high (fanatical, -5% per tick vs -2%)
- [ ] Healing: insurgents auto-heal 2 HP/tick if not engaged (vs regular soldiers require repair)
- [ ] Supply: no supply line needed (fight on local resources)
- [ ] Tactics: guerrilla attacks target supply depots, roads (reduce trade routes)
- [ ] Determinism: identical dissent + legitimacy + seed → identical insurgency timeline

---

### FR-CIV-WAR-010: Shadow Market & Sanctioned Goods Leakage
**ID:** FR-CIV-WAR-010
**Shall:** When factions are sanctioned, embargoed goods leak through shadow markets. Leakage rate is stochastic and depends on enforcement capability (higher enforcement → lower leakage). Shadow goods have inflated prices (2-5x normal).
**Priority:** P2
**Spec Reference:** CIV-0105 (Leakage mechanics); CIV-0100 (Markets & pricing)
**User Journey:** UJ-3 (Default triggers embargo, shadow trade emerges)
**Acceptance Criteria:**
- [ ] Leakage mechanics: L = embargo_fraction × (1 - enforcement_effectiveness) × good_scarcity_factor
- [ ] Leakage rate: up to 20% of embargoed goods leak per tick (if enforcement is weak)
- [ ] Shadow pricing: shadow_price = normal_price × [2.0, 5.0] range (depends on scarcity)
- [ ] Shadow trade events: `economy.shadow_trade.v1` emitted with good_type, quantity, shadow_price, buyer_faction
- [ ] Detection: each shadow transaction has detection_score accumulation (similar to shadow networks)
- [ ] Determinism: identical embargo + enforcement + seed → identical leakage trajectory
- [ ] Analytics: researcher can export shadow_trade log to study sanctions effectiveness

---

### FR-CIV-WAR-011: Attrition & Resource Depletion in Conflict
**ID:** FR-CIV-WAR-011
**Shall:** Long conflicts deplete attacker and defender resources. Attrition_rate per tick depends on conflict_intensity (engagement frequency) and supply_availability. Units in active engagement lose morale −2% per tick; units in shortage lose −5% per tick.
**Priority:** P0
**Spec Reference:** CIV-0105 (War mechanics); FR-CIV-RTS-005 (Supply logistics)
**User Journey:** UJ-4 (Extended war causes attrition, both sides weaken)
**Acceptance Criteria:**
- [ ] Attrition calculation: casualties = unit_count × casualty_rate_per_tick; casualty_rate = base_rate × (1 + intensity_factor)
- [ ] Morale loss: morale -= 2% per tick in active engagement; −5% if supply_status < 25%
- [ ] Casualty routing: when morale < 20%, unit routes (flees combat, becomes unavailable for 50 ticks)
- [ ] Supply depletion: units consume supply per tick; shortage increases casualty rate
- [ ] Recovery: routed units recover morale at −5% per tick (return to duty after 20-tick rest)
- [ ] Determinism: identical unit state + engagement intensity → identical attrition

---

### FR-CIV-WAR-012: Geopolitical Events & Crises
**ID:** FR-CIV-WAR-012
**Shall:** Geopolitical events occur stochastically per tick: border disputes, trade wars, succession crises, natural disasters affecting allies. Events trigger faction responses and can escalate to conflict. Events are logged and traceable to causal chains.
**Priority:** P2
**Spec Reference:** CIV-0105 (Conflict triggers); UJ-3 (Event cascades)
**User Journey:** UJ-3 (Geopolitical event chains)
**Acceptance Criteria:**
- [ ] Event types: BORDER_DISPUTE, TRADE_WAR, SUCCESSION_CRISIS, NATURAL_DISASTER, RELIGIOUS_SCHISM, ECONOMIC_CRISIS
- [ ] Event probability: per_tick_probability = base_rate × tension_level × stability_modifier
- [ ] Faction response: event triggers diplomatic/military response based on faction policy
- [ ] Cascade: event can trigger secondary events (e.g., SUCCESSION_CRISIS → BORDER_DISPUTE as new leader asserts authority)
- [ ] Logging: `geopolitics.event.v1` emitted with event_type, trigger_faction, affected_factions, consequence
- [ ] Determinism: identical geopolitical state + seed → identical event triggers and responses

---

## DOMAIN: Research & Sandbox API (FR-CIV-RES-*)

### FR-CIV-RES-001: Scenario Configuration & Loading
**ID:** FR-CIV-RES-001
**Shall:** Scenarios are specified in YAML with full simulation parameters: geography (map_seed, biome, size), initial_policies, actor roster, resource distribution. Scenario loads and validates against schema. Validation errors reported with actionable messages.
**Priority:** P0
**Spec Reference:** CIV-0100+ (all specs include scenario config sections); UJ-1 (Scenario loading)
**User Journey:** UJ-1 (Scenario loading & validation)
**Acceptance Criteria:**
- [ ] Scenario YAML structure: {name, description, map: {...}, initial_policies: {...}, actors: [{...}], resources: {...}, metadata: {...}}
- [ ] Schema validation: all required fields present, types match, numeric bounds verified
- [ ] Validation errors: non-silent; return full error report with line numbers, field name, constraint violation
- [ ] Canonical scenarios: 5 presets available (temperate-city, island-empire, climate-crisis, economic-collapse, joule-utopia)
- [ ] Custom scenarios: users can create new scenarios by editing templates
- [ ] Determinism: identical scenario YAML + seed → identical simulation (no variations)

---

### FR-CIV-RES-002: Run Execution & Tick Advancement
**ID:** FR-CIV-RES-002
**Shall:** Given a scenario and run config (duration in ticks, seed), the system SHALL execute the simulation: advance tick counter, apply policy, resolve economy, update actors, emit events. Execution is deterministic: same seed → same state trajectory.
**Priority:** P0
**Spec Reference:** CIV-0001 (Core simulation loop, assumed); ADR-003 (Deterministic replay)
**User Journey:** UJ-1 (Run execution)
**Acceptance Criteria:**
- [ ] Tick execution: tick_counter increments from 0 to max_ticks; state transitions per phase order (policy → production → exchange → allocation → update)
- [ ] Tick duration: 40 ms wall-clock per tick (25 Hz) on single CPU; target 200+ ticks/sec with optimization
- [ ] Determinism: given (state_0, policy, seed, max_ticks) → identical state_N (byte-comparison)
- [ ] Checkpointing: can pause and save at any tick; resume from checkpoint advances from same tick
- [ ] Progress reporting: CLI shows tick count, wall-clock elapsed, estimated time remaining
- [ ] Early termination: can stop run early via signal handling (graceful shutdown)

---

### FR-CIV-RES-003: Metrics Snapshot Emission
**ID:** FR-CIV-RES-003
**Shall:** Each tick, the system SHALL emit `metrics_snapshot` containing: population, welfare, Gini, legitimacy, institution states, trade volume, inflation, military strength, supply stress. Snapshots are recorded in time-series format (JSONL) for analysis.
**Priority:** P0
**Spec Reference:** CIV-0100 (metrics schema), CIV-0101 (LOD aggregates), CIV-0103 (institutional metrics), CIV-0105 (military/diplomatic metrics)
**User Journey:** UJ-1 (Metrics export), UJ-3 (Timeline visualization), UJ-5 (Comparative metrics)
**Acceptance Criteria:**
- [ ] Metrics emitted per tick: {tick, population, population_cohorts, avg_welfare, gini, legitimacy, institution_states, trade_volume, inflation, military_strength_per_faction, supply_stress}
- [ ] Format: JSONL (one JSON object per line), write-once append-only
- [ ] Determinism: identical state → identical metrics (floating-point rounding bounded to ±0.0001)
- [ ] Size: typical metrics line = 200 bytes; 10k ticks = ~2 MB
- [ ] Compression: optional GZIP for archive (typical 50% compression ratio)
- [ ] Query interface: can load metrics into memory and query time-series (e.g., `metrics['gini']` → array of Gini per tick)

---

### FR-CIV-RES-004: Event Log & Audit Trail
**ID:** FR-CIV-RES-004
**Shall:** Every material simulation event (policy applied, market cleared, transfer booked, institution transitioned, citizen migrated, conflict event) is recorded in immutable event log. Events include correlation_id for causal tracing.
**Priority:** P0
**Spec Reference:** CIV-0100/0103/0105 (Event contracts per domain); ADR-003 (Deterministic replay)
**User Journey:** UJ-1 (Event log inspection), UJ-3 (Causal analysis), UJ-5 (Mechanism analysis)
**Acceptance Criteria:**
- [ ] Event types: 50+ domain-specific types (policy.applied, economy.market_cleared, citizen.migrated, diplomacy.state_changed, etc.)
- [ ] Event payload: {type, tick, correlation_id, actor_ids, data: {...}, timestamp}
- [ ] Correlation_id: unique per event; enables tracing causal chain (event A.correlation_id → event B.correlation_id)
- [ ] Format: JSONL (one event per line), append-only
- [ ] Size: typical event = 300 bytes; 10k ticks, 10 events/tick = 30 MB
- [ ] Querying: filter events by type, tick range, actor_id, correlation_id chain
- [ ] Causality: reconstruct causal chain by following correlation_id links

---

### FR-CIV-RES-005: Ledger Export & Audit
**ID:** FR-CIV-RES-005
**Shall:** Full ledger trace (all transfers per tick per actor) is exported as CSV for audit. Ledger validates conservation invariant: sum of transfers per currency per tick = 0. Validation report includes any violations and suggests debugging steps.
**Priority:** P0
**Spec Reference:** CIV-0100 (Ledger double-entry accounting)
**User Journey:** UJ-1 (Ledger export & validation), UJ-5 (A/B test ledger comparison)
**Acceptance Criteria:**
- [ ] Ledger CSV: {run_id, tick, from_actor_id, to_actor_id, amount, currency, transfer_type, reason}
- [ ] Export: `civ-sim run export --run-id <id> --format ledger-trace --output ledger.csv`
- [ ] Row count: M transfers × 8 fields; typical 10k ticks → 1M+ rows
- [ ] Validation: sum of all transfers per (currency, tick) must = 0 (within ±0.001 fixed-point tolerance)
- [ ] Violation report: if sum ≠ 0, list suspect transfers (likely candidates for the imbalance)
- [ ] Balance check: per-actor cumulative balance must never go negative (invariant check)
- [ ] Determinism: identical run → identical ledger (byte-stable order, rounding)

---

### FR-CIV-RES-006: Institution & Diplomacy Timeline Export
**ID:** FR-CIV-RES-006
**Shall:** Institution states and diplomatic states are exported as time-series tables (tick, institution_id, state, metrics) and (tick, faction_A_id, faction_B_id, diplomatic_state, pressure_score). Timelines enable state machine visualization.
**Priority:** P0
**Spec Reference:** CIV-0103 (Institution states), CIV-0105 (Diplomatic states)
**User Journey:** UJ-1 (Institution timeline analysis), UJ-3 (Diplomatic crisis tracking)
**Acceptance Criteria:**
- [ ] Institution timeline: {run_id, tick, institution_id, state, legitimacy, capture_score, policy_capacity_mult}
- [ ] Diplomacy timeline: {run_id, tick, faction_a_id, faction_b_id, state, pressure_score, event_reason}
- [ ] State transitions: each state change is a new row; stability periods have constant state rows (no dedupe)
- [ ] Temporal resolution: one row per tick (50k+ rows per time-series in 10k-tick run)
- [ ] Format: Parquet or CSV; Parquet preferred for analyst efficiency
- [ ] Visualization: timeline can be plotted (x=tick, y=state, color=faction); state changes are discontinuities
- [ ] Determinism: identical run → identical timelines (same state transitions at same ticks)

---

### FR-CIV-RES-007: Replay File & Serialization
**ID:** FR-CIV-RES-007
**Shall:** Full simulation state at each tick is optionally recorded to binary replay file (deterministic serialization). Replay file enables rewind/fast-forward operations and offline analysis without re-simulation.
**Priority:** P1
**Spec Reference:** ADR-003 (Deterministic Replay); CIV-0101 (LOD snapshots)
**User Journey:** UJ-1 (Replay export & analysis in Jupyter), UJ-3 (Replay browser with timeline scrubbing)
**Acceptance Criteria:**
- [ ] Replay format: binary, deterministic serialization of full state (population, resources, institutions, diplomacy, etc.)
- [ ] Size: typical 10k-tick run = 50-100 MB (configurable compression)
- [ ] Serialization: use bincode or MessagePack (deterministic, not JSON due to float precision)
- [ ] Indexing: optional tick index table at start of file for O(1) tick lookup
- [ ] API: `Replay::load('replay.civdata')` returns object with `.tick(n)` → full state at tick n
- [ ] Determinism: identical run → identical replay file (bit-for-bit reproducible)
- [ ] Compression: optional zstd compression (10:1 ratio common); 50 MB → 5 MB archived

---

### FR-CIV-RES-008: Scenario Parameter Sweep
**ID:** FR-CIV-RES-008
**Shall:** The system SHALL support batch execution of parameter sweeps: vary one or more parameters (e.g., tax_rate ∈ [0.0, 0.1, 0.2, ...]) and run multiple scenarios in parallel. Results aggregated for sensitivity analysis.
**Priority:** P1
**Spec Reference:** Design doc (P1 Batch execution); UJ-5 (Sensitivity test)
**User Journey:** UJ-5 (Sensitivity analysis: joule rate variation)
**Acceptance Criteria:**
- [ ] Parameter sweep CLI: `civ-sim run sweep --template scenario.yaml --param joule_rate --values "40,50,60" --parallel 3`
- [ ] Execution: spawns 3 independent runs (one per parameter value), CPU-isolated
- [ ] Aggregation: collects metrics from all runs; outputs summary (min/max/mean per metric across sweep)
- [ ] Progress: displays: "Running parameter sweep: 2/3 complete (60%)"
- [ ] Output: summary table and per-run detailed results
- [ ] Determinism: identical parameter values + seed → identical results across runs

---

### FR-CIV-RES-009: Comparative Run Analysis & Diffing
**ID:** FR-CIV-RES-009
**Shall:** The system SHALL support comparing two runs (identical scenario, different allocators or parameters). Diff output shows: which metrics diverged, at which tick, and magnitude of divergence.
**Priority:** P1
**Spec Reference:** UJ-5 (A/B test comparison framework)
**User Journey:** UJ-5 (Market vs Joule comparison)
**Acceptance Criteria:**
- [ ] Diff CLI: `civ-sim run diff --run-id-a market-001 --run-id-b joule-001 --metrics gini,welfare,production`
- [ ] Divergence detection: tick-by-tick comparison of metrics; flag first tick where values differ > 1%
- [ ] Summary table: {metric, tick_divergence, initial_a, initial_b, final_a, final_b, effect_size}
- [ ] Visualization: side-by-side line plots of diverged metrics
- [ ] Root cause: if divergence is likely due to allocator, suggest FR reference (e.g., "See FR-CIV-ECON-002 for market clearing")
- [ ] Determinism: runs with identical scenarios show zero divergence (byte-identical metrics)

---

### FR-CIV-RES-010: Reproducibility Package & Citation
**ID:** FR-CIV-RES-010
**Shall:** For any run, the system SHALL generate a reproducibility package: scenario YAML, random seed, git commit hash of CivLab code, timestamp, parameters. Researcher can publish package and others can reproduce identical results.
**Priority:** P1
**Spec Reference:** UJ-1 (Research reproducibility), UJ-5 (Publication archival)
**User Journey:** UJ-5 (Reproducibility package for peer review)
**Acceptance Criteria:**
- [ ] Package contents: {scenario.yaml, seed.txt, git_hash.txt, timestamp.txt, parameters.json, instructions.md}
- [ ] Generation: `civ-sim run export --run-id <id> --format reproducibility-package --output package.zip`
- [ ] Instructions: README with exact commands to reproduce (civ-sim run create --spec scenario.yaml --seed <seed>)
- [ ] Citation: BibTeX entry with scenario name, git hash, timestamp for academic paper
- [ ] Archival: package is self-contained; can be shared via Zenodo or GitHub for long-term archival
- [ ] Verification: third party can unzip, run instructions, and verify metrics match original

---

## Priority Legend

| Priority | Definition | Timeline |
|----------|-----------|----------|
| **P0** | Core functionality required for MVP; must ship in phase 0 | Weeks 1-4 (sprint) |
| **P1** | High-value features for expanded gameplay & research; ship in phase 1 | Weeks 5-12 (2x sprint) |
| **P2** | Nice-to-have polish features for deep simulation; future phases | Weeks 13+ (future) |

---

## Acceptance Criteria Template (Used in All FRs)

Each FR includes:
- **Shall statement:** Clear, measurable functional requirement
- **Priority:** P0/P1/P2
- **Spec reference:** Link to design document(s)
- **User journey reference:** Link to affected journey(s)
- **Acceptance criteria:** Bulleted checklist, each item testable
  - Data structures and schemas
  - Determinism verification (seed-based testing)
  - Bounds checking and invariants
  - Event logging and audit trail
  - Performance/latency targets (where applicable)
  - Logging/validation to detect failure

---

## Cross-Requirement Dependencies

| FR Dependency | Reason | Impact |
|---------------|--------|--------|
| FR-CIV-ECON-001 (Ledger) | Foundation for all economy FR | All ECON-002+ depend on ledger structure |
| FR-CIV-RES-002 (Tick execution) | Core sim loop | All feature FRs depend on tick advancement |
| FR-CIV-GEO-001 (Terrain) | Map foundation | Movement, structures, resources depend on terrain properties |
| FR-CIV-ACT-001 (Citizen birth) | Population foundation | Migration, jobs, welfare all depend on citizen existence |
| FR-CIV-WAR-001 (Diplomatic states) | Conflict foundation | Sanctions, treaties, coalition all depend on state machine |
| FR-CIV-RES-002 (Execution) + FR-CIV-RES-004 (Events) | Determinism | Replay depends on event ordering (FR-CIV-RES-007) |

---

## Validation & Testing Strategy

**Property-Based Testing:**
- Conservation invariant (ledger): ∀ tick, ∑ transfers = 0
- Determinism invariant: same (state, policy, seed) → identical next state
- Bounds invariant: all metrics in declared range [min, max]

**Integration Testing:**
- Multi-domain scenarios: run economy + climate + actors + war simultaneously
- Stress tests: supply shock cascades through market → migration → institution → conflict

**Regression Testing:**
- Canonical scenarios: preserve expected outcomes (e.g., climate-crisis always triggers migration)
- Replay testing: bit-reproducibility of metrics

