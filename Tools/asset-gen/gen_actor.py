"""
gen_actor.py — Headless Blender script for procedural CC0 actor generation.

Produces rigged, animated .glb files into clients/bevy-ref/assets/models/.
All geometry is procedurally generated — no external meshes, textures, or
licensed assets. Output is CC0 1.0 (public domain).

Usage (headless):
    blender -b -P tools/asset-gen/gen_actor.py -- --variant humanoid
    blender -b -P tools/asset-gen/gen_actor.py -- --variant herd
    blender -b -P tools/asset-gen/gen_actor.py -- --variant humanoid --height 1.8 --limb-scale 1.1

Optional flags:
    --variant     humanoid | herd          (default: humanoid)
    --height      float  body height in m   (default: 1.75 / 1.0 for herd)
    --limb-scale  float  limb length scale  (default: 1.0)
    --out-dir     path   output directory   (default: clients/bevy-ref/assets/models)
"""

import sys
import os
import math
import argparse

# ---------------------------------------------------------------------------
# Parse args after the `--` separator Blender uses for script arguments
# ---------------------------------------------------------------------------
argv = sys.argv
if "--" in argv:
    script_args = argv[argv.index("--") + 1:]
else:
    script_args = []

parser = argparse.ArgumentParser(description="Procedural CC0 actor generator")
parser.add_argument("--variant", choices=["humanoid", "herd"], default="humanoid")
parser.add_argument("--height", type=float, default=None,
                    help="Total height in metres (default: 1.75 for humanoid, 1.0 for herd)")
parser.add_argument("--limb-scale", type=float, default=1.0,
                    help="Limb length multiplier (default: 1.0)")
parser.add_argument("--out-dir", type=str, default=None,
                    help="Output directory (default: clients/bevy-ref/assets/models relative to repo root)")
args = parser.parse_args(script_args)

# ---------------------------------------------------------------------------
# Resolve output directory relative to the repo root (two levels above tools/)
# ---------------------------------------------------------------------------
_script_dir = os.path.dirname(os.path.abspath(__file__))
_repo_root   = os.path.abspath(os.path.join(_script_dir, "..", ".."))

if args.out_dir:
    OUT_DIR = os.path.abspath(args.out_dir)
else:
    OUT_DIR = os.path.join(_repo_root, "clients", "bevy-ref", "assets", "models")

os.makedirs(OUT_DIR, exist_ok=True)

# ---------------------------------------------------------------------------
# Now safe to import bpy (must happen after arg parsing so --help works)
# ---------------------------------------------------------------------------
import bpy  # type: ignore
import mathutils  # type: ignore

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)


def new_material(name: str, r: float, g: float, b: float) -> bpy.types.Material:
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = (r, g, b, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.8
    bsdf.inputs["Metallic"].default_value = 0.0
    return mat


def add_box(name: str, loc, dims, parent=None, mat=None) -> bpy.types.Object:
    """Create a UV-unwrapped box mesh at world location `loc` with given (x,y,z) dims."""
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0, 0, 0))
    obj = bpy.context.active_object
    obj.name = name
    obj.scale = (dims[0], dims[1], dims[2])
    obj.location = (loc[0], loc[1], loc[2])
    bpy.ops.object.transform_apply(scale=True, location=False)
    if mat:
        obj.data.materials.append(mat)
    if parent:
        obj.parent = parent
    return obj


def set_pose_bone_rotation(armature_obj, bone_name: str, euler_xyz):
    """Set a pose bone rotation in XYZ Euler mode."""
    pb = armature_obj.pose.bones.get(bone_name)
    if pb is None:
        return
    pb.rotation_mode = "XYZ"
    pb.rotation_euler = mathutils.Euler(euler_xyz, "XYZ")


def insert_bone_keyframe(armature_obj, bone_name: str, frame: int):
    pb = armature_obj.pose.bones.get(bone_name)
    if pb is None:
        return
    pb.keyframe_insert(data_path="rotation_euler", frame=frame)
    pb.keyframe_insert(data_path="location", frame=frame)


# ---------------------------------------------------------------------------
# HUMANOID BUILDER
# ---------------------------------------------------------------------------

