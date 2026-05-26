# DINOForge Pack Builder - Create warfare-starwars and warfare-modern packs

$packBase = "C:\Users\koosh\Dino\packs"

# ============================================================================
# warfare-starwars Pack Content
# ============================================================================

# Pack directories
$swDirs = @(
    "$packBase\warfare-starwars\doctrines",
    "$packBase\warfare-starwars\waves"
)

foreach ($dir in $swDirs) {
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

# Republic Doctrines
$repDoctrines = @"
# Galactic Republic Doctrines

- id: elite_discipline
  display_name: Elite Discipline
  description: Emphasizes morale, precision, and disciplined squad tactics. Bonus to accuracy and morale retention.
  faction_archetype: order
  modifiers:
    accuracy_bonus: 1.15
    morale_stability: 1.2
    unit_cohesion: 1.1

- id: jedi_leadership
  display_name: Jedi Leadership
  description: Enhanced unit performance through Force-sensitive commanders. Bonus to hero commander effectiveness.
  faction_archetype: order
  modifiers:
    hero_commander_damage: 1.25
    hero_commander_range: 1.1
    leadership_aura_radius: 1.3
"@

# CIS Doctrines
$cisDoctrines = @"
# Confederacy Doctrines

- id: mechanized_attrition
  display_name: Mechanized Attrition
  description: Overwhelm through numbers and mechanical endurance. Bonus to unit count and build speed.
  faction_archetype: industrial_swarm
  modifiers:
    unit_cap_bonus: 0.6
    build_speed_bonus: 0.4
    droid_durability: 1.15

- id: rolling_thunder
  display_name: Rolling Thunder
  description: Aggressive armor tactics with massed vehicle assaults. Bonus to vehicle damage and coordination.
  faction_archetype: industrial_swarm
  modifiers:
    vehicle_damage: 1.2
    vehicle_armor: 1.1
    vehicle_speed: 1.15
"@

Set-Content -Path "$packBase\warfare-starwars\doctrines\republic_doctrines.yaml" -Value $repDoctrines -Encoding UTF8
Set-Content -Path "$packBase\warfare-starwars\doctrines\cis_doctrines.yaml" -Value $cisDoctrines -Encoding UTF8

# Clone Wars Waves
$swWaves = @"
# Clone Wars Campaign Waves

- id: wave_01_droid_probe
  display_name: Droid Probe
  description: Initial reconnaissance wave - light droids testing defensive positions.
  wave_number: 1
  delay_seconds: 30
  is_final_wave: false
  spawn_groups:
    - unit_id: cis_probe_droid
      count: 3
      spawn_delay: 1.0

- id: wave_02_infantry_push
  display_name: Infantry Assault
  description: Main assault wave with battle droid legions.
  wave_number: 2
  delay_seconds: 120
  is_final_wave: false
  spawn_groups:
    - unit_id: cis_b1_battle_droid
      count: 12
      spawn_delay: 0.5
    - unit_id: cis_b1_squad
      count: 8
      spawn_delay: 0.5
    - unit_id: cis_b2_super_battle_droid
      count: 4
      spawn_delay: 1.0

- id: wave_03_armor_assault
  display_name: Armor Assault
  description: Heavy armor support with AAT walkers and Droideka units.
  wave_number: 3
  delay_seconds: 180
  is_final_wave: false
  spawn_groups:
    - unit_id: cis_aat_crew
      count: 6
      spawn_delay: 1.5
    - unit_id: cis_droideka
      count: 8
      spawn_delay: 0.8
    - unit_id: cis_stap_pilot
      count: 5
      spawn_delay: 0.5

- id: wave_04_final_assault
  display_name: Final Assault
  description: Everything - the Separatists commit all forces.
  wave_number: 4
  delay_seconds: 240
  is_final_wave: true
  spawn_groups:
    - unit_id: cis_b1_battle_droid
      count: 16
      spawn_delay: 0.3
    - unit_id: cis_b2_super_battle_droid
      count: 8
      spawn_delay: 0.8
    - unit_id: cis_aat_crew
      count: 8
      spawn_delay: 1.0
    - unit_id: cis_general_grievous
      count: 1
      spawn_delay: 0
  difficulty_scaling:
    count_multiplier: 1.0
    health_multiplier: 1.15
    damage_multiplier: 1.1
"@

Set-Content -Path "$packBase\warfare-starwars\waves\clone_wars_waves.yaml" -Value $swWaves -Encoding UTF8

# ============================================================================
# warfare-modern Pack - CREATE FROM SCRATCH
# ============================================================================

# Create warfare-modern directory structure
$modernDirs = @(
    "$packBase\warfare-modern",
    "$packBase\warfare-modern\factions",
    "$packBase\warfare-modern\units",
    "$packBase\warfare-modern\doctrines",
    "$packBase\warfare-modern\waves"
)

foreach ($dir in $modernDirs) {
    if (!(Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

# Modern Pack YAML
$modernPackYaml = @"
# Modern Warfare Pack
# West vs Classic Enemy - modern military themed total conversion

id: warfare-modern
name: Modern Warfare
version: 0.2.0
framework_version: ">=0.1.0 <1.0.0"
author: DINOForge
description: |
  A total conversion replacing DINO's fantasy armies with contemporary military forces. Deploy modern infantry, vehicles, and artillery in asymmetric warfare between NATO-inspired Western Alliance and a technologically-advanced adversary.

  Factions included:
  - Western Alliance: disciplined combined-arms force with strong research and defensive infrastructure
  - Classic Enemy: adaptive force employing human-wave tactics and scorched-earth doctrine

  Content features:
  - 12 unique units per faction: riflemen, squads, special forces, gunners, mortars, scouts, APCs, armor
  - 4 doctrines: combined arms, human wave, fortified defense, asymmetric tactics
  - 4 campaign waves: early game, mid game, armor support, and final assault
  - Modern weapons and ballistic systems

  Gameplay shifts from medieval pacing to tactical modern warfare with emphasis on combined-arms coordination.
type: total_conversion

depends_on: []
conflicts_with: []

loads:
  factions:
    - factions
  units:
    - units
  doctrines:
    - doctrines
  waves:
    - waves
"@

Set-Content -Path "$packBase\warfare-modern\pack.yaml" -Value $modernPackYaml -Encoding UTF8

Write-Host "SUCCESS: All pack YAML files created!"
Write-Host "- warfare-starwars: doctrines, waves"
Write-Host "- warfare-modern: pack.yaml structure ready for units"
