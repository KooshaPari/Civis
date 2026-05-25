#!/usr/bin/env python3
"""
Stylize asset script for Blender headless CLI.
Imports normalized GLB, applies faction-specific PBR materials, renders preview.

Usage:
  blender --background --python stylize_asset.py -- <input_glb> <output_dir> <palette_json_path>
"""

import bpy
import sys
import json
import os
from pathlib import Path

def hex_to_linear_rgb(hex_color):
    """Convert #RRGGBB hex color to Blender linear RGB (0.0-1.0 per channel, gamma-corrected)."""
    hex_color = hex_color.lstrip('#')
    if len(hex_color) != 6:
        raise ValueError(f"Invalid hex color: {hex_color}")

    r = int(hex_color[0:2], 16)
    g = int(hex_color[2:4], 16)
    b = int(hex_color[4:6], 16)

    def srgb_to_linear(c):
        c = c / 255.0
        if c <= 0.04045:
            return c / 12.92
        else:
            return ((c + 0.055) / 1.055) ** 2.4

    return (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), 1.0)


def create_faction_material(mat_name, hex_color, roughness, metallic):
    """Create a Principled BSDF material with the given color and properties."""
    mat = bpy.data.materials.new(name=mat_name)
    mat.use_nodes = True

    # Clear default nodes
    mat.node_tree.nodes.clear()
    mat.node_tree.links.clear()

    # Create new nodes
    output = mat.node_tree.nodes.new("ShaderNodeOutputMaterial")
    bsdf = mat.node_tree.nodes.new("ShaderNodeBsdfPrincipled")
    mat.node_tree.links.new(bsdf.outputs["BSDF"], output.inputs["Surface"])

    # Set material properties
    bsdf.inputs["Base Color"].default_value = hex_to_linear_rgb(hex_color)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic

    return mat


def apply_faction_materials(objects, palette):
    """Apply faction-specific materials using palette colors."""
    faction = palette.get("faction", "neutral").lower()
    roughness = palette.get("roughness", 0.5)
    metallic = palette.get("metallic", 0.0)

    if faction == "republic":
        # Republic: white armor, navy stripe, gold trim
        primary_hex = palette.get("primary", "#F5F5F5")
        secondary_hex = palette.get("secondary", "#1A3A6B")

        mat_primary = create_faction_material("Republic_Primary", primary_hex, roughness=0.3, metallic=0.1)
        mat_secondary = create_faction_material("Republic_Secondary", secondary_hex, roughness=0.4, metallic=0.05)

        for obj in objects:
            obj.data.materials.clear()
            obj.data.materials.append(mat_primary)
            if len(obj.material_slots) > 1:
                obj.data.materials.append(mat_secondary)

    elif faction == "cis":
        # CIS: tan base, dark brown joints/recesses
        primary_hex = palette.get("primary", "#C8A87A")
        secondary_hex = palette.get("secondary", "#5C3D1E")

        mat_primary = create_faction_material("CIS_Primary", primary_hex, roughness=0.7, metallic=0.2)
        mat_secondary = create_faction_material("CIS_Secondary", secondary_hex, roughness=0.8, metallic=0.1)

        for obj in objects:
            obj.data.materials.clear()
            obj.data.materials.append(mat_primary)
            if len(obj.material_slots) > 1:
                obj.data.materials.append(mat_secondary)

    else:
        # Neutral: single material
        primary_hex = palette.get("primary", "#888888")
        mat = create_faction_material("Neutral_Material", primary_hex, roughness=roughness, metallic=metallic)

        for obj in objects:
            obj.data.materials.clear()
            obj.data.materials.append(mat)


def render_preview(output_dir, asset_name):
    """Render a 256x256 preview thumbnail (non-fatal if it fails)."""
    try:
        scene = bpy.context.scene
        scene.render.engine = 'BLENDER_EEVEE'

        # Detect Blender version for EEVEE_NEXT in 4.0+
        bl_version = bpy.app.version
        if bl_version >= (4, 0, 0):
            scene.render.engine = 'BLENDER_EEVEE_NEXT'

        scene.render.resolution_x = 256
        scene.render.resolution_y = 256
        scene.render.image_settings.file_format = 'PNG'
        scene.render.filepath = os.path.join(output_dir, "preview.png")

        # Frame all objects
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.view3d.view_all(use_all_regions=False)

        # Render
        bpy.ops.render.render(write_still=True)

    except Exception as e:
        # Preview rendering is non-fatal - log but don't fail
        print(f"Warning: Failed to render preview: {e}", file=sys.stderr)


def main():
    # Parse command-line arguments
    argv = sys.argv
    try:
        idx = argv.index("--") + 1
        input_glb = argv[idx]
        output_dir = argv[idx + 1]
        palette_json_path = argv[idx + 2]
    except (ValueError, IndexError):
        print(json.dumps({"success": False, "error": "Invalid arguments"}))
        return False

    os.makedirs(output_dir, exist_ok=True)

    try:
        # Load palette JSON
        with open(palette_json_path, "r") as f:
            palette = json.load(f)

        # Clear default scene
        bpy.ops.wm.read_homefile(use_empty=True)

        # Import normalized GLB
        bpy.ops.import_scene.gltf(filepath=input_glb)

        # Get all mesh objects
        objects = [o for o in bpy.data.objects if o.type == 'MESH']
        if not objects:
            raise RuntimeError(f"No mesh objects found in {input_glb}")

        # Apply faction materials
        apply_faction_materials(objects, palette)

        # Export stylized GLB
        stylized_glb = os.path.join(output_dir, "stylized.glb")
        bpy.ops.export_scene.gltf(filepath=stylized_glb, export_format='GLB')

        # Save .blend file
        stylized_blend = os.path.join(output_dir, "stylized.blend")
        bpy.ops.wm.save_as_mainfile(filepath=stylized_blend)

        # Render preview (non-fatal)
        asset_name = palette.get("asset_name", "asset")
        render_preview(output_dir, asset_name)

        # Generate report
        report = {
            "success": True,
            "faction": palette.get("faction", "neutral"),
            "material_count": len(bpy.data.materials),
            "output_files": ["stylized.glb", "stylized.blend", "preview.png"],
            "output_dir": output_dir
        }

        # Write report to file
        report_path = os.path.join(output_dir, "stylization_report.json")
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)

        # Print to stdout for C# to capture
        print(json.dumps(report))
        return True

    except Exception as e:
        error_report = {
            "success": False,
            "error": str(e),
            "output_dir": output_dir
        }
        print(json.dumps(error_report), file=sys.stderr)
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
