"""
Blender batch converter: real GLB meshes -> FBX for Unity import.
Only converts GLBs above a vertex/size threshold (skips 896/1092B stub placeholders).
Run: blender --background --python convert_real_models.py
"""
import bpy
import os
import sys
import struct
import json

# Worktree-local paths (this file lives in unity-assetbundle-builder/)
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
RAW_ROOT = os.path.join(REPO, "packs", "warfare-starwars", "assets", "raw")
OUT_ROOT = os.path.join(HERE, "Assets", "Models")

MIN_VERTS = 100  # stub cubes have 8-24 verts; real meshes have 855+

os.makedirs(OUT_ROOT, exist_ok=True)


def glb_vertex_count(path):
    try:
        with open(path, "rb") as f:
            data = f.read()
        if data[:4] != b"glTF":
            return -1
        jlen = struct.unpack("<I", data[12:16])[0]
        j = json.loads(data[20:20 + jlen])
        verts = 0
        for m in j.get("meshes", []):
            for pr in m.get("primitives", []):
                pi = pr.get("attributes", {}).get("POSITION")
                if pi is not None:
                    verts += j["accessors"][pi].get("count", 0)
        return verts
    except Exception:
        return -1


results = []
for entry in sorted(os.listdir(RAW_ROOT)):
    entry_path = os.path.join(RAW_ROOT, entry)
    if not os.path.isdir(entry_path):
        continue
    glb = os.path.join(entry_path, "model.glb")
    if not os.path.isfile(glb):
        continue
    verts = glb_vertex_count(glb)
    if verts < MIN_VERTS:
        print(f"[SKIP-STUB] {entry} verts={verts}")
        results.append((entry, "stub"))
        continue

    asset_name = entry.replace("_sketchfab_001", "").replace("_lego", "")
    out_fbx = os.path.join(OUT_ROOT, asset_name + ".fbx")

    try:
        bpy.ops.wm.read_homefile(use_empty=True)
        bpy.ops.import_scene.gltf(filepath=glb)
        bpy.ops.object.select_all(action='SELECT')
        bpy.ops.object.transform_apply(scale=True)
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
            add_leaf_bones=False,
            use_metadata=True,
            path_mode='COPY',
            embed_textures=True,
            axis_forward='-Z',
            axis_up='Y',
        )
        size = os.path.getsize(out_fbx) if os.path.isfile(out_fbx) else 0
        print(f"[OK] {asset_name} verts={verts} -> {out_fbx} ({size}B)")
        results.append((asset_name, "ok"))
    except Exception as e:
        print(f"[ERROR] {asset_name}: {e}")
        results.append((asset_name, f"error: {e}"))

ok = sum(1 for _, s in results if s == "ok")
stub = sum(1 for _, s in results if s == "stub")
err = sum(1 for _, s in results if s.startswith("error"))
print(f"\n=== Summary === Converted: {ok}  Stubs-skipped: {stub}  Errors: {err}")
sys.exit(0)
