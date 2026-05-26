#!/usr/bin/env python3
"""
blender_assemble_buildings.py

Blender batch assembly script for Star Wars building models.
Imports Kenney source FBX, applies faction textures, and exports game-ready models.

Run with:
  blender --python blender_assemble_buildings.py -- --output assets/meshes/buildings

This script is designed to run in Blender's embedded Python environment.
It can also be called from command line with appropriate Blender version.
"""

import bpy
import sys
import os
from pathlib import Path
from dataclasses import dataclass
from typing import Optional, Tuple

print(f"Blender {bpy.app.version_string}")
print(f"Python {sys.version}")


@dataclass
class BuildingConfig:
    """Configuration for a single building assembly."""
    building_id: str              # e.g., "rep_house_clone_quarters"
    source_fbx: str               # Kenney source, e.g., "structure.fbx"
    faction: str                  # "rep" or "cis"
    faction_palette: dict         # RGB color values
    description: str              # For logging
    target_tris: Tuple[int, int] = (300, 600)  # Min-max tri count
    add_details: bool = True      # Add faction-specific details
    custom_details: Optional[str] = None  # Custom Blender operations


class BuildingAssembler:
    """Manages building assembly workflow in Blender."""

    def __init__(self, source_dir: Path, output_meshes_dir: Path, output_textures_dir: Path):
        self.source_dir = Path(source_dir)
        self.output_meshes = Path(output_meshes_dir)
        self.output_textures = Path(output_textures_dir)

        # Ensure output directories exist
        self.output_meshes.mkdir(parents=True, exist_ok=True)
        self.output_textures.mkdir(parents=True, exist_ok=True)

    def clear_scene(self):
        """Remove all objects from the current scene."""
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.delete(use_global=False)

    def import_kenney_fbx(self, relative_fbx_path: str) -> Optional[bpy.types.Object]:
        """
        Import a Kenney FBX model.

        Args:
            relative_fbx_path: Path relative to source/kenney/, e.g., "space-kit/structure.fbx"

        Returns:
            Imported object, or None if failed
        """
        # Construct full path to Kenney model
        fbx_path = self.source_dir / "space-kit" / "Models" / "FBX format" / relative_fbx_path

        if not fbx_path.exists():
            print(f"  [ERROR] FBX not found: {fbx_path}")
            return None

        print(f"  Importing: {fbx_path}")

        try:
            bpy.ops.import_scene.fbx(filepath=str(fbx_path))
            imported = bpy.context.selected_objects[0] if bpy.context.selected_objects else None
            return imported
        except Exception as e:
            print(f"  [ERROR] Import failed: {e}")
            return None

    def apply_faction_material(self, obj: bpy.types.Object, palette: dict, texture_path: Path):
        """
        Apply faction material and texture to object.

        Args:
            obj: Blender object
            palette: Dict with RGB colors (primary, accent, light, dark, energy)
            texture_path: Path to faction texture PNG
        """
        # Create material
        mat = bpy.data.materials.new(name=f"{obj.name}_faction_material")
        mat.use_nodes = True
        nodes = mat.node_tree.nodes
        links = mat.node_tree.links

        # Clear default nodes
        nodes.clear()

        # Create node tree: Texture → Principled BSDF → Output
        texture_node = nodes.new(type='ShaderNodeTexImage')
        bsdf_node = nodes.new(type='ShaderNodeBsdfPrincipled')
        output_node = nodes.new(type='ShaderNodeOutputMaterial')

        # Load texture if it exists
        if texture_path.exists():
            image = bpy.data.images.load(str(texture_path))
            texture_node.image = image
            links.new(texture_node.outputs['Color'], bsdf_node.inputs['Base Color'])
        else:
            # Fallback: use palette primary color directly
            bsdf_node.inputs['Base Color'].default_value = (
                palette['primary'][0] / 255,
                palette['primary'][1] / 255,
                palette['primary'][2] / 255,
                1.0
            )

        # Configure BSDF
        bsdf_node.inputs['Metallic'].default_value = 0.2
        bsdf_node.inputs['Roughness'].default_value = 0.8

        # Output
        links.new(bsdf_node.outputs['BSDF'], output_node.inputs['Surface'])

        # Assign material to object
        if obj.data.materials:
            obj.data.materials[0] = mat
        else:
            obj.data.materials.append(mat)

        print(f"  Applied material: {mat.name}")

    def optimize_object(self, obj: bpy.types.Object, target_tris: Tuple[int, int]) -> int:
        """
        Optimize object poly count.

        Args:
            obj: Blender object
            target_tris: (min_tris, max_tris) target range

        Returns:
            Actual tri count
        """
        # Get current tri count
        mesh = obj.data
        tri_count = len(mesh.polygons)

        print(f"  Poly count: {tri_count} (target: {target_tris[0]}-{target_tris[1]})")

        if tri_count > target_tris[1]:
            print(f"  [WARNING] Poly count exceeds target; consider decimation or manual retopology")

        return tri_count

    def add_faction_details(self, obj: bpy.types.Object, faction: str):
        """
        Add faction-specific details (decals, markers, etc.).
        This is a placeholder for manual detail work in Blender.

        Args:
            obj: Blender object
            faction: "rep" or "cis"
        """
        print(f"  Adding {faction.upper()} faction details...")

        # Placeholder: In production, this would add:
        # - Republic: emblems, panel lines, blue stripes
        # - CIS: orange stripes, rust overlays, droid markings

        if faction == "rep":
            # Republic details would go here
            pass
        elif faction == "cis":
            # CIS details would go here
            pass

    def assemble_building(self, config: BuildingConfig) -> bool:
        """
        Assemble a single building.

        Args:
            config: BuildingConfig with assembly instructions

        Returns:
            True if successful, False otherwise
        """
        print(f"\n[ASSEMBLE] {config.building_id}")
        print(f"  Description: {config.description}")
        print(f"  Faction: {config.faction.upper()}")

        try:
            self.clear_scene()

            # Import source FBX
            imported = self.import_kenney_fbx(config.source_fbx)
            if not imported:
                return False

            # Apply faction material
            texture_path = self.output_textures / f"{config.building_id}_albedo.png"
            self.apply_faction_material(imported, config.faction_palette, texture_path)

            # Add faction details
            if config.add_details:
                self.add_faction_details(imported, config.faction)

            # Optimize
            tri_count = self.optimize_object(imported, config.target_tris)

            # Export as FBX
            output_fbx = self.output_meshes / f"{config.building_id}.fbx"
            print(f"  Exporting: {output_fbx}")

            bpy.ops.export_scene.fbx(
                filepath=str(output_fbx),
                use_selection=True,
                use_active_layer=True,
                bake_anim_use_nla_strips=False,
            )

            print(f"  [OK] {config.building_id}.fbx ({tri_count} tris)")
            return True

        except Exception as e:
            print(f"  [ERROR] {e}")
            return False


