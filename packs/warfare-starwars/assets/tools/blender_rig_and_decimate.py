#!/usr/bin/env python3
"""
blender_rig_and_decimate.py — RIGGING-OPTIMIZER (#991)

Retarget a static SW unit mesh onto a DINO-compatible reference armature,
decimate to LOD budgets, and export rigged GLB for Unity bundle import.

Run (Blender 3.x+):
  blender --background --python blender_rig_and_decimate.py -- \\
    --source path/to/static.glb \\
    --reference path/to/dino_reference_rigged.glb \\
    --output path/to/working/asset_id/rigged.glb \\
    --lod0-tris 2000 --lod1-ratio 0.6 --lod2-ratio 0.3

Reference armature must already match DINO vanilla bindpose count/names.
Use weight transfer from reference mesh → SW mesh (not Rigify-from-scratch).
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
except ImportError:
    print("Run inside Blender: blender --background --python blender_rig_and_decimate.py -- ...")
    sys.exit(1)


def parse_args(argv: list[str]) -> argparse.Namespace:
    if "--" in argv:
        argv = argv[argv.index("--") + 1 :]
    p = argparse.ArgumentParser(description="Rig + decimate SW asset for DINOForge")
    p.add_argument("--source", required=True, help="Static GLB/GLTF/FBX to retarget")
    p.add_argument("--reference", required=True, help="DINO reference rigged GLB with target skeleton")
    p.add_argument("--output", required=True, help="Output rigged GLB path")
    p.add_argument("--lod0-tris", type=int, default=2000, help="Target triangle count for LOD0")
    p.add_argument("--lod1-ratio", type=float, default=0.6, help="LOD1 tris as fraction of LOD0")
    p.add_argument("--lod2-ratio", type=float, default=0.3, help="LOD2 tris as fraction of LOD0")
    p.add_argument("--validation-json", default="", help="Optional validation_report.json path")
    return p.parse_args(argv)


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)


def import_file(path: Path) -> list[bpy.types.Object]:
    suffix = path.suffix.lower()
    if suffix in {".glb", ".gltf"}:
        bpy.ops.import_scene.gltf(filepath=str(path))
    elif suffix == ".fbx":
        bpy.ops.import_scene.fbx(filepath=str(path))
    else:
        raise ValueError(f"Unsupported format: {path}")
    return list(bpy.context.selected_objects)


def mesh_objects(objs: list[bpy.types.Object]) -> list[bpy.types.Object]:
    return [o for o in objs if o.type == "MESH"]


def armature_object(objs: list[bpy.types.Object]) -> bpy.types.Object | None:
    for o in objs:
        if o.type == "ARMATURE":
            return o
    return None


def apply_decimate(obj: bpy.types.Object, target_tris: int) -> None:
    current = len(obj.data.polygons)
    if current <= target_tris:
        return
    ratio = max(0.01, min(1.0, target_tris / current))
    mod = obj.modifiers.new(name="DecimateLOD", type="DECIMATE")
    mod.ratio = ratio
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=mod.name)


def transfer_weights(source_obj: bpy.types.Object, target_obj: bpy.types.Object, armature: bpy.types.Object) -> None:
    """Copy vertex groups from reference mesh via Data Transfer."""
    for vg in list(target_obj.vertex_groups):
        target_obj.vertex_groups.remove(vg)
    for vg in source_obj.vertex_groups:
        target_obj.vertex_groups.new(name=vg.name)
    mod = target_obj.modifiers.new(name="WeightTransfer", type="DATA_TRANSFER")
    mod.object = source_obj
    mod.use_vert_data = False
    mod.use_loop_data = False
    mod.use_poly_data = False
    mod.use_vertex_groups = True
    mod.vertex_group_transfer_method = "NEAREST"
    bpy.context.view_layer.objects.active = target_obj
    bpy.ops.object.modifier_apply(modifier=mod.name)
    target_obj.parent = armature
    target_obj.parent_type = "ARMATURE"
    arm_mod = target_obj.modifiers.new(name="Armature", type="ARMATURE")
    arm_mod.object = armature


def validate_rigged(mesh: bpy.types.Object, armature: bpy.types.Object) -> dict:
    issues: list[str] = []
    if mesh.vertex_groups is None or len(mesh.vertex_groups) == 0:
        issues.append("no_vertex_groups")
    bone_count = len(armature.data.bones)
    if bone_count == 0:
        issues.append("empty_armature")
    return {
        "mesh": mesh.name,
        "armature": armature.name,
        "triangles_lod0": len(mesh.data.polygons),
        "vertex_groups": len(mesh.vertex_groups),
        "armature_bones": bone_count,
        "ok": len(issues) == 0,
        "issues": issues,
    }


def export_glb(path: Path, objects: list[bpy.types.Object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for o in objects:
        o.select_set(True)
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_skins=True,
        export_def_bones=True,
        export_animations=False,
    )


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    source = Path(args.source).resolve()
    reference = Path(args.reference).resolve()
    output = Path(args.output).resolve()

    if not source.is_file():
        print(f"[ERROR] source not found: {source}")
        return 1
    if not reference.is_file():
        print(f"[ERROR] reference not found: {reference}")
        return 1

    clear_scene()
    ref_objs = import_file(reference)
    ref_meshes = mesh_objects(ref_objs)
    ref_arm = armature_object(ref_objs)
    if ref_arm is None or not ref_meshes:
        print("[ERROR] reference must contain mesh + armature")
        return 1
    ref_mesh = ref_meshes[0]

    src_objs = import_file(source)
    src_meshes = mesh_objects(src_objs)
    if not src_meshes:
        print("[ERROR] source has no mesh")
        return 1
    target = src_meshes[0]

    transfer_weights(ref_mesh, target, ref_arm)
    apply_decimate(target, args.lod0_tris)

    report = validate_rigged(target, ref_arm)
    report["lod0_target_tris"] = args.lod0_tris
    report["lod1_ratio"] = args.lod1_ratio
    report["lod2_ratio"] = args.lod2_ratio

    export_glb(output, [target, ref_arm])

    if args.validation_json:
        Path(args.validation_json).write_text(json.dumps(report, indent=2), encoding="utf-8")

    print(json.dumps(report))
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
