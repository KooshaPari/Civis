#!/bin/bash
set -e

echo "=== Testing Pack Loading ==="
echo ""

# List all deployed packs
echo "Deployed packs:"
ls -1 "G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/dinoforge_packs/"
echo ""

# Check warfare pack manifests
echo "Checking warfare pack manifests..."
for pack in warfare-modern warfare-starwars warfare-guerrilla; do
    pack_dir="G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/dinoforge_packs/$pack"
    echo "  $pack:"
    grep "^id\|^name\|^version\|^conflicts_with" "$pack_dir/pack.yaml" | head -4
    echo ""
done

echo "=== Checking for YAML files ==="
echo ""
for pack in warfare-modern warfare-starwars warfare-guerrilla; do
    pack_dir="G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/dinoforge_packs/$pack"
    echo "$pack YAML files:"
    find "$pack_dir" -name "*.yaml" | wc -l
    echo "  Units: $(ls -1 "$pack_dir/units" 2>/dev/null | wc -l)"
    echo "  Buildings: $(ls -1 "$pack_dir/buildings" 2>/dev/null | wc -l)"
    echo "  Weapons: $(ls -1 "$pack_dir/weapons" 2>/dev/null | wc -l)"
    echo "  Doctrines: $(ls -1 "$pack_dir/doctrines" 2>/dev/null | wc -l)"
    echo ""
done

