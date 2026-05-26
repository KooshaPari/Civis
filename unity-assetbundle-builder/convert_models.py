"""
Blender batch converter: GLB/GLTF/FBX → FBX for Unity import.
Run: blender --background --python convert_models.py
"""
import bpy
import os
import sys

RAW_ROOT = r"C:\Users\koosh\Dino\packs\warfare-starwars\assets\raw"
OUT_ROOT  = r"C:\Users\koosh\Dino\unity-assetbundle-builder\Assets\Models"

SUPPORTED = (".glb", ".gltf", ".fbx", ".obj")

os.makedirs(OUT_ROOT, exist_ok=True)

results = []

for entry in sorted(os.listdir(RAW_ROOT)):
    entry_path = os.path.join(RAW_ROOT, entry)
    if not os.path.isdir(entry_path):
        continue

    # Find first supported model file
    model_file = None
    for fname in ("model.glb", "source_download.glb"):
        fp = os.path.join(entry_path, fname)
        if os.path.isfile(fp):
            model_file = fp
            break

    if model_file is None:
        # Try other formats
        for fname in os.listdir(entry_path):
            if fname.lower().endswith(SUPPORTED):
                model_file = os.path.join(entry_path, fname)
                break

    if model_file is None:
        print(f"[SKIP] No model in {entry}")
        continue

    # Clean name: strip _sketchfab_001 suffix
    asset_name = entry.replace("_sketchfab_001", "").replace("_lego", "")
    out_fbx = os.path.join(OUT_ROOT, asset_name + ".fbx")

    if os.path.isfile(out_fbx):
        print(f"[SKIP] Already converted: {asset_name}")
        results.append((asset_name, "skipped"))
        continue

    try:
        # Clear scene
        bpy.ops.wm.read_homefile(use_empty=True)

        ext = os.path.splitext(model_file)[1].lower()
        if ext in (".glb", ".gltf"):
            bpy.ops.import_scene.gltf(filepath=model_file)
        elif ext == ".fbx":
            bpy.ops.import_scene.fbx(filepath=model_file)
        elif ext == ".obj":
            bpy.ops.wm.obj_import(filepath=model_file)

        # Center and normalize scale
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.transform_apply(scale=True)

        # Export as FBX
        bpy.ops.export_scene.fbx(
            filepath=out_fbx,
            use_selection=False,
            global_scale=1.0,
            apply_unit_scale=True,
            apply_scale_options='FBX_SCALE_NONE',
            bake_space_transform=False,
            object_types={'MESH'},
            use_mesh_modifiers=True,
            mesh_smooth_type='FACE',
            use_armature_deform_only=True,
            add_leaf_bones=False,
            primary_bone_axis='Y',
            secondary_bone_axis='X',
            use_metadata=True,
            path_mode='COPY',
            embed_textures=True,
            axis_forward='-Z',
            axis_up='Y',
        )

        print(f"[OK] {asset_name} → {out_fbx}")
        results.append((asset_name, "ok"))
    except Exception as e:
        print(f"[ERROR] {asset_name}: {e}")
        results.append((asset_name, f"error: {e}"))

print("\n=== Summary ===")
ok  = sum(1 for _, s in results if s == "ok")
sk  = sum(1 for _, s in results if s == "skipped")
err = sum(1 for _, s in results if s.startswith("error"))
print(f"  Converted: {ok}  Skipped: {sk}  Errors: {err}")

sys.exit(0)
