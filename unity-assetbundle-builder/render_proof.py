"""Render-proof: import each generated model.glb, render a thumbnail PNG.
Run: blender --background --python render_proof.py
Outputs to docs/screenshots/model-previews/sw-procedural/<unit>.png
"""
import bpy, os, math

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
RAW = os.path.join(REPO, "packs", "warfare-starwars", "assets", "raw")
OUT = os.path.join(REPO, "docs", "screenshots", "model-previews", "sw-procedural")
os.makedirs(OUT, exist_ok=True)

UNITS = [
    "sw_general_grievous_sketchfab_001", "sw_at_te_walker_sketchfab_001",
    "cis_tri_fighter_sketchfab_001", "sw_aat_walker_sketchfab_001",
    "rep_v19_torrent_sketchfab_001", "cis_dwarf_spider_droid_sketchfab_001",
    "cis_laat_gunship_sketchfab_001", "sw_droideka_sketchfab_001",
    "rep_jedi_knight_sketchfab_001", "cis_stap_speeder_sketchfab_001",
]

for u in UNITS:
    glb = os.path.join(RAW, u, "model.glb")
    if not os.path.exists(glb):
        print(f"SKIP {u} (no glb)"); continue
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=glb)
    # camera
    cam_data = bpy.data.cameras.new("cam"); cam = bpy.data.objects.new("cam", cam_data)
    bpy.context.scene.collection.objects.link(cam)
    cam.location = (4, -4, 3); cam.rotation_euler = (math.radians(65), 0, math.radians(45))
    bpy.context.scene.camera = cam
    # light
    ld = bpy.data.lights.new("sun", 'SUN'); ld.energy = 4
    lo = bpy.data.objects.new("sun", ld); bpy.context.scene.collection.objects.link(lo)
    lo.rotation_euler = (math.radians(45), math.radians(20), 0)
    sc = bpy.context.scene
    sc.render.engine = 'BLENDER_EEVEE_NEXT'
    sc.render.resolution_x = 256; sc.render.resolution_y = 256
    sc.render.film_transparent = True
    sc.render.filepath = os.path.join(OUT, u + ".png")
    bpy.ops.render.render(write_still=True)
    print(f"RENDERED {u}")
print("DONE")
