# Warfare Domain

The Warfare domain plugin is the first and reference domain plugin for DINOForge. It extends the platform with combat-oriented modding: factions, doctrines, unit taxonomies, weapon systems, wave composition, and themed content packs.

## Concept

DINO is a medieval siege defense RTS where you defend against massive hordes (25,000+ enemies). The Warfare domain reimagines these battles with themed factions — modern military, Star Wars Clone Wars, and more.

The key insight: every battle is the same mechanical structure (line infantry, anti-armor, artillery, defenses, waves), just skinned differently. DINOForge makes that skin-swap declarative.

## Three Faction Archetypes

All factions map to one of three mechanical families:

```mermaid
graph LR
    O[Order<br/>Quality Army] --- IS[Industrial Swarm<br/>Attrition Army]
    IS --- A[Asymmetric<br/>Insurgent Army]

    style O fill:#0f3460,stroke:#fff,color:#fff
    style IS fill:#533483,stroke:#fff,color:#fff
    style A fill:#e94560,stroke:#fff,color:#fff
```

| Archetype | Traits | Factions |
|-----------|--------|----------|
| **Order** | Strong line infantry, reliable DPS, better defenses, higher unit cost | Republic, West |
| **Industrial Swarm** | Larger numbers, cheaper core, expendable, strong siege | CIS, Classic West Enemy |
| **Asymmetric** | Light units, mobility, ambush, raid pressure | Guerrilla West Enemy |

## Theme Packs

| Pack | Factions | Theme |
|------|----------|-------|
| Republic vs CIS | Galactic Republic, CIS | Star Wars Clone Wars |
| West vs Enemies | West, Classic West Enemy, Guerrilla West Enemy | Modern Warfare |

## Implementation Order

1. **West vs Classic West Enemy** — Easiest, proves the framework works
2. **Republic vs CIS** — Harder art/audio, proves theme-skin abstraction
3. **West vs Guerrilla Enemy** — Asymmetry, hardest balance challenge

## Content Production Strategy

### Level 1 — Zero-New-Model Build (MVP)

Only: renamed units, new icons, recolors, projectile/VFX swaps, stat changes, wave logic, SFX swaps, text/UI replacement. Gets a playable framework fast.

### Level 2 — Selective Model Swaps

Add: hero units, key vehicles, signature defenses, faction command buildings. Only highest-visibility pieces.

### Level 3 — Full Asset Conversion

Only after the framework is proven fun. This is the difference between a released mod and a dead dream repo.

## Module Split

| Module | Responsibility |
|--------|---------------|
| `DinoWarfare.Core` | Schemas, faction definitions, balance tables, shared enums |
| `DinoWarfare.ECS` | Entity queries, ECS patching, component mutation, spawn overrides |
| `DinoWarfare.Assets` | Icon registry, localization, audio routing, VFX registry |
| `DinoWarfare.Campaign` | Wave composition, faction matchup rules, mission overrides |
| `DinoWarfare.Theme.StarWars` | Republic/CIS content pack |
| `DinoWarfare.Theme.Modern` | West/Classic Enemy/Guerrilla content pack |
| `DinoWarfare.Debug` | Overlay, inspector, hot reload, test-spawn menus |

## Next Steps

- [Factions](/warfare/factions) — Detailed faction guide
- [Unit Roles](/warfare/unit-roles) — The 13 required role slots
- [Schema Reference](/reference/schemas) — Faction and unit schemas
