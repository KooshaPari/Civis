---
title: Compatibility Matrix
description: DINOForge version compatibility with game, BepInEx, and Unity versions
---

# Compatibility Matrix

DINOForge uses a 6-tier status ladder to communicate version compatibility. This page helps you determine if your DINOForge version works with your DINO game installation.

## Status Ladder

The following statuses indicate compatibility levels and recommended actions:

| Status | Color | Description | Installer Action |
|--------|-------|-------------|-------------------|
| **Optimal** | Green (#22c55e) | Fully tested and verified. All features work. Recommended configuration. | Proceed |
| **Stable** | Lime (#84cc16) | Works reliably. Minor edge cases possible. Community tested. | Proceed |
| **Maintenance** | Yellow (#eab308) | Functional but receiving security fixes only. Upgrade recommended. | Warn |
| **Unstable** | Orange (#f97316) | Known issues. Partial functionality. Use at own risk. | Warn Strongly |
| **Broken** | Red (#ef4444) | Known incompatible. Will not function correctly. | Block |
| **Unknown** | Gray (#6b7280) | Untested combination. No compatibility data available. | Warn |

## Compatibility Matrix

### Current Releases

| DINOForge | DINO Game | BepInEx | Unity | Status | Notes |
|-----------|-----------|---------|-------|--------|-------|
| 0.5.x | `>=1.0.0` | `>=5.4.21` `&lt;6.0.0` | 2021.3.* | **Optimal** | Current stable release. Fully tested. |
| 0.4.x | `>=1.0.0` | `>=5.4.21` `&lt;6.0.0` | 2021.3.* | **Maintenance** | Security fixes only. Upgrade to 0.5.x recommended. |
| 0.3.x | `>=1.0.0` | `>=5.4.21` `&lt;6.0.0` | 2021.3.* | **Broken** | No longer supported. Upgrade to 0.5.x required. |

### Experimental / Future

| DINOForge | DINO Game | BepInEx | Unity | Status | Notes |
|-----------|-----------|---------|-------|--------|-------|
| 0.5.x | `>=1.0.0` | `>=6.0.0` | 2021.3.* | **Unknown** | BepInEx 6.x (Mono) support planned for DINOForge 1.0. Not yet tested. |
| 0.5.x | `>=2.0.0` | `>=5.4.21` `&lt;6.0.0` | 2022.* | **Unknown** | Future DINO game version. No compatibility data available yet. |

## Component Compatibility

ECS component layouts may change between game versions. The following components have been tested:

| Component | Field | DINO Versions | Status | Notes |
|-----------|-------|---------------|--------|-------|
| `Components.Health` | `currentHealth` | `>=1.0` | **Optimal** | Confirmed via entity dump. Core health tracking for units and buildings. |
| `Components.HealthBase` | `_maxHealthMultiplier` | `>=1.0` | **Stable** | Private field, may change in patches. Max health multiplier mutable value. |
| `Components.Unit` | (tag) | `>=1.0` | **Optimal** | Zero-sized tag marking entities as units. Verified present on all units. |
| `Components.ArmorData` | `type` | `>=1.0` | **Optimal** | ArmorType enum. Confirmed layout stable across patches. |
| `Components.AttackCooldown` | `value` | `>=1.0` | **Stable** | Cooldown timer. Also carries fixedProjectileSpeed field. |
| `Components.ProjectileDataBase` | (blob ref) | `>=1.0` | **Optimal** | `BlobAssetReference&lt;ProjectileData&gt;`. Immutable, verified on projectile entities. |
| `Components.BuildingBase` | (core) | `>=1.0` | **Optimal** | Present on all buildings. Core building data container. |
| `Components.Enemy` | (faction marker) | `>=1.0` | **Stable** | Enemy faction marker (`BlobAssetReference&lt;EnemyBaseData&gt;`). Player units lack this tag. |

## Upgrading Between Versions

### 0.3.x → 0.5.x

0.3.x is no longer supported and incompatible with 0.5.x. A full reinstall is required:

1. Remove all 0.3.x mod packs from `BepInEx/plugins/`
2. Update DINOForge Runtime plugin to 0.5.x
3. Reinstall mod packs targeting 0.5.x

### 0.4.x → 0.5.x

0.4.x packs should be compatible with 0.5.x, but it is recommended to update when the pack author publishes a 0.5.x version.

## Important Notes

- **DINO Game Version**: The game uses Unity 2021.3.45f2. Game updates may change ECS component layouts without warning.
- **BepInEx 6.x**: Full support planned for DINOForge 1.0. Currently using BepInEx 5.4.23.5.
- **Mono Runtime**: DINO uses the Mono runtime (not IL2CPP), which is important for BepInEx compatibility.

## Reporting Compatibility Issues

If you encounter incompatibility issues not listed here, please report them on the [GitHub Issues](https://github.com/KooshaPari/Dino/issues) page.

## Machine-Readable Data

For programmatic access to compatibility data, see the [compat.json](/compat.json) file.
