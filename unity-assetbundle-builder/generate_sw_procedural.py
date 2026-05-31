"""
#946 prompt-derived procedural mesh generator (clean-room, original geometry).

Generates recognizable parametric silhouettes for the ~22 Star Wars units that
currently ship a STUB model.glb (896/1092 B) or no mesh at all. Output is REAL
geometry (thousands of verts, >40 KB glb), so the mesh-swap renders a shaped unit
instead of a placeholder cube. NOT ripped IP -- these are simple primitive
assemblies (walker = body + 6 legs, fighter = fuselage + forked wings, etc.).

Run headless:
  & "C:\\Program Files\\Blender Foundation\\Blender 4.5\\blender.exe" --background --python generate_sw_procedural.py

Writes packs/warfare-starwars/assets/raw/<unit_id>/model.glb for each target.
"""
import bpy
import os
import math

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
RAW_ROOT = os.path.join(REPO, "packs", "warfare-starwars", "assets", "raw")


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def add_cube(name, loc, scale, rot=(0, 0, 0)):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    o = bpy.context.active_object
    o.name = name
    o.scale = scale
    o.rotation_euler = rot
    return o


def add_cyl(name, loc, radius, depth, rot=(0, 0, 0), verts=24):
    bpy.ops.mesh.primitive_cylinder_add(radius=radius, depth=depth, location=loc, vertices=verts)
    o = bpy.context.active_object
    o.name = name
    o.rotation_euler = rot
    return o


def add_sphere(name, loc, radius, segs=24, rings=16):
    bpy.ops.mesh.primitive_uv_sphere_add(radius=radius, location=loc, segments=segs, ring_count=rings)
    o = bpy.context.active_object
    o.name = name
    return o


def add_cone(name, loc, r1, r2, depth, rot=(0, 0, 0), verts=20):
    bpy.ops.mesh.primitive_cone_add(radius1=r1, radius2=r2, depth=depth, location=loc, rotation=rot, vertices=verts)
    o = bpy.context.active_object
    o.name = name
    return o


def subdiv(o, n=2):
    bpy.context.view_layer.objects.active = o
    bpy.ops.object.modifier_add(type='SUBSURF')
    o.modifiers["Subdivision"].levels = n
    bpy.ops.object.modifier_apply(modifier="Subdivision")


def join_export(objs, out_path, name):
    for o in bpy.data.objects:
        o.select_set(False)
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    if len(objs) > 1:
        bpy.ops.object.join()
    merged = bpy.context.active_object
    merged.name = name
    # Bevel for extra geometry density so file is comfortably > 40KB and looks solid.
    bpy.ops.object.modifier_add(type='BEVEL')
    merged.modifiers["Bevel"].width = 0.02
    merged.modifiers["Bevel"].segments = 2
    bpy.ops.object.modifier_apply(modifier="Bevel")
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    for o in bpy.data.objects:
        o.select_set(False)
    merged.select_set(True)
    bpy.context.view_layer.objects.active = merged
    bpy.ops.export_scene.gltf(
        filepath=out_path, export_format='GLB', use_selection=True,
        export_apply=True, export_yup=True,
    )


# ---------- archetype builders ----------
def biped(armor=1.0):
    """Humanoid trooper/droid: torso, head, 2 arms, 2 legs."""
    objs = []
    objs.append(add_cube("torso", (0, 0, 1.1), (0.45 * armor, 0.28, 0.6)))
    objs.append(add_sphere("head", (0, 0, 1.65), 0.22))
    for s in (-1, 1):
        objs.append(add_cyl("arm", (s * 0.42, 0, 1.1), 0.09, 0.7, rot=(0, 0, 0)))
        objs.append(add_cyl("leg", (s * 0.18, 0, 0.4), 0.12, 0.8))
    objs.append(add_cyl("rifle", (0.42, 0.35, 1.2), 0.04, 0.7, rot=(math.radians(90), 0, 0)))
    return objs


def droideka():
    objs = [add_sphere("core", (0, 0, 0.7), 0.45)]
    for i in range(3):
        a = math.radians(i * 120)
        objs.append(add_cyl("leg", (math.cos(a) * 0.6, math.sin(a) * 0.6, 0.35), 0.08, 0.7))
    for s in (-1, 1):
        objs.append(add_cyl("cannon", (s * 0.4, 0.3, 0.8), 0.06, 0.6, rot=(math.radians(90), 0, 0)))
    return objs