def build_humanoid(height: float = 1.75, limb_scale: float = 1.0):
    """Procedurally build a low-poly box humanoid with armature + animations."""
    clear_scene()

    # Proportions derived from height
    h = height
    torso_h  = h * 0.30
    head_h   = h * 0.15
    head_w   = h * 0.12
    arm_h    = h * 0.28 * limb_scale
    arm_w    = h * 0.07
    leg_h    = h * 0.36 * limb_scale
    leg_w    = h * 0.09
    torso_w  = h * 0.22
    torso_d  = h * 0.14

    skin_mat  = new_material("Skin",  0.87, 0.70, 0.55)
    cloth_mat = new_material("Cloth", 0.25, 0.40, 0.65)

    # ---- Meshes ----
    torso = add_box("torso", (0, 0, leg_h + torso_h * 0.5), (torso_w, torso_d, torso_h), mat=cloth_mat)
    head  = add_box("head",  (0, 0, leg_h + torso_h + head_h * 0.5), (head_w, head_w, head_h), mat=skin_mat)

    # Arms (shoulder offset = torso_w/2 + arm_w/2)
    arm_offset_x = torso_w * 0.5 + arm_w * 0.5
    arm_y = leg_h + torso_h - arm_h * 0.5
    l_arm = add_box("arm_L", (-arm_offset_x, 0, arm_y), (arm_w, arm_w, arm_h), mat=skin_mat)
    r_arm = add_box("arm_R", ( arm_offset_x, 0, arm_y), (arm_w, arm_w, arm_h), mat=skin_mat)

    # Legs
    leg_offset_x = torso_w * 0.30
    l_leg = add_box("leg_L", (-leg_offset_x, 0, leg_h * 0.5), (leg_w, leg_w, leg_h), mat=cloth_mat)
    r_leg = add_box("leg_R", ( leg_offset_x, 0, leg_h * 0.5), (leg_w, leg_w, leg_h), mat=cloth_mat)

    # ---- Armature ----
    bpy.ops.object.armature_add(enter_editmode=True, location=(0, 0, 0))
    arm_obj = bpy.context.active_object
    arm_obj.name = "Armature"
    arm_data = arm_obj.data
    arm_data.name = "Armature"

    # Remove default bone
    for b in arm_data.edit_bones:
        arm_data.edit_bones.remove(b)

    def eb(name, head_pos, tail_pos, parent_name=None):
        bone = arm_data.edit_bones.new(name)
        bone.head = mathutils.Vector(head_pos)
        bone.tail = mathutils.Vector(tail_pos)
        if parent_name:
            bone.parent = arm_data.edit_bones[parent_name]
        return bone

    # Bone layout (head=joint, tail=distal)
    eb("root",   (0, 0, 0),                        (0, 0, leg_h * 0.1))
    eb("pelvis", (0, 0, leg_h * 0.1),              (0, 0, leg_h + torso_h * 0.5), "root")
    eb("spine",  (0, 0, leg_h + torso_h * 0.5),    (0, 0, leg_h + torso_h),       "pelvis")
    eb("neck",   (0, 0, leg_h + torso_h),           (0, 0, leg_h + torso_h + head_h), "spine")
    eb("leg.L",  (-leg_offset_x, 0, leg_h),        (-leg_offset_x, 0, 0),         "pelvis")
    eb("leg.R",  ( leg_offset_x, 0, leg_h),        ( leg_offset_x, 0, 0),         "pelvis")
    eb("arm.L",  (-arm_offset_x, 0, leg_h + torso_h), (-arm_offset_x, 0, arm_y - arm_h * 0.5), "spine")
    eb("arm.R",  ( arm_offset_x, 0, leg_h + torso_h), ( arm_offset_x, 0, arm_y - arm_h * 0.5), "spine")

    bpy.ops.object.mode_set(mode="OBJECT")

    # ---- Skin meshes to armature ----
    for mesh_obj in [torso, head, l_arm, r_arm, l_leg, r_leg]:
        mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    bpy.ops.object.select_all(action="DESELECT")

    # ---- Animations ----
    bpy.context.scene.render.fps = 24

    # --- IDLE action (subtle sway, 48 frames / 2 s) ---
    idle_action = bpy.data.actions.new("idle")
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = idle_action

    for frame in [1, 13, 25, 37, 49]:
        bpy.context.scene.frame_set(frame)
        sway = 0.03 if frame in [1, 25, 49] else -0.03
        set_pose_bone_rotation(arm_obj, "spine", (0, sway, 0))
        set_pose_bone_rotation(arm_obj, "neck",  (0, -sway * 0.5, 0))
        insert_bone_keyframe(arm_obj, "spine", frame)
        insert_bone_keyframe(arm_obj, "neck", frame)

    # --- WALK action (arm/leg swing, 24 frames / 1 s loop) ---
    walk_action = bpy.data.actions.new("walk")
    arm_obj.animation_data.action = walk_action

    swing = 0.45  # radians
    leg_pairs = [
        (1,  "leg.L",  swing,  "leg.R", -swing,  "arm.L", -swing * 0.6, "arm.R",  swing * 0.6),
        (13, "leg.L",  0,      "leg.R",  0,       "arm.L",  0,           "arm.R",  0),
        (25, "leg.L", -swing,  "leg.R",  swing,   "arm.L",  swing * 0.6, "arm.R", -swing * 0.6),
    ]
    for (frame, ll, la, rl, ra, al, aa, ar, ab) in leg_pairs:
        bpy.context.scene.frame_set(frame)
        set_pose_bone_rotation(arm_obj, ll, (la, 0, 0))
        set_pose_bone_rotation(arm_obj, rl, (ra, 0, 0))
        set_pose_bone_rotation(arm_obj, al, (aa, 0, 0))
        set_pose_bone_rotation(arm_obj, ar, (ab, 0, 0))
        for bn in [ll, rl, al, ar]:
            insert_bone_keyframe(arm_obj, bn, frame)

    # Mirror frame 1 at frame 25+12=37 for smooth loop
    bpy.context.scene.frame_set(37)
    set_pose_bone_rotation(arm_obj, "leg.L",  ( swing, 0, 0))
    set_pose_bone_rotation(arm_obj, "leg.R",  (-swing, 0, 0))
    set_pose_bone_rotation(arm_obj, "arm.L",  (-swing * 0.6, 0, 0))
    set_pose_bone_rotation(arm_obj, "arm.R",  ( swing * 0.6, 0, 0))
    for bn in ["leg.L", "leg.R", "arm.L", "arm.R"]:
        insert_bone_keyframe(arm_obj, bn, 37)

    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end   = 48

    return arm_obj


