#!/usr/bin/env python3
"""
Generate faction-color texture variants for DINO Star Wars buildings.
Uses PIL to recolor base Kenney textures via HSL shift.
"""

import os
import sys
from PIL import Image
import colorsys

# Output directory
OUTPUT_DIR = r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\textures\buildings"

# Kenney source textures (representative base textures)
KENNEY_SOURCES = {
    "house": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_01.png",
    "farm": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_04.png",
    "granary": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_07.png",
    "hospital": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_10.png",
    "forester": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_13.png",
    "stone": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_16.png",
    "iron": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\modular-space-kit\Models\Textures\variation-a.png",
    "soul": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\modular-space-kit\Models\Textures\variation-b.png",
    "builder": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_19.png",
    "guild": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\sci-fi-rts\PNG\Default size\Environment\scifiEnvironment_20.png",
    "gate": r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\source\kenney\modular-space-kit\Models\Textures\colormap.png",
}

# Faction color schemes
FACTION_COLORS = {
    "rep": {
        "name": "Republic",
        "body_hex": "#F5F5F5",
        "accent_hex": "#1A3A6B",
        "body_rgb": (245, 245, 245),
        "accent_rgb": (26, 58, 107),
    },
    "cis": {
        "name": "CIS",
        "body_hex": "#444444",
        "accent_hex": "#B35A00",
        "body_rgb": (68, 68, 68),
        "accent_rgb": (179, 90, 0),
    },
}

# Building IDs (24 total)
BUILDINGS = {
    "rep": [
        ("house", "clone_quarters"),
        ("farm", "hydroponic"),
        ("granary", "synthesizer"),
        ("hospital", "medbay"),
        ("forester", "extraction_post"),
        ("stone", "durasteel_refinery"),
        ("iron", "deep_core_rig"),
        ("iron", "tibanna_extractor"),
        ("soul", "crystal_excavator"),
        ("builder", "engineer_corps"),
        ("guild", "engineering_lab"),
        ("gate", "blast_gate"),
    ],
    "cis": [
        ("house", "droid_pod"),
        ("farm", "fuel_harvester"),
        ("granary", "power_depot"),
        ("hospital", "repair_station"),
        ("forester", "raw_extractor"),
        ("stone", "scrap_works"),
        ("iron", "ore_plant"),
        ("iron", "endless_extractor"),
        ("soul", "dark_energy_tap"),
        ("builder", "droid_bay"),
        ("guild", "techno_workshop"),
        ("gate", "security_barrier"),
    ],
}

def recolor_image(base_path, target_body_rgb, target_accent_rgb, output_size=512):
    """Recolor base texture via HSL shift."""
    try:
        img = Image.open(base_path).convert('RGB')
    except Exception as e:
        print(f"ERROR: Could not load {base_path}: {e}")
        return None

    img = img.resize((output_size, output_size), Image.Resampling.LANCZOS)

    pixels = img.load()
    width, height = img.size

    body_h, body_s, body_l = colorsys.rgb_to_hls(
        target_body_rgb[0]/255.0, target_body_rgb[1]/255.0, target_body_rgb[2]/255.0
    )
    accent_h, accent_s, accent_l = colorsys.rgb_to_hls(
        target_accent_rgb[0]/255.0, target_accent_rgb[1]/255.0, target_accent_rgb[2]/255.0
    )

    for y in range(height):
        for x in range(width):
            r, g, b = pixels[x, y][:3] if len(pixels[x, y]) > 3 else pixels[x, y]

            h, l, s = colorsys.rgb_to_hls(r/255.0, g/255.0, b/255.0)

            if l < 0.5:
                new_h, new_s = accent_h, accent_s
            else:
                new_h, new_s = body_h, body_s

            new_r, new_g, new_b = colorsys.hls_to_rgb(new_h, l, new_s)
            pixels[x, y] = (
                int(new_r * 255),
                int(new_g * 255),
                int(new_b * 255),
            )

    return img

def create_placeholder(faction, building_id, output_size=512):
    """Create simple faction-colored placeholder."""
    faction_data = FACTION_COLORS[faction]
    body_rgb = faction_data["body_rgb"]
    accent_rgb = faction_data["accent_rgb"]

    img = Image.new('RGB', (output_size, output_size), body_rgb)
    pixels = img.load()

    accent_width = output_size // 4
    for y in range(output_size):
        for x in range(output_size - accent_width, output_size):
            pixels[x, y] = accent_rgb

    return img

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    generated = []
    failed = []

    for faction, building_list in BUILDINGS.items():
        faction_data = FACTION_COLORS[faction]
        body_rgb = faction_data["body_rgb"]
        accent_rgb = faction_data["accent_rgb"]

        print(f"\n[{faction.upper()}] Generating {len(building_list)} textures...")

        for building_type, building_suffix in building_list:
            if building_type == "iron" and building_suffix == "tibanna_extractor":
                output_name = f"{faction}_iron_tibanna_extractor_albedo.png"
            elif building_type == "iron" and building_suffix == "endless_extractor":
                output_name = f"{faction}_iron_endless_extractor_albedo.png"
            else:
                output_name = f"{faction}_{building_type}_{building_suffix}_albedo.png"

            output_path = os.path.join(OUTPUT_DIR, output_name)

            if building_type not in KENNEY_SOURCES:
                print(f"  WARN: No Kenney source for {building_type}, using placeholder")
                img = create_placeholder(faction, output_name)
                if img:
                    img.save(output_path, 'PNG')
                    generated.append((output_name, "placeholder"))
                continue

            kenney_path = KENNEY_SOURCES[building_type]

            if not os.path.exists(kenney_path):
                print(f"  WARN: Kenney source not found: {kenney_path}, using placeholder")
                img = create_placeholder(faction, output_name)
                if img:
                    img.save(output_path, 'PNG')
                    generated.append((output_name, "placeholder"))
                continue

            try:
                img = recolor_image(kenney_path, body_rgb, accent_rgb, output_size=512)
                if img:
                    img.save(output_path, 'PNG')
                    print(f"  OK: {output_name}")
                    generated.append((output_name, "kenney"))
                else:
                    failed.append((output_name, "recolor_failed"))
            except Exception as e:
                print(f"  ERR: {output_name}: {e}")
                failed.append((output_name, str(e)))

    print(f"\n{'='*60}")
    print(f"GENERATED: {len(generated)}/24 textures")
    print(f"FAILED: {len(failed)}/24")

    if failed:
        print(f"\nFailed textures:")
        for name, reason in failed:
            print(f"  - {name}: {reason}")

    return 0 if len(generated) == 24 else 1

if __name__ == '__main__':
    sys.exit(main())
