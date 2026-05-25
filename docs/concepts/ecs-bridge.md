# ECS Bridge

DINO uses full Unity ECS (DOTS) with Burst compilation. DINOForge bridges between its declarative pack system and DINO's actual ECS components.

## Why ECS-Native?

Standard BepInEx modding uses Harmony to patch managed C# methods. In DINO this carries severe performance costs:

- Harmony prefix/postfix patches are **compatible** but slow
- Transpiler patches are **not recommended** due to ECS architecture
- Removing the Burst-generated DLL can **halve frame rate**
- DINO handles 25,000+ enemy entities per wave — performance matters

DINOForge uses **ECS-native modding** wherever possible: writing actual ECS systems that integrate with DINO's system groups.

## ECS System Registration

DINO ECS systems inherit `SystemBaseSimulation` (not plain `SystemBase`):

```csharp
[UpdateAfter(typeof(SomeSystem))]
[UpdateInGroup(typeof(SomeGroup))]
public class MyModSystem : SystemBaseSimulation
{
    protected override void OnCreateSimulation() { }
    protected override void OnUpdateSimulation() { }
}
```

Key patterns from existing DINO mods:
- `ComponentDataFromEntity&lt;T&gt;` for entity data access
- `GetSingletonEntity&lt;T&gt;()` / `GetSingleton&lt;T&gt;()` for singletons
- `[UpdateAfter]` and `[UpdateInGroup]` attributes for system ordering

## Game Namespaces

DINO organizes code under these top-level namespaces:

| Namespace | Contents |
|-----------|----------|
| `Components.*` | ECS component definitions |
| `Systems.*` | ECS system implementations |
| `Utility.*` | Helper classes |
| `UI.*` | UI systems |

## Component Catalog

DINOForge's Runtime layer includes an **Entity Dumper** that catalogs all discovered components at boot. This dump feeds the SDK's mapping tables.

### Vanilla Unit-to-Role Mapping

| Vanilla Element | Abstract DINOForge Role |
|----------------|------------------------|
| Peasant worker | Economy worker |
| Swordsman | Line infantry |
| Archer / ranged | Ranged infantry |
| Cavalry | Fast strike unit |
| Catapult / trebuchet | Siege / artillery |
| Walls / towers | Static defense |
| Farms | Economy primary |
| Town hall / castle | Command center |
| Magic / soul | Special ability proxy |

## How Packs Map to ECS

When a pack defines a unit in YAML:

```yaml
id: clone_trooper
unit_class: CoreLineInfantry
vanilla_mapping: swordsman
stats:
  hp: 120
  damage: 18
```

The runtime:
1. Looks up `vanilla_mapping: swordsman` in the component catalog
2. Finds the corresponding ECS entity archetype
3. Applies stat overrides (`hp`, `damage`, etc.) to the relevant components
4. Registers visual/audio overrides if defined

This means pack authors never touch ECS code directly. The bridge handles translation.

## Resource Types

DINO has 5 base resource types plus special resources:

| Resource | Internal Name |
|----------|--------------|
| Food | `Food` |
| Wood | `Wood` |
| Stone | `Stone` |
| Iron | `Iron` |
| Gold | `Money` |
| Souls | `Souls` |
| Bones | `Bones` |
| Corpse | `Corpse` |

## Harmony Fallback

For edge cases where ECS-native modding is insufficient, DINOForge supports controlled Harmony patches through the Patch Layer (Layer 4). These are:

- Explicitly marked as `unsafe`
- Performance-gated
- Version-checked at load time
- Logged with full diagnostics

Use Harmony only when no ECS-native path exists.
