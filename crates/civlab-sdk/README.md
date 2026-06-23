# civlab-sdk

`civlab-sdk` is the modder-facing SDK for Civis. It stays on the public side of
the architecture: mods implement traits, hosts own the adapters.

## What it provides

- Material registration on top of `civ-voxel` taxonomy types
- Building and recipe registration
- Simulation event hooks for births, deaths, and tech changes
- Manifest loading from `mods/` directories in JSON or RON

## Example

```rust
use civlab_sdk::{
    BuildingBlueprint, BuildingCatalog, BuildingKind, BuildingRegistration, BirthEvent,
    CustomMaterial, DeathEvent, MaterialCatalog, MaterialRegistrar, MaterialSpec,
    RecipeCatalog, RecipeDefinition, RecipeRegistrar, SimulationEventHook, TechEvent,
};
use civ_voxel::{MaterialId, Phase};
use civ_agents::{Civilian, Position3d};

#[derive(Default)]
struct MarbleMod;

impl MaterialRegistrar for MarbleMod {
    fn register_materials(&self, catalog: &mut MaterialCatalog) {
        catalog.register(CustomMaterial {
            base_id: None,
            spec: MaterialSpec {
                name: "Marble".to_owned(),
                phase: Phase::Solid,
                density: 2_700,
                flow_rate: 0,
                viscosity: 0,
                angle_of_repose: None,
                color: [240, 240, 244, 255],
            },
        });
    }
}

impl civlab_sdk::building::BuildingRegistrar for MarbleMod {
    fn register_buildings(&self, catalog: &mut BuildingCatalog) {
        catalog.register(BuildingRegistration {
            blueprint: BuildingBlueprint {
                id: "marble-cottage".to_owned(),
                name: "Marble Cottage".to_owned(),
                kind: BuildingKind::Residential,
                preferred_materials: vec![MaterialId(12)],
                era_min: 2,
            },
        });
    }
}

impl RecipeRegistrar for MarbleMod {
    fn register_recipes(&self, catalog: &mut RecipeCatalog) {
        catalog.register(civlab_sdk::RecipeRegistration {
            recipe: RecipeDefinition {
                id: "marble-block".to_owned(),
                name: "Marble Block".to_owned(),
                inputs: vec![(MaterialId(12), 4)],
                outputs: vec![(MaterialId(12), 1)],
                building: Some("marble-cottage".to_owned()),
            },
        });
    }
}

impl SimulationEventHook for MarbleMod {
    fn on_birth(&mut self, event: &BirthEvent) {
        let _ = (event.tick, &event.civilian, &event.position);
    }

    fn on_death(&mut self, event: &DeathEvent) {
        let _ = (event.tick, &event.civilian);
    }

    fn on_tech(&mut self, event: &TechEvent) {
        let _ = (&event.tech_id, &event.civilian);
    }
}
```

## Manifest example

`mods/marble/manifest.json`

```json
{
  "mod": {
    "id": "marble",
    "name": "Marble Mod",
    "version": "1.2.3",
    "author": "CivLab",
    "description": "Adds marble materials and a cottage line",
    "entrypoint": "marble.wasm"
  },
  "materials": [
    {
      "name": "Marble",
      "phase": "Solid",
      "density": 2700,
      "flow_rate": 0,
      "viscosity": 0,
      "angle_of_repose": null,
      "color": [240, 240, 244, 255]
    }
  ],
  "buildings": [],
  "recipes": [],
  "events": ["birth", "tech"]
}
```

Load all manifests from a folder:

```rust
let manifests = civlab_sdk::load_manifests_from_dir("mods")?;
```
