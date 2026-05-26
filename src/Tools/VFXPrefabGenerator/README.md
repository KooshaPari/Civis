# VFX Prefab Generator

**Status**: Ready for Unity Editor Integration
**Version**: 1.0
**Framework**: DINOForge ≥0.1.0
**Game Target**: DINO (Diplomacy is Not an Option) with warfare-starwars pack

## Overview

The VFX Prefab Generator is an Editor-only C# utility that automatically creates 11 Unity binary prefabs for the warfare-starwars content pack. These prefabs contain configured ParticleSystem components used by the VFXPoolManager at runtime.

**Prefabs Generated** (11 total):
1. **BlasterBolt_Rep.prefab** — Republic blaster projectile trail (bright blue, #4488FF)
2. **BlasterBolt_CIS.prefab** — CIS blaster projectile trail (orange-red, #FF4400)
3. **LightsaberVFX_Rep.prefab** — Republic melee swing trail (blue glow)
4. **LightsaberVFX_CIS.prefab** — CIS melee swing trail (orange glow)
5. **BlasterImpact_Rep.prefab** — Republic hit spark burst (blue)
6. **BlasterImpact_CIS.prefab** — CIS hit spark burst (orange)
7. **UnitDeathVFX_Rep.prefab** — Republic unit death effect (ascending disintegration)
8. **UnitDeathVFX_CIS.prefab** — CIS unit death effect (explosive burst)
9. **BuildingCollapse_Rep.prefab** — Republic building collapse (dust cloud, blue accent)
10. **BuildingCollapse_CIS.prefab** — CIS building collapse (dust cloud, orange accent)
11. **Explosion_CIS.prefab** — Large explosion (orange violent burst)

## Usage

### Option A: Unity Editor Menu (Recommended)

1. **Ensure Unity 2021.3.45f2 is open** with the DINO project loaded
2. Open the menu: **DINOForge → Generate VFX Prefabs**
3. Wait for completion dialog
4. Prefabs are created in: `Assets/warfare-starwars/vfx/` (in the Unity project)

### Option B: Manual Execution (Advanced)

```csharp
// In any Editor script or the Console
DINOForge.Tools.VFXPrefabGenerator.VFXPrefabGenerator.GenerateAllPrefabs();
```

## Technical Details

### Configuration Per Prefab Type

Each prefab is configured with faction-appropriate particle system settings:

| Type | Duration | Start Lifetime | Emission Rate | Max Particles | Usage |
|------|----------|----------------|---------------|---------------|-------|
| **BlasterBolt** | 0.5s | 0.3s | 50 p/s | 80 | Projectile flight trails |
| **LightsaberVFX** | 0.6s | 0.4s | 40 p/s | 100 | Melee swing effects |
| **BlasterImpact** | 0.3s | 0.25s | 100 p/s | 50 | Hit feedback sparks |
| **UnitDeathVFX** | 0.8s | 0.6s | 80 p/s | 120 | Unit destruction |
| **BuildingCollapse** | 1.0s | 0.8s | 60 p/s | 150 | Building destruction |
| **Explosion** | 0.6s | 0.5s | 150 p/s | 200 | Large explosions (AoE) |

### Faction Colors

- **Republic**: `#4488FF` (bright blue), `#64A0DC` (light accent)
- **CIS**: `#FF4400` (rust orange), `#B35A00` (dark accent)

### Material Properties

All prefabs use the **Particles/Standard Unlit** shader with:
- **Base Color**: Faction-specific (blue or orange)
- **Emission**: 2.0x intensity for visible glow
- **Render Queue**: 3000 (Transparent/Additive)
- **Blend Mode**: Additive (for glow effect)

### LOD Support

Prefabs support LOD via `max_particles` configuration:
- **HIGH**: Full particle count (80-200)
- **MEDIUM**: 60% of max (~50-120)
- **LOW/CULLED**: 30% of max (~20-60)

Implemented by VFXPoolManager or LevelManager via ParticleSystem.maxParticles property.

## Output Location

**In Unity Project**: `Assets/warfare-starwars/vfx/`

**In Final Pack**: `packs/warfare-starwars/assets/vfx/`

The prefabs are **binary .prefab files** (not YAML), serialized in Unity's native format.

## Integration with VFXPoolManager

The generated prefabs are consumed by `src/Runtime/Bridge/VFXPoolManager.cs`:

```csharp
// VFXPoolManager expects these paths:
AllocatePool("vfx/BlasterBolt_Rep.prefab", 12);
AllocatePool("vfx/BlasterBolt_CIS.prefab", 12);
AllocatePool("vfx/LightsaberVFX_Rep.prefab", 8);
// ... etc (11 total)
```

At runtime, the pool manager loads prefabs from the pack bundle and pre-allocates instances for reuse.

## Customization

### Adjusting Particle Counts

Edit `VFXPrefabGenerator.cs`, method `ConfigureParticleSystem()`:

```csharp
case VFXType.BlasterBolt:
    main.maxParticles = 80;  // Increase for denser trails
    emission.rateOverTime = new ParticleSystem.MinMaxCurve(50);  // More particles/sec
```

### Changing Faction Colors

Modify the color constants at the top of the class:

```csharp
private static readonly Color RepublicBlue = new Color(0.267f, 0.533f, 1.0f, 1.0f);  // #4488FF
private static readonly Color CISRed = new Color(1.0f, 0.267f, 0.0f, 1.0f);           // #FF4400
```

### Adding New Prefab Types

1. Add a new `VFXType` enum variant
2. Add a `case` in `ConfigureParticleSystem()`
3. Add a `GenerateXxxPrefab()` method
4. Call it from `GenerateAllPrefabs()`

## Validation

After generation, verify:

```bash
# Check files exist
ls -lh Assets/warfare-starwars/vfx/*.prefab

# Validate pack structure
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars

# Build pack bundle
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
```

## Dependencies

- **Unity**: 2021.3.45f2+ (required for Editor scripting)
- **Framework**: DINOForge ≥0.1.0
- **C#**: .NET Framework 4.7.2 (Editor target)

## Troubleshooting

### Issue: Menu option doesn't appear

**Solution**: Ensure the script is in `Assets/Editor/` or in any folder under the project. Rebuild the Visual Studio project and refresh Unity.

### Issue: "Shader not found" error

**Solution**: Unity is missing the default `Particles/Standard Unlit` shader. This is built-in; if missing, restart the Editor.

### Issue: Prefabs created but VFXPoolManager can't load them

**Solution**: Ensure the output path in the generator matches what VFXPoolManager expects:
- Generator creates: `vfx/BlasterBolt_Rep.prefab`
- VFXPoolManager loads: `vfx/BlasterBolt_Rep.prefab`

Paths must match exactly (case-sensitive).

## Future Enhancements

- [ ] Per-prefab configuration UI (Editor window)
- [ ] Live preview in Editor scene
- [ ] Export to Addressables catalog automatically
- [ ] Variant generation (low-poly, high-fidelity)
- [ ] Sound effect integration
- [ ] LOD generator (auto-create MEDIUM/LOW variants)

## References

- **VFX Design Spec**: `packs/warfare-starwars/VFX_SYSTEM_DESIGN.md`
- **VFXPoolManager**: `src/Runtime/Bridge/VFXPoolManager.cs`
- **Unity Particle System Docs**: https://docs.unity3d.com/Manual/class-ParticleSystem.html

---

**Generated by**: Claude Code (Haiku Agent)
**Last Updated**: 2026-03-12
