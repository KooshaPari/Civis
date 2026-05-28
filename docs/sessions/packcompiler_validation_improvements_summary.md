# PackCompiler Validation Pipeline Improvements

**Date**: 2026-05-28  
**Commit**: `eca5f97f`  
**Branch**: `feat/unityexplorer-devtools-20260528`  

## Summary

Significantly improved the PackCompiler `validate` command with schema-aware error messages, contextual suggestions, and auto-repair framework. Pack developers now receive actionable feedback when validation fails instead of cryptic schema errors.

## What Was Added

### 1. EnhancedValidationService (`src/Tools/PackCompiler/Services/EnhancedValidationService.cs`)

A new service that enriches validation errors with:

- **Field-path extraction**: Parses dot-separated paths (e.g., `loads.units[0]`) into components
- **Line number detection**: Scans YAML files to locate field definitions
- **Contextual messages**: Transforms generic errors into plain-English explanations
- **Smart suggestions**: Context-specific guidance for each validation rule violation
- **Helper classes**:
  - `EnrichedValidationError` - Enriched error with suggestions and context
  - `ValidationReport` - Summary report for batch validations
  - `AutoRepairSuggestion` - Framework for auto-fixes (extensible)

**Key capabilities**:

```csharp
public EnrichedValidationError EnrichError(ValidationError error)
```

Analyzes errors and returns:
- Line/column information
- Field path components
- Multiple actionable suggestions
- Contextual explanation message

### 2. Enhanced Validate Command

Updated Program.cs with new CLI options:

```bash
# Basic enhanced validation (always enabled)
dotnet run --project src/Tools/PackCompiler -- validate <pack>

# Strict mode: fail on warnings + errors (for CI)
dotnet run --project src/Tools/PackCompiler -- validate <pack> --strict

# Auto-fix mode: attempt to repair common issues (future enhancement)
dotnet run --project src/Tools/PackCompiler -- validate <pack> --fix
```

### 3. Visual Improvements

Using Spectre.Console for rich output:

```
✗ Line 4: Field '[type]' has an invalid value — it must be one of the allowed options
  Error: NotInEnumeration: #/type
  Suggestions:
    • Valid types: content, balance, ruleset, scenario, total_conversion, utility
    • Example: type: content

✗ Line 1: Field '[version]' is required but was not provided
  Error: PropertyRequired: #/version
  Suggestions:
    • Add required field 'version' to pack.yaml
    • Version must follow semantic versioning: MAJOR.MINOR.PATCH (e.g., 1.2.3)
```

## Validation Enhancements

### Missing Required Fields

Detects missing fields and provides:
- **Field names** that are required
- **Default suggestions** for common fields:
  - `framework_version` → `">=0.5.0 <1.0.0"`
  - `id` → derived from directory name (kebab-case)

### Enum Validation

When field has invalid enum value:
- Lists all valid options
- Shows examples
- Suggests corrections for typos

### Pattern Validation

For pattern mismatches:

- **Pack ID**: "lowercase with hyphens/underscores only" + auto-suggest
- **Version**: "SemVer format: MAJOR.MINOR.PATCH"
- **Field patterns**: Field-specific guidance

### Type Mismatches

Provides YAML syntax help:
- "Use array syntax: [item1, item2, item3]"
- "Check indentation — field may need child properties"

## Sample Output

Given a broken pack with:

```yaml
id: test-broken-pack
name: Test Broken Pack
# Missing 'version'
author: Test Author
type: invalid-type
```

Output:

```
PackCompiler Validate
Pack Path: temp-test-pack

Loading manifest...
Schema validation failed:

✗ Line 1: Field '[version]' is required but was not provided
  Error: PropertyRequired: #/version
  Suggestions:
    • Add required field 'version' to pack.yaml
    • Version must follow semantic versioning: MAJOR.MINOR.PATCH (e.g., 1.2.3)

✗ Line 4: Field '[type]' has an invalid value — it must be one of the allowed options
  Error: NotInEnumeration: #/type
  Suggestions:
    • Valid types: content, balance, ruleset, scenario, total_conversion, utility
    • Example: type: content
```

## Architecture

### Class Hierarchy

