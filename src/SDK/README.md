# DINOForge.SDK

The DINOForge SDK is the mod-author entry point for building content packs and domain plugins for **Diplomacy is Not an Option** (DINO).

## Install

```bash
dotnet add package DINOForge.SDK
```

Targets `netstandard2.0` for BepInEx 5.4 (Mono CLR 4.0) compatibility.

## What's Included

- **Registries** — typed registries for units, factions, buildings, weapons, projectiles, doctrines, skills, waves, squads.
- **ContentLoader** — pack discovery, YAML/JSON manifest loading, schema validation, dependency resolution.
- **Models** — immutable data records for all moddable content shapes.
- **Validation** — NJsonSchema-backed validators for `pack.yaml` and all 29 content schemas.
- **Assets** — addressables catalog helpers and asset bundle metadata.
- **Universe Bible** — total-conversion override hooks.

## Quick Start

```csharp
using DINOForge.SDK.Registry;
using DINOForge.SDK.Models;

// Register a unit in your domain plugin
var registry = serviceProvider.GetRequiredService<IRegistry<UnitDefinition>>();
registry.Register(new UnitDefinition {
    Id = "my-pack.tank",
    Health = 500,
    // ...
});
```

## Documentation

- Mod author guide: https://kooshapari.github.io/Dino/guides/mod-authoring
- Pack schema reference: https://kooshapari.github.io/Dino/schemas
- Main repository: https://github.com/KooshaPari/Dino

## License

MIT — see repository root for full text.
