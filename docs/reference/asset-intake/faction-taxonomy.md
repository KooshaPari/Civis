# Faction Taxonomy and Gameplay Tags

## Core Factions (Prototype)

- `republic`
- `cis`
- `empire`
- `rebel`
- `jedi`
- `sith`
- `neutral`
- `scum`

## Gameplay Tags

```json
{
  "faction": "republic|cis|empire|rebel|jedi|sith|neutral|scum",
  "unit_role": "line_infantry|elite_infantry|anti_armor|recon|anti_air|support|hero",
  "armor_class": "light|medium|heavy|walker",
  "weapon_type": "blaster|rifle|cannon|beam|explosive|melee|none",
  "mobility": "foot|hover|vehicle|walker|air",
  "era": "clone_wars|classical|future|original|mixed",
  "tone": "military|ceremonial|aggressive|stealth|siege",
  "scale_hint": "small|standard|large|capital"
}
```

## Query Examples

- all CIS anti-armor line units: `faction=cis`, `unit_role=anti_armor`
- all Republic mobile units: `faction=republic`, `mobility=vehicle`
- all props and environmental kits: `unit_role` omitted, `era=clone_wars`

## Governance

- `faction` and `unit_role` are required in candidate manifests intended for registry registration.
- Optional tags can be added without schema change but must remain lowercase snake_case.
