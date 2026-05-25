# new-pack

Create a new DINOForge content pack with the given name and type.

**Usage**: `/new-pack <pack-id> [type]`

**Types**: `content` | `balance` | `ruleset` | `total_conversion` | `utility` (default: `content`)

**Arguments**: $ARGUMENTS

## Steps

1. Parse pack-id and type from arguments (e.g. "my-pack content" or just "my-pack")
2. Create directory `packs/<pack-id>/`
3. Create `packs/<pack-id>/pack.yaml` with proper manifest (id, name, version: 0.1.0, type, author, depends_on: [], loads section)
4. Create appropriate subdirectories based on type (units/, buildings/, factions/ for content; stats/ for balance)
5. Create a sample YAML file in each subdirectory
6. Validate the pack: `dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-id>/`
7. Report success with next steps