def get_republic_palette() -> dict:
    """Return Republic faction palette."""
    return {
        'primary': (245, 245, 245),      # #F5F5F5
        'accent': (26, 58, 107),         # #1A3A6B
        'light': (238, 238, 238),        # #EEEEEE
        'dark': (100, 100, 100),         # #646464
        'energy': (68, 136, 255),        # #4488FF
    }


def get_cis_palette() -> dict:
    """Return CIS faction palette."""
    return {
        'primary': (68, 68, 68),         # #444444
        'accent': (179, 90, 0),          # #B35A00
        'light': (85, 85, 85),           # #555555
        'dark': (42, 42, 42),            # #2A2A2A
        'energy': (255, 102, 0),         # #FF6600
    }


def main():
    """Main assembly workflow."""

    # Parse command-line arguments
    # Note: Blender passes args after "--" to Python script
    output_meshes_dir = Path("assets/meshes/buildings")
    output_textures_dir = Path("assets/textures/buildings")
    source_dir = Path("assets/source/kenney")

    # Command-line argument parsing for Blender
    if "--output" in sys.argv:
        idx = sys.argv.index("--output")
        output_meshes_dir = Path(sys.argv[idx + 1])

    print(f"\n=== Star Wars Building Assembler ===")
    print(f"Source: {source_dir}")
    print(f"Output meshes: {output_meshes_dir}")
    print(f"Output textures: {output_textures_dir}")

    assembler = BuildingAssembler(source_dir, output_meshes_dir, output_textures_dir)

    # Define buildings to assemble (pilot: just 2)
    buildings = [
        # Pilot buildings (simple structures)
        BuildingConfig(
            building_id="rep_house_clone_quarters",
            source_fbx="structure.fbx",
            faction="rep",
            faction_palette=get_republic_palette(),
            description="Modular circular habitat unit",
            target_tris=(300, 400),
        ),
        BuildingConfig(
            building_id="cis_house_droid_pod",
            source_fbx="structure.fbx",
            faction="cis",
            faction_palette=get_cis_palette(),
            description="Stacked droid deactivation rack",
            target_tris=(300, 400),
        ),
    ]

    # Assemble buildings
    success_count = 0
    fail_count = 0

    for config in buildings:
        if assembler.assemble_building(config):
            success_count += 1
        else:
            fail_count += 1

    print(f"\n=== SUMMARY ===")
    print(f"Success: {success_count}")
    print(f"Failed: {fail_count}")

    return 0 if fail_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
