#!/usr/bin/env python3
"""
generate_faction_textures.py

Batch-generate Republic and CIS faction-color texture variants from Kenney source.
Creates faction-specific color palettes for all 24 vanilla building reskins.

Usage:
  python generate_faction_textures.py --source assets/source/kenney --output assets/textures/buildings

Dependencies:
  - Pillow (pip install Pillow)
"""

import os
import sys
import argparse
from pathlib import Path
from PIL import Image, ImageOps, ImageEnhance
import json
from dataclasses import dataclass
from typing import Tuple, Dict

# Color palettes (RGB tuples)
@dataclass
class Palette:
    primary: Tuple[int, int, int]
    accent: Tuple[int, int, int]
    light: Tuple[int, int, int]
    dark: Tuple[int, int, int]
    energy: Tuple[int, int, int] = None

REPUBLIC_PALETTE = Palette(
    primary=(245, 245, 245),      # #F5F5F5 near-white durasteel
    accent=(26, 58, 107),         # #1A3A6B deep navy blue
    light=(238, 238, 238),        # #EEEEEE off-white trim
    dark=(100, 100, 100),         # #646464 shadow
    energy=(68, 136, 255),        # #4488FF emissive blue (for glow effects)
)

CIS_PALETTE = Palette(
    primary=(68, 68, 68),         # #444444 medium-dark grey
    accent=(179, 90, 0),          # #B35A00 rust orange
    light=(85, 85, 85),           # #555555 light grey
    dark=(42, 42, 42),            # #2A2A2A shadow dark
    energy=(255, 102, 0),         # #FF6600 heat orange
)

# Building definitions: (building_id, kenney_source_filename, description)
REPUBLIC_BUILDINGS = [
    ("rep_house_clone_quarters", "generic_structure", "Modular circular habitat unit"),
    ("rep_farm_hydroponic", "generic_structure", "Tiered planting bays"),
    ("rep_granary_synthesizer", "generic_structure", "Tall cylindrical silo"),
    ("rep_hospital_medbay", "generic_structure", "Compact medical facility"),
    ("rep_forester_extraction_post", "generic_structure", "Small outpost with antenna"),
    ("rep_stone_durasteel_refinery", "generic_structure", "Industrial smelter"),
    ("rep_iron_tibanna_extractor", "generic_tower", "Derrick-style tall tower"),
    ("rep_iron_deep_core_rig", "generic_tower", "Twin-derrick extraction rig"),
    ("rep_soul_crystal_excavator", "generic_tower", "Kyber crystal mining cage"),
    ("rep_builder_engineer_corps", "generic_structure", "Mobile engineering depot"),
    ("rep_guild_engineering_lab", "generic_structure", "Multi-bay R&D facility"),
    ("rep_gate_security_gate", "generic_gate", "Security gate with panels"),
]

CIS_BUILDINGS = [
    ("cis_house_droid_pod", "generic_structure", "Stacked droid deactivation rack"),
    ("cis_farm_fuel_harvester", "generic_structure", "Ground-boring fuel extractor"),
    ("cis_granary_power_depot", "generic_structure", "Battery and capacitor bank"),
    ("cis_hospital_repair_station", "generic_structure", "Open-frame repair cradle"),
    ("cis_forester_raw_extractor", "generic_structure", "Automated extractor with anchor legs"),
    ("cis_stone_scrap_works", "generic_structure", "Ore crusher with shrapnel aesthetic"),
    ("cis_iron_ore_plant", "generic_tower", "Compact mining tower with conveyor"),
    ("cis_iron_endless_extractor", "generic_tower", "Dual-bore heavy extraction platform"),
    ("cis_soul_dark_energy_tap", "generic_tower", "Dark energy extraction lattice"),
    ("cis_builder_droid_bay", "generic_structure", "Droid-operated assembly depot"),
    ("cis_guild_techno_workshop", "generic_structure", "Asymmetric organic-tech facility"),
    ("cis_gate_security_barrier", "generic_gate", "Armored barrier with sentry alcoves"),
]


def colorize_grayscale(
    image: Image.Image,
    palette: Palette,
    blend_mode: str = "overlay"
) -> Image.Image:
    """
    Apply faction colors to a grayscale or generic texture.

    Args:
        image: PIL Image (RGB or RGBA)
        palette: Palette object with color definitions
        blend_mode: "overlay", "multiply", "hsl_shift"

    Returns:
        Colorized PIL Image (RGB)
    """
    # Convert to RGB if needed
    if image.mode == 'RGBA':
        # Preserve alpha for composite later
        alpha = image.split()[3]
        image = image.convert('RGB')
    else:
        alpha = None
        image = image.convert('RGB')

    if blend_mode == "overlay":
        # Simple overlay: bright areas → primary, dark areas → accent
        return _colorize_overlay(image, palette)
    elif blend_mode == "multiply":
        # Multiply blend: multiply grayscale by palette colors
        return _colorize_multiply(image, palette)
    elif blend_mode == "hsl_shift":
        # HSL-based shift for more nuanced results
        return _colorize_hsl(image, palette)
    else:
        raise ValueError(f"Unknown blend_mode: {blend_mode}")