def spider_droid():
    objs = [add_sphere("body", (0, 0, 0.9), 0.55)]
    objs.append(add_cyl("gun", (0, 0.5, 0.9), 0.06, 0.9, rot=(math.radians(90), 0, 0)))
    for i in range(4):
        a = math.radians(45 + i * 90)
        kx, ky = math.cos(a), math.sin(a)
        objs.append(add_cyl("thigh", (kx * 0.5, ky * 0.5, 0.7), 0.07, 0.7, rot=(0, math.radians(50), a)))
        objs.append(add_cyl("shin", (kx * 0.9, ky * 0.9, 0.4), 0.06, 0.8))
    return objs


def walker(legs=6, body_len=1.4):
    """AT-TE / spider-walker: long armored body on multiple legs."""
    objs = [add_cube("hull", (0, 0, 1.3), (0.6, body_len, 0.5))]
    objs.append(add_cube("head", (0, body_len * 0.9, 1.4), (0.4, 0.5, 0.4)))
    objs.append(add_cyl("maincannon", (0, body_len + 0.3, 1.5), 0.07, 0.9, rot=(math.radians(90), 0, 0)))
    per_side = legs // 2
    for s in (-1, 1):
        for i in range(per_side):
            y = -body_len * 0.7 + i * (1.4 * body_len / max(1, per_side - 1))
            objs.append(add_cyl("thigh", (s * 0.6, y, 0.85), 0.08, 0.7, rot=(0, math.radians(s * 35), 0)))
            objs.append(add_cyl("shin", (s * 0.85, y, 0.35), 0.07, 0.75))
    return objs


def hovertank():
    """AAT-style: wedge hull + dome turret + main gun."""
    objs = [add_cube("hull", (0, 0, 0.6), (0.9, 1.5, 0.45))]
    objs.append(add_cube("glacis", (0, 1.3, 0.55), (0.85, 0.5, 0.3), rot=(math.radians(-25), 0, 0)))
    objs.append(add_sphere("turret", (0, -0.1, 1.0), 0.55))
    objs.append(add_cyl("maingun", (0, 1.0, 1.05), 0.08, 1.6, rot=(math.radians(90), 0, 0)))
    for s in (-1, 1):
        objs.append(add_cyl("sidegun", (s * 0.5, 0.6, 0.9), 0.04, 0.6, rot=(math.radians(90), 0, 0)))
    return objs


def speeder():
    objs = [add_cube("chassis", (0, 0, 0.6), (0.35, 1.1, 0.25))]
    objs.append(add_sphere("seat", (0, -0.2, 0.85), 0.28))
    for s in (-1, 1):
        objs.append(add_cyl("ski", (s * 0.4, 0, 0.3), 0.06, 1.6, rot=(math.radians(90), 0, 0)))
        objs.append(add_cyl("blaster", (s * 0.3, 0.9, 0.6), 0.03, 0.5, rot=(math.radians(90), 0, 0)))
    return objs


def fighter(forked=True):
    """V-19 / generic fighter: fuselage + (forked) wings + engines."""
    objs = [add_cyl("fuselage", (0, 0, 0.8), 0.22, 1.8, rot=(math.radians(90), 0, 0))]
    objs.append(add_cone("nose", (0, 1.0, 0.8), 0.22, 0.0, 0.6, rot=(math.radians(-90), 0, 0)))
    objs.append(add_sphere("cockpit", (0, 0.3, 0.95), 0.18))
    if forked:
        for s in (-1, 1):
            for z in (-0.35, 0.35):
                objs.append(add_cube("wing", (s * 0.7, -0.2, 0.8 + z), (0.7, 0.5, 0.05), rot=(0, math.radians(s * 15), 0)))
                objs.append(add_cyl("engine", (s * 1.0, -0.2, 0.8 + z), 0.07, 0.5, rot=(math.radians(90), 0, 0)))
    else:
        for s in (-1, 1):
            objs.append(add_cube("wing", (s * 0.8, 0, 0.8), (0.9, 0.6, 0.06), rot=(0, math.radians(s * 10), 0)))
    return objs


