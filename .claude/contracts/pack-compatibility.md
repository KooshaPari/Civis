# Pack Compatibility Contract

## Pack Manifest Schema

The pack manifest schema is versioned alongside the framework.

## Pack Requirements

- All packs must declare `framework_version` range (e.g., `">=0.1.0 <1.0.0"`)
- All packs must declare `id`, `name`, `version`, and `author`
- Packs may declare `depends_on` and `conflicts_with` dependencies
- Pack IDs must be globally unique within a deployment

## Schema Evolution

- Breaking schema changes require a migration path for existing packs
- Schema version changes must be documented in CHANGELOG.md
- Validator must handle graceful degradation for older pack formats

## Validation

- All packs must pass schema validation before inclusion
- Pack compiler must reject non-compliant manifests with clear error messages
- Content references (units, buildings, factions) must be validated for consistency
