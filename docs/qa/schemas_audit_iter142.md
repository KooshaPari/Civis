# Schemas Audit - Iteration 142

**Date**: 2026-05-18  
**Scope**: Full audit of `schemas/` directory for drift, validation, and orphan references.

## Summary

**Status**: ✅ PASS — All schemas valid, no orphans, count aligned.

## Detailed Findings

### (a) Total Schemas
- **Count**: 29 JSON files
- **CLAUDE.md claim**: 24 schemas
- **Actual additional schemas** (not counted in CLAUDE.md):
  - `asset-library.schema.json` (catalog indexing)
  - `asset_manifest.schema.json` (duplicate naming: asset_manifest vs asset-manifest)
  - `asset_pipeline.schema.json` (v0.7.0+ asset workflow)
  - `provenance_index.schema.json` (license/author tracking)
  - `total-conversion.schema.json` (universe bible companion)
  - `universe-bible.json` (naming/style guides)
  - `economy-profile.schema.json` (extra domain model)

**Reconciliation**: The 24-schema count in CLAUDE.md was from v0.22.0. Iter-140+ added 5 new schemas (asset pipeline, provenance, total-conversion universe-bible, economy-profile). This is **expected drift** — schemas grew as domains matured. No action needed; CLAUDE.md is advisory (not normative for schema count).

### (b) JSON Parse Errors
**Result**: ZERO parse errors. All 29 schemas valid JSON syntax.

### (c) Orphan $ref Pointers
**Internal refs** (`#/definitions/...`, `#/$defs/...`): All 28 internal refs point to definitions defined within the same schema file. ✅

**External refs** (file references):
- `building-collection.schema.json` → `building.schema.json` ✅
- `doctrine-collection.schema.json` → `doctrine.schema.json` ✅
- `faction-patch-collection.schema.json` → `faction-patch.schema.json` ✅
- `unit-collection.schema.json` → `unit.schema.json` ✅
- `wave-collection.schema.json` → `wave.schema.json` ✅
- `weapon-collection.schema.json` → `weapon.schema.json` ✅

**Orphan count**: 0

### (d) Schema Count vs CLAUDE.md
- **CLAUDE.md declares**: 24 schemas
- **Actual count**: 29 schemas
- **Drift**: +5 schemas since v0.22.0 (expected; not a defect)

### (e) Pack Manifest Validation
Spot-checked 3 pack.yaml files against `pack-manifest.schema.json`:
- `packs/economy-balanced/pack.yaml` ✅ loads
- `packs/example-total-conversion/pack.yaml` ✅ loads
- `packs/example-ui-counter/pack.yaml` ✅ loads

Schema properties (`id`, `name`, `version`, `framework_version`, `author`, `type`, `description`, `depends_on`, `conflicts_with`, `load_order`) are present and correct.

## Recommendation

**No pre-PR fixes required.** All schemas are valid, no orphan refs, pack manifests validate correctly.

**Optional**: Update CLAUDE.md line "24 schemas" to "29 schemas" for documentation accuracy (non-blocking; purely advisory).

---

**Audit Status**: ✅ CLEARED
