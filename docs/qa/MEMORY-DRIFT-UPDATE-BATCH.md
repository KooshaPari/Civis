# MEMORY.md Drift Update Batch — Task #850

**Date**: 2026-05-21
**Status**: PROPOSED (user-gated). DO NOT apply without explicit approval.
**Scope**: Read-only prep. No edits to MEMORY.md or CLAUDE.md were performed.

## Purpose

Consolidate stale numeric/status claims in `C:\Users\koosh\.claude\projects\C--Users-koosh-Dino\memory\MEMORY.md` (Milestone Status and Architecture sections) into a single review-ready patch batch.

## Drift Items Collected (6)

### 1. M4 Warfare test count
- **Current MEMORY.md**: "M4: Warfare Domain - DONE (..., 31 tests)"
- **Actual**: 129 tests
- **Source**: session audit (this iter)
- **Proposed patch**:
  `- **M4**: Warfare Domain - DONE (archetypes, doctrines, roles, waves, balance, 129 tests)`

### 2. M6 Economy test count
- **Current**: "M6 Economy: DONE — ..., 48 tests, ..."
- **Actual**: 264 tests
- **Proposed patch**:
  `- **M6 Economy**: DONE — EconomyPlugin, 6 models, 3 registries, ProductionCalculator, TradeEngine, 264 tests, economy-balanced pack, economy-profile.schema.json.`

### 3. M11 UI test count
- **Current**: "M11 UI Domain: DONE — ..., 251 UI tests, ..."
- **Actual**: 289 effective (259 method count) — pick effective per existing convention
- **Proposed patch**:
  `- **M11 UI Domain**: DONE — UiDomainPlugin, HudElementRegistry, MenuRegistry, ThemeRegistry, 289 UI tests, ui-overlay.schema.json, ui-hud-minimal pack.`

### 4. ComponentMap mappings
- **Current** (Architecture): "ComponentMap (30+ mappings)"
- **Actual**: 57 mappings
- **Proposed patch**:
  `... ECS Bridge: ComponentMap (57 mappings), EntityQueries, StatModifierSystem, VanillaCatalog`

### 5. Pattern #231 status
- **Current**: Not listed under retired patterns in MEMORY.md milestone notes.
- **Actual**: RETIRED in v0.26.0 (see `docs/qa/pattern-231-CLOSURE-v0.26.0.md`).
- **Proposed addition** (append to Iter-143 w2 retro line or new iter-144 line):
  `- Pattern #231 RETIRED (v0.26.0): all 11 HIGH NuGet-surface static-init I/O sites refactored to lazy. See pattern-231-CLOSURE-v0.26.0.md.`

### 6. Iter-144 milestones
- **Status**: Already covered by `project_iter144_*` memory entries (#707/#570/#576 cross-refs).
- **Action**: No new MEMORY.md line needed; confirm cross-refs intact during review.

## Batch Application Order

Apply items 1-5 as line-level replacements in MEMORY.md "Milestone Status" / "Architecture" sections. Item 6 is informational only.

## Verification After Apply

1. Re-grep `31 tests`, `48 tests`, `251 UI tests`, `30+ mappings` — should return 0 matches in MEMORY.md.
2. Confirm Pattern #231 appears in retired list.
3. No churn elsewhere in MEMORY.md.

---

**Reminder**: This file is the PROPOSED batch. MEMORY.md edits require explicit user approval per session protocol.