# ---------------------------------------------------------------------------
# QUADRUPED (HERD) BUILDER
# ---------------------------------------------------------------------------

def build_herd(height: float = 1.0, limb_scale: float = 1.0):
    """Procedurally build a low-poly box quadruped (horse-like) with armature + animations."""
    clear_scene()

    h = height
    body_l  = h * 1.60
    body_h  = h * 0.50
    body_w  = h * 0.35
    neck_h  = h * 0.50
    neck_w  = h * 0.18
    head_l  = h * 0.35
    head_h  = h * 0.22
    leg_h   = h * 0.55 * limb_scale
    leg_w   = h * 0.10

    body_bottom_z = leg_h
    body_center_z = leg_h + body_h * 0.5

    coat_mat  = new_material("Coat",  0.55, 0.35, 0.18)
    mane_mat  = new_material("Mane",  0.25, 0.15, 0.08)

    # Body
    body = add_box("body", (0, 0, body_center_z), (body_w, body_l, body_h), mat=coat_mat)

    # Neck (front, along Y+)
    neck_y   = body_l * 0.40
    neck_z   = body_center_z + body_h * 0.3 + neck_h * 0.3
    neck = add_box("neck", (0, neck_y, neck_z), (neck_w, neck_w, neck_h * 0.8), mat=coat_mat)

    # Head
    head_z = neck_z + neck_h * 0.55
    head_y = neck_y + head_l * 0.3
    head = add_box("head", (0, head_y, head_z), (head_h, head_l, head_h), mat=coat_mat)

    # Legs — 4 corners
    leg_positions = [
        ("leg_FL", ( leg_w * 1.2,  body_l * 0.35, leg_h * 0.5)),
        ("leg_FR", (-leg_w * 1.2,  body_l * 0.35, leg_h * 0.5)),
        ("leg_RL", ( leg_w * 1.2, -body_l * 0.35, leg_h * 0.5)),
        ("leg_RR", (-leg_w * 1.2, -body_l * 0.35, leg_h * 0.5)),
    ]
    leg_objs = {}
    for (name, pos) in leg_positions:
        leg_objs[name] = add_box(name, pos, (leg_w, leg_w, leg_h), mat=coat_mat)

    # ---- Armature ----
    bpy.ops.object.armature_add(enter_editmode=True, location=(0, 0, 0))
    arm_obj = bpy.context.active_object
    arm_obj.name = "Armature"
    arm_obj.data.name = "Armature"

    for b in arm_obj.data.edit_bones:
        arm_obj.data.edit_bones.remove(b)

    def eb(name, head_pos, tail_pos, parent_name=None):
        bone = arm_obj.data.edit_bones.new(name)
        bone.head = mathutils.Vector(head_pos)
        bone.tail = mathutils.Vector(tail_pos)
        if parent_name:
            bone.parent = arm_obj.data.edit_bones[parent_name]
        return bone

    eb("root",   (0, 0, 0),                      (0, 0, body_bottom_z * 0.1))
    eb("spine",  (0, 0, body_center_z),           (0, body_l * 0.1, body_center_z + body_h * 0.3), "root")
    eb("neck_b", (0, neck_y * 0.5, body_center_z + body_h * 0.3),
                 (0, neck_y, neck_z), "spine")
    eb("head_b", (0, neck_y, neck_z), (0, head_y + head_l * 0.3, head_z), "neck_b")

    for (name, pos) in leg_positions:
        bone_name = f"bone_{name}"
        top = (pos[0], pos[1], body_bottom_z)
        bot = (pos[0], pos[1], 0)
        eb(bone_name, top, bot, "root")

    bpy.ops.object.mode_set(mode="OBJECT")

    # Skin
    for mesh_obj in [body, neck, head] + list(leg_objs.values()):
        mesh_obj.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.parent_set(type="ARMATURE_AUTO")
    bpy.ops.object.select_all(action="DESELECT")

    # ---- Animations ----
    bpy.context.scene.render.fps = 24

    # --- IDLE (head nod, 48 frames) ---
    idle_action = bpy.data.actions.new("idle")
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = idle_action

    for (frame, angle) in [(1, 0.0), (12, 0.08), (24, 0.0), (36, -0.04), (48, 0.0)]:
        bpy.context.scene.frame_set(frame)
        set_pose_bone_rotation(arm_obj, "neck_b", (angle, 0, 0))
        set_pose_bone_rotation(arm_obj, "head_b", (-angle * 0.5, 0, 0))
        insert_bone_keyframe(arm_obj, "neck_b", frame)
        insert_bone_keyframe(arm_obj, "head_b", frame)

    # --- WALK (leg sequence, 24 frames, diagonal pairs) ---
    walk_action = bpy.data.actions.new("walk")
    arm_obj.animation_data.action = walk_action

    sw = 0.4
    # Diagonal gait: FL+RR swing together, FR+RL opposite
    gait = [
        (1,  "bone_leg_FL",  sw, "bone_leg_RR",  sw, "bone_leg_FR", -sw, "bone_leg_RL", -sw),
        (13, "bone_leg_FL",   0, "bone_leg_RR",   0, "bone_leg_FR",   0, "bone_leg_RL",   0),
        (25, "bone_leg_FL", -sw, "bone_leg_RR", -sw, "bone_leg_FR",  sw, "bone_leg_RL",  sw),
    ]
    for row in gait:
        frame = row[0]
        pairs = [(row[i], row[i+1]) for i in range(1, len(row), 2)]
        bpy.context.scene.frame_set(frame)
        for (bname, angle) in pairs:
            set_pose_bone_rotation(arm_obj, bname, (angle, 0, 0))
            insert_bone_keyframe(arm_obj, bname, frame)

    # Loop closure at 37
    bpy.context.scene.frame_set(37)
    for (bname, angle) in [("bone_leg_FL", sw), ("bone_leg_RR", sw),
                            ("bone_leg_FR", -sw), ("bone_leg_RL", -sw)]:
        set_pose_bone_rotation(arm_obj, bname, (angle, 0, 0))
        insert_bone_keyframe(arm_obj, bname, 37)

    bpy.context.scene.frame_start = 1
    bpy.context.scene.frame_end   = 48

    return arm_obj


