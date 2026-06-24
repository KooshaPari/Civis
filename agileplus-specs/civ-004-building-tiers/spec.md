---
spec_id: civ-004
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-05-29
---

# Specification: Building Tiers and Production Chains

**Slug**: civ-004-building-tiers | **Epic**: E2 | **Date**: 2026-05-29 | **State**: ACTIVE

## Problem Statement

Buildings are the primary production nodes in the Civis simulation. They must support tiered complexity (primitive, artisan, industrial, advanced) with chained input/output dependencies so that supply chain emergent behavior occurs. Building type parameters must be scenario-YAML-configurable to allow research iteration without code changes.

## Target Users

- Economy crate developers wiring production into `civ-economy`
- Game designers defining scenario building sets in YAML
- Bevy client developers rendering buildings by tier in the 3D world

## Functional Requirements

- [ ] **FR-CIV-BUILD-001**: Building entity has `tier: BuildingTier` enum (Primitive, Artisan, Industrial, Advanced); tier determines input slots, output slots, and base Joule cost
- [ ] **FR-CIV-BUILD-002**: Production chain — each building type specifies `inputs: Vec<(GoodType, u32)>` and `outputs: Vec<(GoodType, u32)>`; production halts deterministically when any required input is zero
- [ ] **FR-CIV-BUILD-003**: All building type parameters (production rate, input/output ratios, Joule cost) configurable in scenario YAML; schema validated on load; invalid YAML → descriptive error

## Non-Functional Requirements

- Building type registry uses `BTreeMap<BuildingTypeId, BuildingSpec>` for determinism
- Production events logged with entity ID, good type, quantity, tick number
- Building specs loadable via `crates/engine/src/scenario.rs` YAML loader

## Constraints and Dependencies

- Depends on: FR-CORE-002 (ECS entity model) for building entity registration
- Depends on: FR-ECON-001 (production system) for per-tick production dispatch
- Depends on: FR-API-001 (scenario YAML) for building spec configuration

## Acceptance Criteria

- [ ] All four `BuildingTier` variants creatable with distinct slot counts
- [ ] Production chain halts (no phantom output) when any input good is zero
- [ ] Building spec overrides in scenario YAML apply without recompile
- [ ] Schema validation rejects missing required fields with field-path error
- [ ] Production events emitted to event log with correct entity ID and quantities

## Status

| Story | Status |
|-------|--------|
| Building entity ECS registration | Partial (buildings in engine ECS; tier enum not defined) |
| Production chain halt logic | Partial (`EconomyState::step` exists; chain halt not verified) |
| Scenario YAML building config | Partial (scenario loader exists; building spec schema not validated) |