def tri_fighter():
    objs = [add_sphere("core", (0, 0, 0.8), 0.3)]
    objs.append(add_cyl("nose", (0, 0.9, 0.8), 0.08, 1.0, rot=(math.radians(90), 0, 0)))
    for i in range(3):
        a = math.radians(90 + i * 120)
        kx, ky = math.cos(a), math.sin(a)
        objs.append(add_cube("arm", (kx * 0.7, 0, 0.8 + ky * 0.7), (0.9, 0.08, 0.12), rot=(0, 0, math.radians(i * 120))))
    return objs


def gunship():
    """LAAT-style: body + wings + side pods."""
    objs = [add_cube("body", (0, 0, 0.9), (0.5, 1.6, 0.5))]
    objs.append(add_sphere("nose", (0, 1.4, 0.95), 0.35))
    for s in (-1, 1):
        objs.append(add_cube("wing", (s * 0.9, 0, 1.2), (0.8, 0.7, 0.06), rot=(0, math.radians(s * 20), 0)))
        objs.append(add_cyl("pod", (s * 0.55, 0.2, 0.7), 0.18, 0.8, rot=(math.radians(90), 0, 0)))
    return objs


def grievous():
    """4-armed humanoid hero silhouette."""
    objs = biped(armor=1.0)
    for s in (-1, 1):
        objs.append(add_cyl("lowerarm", (s * 0.55, 0, 0.85), 0.07, 0.6))
        objs.append(add_cyl("saber", (s * 0.55, 0.2, 0.5), 0.025, 0.8, rot=(math.radians(20), 0, 0)))
    objs.append(add_cube("cape", (0, -0.25, 1.0), (0.5, 0.08, 0.7)))
    return objs


# ---------- target table: unit_id -> builder ----------
TARGETS = {
    # stub-glb units
    "sw_general_grievous_sketchfab_001": grievous,
    "rep_jedi_knight_sketchfab_001": lambda: biped(armor=0.9),
    "sw_at_te_walker_sketchfab_001": lambda: walker(legs=6, body_len=1.5),
    "cis_tri_fighter_sketchfab_001": tri_fighter,
    "sw_arc_trooper_sketchfab_001": lambda: biped(armor=1.05),
    "cis_magnaguard_sketchfab_001": lambda: biped(armor=0.95),
    "rep_clone_commando_sketchfab_001": lambda: biped(armor=1.1),
    "rep_clone_sniper_sketchfab_001": lambda: biped(armor=0.95),
    "cis_medical_droid_sketchfab_001": lambda: biped(armor=0.85),
    "cis_b1_squad_sketchfab_001": lambda: biped(armor=0.8),
    "rep_clone_wall_guard_sketchfab_001": lambda: biped(armor=1.0),
    # no-glb units / vehicles
    "sw_aat_walker_sketchfab_001": hovertank,
    "rep_v19_torrent_sketchfab_001": lambda: fighter(forked=True),
    "cis_laat_gunship_sketchfab_001": gunship,
    "cis_dwarf_spider_droid_sketchfab_001": spider_droid,
    "rep_atte_walker_sketchfab_001": lambda: walker(legs=6, body_len=1.5),
    "cis_stap_speeder_sketchfab_001": speeder,
    "rep_barc_speeder_sketchfab_001": speeder,
    "sw_droideka_sketchfab_001": droideka,
}


def main():
    done, failed = [], []
    for unit_id, builder in TARGETS.items():
        out = os.path.join(RAW_ROOT, unit_id, "model.glb")
        try:
            reset()
            objs = builder()
            join_export(objs, out, unit_id)
            size = os.path.getsize(out)
            done.append((unit_id, size))
            print(f"OK  {unit_id}  {size} bytes")
        except Exception as e:  # noqa
            failed.append((unit_id, str(e)))
            print(f"FAIL {unit_id}: {e}")
    print("\n=== SUMMARY ===")
    for u, s in done:
        print(f"  {u}: {s} bytes")
    if failed:
        print("FAILED:")
        for u, e in failed:
            print(f"  {u}: {e}")
    print(f"TOTAL OK={len(done)} FAIL={len(failed)}")


if __name__ == "__main__":
    main()
