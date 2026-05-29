# dinoforge-pack-diff

Compare two packs and show differences in content, schemas, and dependencies.

**Usage**: `/dinoforge-pack-diff <pack-1-id> <pack-2-id> [--show-stats] [--format <json|text>]`

**Arguments**:
- `<pack-1-id>`: First pack to compare
- `<pack-2-id>`: Second pack to compare
- `--show-stats`: Include row counts and file sizes in output (default: false)
- `--format`: Output format (json, text). Default: text

## Purpose

Identifies what changed between two pack versions or compares two completely different packs. Useful for:
- Reviewing a pack update before deploying
- Understanding what a balance mod changed
- Merging contributions from multiple modders

## Steps

1. **Validate both packs exist**:
   - Check `packs/<pack-1-id>/pack.yaml` and `packs/<pack-2-id>/pack.yaml`
   - If either missing, error with helpful message

2. **Compare manifests**:
   - Differences in version, author, description, dependencies, conflicts
   - Highlight breaking changes (type change, new required deps)

3. **Compare content** (units, buildings, factions, weapons, etc.):
   - List added entries (in pack-2 but not pack-1)
   - List removed entries (in pack-1 but not pack-2)
   - List modified entries (differ in stats/properties)
   - Show before/after diffs for modified entries

4. **Report dependencies**:
   - Which new packs does pack-2 depend on?
   - Which packs did pack-1 need that pack-2 doesn't?

5. **Output format**:
   - **Text** (default): human-readable with emoji markers:
     - ✅ Added: `sw-rep-clone-trooper`
     - ❌ Removed: `republic-faction` (renamed? check manually)
     - ⚠️ Modified: `unit-health: 100 → 120`
   - **JSON**: structured diff for scripting (suitable for CI/CD)
   - **--show-stats**: append row counts and file sizes

## Example Output (text)

```
Pack Manifest
  Version: 0.1.0 → 0.2.0
  Author: (unchanged)
  Dependencies: (no change)

Units (28 items)
  ✅ Added: sw-rep-clone-trooper, sw-rep-clone-specialist (2 new)
  ❌ Removed: republic-soldier (1 deprecated)
  ⚠️ Modified: sw-rep-soldier-health (100 → 120)
  ⚠️ Modified: sw-cis-droid-armor (10 → 15)

Factions (1 item)
  (no changes)

Overall Stats
  Files changed: 18
  Lines added: 342
  Lines removed: 89
  Net change: +253
```

## Use When

- Code reviewing a pack contribution
- Understanding balance changes before deployment
- Merging multiple modders' work
- Pre-deployment verification (compare staged vs deployed)
- Documenting changelog entries

## Time

~5 seconds (load + diff + format).
