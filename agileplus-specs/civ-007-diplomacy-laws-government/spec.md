---
spec_id: civ-007
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Diplomacy, Laws, and Government

**Slug**: civ-007-diplomacy-laws-government | **Epic**: E4 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Nations require a formal governmental structure (type, laws, legitimacy) and a diplomatic layer (FSM states, treaties, influence capital) to model interstate interaction. Laws constrain what policies are permissible; government type determines legitimacy recovery rate. The diplomatic FSM from CIV-0105 defines 8 states from Cooperative through Alliance and must transition deterministically given policy bundle inputs. Shadow networks (covert flows) operate as a sub-layer under the diplomatic FSM.

## Target Users

- Policy/diplomacy crate developers (`civ-policy`, `civ-laws`)
- Scenario designers specifying starting diplomatic relations in YAML
- Research agents modeling the sanctions-leakage and backfire theorems from CIV-0104/0105

## Functional Requirements

- [ ] **FR-CIV-DIPLO-001**: Diplomatic FSM — 8 states per ordered actor pair: `Cooperative, Strained, Sanctioned, Escalating, ActiveConflict, Deescalating, ColdWar, Alliance`; deterministic transitions given state + policy bundle; all thresholds config-driven
- [ ] **FR-CIV-DIPLO-002**: Influence capital system — nations accumulate influence via trade surplus and alliance alignment; influence spent on alliance formation and sanction lifting; logged each tick
- [ ] **FR-CIV-DIPLO-003**: Shadow network — covert flows of finance/info/materiel persist under enforcement; leakage conserved (non-negative); every shadow flow logged; enforcement intensity feeds legitimacy modifier with overreach detection
- [ ] **FR-CIV-GOV-001**: Government type enum `(Monarchy, Republic, Oligarchy, Theocracy, Democracy, Dictatorship)`; type determines legitimacy recovery rate, max tax rate, and rebellion threshold multiplier
- [ ] **FR-CIV-GOV-002**: Laws as RON-serialized policy bundles; each law specifies `domain: LawDomain`, `effect: LawEffect`, `constraint: Optional<Constraint>`; laws loaded from `civ-laws` RON stubs at scenario init

## Non-Functional Requirements

- Crate: `civ-policy` (diplomacy + shadow) + `civ-laws` (law RON stubs)
- All FSM transitions deterministic given same state; no system clock or float
- Actor-pair FSM keyed by `(ActorId, ActorId)` where `a < b` (stable sort)
- Influence capital stored as `Fixed` type; sanction leakage non-negative enforced by type
- Transitions operate in Phase 2 (Policy Phase) of tick cycle; RNG in Phase 4

## Constraints and Dependencies

- Depends on: FR-CORE-005 (policy evaluation phase) for policy bundle evaluation
- Depends on: FR-CIV-WAR-001 (military units) for `ActiveConflict` state mechanics
- Depends on: FR-CIV-SOCIAL-001 (institutions) for governmental institution membership
- Sanctions Leakage Threshold L₀, Authoritarian Backfire E*, Coalition Stability C₀ computed each tick and exposed as metrics

## Acceptance Criteria

- [ ] Diplomatic FSM transitions deterministically from `Cooperative → Strained` when `grievance_score >= STRAIN_THRESHOLD`
- [ ] `Cooperative → Alliance` fires when `influence_capital >= ALLIANCE_COST AND ideology_alignment >= ALIGN_FLOOR`
- [ ] Shadow flow total is non-negative and conserved across tick
- [ ] Shadow flow events logged with source, destination, type, quantity, tick
- [ ] Government type correctly modifies legitimacy recovery rate and max tax rate
- [ ] Laws load from RON stubs at scenario init without crash; invalid RON produces descriptive error

## Status

| Story | Status |
|-------|--------|
| Diplomatic FSM struct and state machine | Planned |
| Influence capital tracking | Planned |
| Shadow network flow model | Planned |
| Government type enum | Planned |
| Laws RON stubs | Partial (`civ-laws` RON stubs exist; schema not validated) |
| Threshold metric monitoring | Planned |
