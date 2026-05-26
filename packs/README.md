# DINOForge Packs

Official example and reference packs for DINOForge.

## Directory Structure

This directory is the canonical source for all official packs. Each subdirectory is a
self-contained content pack with a `pack.yaml` manifest:

- **example-balance** - Simple balance pack demonstrating basic unit and faction definitions
- **warfare-modern** - Modern warfare theme with contemporary units and weapons
- **warfare-starwars** - Star Wars theme with Republic vs CIS factions and units
- **warfare-guerrilla** - Guerrilla warfare theme with asymmetric faction composition
- **economy-balanced** - Economy system demonstration with resource rates and trade routes
- **scenario-tutorial** - Scenario pack with tutorial, survival, and resource challenges

## Pack Manifest Format

Each pack must contain a `pack.yaml` manifest. See the example packs for templates.

## Validation

Validate any pack directory with:

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-name>
```

Or via the CLI:

```bash
dinoforge validate <pack-directory>
```

## Building a Pack

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/<pack-name>
```

## Adding New Packs

To create a new pack:

1. Create a new subdirectory under `packs/` with your pack name.
2. Add a `pack.yaml` manifest (use an existing pack as reference).
3. Add your content files (units, factions, buildings, etc.) under the appropriate subdirectories.
4. Validate: `dotnet run --project src/Tools/PackCompiler -- validate packs/<your-pack-name>`

## References

- [Pack Manifest Schema](../schemas/pack-manifest.schema.json)
- [DINOForge Documentation](../docs/)
- [Contributing Guide](../CONTRIBUTING.md)