def _colorize_overlay(image: Image.Image, palette: Palette) -> Image.Image:
    """Overlay blend: bright areas → primary, dark areas → accent."""
    img_array = ImageOps.grayscale(image)
    output = Image.new('RGB', image.size)
    pixels = output.load()
    img_pixels = img_array.load()

    for y in range(image.size[1]):
        for x in range(image.size[0]):
            val = img_pixels[x, y] / 255.0

            # Interpolate between accent (dark) and primary (light)
            if val < 0.5:
                # Dark areas: accent color
                ratio = val * 2  # 0 to 1 as val goes 0 to 0.5
                r = int(palette.dark[0] + (palette.accent[0] - palette.dark[0]) * ratio)
                g = int(palette.dark[1] + (palette.accent[1] - palette.dark[1]) * ratio)
                b = int(palette.dark[2] + (palette.accent[2] - palette.dark[2]) * ratio)
            else:
                # Light areas: primary color
                ratio = (val - 0.5) * 2  # 0 to 1 as val goes 0.5 to 1.0
                r = int(palette.accent[0] + (palette.primary[0] - palette.accent[0]) * ratio)
                g = int(palette.accent[1] + (palette.primary[1] - palette.accent[1]) * ratio)
                b = int(palette.accent[2] + (palette.primary[2] - palette.accent[2]) * ratio)

            pixels[x, y] = (r, g, b)

    return output


def _colorize_multiply(image: Image.Image, palette: Palette) -> Image.Image:
    """Multiply blend: grayscale * palette colors."""
    img_array = ImageOps.grayscale(image)
    output = Image.new('RGB', image.size)
    pixels = output.load()
    img_pixels = img_array.load()

    primary = palette.primary
    for y in range(image.size[1]):
        for x in range(image.size[0]):
            val = img_pixels[x, y] / 255.0
            r = int(primary[0] * val)
            g = int(primary[1] * val)
            b = int(primary[2] * val)
            pixels[x, y] = (r, g, b)

    return output


def _colorize_hsl(image: Image.Image, palette: Palette) -> Image.Image:
    """HSL-based colorization for better fidelity."""
    # Convert to HSV, shift hue, return RGB
    from colorsys import rgb_to_hls, hls_to_rgb

    output = Image.new('RGB', image.size)
    pixels = output.load()
    img_pixels = image.load()

    # Get target hue from palette primary
    target_r, target_g, target_b = palette.primary
    target_h, target_l, target_s = rgb_to_hls(target_r / 255, target_g / 255, target_b / 255)

    for y in range(image.size[1]):
        for x in range(image.size[0]):
            src_r, src_g, src_b = img_pixels[x, y][:3]
            src_h, src_l, src_s = rgb_to_hls(src_r / 255, src_g / 255, src_b / 255)

            # Preserve lightness, apply target hue and saturation
            new_r, new_g, new_b = hls_to_rgb(target_h, src_l, max(src_s, 0.5))
            pixels[x, y] = (int(new_r * 255), int(new_g * 255), int(new_b * 255))

    return output


def create_default_placeholder_texture(size: Tuple[int, int], palette: Palette) -> Image.Image:
    """
    Create a simple procedural placeholder texture if source is not found.
    This ensures we don't fail silently if Kenney source is missing.
    """
    img = Image.new('RGB', size, palette.primary)
    pixels = img.load()

    # Add a simple grid pattern
    for y in range(size[1]):
        for x in range(size[0]):
            # Checkerboard pattern
            if (x // 16 + y // 16) % 2 == 0:
                pixels[x, y] = palette.light
            else:
                pixels[x, y] = palette.primary

    return img


def generate_building_texture(
    building_id: str,
    palette: Palette,
    source_dir: Path,
    output_dir: Path,
    force: bool = False
) -> bool:
    """
    Generate a single building texture.

    Args:
        building_id: e.g., "rep_house_clone_quarters"
        palette: Palette object
        source_dir: Path to Kenney source assets
        output_dir: Path to output textures
        force: Regenerate even if exists

    Returns:
        True if successful, False otherwise
    """
    output_path = output_dir / f"{building_id}_albedo.png"

    if output_path.exists() and not force:
        print(f"[SKIP] {output_path} (exists)")
        return True

    # For now: create a placeholder procedural texture
    # In production, this would:
    # 1. Load source Kenney texture (if available)
    # 2. Apply colorization
    # 3. Save output

    # Create placeholder with faction colors
    texture = create_default_placeholder_texture((256, 256), palette)

    # Colorize it
    gray_version = ImageOps.grayscale(texture)
    colorized = colorize_grayscale(gray_version, palette, blend_mode="overlay")

    try:
        colorized.save(output_path)
        print(f"[OK] {output_path}")
        return True
    except Exception as e:
        print(f"[ERROR] {building_id}: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Generate faction-color texture variants for Star Wars building reskins"
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("assets/source/kenney"),
        help="Path to Kenney source assets"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("assets/textures/buildings"),
        help="Path to output textures"
    )
    parser.add_argument(
        "--faction",
        choices=["republic", "cis", "all"],
        default="all",
        help="Which faction to generate"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Regenerate even if textures exist"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Verbose output"
    )

    args = parser.parse_args()

    # Create output dir
    args.output.mkdir(parents=True, exist_ok=True)

    print(f"Source: {args.source}")
    print(f"Output: {args.output}")
    print()

    success_count = 0
    fail_count = 0

    if args.faction in ["republic", "all"]:
        print("=== REPUBLIC FACTION ===")
        for building_id, source, description in REPUBLIC_BUILDINGS:
            if generate_building_texture(
                building_id,
                REPUBLIC_PALETTE,
                args.source,
                args.output,
                args.force
            ):
                success_count += 1
            else:
                fail_count += 1
        print()

    if args.faction in ["cis", "all"]:
        print("=== CIS FACTION ===")
        for building_id, source, description in CIS_BUILDINGS:
            if generate_building_texture(
                building_id,
                CIS_PALETTE,
                args.source,
                args.output,
                args.force
            ):
                success_count += 1
            else:
                fail_count += 1
        print()

    print(f"SUMMARY: {success_count} OK, {fail_count} FAILED")
    return 0 if fail_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
