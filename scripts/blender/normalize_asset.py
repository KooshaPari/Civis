#!/usr/bin/env python3
"""
Normalize asset script for Blender headless CLI.
Imports GLB, merges materials, generates LOD levels, exports normalized + LOD files.

Usage:
  blender --background --python normalize_asset.py -- <input_glb> <output_dir> <target_polycount>
"""

import bpy
import sys
import json
import os
from pathlib import Path

def main():
    # Parse command-line arguments (everything after --)
    argv = sys.argv
    try:
        idx = argv.index("--") + 1
        input_glb = argv[idx]
        output_dir = argv[idx + 1]
        target_polycount = int(argv[idx + 2])
    except (ValueError, IndexError):
        print(json.dumps({"success": False, "error": "Invalid arguments"}))
        return False

    os.makedirs(output_dir, exist_ok=True)

    try:
        # Clear default scene
        bpy.ops.wm.read_homefile(use_empty=True)

        # Import GLB file
        bpy.ops.import_scene.gltf(filepath=input_glb)

        # Get all mesh objects
        objects = [o for o in bpy.data.objects if o.type == 'MESH']
        if not objects:
            raise RuntimeError(f"No mesh objects found in {input_glb}")

        # Calculate original polycount
        original_poly = sum(len(o.data.polygons) for o in objects)

        # Create a single Principled BSDF material for normalization
        mat = bpy.data.materials.new(name="Normalized_PBR")
        mat.use_nodes = True
        # Access the Principled BSDF node (created by default in new material)
        # For Blender 3.x and 4.x compatibility, we clear and rebuild
        mat.node_tree.nodes.clear()
        mat.node_tree.links.clear()

        # Create output node
        output_node = mat.node_tree.nodes.new("ShaderNodeOutputMaterial")
        # Create Principled BSDF
        bsdf = mat.node_tree.nodes.new("ShaderNodeBsdfPrincipled")
        # Connect
        mat.node_tree.links.new(bsdf.outputs["BSDF"], output_node.inputs["Surface"])

        # Assign material to all meshes and clear their existing materials
        for obj in objects:
            obj.data.materials.clear()
            obj.data.materials.append(mat)

        # Export LOD0 (full quality, no decimation)
        lod0_path = os.path.join(output_dir, "normalized.glb")
        bpy.ops.export_scene.gltf(filepath=lod0_path, export_format='GLB')
        lod0_poly = original_poly

        # Apply 50% decimate for LOD1
        for obj in objects:
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.select_all(action='DESELECT')
            obj.select_set(True)

            mod = obj.modifiers.new(name="Decimate", type='DECIMATE')
            mod.decimate_type = 'COLLAPSE'
            mod.ratio = 0.5

            bpy.ops.object.modifier_apply(modifier=mod.name)

        # Export LOD1
        lod1_path = os.path.join(output_dir, "lod1.glb")
        bpy.ops.export_scene.gltf(filepath=lod1_path, export_format='GLB')
        lod1_poly = sum(len(o.data.polygons) for o in objects)

        # Apply another 50% decimate for LOD2 (total ~25% of original)
        for obj in objects:
            bpy.context.view_layer.objects.active = obj
            bpy.ops.object.select_all(action='DESELECT')
            obj.select_set(True)

            mod = obj.modifiers.new(name="Decimate", type='DECIMATE')
            mod.decimate_type = 'COLLAPSE'
            mod.ratio = 0.5

            bpy.ops.object.modifier_apply(modifier=mod.name)

        # Export LOD2
        lod2_path = os.path.join(output_dir, "lod2.glb")
        bpy.ops.export_scene.gltf(filepath=lod2_path, export_format='GLB')
        lod2_poly = sum(len(o.data.polygons) for o in objects)

        # Generate report
        report = {
            "success": True,
            "original_polycount": original_poly,
            "lod0_polycount": lod0_poly,
            "lod1_polycount": lod1_poly,
            "lod2_polycount": lod2_poly,
            "material_count": 1,
            "output_files": ["normalized.glb", "lod1.glb", "lod2.glb"],
            "output_dir": output_dir
        }

        # Write report to file
        report_path = os.path.join(output_dir, "normalization_report.json")
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)

        # Also print to stdout for C# to capture
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
