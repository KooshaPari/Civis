# Pattern #220 Audit: Unsealed Concrete Classes with Mutable State

**Status**: Catalog-promoted (iter-123)

**Date**: 2026-05-18  
**Auditor**: Claude Code (Haiku)  
**Scope**: `src/` excluding `bin/`, `obj/`, `Tests/` projects

---

## Executive Summary

**Pattern #220 is ENDEMIC.** With 32 violations across the codebase at moderate-to-low frequency (not epidemic), the pattern warrants **promotion to Pattern Catalog** with a **Roslyn analyzer** (DF1012) as future work. Current recommendation: document governance in CLAUDE.md, add allowlist at `docs/qa/pattern-220-allowlist.txt`, and schedule analyzer for v0.25.0+.

---

## Audit Results

### Script & Methodology
- **Script**: `scripts/ci/audit_unsealed_concrete_classes.py` (68 lines)
- **Detection Logic**:
  1. Parse C# files for `public class <Name>` (not `sealed`, `abstract`, `static`)
  2. Check for mutable state: `private` field with list/dict/set/generic type OR `_fieldName` pattern
  3. Check for inheritance contract: `protected virtual` or `protected abstract` members
  4. Violation = mutable state + NO inheritance contract (open for accidents)
- **False Positive Mitigation**: Regex only (not Roslyn); excludes test projects; checks for explicit inheritance intent

### Violation Summary
- **Total Violations**: 32
- **Tier Classification**:
  - **Tier 1 - Endemic Risk** (>200): None
  - **Tier 2 - Moderate** (30–200): **Pattern #220** ✓
  - **Tier 3 - Low Frequency** (<30): None

### Top 10 Violations by File

| File | Line | Class Name | Reason |
|------|------|-----------|--------|
| Runtime\Assets\AssetService.cs | 18 | AssetService | mutable_state_no_inheritance_contract |
| Runtime\Aviation\AerialSpawnSystem.cs | 39 | AerialSpawnSystem | mutable_state_no_inheritance_contract |
| Runtime\Aviation\AerialTargetingSystem.cs | 43 | AerialTargetingSystem | mutable_state_no_inheritance_contract |
| Runtime\Bridge\AssetBundleCache.cs | 12 | AssetBundleCache | mutable_state_no_inheritance_contract |
| Runtime\Bridge\AssetSwapSystem.cs | 50 | AssetSwapSystem | mutable_state_no_inheritance_contract |
| Runtime\Bridge\BuildingDestructionVFXSystem.cs | 15 | BuildingDestructionVFXSystem | mutable_state_no_inheritance_contract |
| Runtime\Bridge\KeyInputSystem.cs | 25 | KeyInputSystem | mutable_state_no_inheritance_contract |
| Runtime\Bridge\PackUnitSpawner.cs | 45 | PackUnitSpawner | mutable_state_no_inheritance_contract |
| Runtime\Bridge\ProjectileVFXSystem.cs | 15 | ProjectileVFXSystem | mutable_state_no_inheritance_contract |
| Runtime\Bridge\StatModifierSystem.cs | 86 | StatModifierSystem | mutable_state_no_inheritance_contract |

*Full audit data: `docs/qa/pattern_220_audit_raw.csv`*

### Distribution by Layer
- **Runtime/**: 18 violations (56%) — ECS systems, UI components, hot-reload bridge
- **SDK/**: 3 violations (9%) — AssetService, ContentLoader, PackFileWatcher
- **Tools/**: 3 violations (9%) — PackCompiler services (Addressables, Optimization, Prefab)
- **Bridge/**: 8 violations (25%) — ECS bridge systems and caches

---

## Governance Recommendation

### Why Pattern #220 Matters
1. **Accident-Prone Inheritance**: Subclasses could mutate internal dictionaries/lists unsafely
2. **Serialization Fragility**: Frameworks may instantiate subclasses and lose field mappings
3. **Maintenance Liability**: No explicit contract means future refactoring risks silent breakage

### Proposed Remediation Strategy

**Tier 1 (Immediate — v0.24.0)**
- Add governance section to CLAUDE.md (similar to Pattern #124: Unsealed Public Classes)
- Create `docs/qa/pattern-220-allowlist.txt` for known-safe classes (internal, no subclassing intended)
- Allowlist Runtime/ECS systems (AerialSpawnSystem, etc.) with reason: `// sealed-by-design: ECS system, no subclassing intended`

**Tier 2 (Next Sprint — v0.25.0)**
- Implement Roslyn analyzer **DF1012** in `src/DINOForge.Analyzers/`
- Scope: NuGet-published only (SDK/, Bridge.Protocol/Client/)
- Rule: Flag public unsealed class with mutable private state and no virtual members
- Configurable severity: WARN (default) or ERROR for published APIs

**Tier 3 (Backlog — v0.26.0+)**
- Retrofit high-impact classes in SDK/ and Bridge/ with `sealed` keyword
- For cases where subclassing IS intended, add `protected virtual` event hooks or documented extension points

### Exemptions (Documented in CLAUDE.md)
- **MonoBehaviour subclasses** in Runtime/UI/ and Runtime/ — Unity inheritance model is intentional (can be marked with `// sealed-by-design: unity-inheritance` allowlist)
- **Internal classes** not published in NuGet — may remain unsealed if no external subclassing
- **Service implementations** (AssetService, ContentLoader, PackFileWatcher) can remain unsealed if dependency-injected (allows test doubles) — document in `// unsealed-by-design: DI-testability`

---

## Judgment

**Pattern #220 merits promotion to Pattern Catalog.** 

With 32 violations (~2% of codebase), the pattern is **endemic at moderate frequency** but **not urgent** (no production breakage observed). Governance via CLAUDE.md + allowlist is sufficient until Roslyn analyzer ships. The presence of multiple ECS systems and services with mutable internal state warrants explicit sealing contracts to prevent accidental inheritance.

**Recommendation**: Add Pattern #220 to CLAUDE.md (between #124 and #210), create allowlist, and schedule Roslyn DF1012 for v0.25.0.

---

## Artifacts

- **Audit Script**: `scripts/ci/audit_unsealed_concrete_classes.py`
- **Raw Data**: `docs/qa/pattern_220_audit_raw.csv`
- **This Report**: `docs/qa/pattern_220_audit.md`
- **Allowlist** (pending): `docs/qa/pattern-220-allowlist.txt`
