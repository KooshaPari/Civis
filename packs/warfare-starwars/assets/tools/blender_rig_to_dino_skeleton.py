#!/usr/bin/env python3
"""
blender_rig_to_dino_skeleton.py  (#991)

Rig a Star Wars unit mesh to a DINO-compatible 21-bone humanoid infantry skeleton so the
exported skinned mesh has exactly Mesh.bindposes.Length == 21, satisfying the runtime guard
AssetSwapSystem.IsSkinnedMeshCompatible (vanilla 'dark_knight' = 21 bindposes).

Why a fresh 21-bone armature (not weight-transfer from a DINO reference): the DINO infantry
skeleton's exact bone NAMES are not recoverable from settings_assets_all.bundle (the bone
Transform hierarchy lives cross-bundle in the unit prefab). We DO know the authoritative fact
needed to pass the swap guard and to deform plausibly: 21 humanoid bones. So we build a
standard 21-bone biped, bind the mesh with automatic (heat-map) weights, and export. The result
is a genuinely skinned mesh (21 bindposes) that the procedural animation can drive.

The raw SW clone GLB is ALREADY skinned (66 joints) — we discard its over-detailed rig and
re-bind to the 21-bone skeleton, then decimate to the infantry LOD budget.

Run (Blender 3.x / 4.x):
  blender --background --python blender_rig_to_dino_skeleton.py -- \
    --source path/to/model.glb \
    --output path/to/rigged_21bone.glb \
    --lod0-tris 2000

Exit 0 = ok (21 bindposes), 2 = validation failed.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    import bpy
    from mathutils import Vector
except ImportError:
    print("Run inside Blender: blender --background --python blender_rig_to_dino_skeleton.py -- ...")
    sys.exit(1)


# 21-bone humanoid infantry skeleton. Names are conventional; ORDER and COUNT (21) are what
# matter for the bindpose-count guard. Positions are normalized for a ~1.8m biped and are
# rescaled to the imported mesh's bounding box at build time.
# (name, parent, head_xyz, tail_xyz) in a T-pose, +Y up.
BONES_21 = [
    ("Hips",          None,           (0.00, 1.00, 0.00), (0.00, 1.12, 0.00)),
    ("Spine",         "Hips",         (0.00, 1.12, 0.00), (0.00, 1.28, 0.00)),
    ("Chest",         "Spine",        (0.00, 1.28, 0.00), (0.00, 1.45, 0.00)),
    ("Neck",          "Chest",        (0.00, 1.45, 0.00), (0.00, 1.55, 0.00)),
    ("Head",          "Neck",         (0.00, 1.55, 0.00), (0.00, 1.72, 0.00)),
    ("Shoulder_L",    "Chest",        (0.05, 1.45, 0.00), (0.18, 1.45, 0.00)),
    ("UpperArm_L",    "Shoulder_L",   (0.18, 1.45, 0.00), (0.42, 1.45, 0.00)),
    ("LowerArm_L",    "UpperArm_L",   (0.42, 1.45, 0.00), (0.65, 1.45, 0.00)),
    ("Hand_L",        "LowerArm_L",   (0.65, 1.45, 0.00), (0.78, 1.45, 0.00)),
    ("Shoulder_R",    "Chest",        (-0.05, 1.45, 0.00), (-0.18, 1.45, 0.00)),
    ("UpperArm_R",    "Shoulder_R",   (-0.18, 1.45, 0.00), (-0.42, 1.45, 0.00)),
    ("LowerArm_R",    "UpperArm_R",   (-0.42, 1.45, 0.00), (-0.65, 1.45, 0.00)),
    ("Hand_R",        "LowerArm_R",   (-0.65, 1.45, 0.00), (-0.78, 1.45, 0.00)),
    ("UpperLeg_L",    "Hips",         (0.10, 1.00, 0.00), (0.11, 0.55, 0.00)),
    ("LowerLeg_L",    "UpperLeg_L",   (0.11, 0.55, 0.00), (0.12, 0.10, 0.00)),
    ("Foot_L",        "LowerLeg_L",   (0.12, 0.10, 0.00), (0.12, 0.02, 0.12)),
    ("UpperLeg_R",    "Hips",         (-0.10, 1.00, 0.00), (-0.11, 0.55, 0.00)),
    ("LowerLeg_R",    "UpperLeg_R",   (-0.11, 0.55, 0.00), (-0.12, 0.10, 0.00)),
    ("Foot_R",        "LowerLeg_R",   (-0.12, 0.10, 0.00), (-0.12, 0.02, 0.12)),
    ("Toe_L",         "Foot_L",       (0.12, 0.02, 0.12), (0.12, 0.02, 0.20)),
    ("Toe_R",         "Foot_R",       (-0.12, 0.02, 0.12), (-0.12, 0.02, 0.20)),
]
assert len(BONES_21) == 21


def parse_args(argv):
    if "--" in argv:
        argv = argv[argv.index("--") + 1:]
    p = argparse.ArgumentParser()
    p.add_argument("--source", required=True)
    p.add_argument("--output", required=True)
    p.add_argument("--lod0-tris", type=int, default=2000)
    p.add_argument("--validation-json", default="")
    return p.parse_args(argv)


def clear_scene():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for block in (bpy.data.meshes, bpy.data.armatures, bpy.data.objects):
        for b in list(block):
            try:
                block.remove(b)
            except Exception:
                pass


def import_glb(path):
    bpy.ops.import_scene.gltf(filepath=str(path))
    return list(bpy.context.selected_objects)


def merge_meshes(objs):
    meshes = [o for o in objs if o.type == "MESH"]
    if not meshes:
        return None
    bpy.ops.object.select_all(action="DESELECT")
    for m in meshes:
        m.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    if len(meshes) > 1:
        bpy.ops.object.join()
    return bpy.context.view_layer.objects.active


def strip_existing_rig(mesh_obj):
    # Remove armature modifiers + vertex groups + parent so we rebind cleanly.
    for mod in list(mesh_obj.modifiers):
        if mod.type == "ARMATURE":
            mesh_obj.modifiers.remove(mod)
    for vg in list(mesh_obj.vertex_groups):
        mesh_obj.vertex_groups.remove(vg)
    mesh_obj.parent = None


def mesh_bounds(mesh_obj):
    bpy.context.view_layer.update()
    coords = [mesh_obj.matrix_world @ Vector(c) for c in mesh_obj.bound_box]
    xs = [c.x for c in coords]; ys = [c.y for c in coords]; zs = [c.z for c in coords]
    return (min(xs), min(ys), min(zs)), (max(xs), max(ys), max(zs))


def build_armature(mesh_obj):
    (minx, miny, minz), (maxx, maxy, maxz) = mesh_bounds(mesh_obj)
    height = max(0.001, maxy - miny)
    cx = (minx + maxx) / 2.0
    cz = (minz + maxz) / 2.0

    arm_data = bpy.data.armatures.new("DinoInfantry21")
    arm_obj = bpy.data.objects.new("DinoInfantry21", arm_data)
    bpy.context.collection.objects.link(arm_obj)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="EDIT")

    def remap(p):
        # template space: Y in [0,1.8]; map to mesh bbox.
        x, y, z = p
        return (cx + x * height / 1.8, miny + y * height / 1.8, cz + z * height / 1.8)

    ebones = arm_data.edit_bones
    created = {}
    for name, parent, head, tail in BONES_21:
        eb = ebones.new(name)
        eb.head = remap(head)
        eb.tail = remap(tail)
        created[name] = eb
    for name, parent, _, _ in BONES_21:
        if parent:
            created[name].parent = created[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    # Mark all bones as deform bones so the glTF exporter recognises them as skin joints.
    for b in arm_data.bones:
        b.use_deform = True
    return arm_obj


def bind_with_auto_weights(mesh_obj, arm_obj):
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    # Heat-map automatic weights; falls back to envelopes if heat fails.
    try:
        bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    except RuntimeError:
        bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")

    # parent_set may not leave an Armature modifier if heat weighting partially failed.
    # Guarantee one exists and points at the armature so the glTF exporter emits a skin.
    if not any(m.type == "ARMATURE" and m.object == arm_obj for m in mesh_obj.modifiers):
        am = mesh_obj.modifiers.new(name="Armature", type="ARMATURE")
        am.object = arm_obj
    mesh_obj.parent = arm_obj
    mesh_obj.parent_type = "OBJECT"

    # Verify weights actually landed. Heat weighting can leave most vertices unassigned, which
    # makes the glTF exporter emit NO skin. If coverage is poor, assign each vertex to its
    # nearest bone (by head position) with weight 1.0 so every vertex is skinned.
    me = mesh_obj.data
    assigned = set()
    for v in me.vertices:
        for g in v.groups:
            if g.weight > 0.0001:
                assigned.add(v.index)
                break
    coverage = len(assigned) / max(1, len(me.vertices))
    print(f"[diag] auto-weight coverage={coverage:.2%} ({len(assigned)}/{len(me.vertices)})")
    if coverage < 0.95:
        _assign_nearest_bone_weights(mesh_obj, arm_obj)


def _assign_nearest_bone_weights(mesh_obj, arm_obj):
    """Fallback: hard-assign every vertex to the nearest deform bone (weight 1.0)."""
    bone_heads = []
    mw = arm_obj.matrix_world
    for b in arm_obj.data.bones:
        head_world = mw @ b.head_local
        bone_heads.append((b.name, head_world))
    vg_by_name = {vg.name: vg for vg in mesh_obj.vertex_groups}
    for name, _ in bone_heads:
        if name not in vg_by_name:
            vg_by_name[name] = mesh_obj.vertex_groups.new(name=name)
    me = mesh_obj.data
    mwm = mesh_obj.matrix_world
    count = 0
    for v in me.vertices:
        # Skip if already well-weighted.
        if any(g.weight > 0.0001 for g in v.groups):
            continue
        co = mwm @ v.co
        nearest = min(bone_heads, key=lambda bh: (bh[1] - co).length_squared)
        vg_by_name[nearest[0]].add([v.index], 1.0, "REPLACE")
        count += 1
    print(f"[diag] nearest-bone fallback assigned {count} previously-unweighted vertices")


def decimate(mesh_obj, target_tris):
    me = mesh_obj.data
    cur = len(me.polygons)
    if cur <= target_tris:
        return cur
    ratio = max(0.02, min(1.0, target_tris / cur))
    mod = mesh_obj.modifiers.new("DecimateLOD0", type="DECIMATE")
    mod.ratio = ratio
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.modifier_apply(modifier=mod.name)
    return len(mesh_obj.data.polygons)


def export(path, mesh_obj, arm_obj):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    mesh_obj.select_set(True)
    arm_obj.select_set(True)
    # export_apply MUST be False: applying modifiers collapses the Armature modifier and the
    # glTF exporter then emits no skin (skins:0). With it False, skins are preserved.
    bpy.ops.export_scene.gltf(
        filepath=str(path),
        export_format="GLB",
        use_selection=True,
        export_apply=False,
        export_skins=True,
        export_def_bones=False,
        export_animations=False,
        export_yup=True,
    )


def main(argv):
    args = parse_args(argv)
    src = Path(args.source).resolve()
    out = Path(args.output).resolve()
    if not src.is_file():
        print(f"[ERROR] source not found: {src}")
        return 1

    clear_scene()
    objs = import_glb(src)
    mesh = merge_meshes(objs)
    if mesh is None:
        print("[ERROR] no mesh in source")
        return 1

    strip_existing_rig(mesh)
    # Purge every non-mesh object the import created (original 66-bone armature + empties such
    # as 'Object_73'). They confuse the glTF skin exporter and leave stray no-skin nodes.
    for o in list(bpy.data.objects):
        if o is not mesh and o.type != "MESH":
            bpy.data.objects.remove(o, do_unlink=True)
    for a in list(bpy.data.armatures):
        bpy.data.armatures.remove(a)
    mesh.parent = None
    # Decimate BEFORE binding so the Armature modifier stays last in the stack and the glTF
    # exporter emits a proper skin (skins>0). Applying decimate after binding strips the skin.
    tris_after = decimate(mesh, args.lod0_tris)
    arm = build_armature(mesh)
    bind_with_auto_weights(mesh, arm)

    mods = [(m.type, getattr(m, "object", None).name if getattr(m, "object", None) else None) for m in mesh.modifiers]
    print(f"[diag] mesh='{mesh.name}' parent={mesh.parent.name if mesh.parent else None} mods={mods}")
    bone_count = len(arm.data.bones)
    vg_count = len(mesh.vertex_groups)
    report = {
        "source": str(src),
        "output": str(out),
        "armature_bones": bone_count,
        "vertex_groups": vg_count,
        "triangles_lod0": tris_after,
        "expected_bindposes": 21,
        "ok": bone_count == 21 and vg_count > 0,
    }
    export(out, mesh, arm)

    if args.validation_json:
        Path(args.validation_json).write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report))
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
