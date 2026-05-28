# Example output of: dinoforge pack diff warfare-starwars warfare-modern
# This demonstrates what the command would produce

$sampleOutput = @"
Pack Diff: warfare-starwars vs warfare-modern

┌─────────────────────────────────────────────────────────────┐
│                          Units                              │
├──────────────────────┬──────────────────────┬───────────────┤
│ In A Only (green)    │ In B Only (blue)     │ In Both (yellow)
├──────────────────────┼──────────────────────┼───────────────┤
│ rep_clone_militia    │ western_rifleman     │ rep_v19_torrent│
│ rep_clone_trooper    │ western_squad        │ cis_tri_fighter│
│ rep_arc_trooper      │ western_scout        │               │
│ rep_heavy_trooper    │ western_gunner       │               │
│ cis_battle_droid     │ enemy_militia        │               │
│ cis_super_droid      │ enemy_heavy          │               │
└──────────────────────┴──────────────────────┴───────────────┘

┌─────────────────────────────────────────────────────────────┐
│                       Buildings                             │
├──────────────────────┬──────────────────────┬───────────────┤
│ In A Only (green)    │ In B Only (blue)     │ In Both (yellow)
├──────────────────────┼──────────────────────┼───────────────┤
│ rep_clone_facility   │ western_barracks     │ base_tower    │
│ cis_droid_factory    │ western_tech_lab     │ base_wall     │
│ shield_generator     │ enemy_barracks       │               │
└──────────────────────┴──────────────────────┴───────────────┘

┌─────────────────────────────────────────────────────────────┐
│                       Factions                              │
├──────────────────────┬──────────────────────┬───────────────┤
│ In A Only (green)    │ In B Only (blue)     │ In Both (yellow)
├──────────────────────┼──────────────────────┼───────────────┤
│ republic             │ western_alliance     │ classic_enemy │
│ cis                  │ enemy_faction        │               │
└──────────────────────┴──────────────────────┴───────────────┘

Stat Differences in Units:
  rep_v19_torrent:
    hp: 110.0 → 125.0
    damage: 18.0 → 20.0
    armor: 8.0 → 10.0
  cis_tri_fighter:
    hp: 100.0 → 115.0
    fire_rate: 3.5 → 4.0
"@

Write-Host $sampleOutput -ForegroundColor White

Write-Host "`n" -ForegroundColor White
Write-Host "Example JSON output:" -ForegroundColor Cyan
Write-Host @"
{
  "packA": "warfare-starwars",
  "packB": "warfare-modern",
  "units": {
    "onlyInA": [ "rep_clone_militia", "rep_clone_trooper", "rep_arc_trooper", ... ],
    "onlyInB": [ "western_rifleman", "western_squad", "western_scout", ... ],
    "inBoth": [ "rep_v19_torrent", "cis_tri_fighter" ],
    "statDiffs": {
      "rep_v19_torrent": {
        "hp": [ 110.0, 125.0 ],
        "damage": [ 18.0, 20.0 ],
        "armor": [ 8.0, 10.0 ]
      }
    }
  },
  "buildings": { ... },
  "factions": { ... },
  "weapons": { ... },
  "doctrines": { ... }
}
"@ -ForegroundColor Gray

Write-Host "`nCommand Usage:" -ForegroundColor Green
Write-Host "
# Compare two packs (table format)
dinoforge pack diff warfare-starwars warfare-modern

# Compare with detailed stat differences
dinoforge pack diff warfare-starwars warfare-modern --show-stats

# Output as JSON for machine processing
dinoforge pack diff warfare-starwars warfare-modern --format json

# Full example with all options
dinoforge pack diff warfare-starwars warfare-modern --show-stats --format json
" -ForegroundColor Cyan
