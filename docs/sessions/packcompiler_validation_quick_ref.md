# PackCompiler Validate — Quick Reference

## Basic Usage

```bash
cd C:\Users\koosh\Dino

# Validate a single pack
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

# Validate all packs in directory (auto-discovers pack.yaml)
dotnet run --project src/Tools/PackCompiler -- validate packs/

# Validate with strict mode (fail on warnings)
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack --strict

# Validate with JSON output (for CI/tooling)
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack --format json
```

## Understanding Error Messages

### Format

```
✗ Line N: Field '[field-name]' <contextual-message>
  Error: <schema-rule>
  Suggestions:
    • Suggestion 1
    • Suggestion 2
    • ...
```

### Common Errors

#### Missing Required Field

```
✗ Line 1: Field '[version]' is required but was not provided
  Error: PropertyRequired
  Suggestions:
    • Add required field 'version' to pack.yaml
    • Version must follow semantic versioning: MAJOR.MINOR.PATCH (e.g., 1.2.3)
```

**Fix**: Add the field with the suggested value.

#### Invalid Enum Value

```
✗ Line 6: Field '[type]' has an invalid value — it must be one of the allowed options
  Error: NotInEnumeration
  Suggestions:
    • Valid types: content, balance, ruleset, scenario, total_conversion, utility
    • Example: type: content
```

**Fix**: Use one of the listed valid values.

#### Pattern Mismatch

```
✗ Line 1: Field '[id]' does not match the required format
  Error: Pattern validation failed
  Suggestions:
    • Pack ID must be lowercase with hyphens/underscores only
    • Suggested value: "my-cool-pack"
```

**Fix**: Follow the format rules shown in suggestions.

#### Type Mismatch

```
✗ Line 3: Field '[depends_on]' has the wrong type
  Error: Type mismatch
  Suggestions:
    • Use YAML array syntax: [item1, item2, item3]
    • Example: depends_on: [warfare-base, economy-modern]
```

**Fix**: Use correct YAML syntax (array vs. string vs. object).

## Common Pack.YAML Template

```yaml
# Required fields
id: my-cool-pack                          # lowercase, hyphens/underscores
name: My Cool Pack                        # Human-readable name
version: 0.1.0                            # SemVer: MAJOR.MINOR.PATCH
author: Your Name                         # Your name or org
type: content                             # One of: content, balance, ruleset, scenario, total_conversion, utility

# Recommended fields
framework_version: ">=0.5.0 <1.0.0"      # DINOForge version constraint
description: "Brief description of what this pack does"

# Optional fields
game_version: "1.0.0"                     # Specific game version if needed
load_order: 100                           # Load priority (default: 100)

# Pack dependencies
depends_on:
  - warfare-base
  - economy-modern

conflicts_with:
  - warfare-old

# Content to load
loads:
  units:
    - units/my-unit.yaml
  factions:
    - factions/my-faction.yaml
  buildings:
    - buildings/my-building.yaml
  doctrines:
    - doctrines/my-doctrine.yaml
  weapons:
    - weapons/my-weapon.yaml
```

## Validation Checklist

Before running validate, ensure:

- [ ] `pack.yaml` exists in pack directory
- [ ] All required fields present: `id`, `name`, `version`, `author`, `type`
- [ ] `framework_version` specified (e.g., `">=0.5.0 <1.0.0"`)
- [ ] Pack ID is lowercase with hyphens/underscores only
- [ ] Version follows SemVer format (MAJOR.MINOR.PATCH)
- [ ] Type is one of valid enum values
- [ ] All dependencies exist and have valid IDs
- [ ] YAML indentation is correct (2 spaces, no tabs)
- [ ] Arrays use YAML syntax: `[item1, item2]` or list format with `-`

## Exit Codes

```
0 = Validation successful
1 = Validation failed (errors found)
2 = Invalid arguments or fatal error
```

## Integration with CI/CD

For GitHub Actions workflows:

```yaml
- name: Validate Packs
  run: |
    cd ${{ github.workspace }}
    dotnet run --project src/Tools/PackCompiler -- validate packs/ --strict
```

For pre-commit hooks:

```bash
#!/bin/bash
# .git/hooks/pre-commit
dotnet run --project src/Tools/PackCompiler -- validate packs/ --strict
if [ $? -ne 0 ]; then
  echo "Pack validation failed. Fix errors and try again."
  exit 1
fi
```

## Troubleshooting

### "pack.yaml not found"

Make sure your pack directory structure is:
```
my-pack/
├── pack.yaml          ← Must exist here
├── units/
├── factions/
└── ...
```

### "Unknown property 'X'"

Remove the invalid property from pack.yaml. Valid properties are:
- Required: `id`, `name`, `version`, `author`, `type`
- Recommended: `framework_version`, `description`
- Optional: `game_version`, `load_order`, `depends_on`, `conflicts_with`, `loads`, `overrides`

### "PropertyRequired for X"

Add the missing required field. Use suggestions for default values.

### "NotInEnumeration"

Check the field's valid values from suggestions. For `type`, use one of:
- `content` — Standard content pack (units, factions, balance)
- `balance` — Purely balance adjustments
- `ruleset` — Game rule changes
- `scenario` — Scenario/mission content
- `total_conversion` — Complete game overhaul
- `utility` — Tools and utilities for modders

## Next Steps

1. Run validation: `dotnet run --project src/Tools/PackCompiler -- validate <pack>`
2. Read suggestions carefully for each error
3. Fix issues based on guidance provided
4. Re-run validation to verify fixes
5. Consider adding to CI pipeline with `--strict` flag

## See Also

- `docs/sessions/packcompiler_validation_improvements_summary.md` — Architecture and design
- `docs/sessions/packcompiler_before_after_example.md` — Detailed examples
- `schemas/pack-manifest.schema.json` — Full validation schema
- `CLAUDE.md` — Pack system documentation