# ---------------------------------------------------------------------------
# EXPORT
# ---------------------------------------------------------------------------

def export_glb(out_path: str):
    # Blender 4.x glTF exporter — 'export_colors' was removed; use vertex-color
    # attribute export via export_attributes instead.
    bpy.ops.export_scene.gltf(
        filepath=out_path,
        export_format="GLB",
        export_animations=True,
        export_skins=True,
        export_morph=False,
        export_materials="EXPORT",
        use_active_scene=True,
    )
    size = os.path.getsize(out_path) if os.path.exists(out_path) else 0
    print(f"[gen_actor] Exported {out_path} ({size} bytes)")


# ---------------------------------------------------------------------------
# MAIN
# ---------------------------------------------------------------------------

variant = args.variant

if variant == "humanoid":
    height    = args.height if args.height is not None else 1.75
    build_humanoid(height=height, limb_scale=args.limb_scale)
    out_path = os.path.join(OUT_DIR, "humanoid_gen.glb")
else:
    height    = args.height if args.height is not None else 1.0
    build_herd(height=height, limb_scale=args.limb_scale)
    out_path = os.path.join(OUT_DIR, "herd_gen.glb")

export_glb(out_path)
print(f"[gen_actor] Done — variant={variant} height={height} limb_scale={args.limb_scale}")
