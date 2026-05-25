# Extension Points

Documented extension points that agents may use to extend DINOForge.

---

## Registry Extensions

Agents may register new entries in any existing registry:

| Registry | Extension Method | Schema |
|----------|-----------------|--------|
| Units | `register_unit_archetype(...)` | `schemas/unit.schema.yaml` |
| Buildings | `register_building(...)` | `schemas/building.schema.yaml` |
| Weapons | `register_weapon(...)` | `schemas/weapon.schema.yaml` |
| Projectiles | `register_projectile(...)` | `schemas/projectile.schema.yaml` |
| Factions | `register_faction_pack(...)` | `schemas/faction.schema.yaml` |
| Doctrines | `register_doctrine(...)` | `schemas/doctrine.schema.yaml` |
| Wave Templates | `replace_wave_table(...)` | `schemas/wave.schema.yaml` |
| Localization | `patch_localization_bundle(...)` | `schemas/localization.schema.yaml` |
| Audio | `register_audio_pack(...)` | `schemas/audio.schema.yaml` |
| VFX | `override_projectile_family(...)` | `schemas/vfx.schema.yaml` |
| Behaviors | `attach_behavior_modifier(...)` | `schemas/behavior.schema.yaml` |
| Prefabs | `replace_prefab_binding(...)` | `schemas/prefab.schema.yaml` |

## Domain Plugin Extension

New domain plugins may be added under `src/Domains/<DomainName>/`:

Requirements:
- Must register domain-specific registries with core
- Must provide schemas in `schemas/domains/<domain>/`
- Must include validation rules
- Must NOT depend on other domain plugins
- Must include own test suite

## Pack Extension

New packs added under `packs/<pack-name>/`:

Requirements:
- Must include `pack.yaml` conforming to `schemas/pack-manifest.schema.yaml` (or `.json`)
- Must pass `dotnet run --project src/Tools/PackCompiler -- validate`
- Must declare all dependencies and conflicts
- See `packs/example-balance/` for a minimal working example

## Agents MUST NOT:
- Create new registry types without architect approval
- Modify core extension points without ADR
- Add undocumented extension points
- Bypass schema validation