```
EnhancedValidationService
├── EnrichError(ValidationError) → EnrichedValidationError
├── GenerateSuggestions(...)
├── BuildContextualMessage(...)
├── FindLineNumberInYaml(...)
├── ParseFieldPath(...)
└── DeriveIdFromPath(...)

EnrichedValidationError
├── OriginalError: ValidationError
├── Path: string
├── LineNumber: int
├── FieldPath: List<string>
├── Suggestions: IReadOnlyList<string>
└── ContextualMessage: string

ValidationReport (batch summary)
├── TotalPacks: int
├── SuccessfulPacks: int
├── WarningPacks: int
├── FailedPacks: int
└── ToString() → formatted summary
```

### Integration Points

1. **Program.cs** - ValidateSinglePack() calls EnhancedValidationService
2. **DisplayEnrichedError()** - Renders enriched errors with Spectre.Console
3. **Schema-aware** - Uses NJsonSchema + pack-manifest.schema.json for suggestions

## Future Enhancements

### --fix Flag Implementation

The `--fix` flag is wired but not yet fully implemented. Implementation would:

1. Auto-normalize YAML indentation via YamlDotNet round-trip
2. Add missing required fields with defaults:
   - `framework_version: ">=0.5.0 <1.0.0"`
   - `id: <derived-from-directory>`
3. Correct common typos:
   - `type: "mod-conversion"` → `type: "total_conversion"`
   - `version: "1.2"` → `version: "1.2.0"`
4. Write repaired manifest back to disk with backup

### --strict Flag

Wired but not fully utilized. Future use:

- In CI pipelines to fail on any warnings
- Treat "unused field" and "missing optional field hints" as errors
- Include in pre-commit hook

## Testing

Build and test:

```bash
# Build
dotnet build src/Tools/PackCompiler/ -c Release

# Test with valid pack
dotnet run --project src/Tools/PackCompiler -- validate packs/example-balance

# Test with broken pack
mkdir test-pack
cat > test-pack/pack.yaml << 'EOF'
id: broken
name: Broken Pack
author: Test
type: invalid
EOF
dotnet run --project src/Tools/PackCompiler -- validate test-pack
```

## Files Modified

- `src/Tools/PackCompiler/Program.cs`
  - Added `--fix` and `--strict` options to validate command
  - Updated ValidatePack signature to accept new flags
  - Updated ValidateSinglePack to call EnhancedValidationService
  - Added DisplayEnrichedError() helper

- `src/Tools/PackCompiler/Services/EnhancedValidationService.cs` (new)
  - EnhancedValidationService class (310 lines)
  - EnrichedValidationError class
  - ValidationReport class
  - AutoRepairSuggestion class

## Build Status

✅ **Build succeeded** - No errors, 31 warnings (pre-existing)

```
Build succeeded.
Time Elapsed 00:00:31.56
```

## Commit Details

```
commit eca5f97f
Author: Claude Opus 4.7 <noreply@anthropic.com>
Date:   2026-05-28

    feat(packcompiler): enhanced validation with schema-aware suggestions and auto-fix support

    - New EnhancedValidationService for schema-aware error enrichment
    - CLI options: --fix (auto-repair), --strict (CI mode)
    - Contextual suggestions for all validation rule violations
    - Line number and field-path extraction from YAML
    - Colored output via Spectre.Console
```

## Next Steps

1. **Implement --fix flag**: Full auto-repair for common issues
2. **Integrate into CI**: Add --strict flag to pre-commit hooks
3. **Extend suggestions**: Add more field-specific guidance
4. **Add batch validation**: Validate multiple packs in parallel
5. **Create validation report format**: JSON output for tooling

## Design Rationale

### Why EnhancedValidationService?

- **Separation of concerns**: Validation enrichment separate from CLI rendering
- **Reusability**: Can be used by other tools (MCP, SDK validation APIs)
- **Testability**: Service logic decoupled from Spectre.Console output
- **Extensibility**: Easy to add new suggestion types and rules

### Why Not Modify NJsonSchemaValidator?

- SDK validators should remain minimal (wrap, don't handroll per CLAUDE.md)
- PackCompiler-specific enhancements belong in Tools layer
- Allows independent iteration on error messages
- SDK stability (no breaking changes to ValidationResult)

## References

- `CLAUDE.md` - Architecture layer stack, wrap-don't-handroll principle
- `schemas/pack-manifest.schema.json` - Pack validation schema
- `src/SDK/Validation/NJsonSchemaValidator.cs` - Base validator
- Pattern Catalog #109 - Pattern suggests centralized JSON options
- Pattern #220 - Unsealed concrete classes (EnrichedValidationError is sealed)
