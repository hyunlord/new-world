"""Contact sheets for visual review (not part of the asset build).

blender -b --factory-startup -P tools/3d/blender/render_contact.py -- [out_dir] [id_filter...]

Writes <out_dir>/blender_models_<YYYYmmdd_HHMM>.png (every exported .glb, front-3/4 view, labelled)
and <out_dir>/blender_poses_<YYYYmmdd_HHMM>.png (each character re-posed FROM THE VAT .bin at the
middle frame of every clip, coloured by the COLOR_0 tint mask, or by the atlas through UV0 for textured
characters) so the baked data is checked too.
"""
import datetime
import json
import math
import os
import sys

import bpy
import numpy as np
from mathutils import Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

CELL = 300
VIEW_DIR = Vector((0.85, -1.0, 0.7)).normalized()  # front (-Y) right, from above


def setup_scene():
    C.reset_scene()
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.render.resolution_x = sc.render.resolution_y = CELL
    sc.render.resolution_percentage = 100
    sc.render.image_settings.file_format = "PNG"
    sh = sc.display.shading
    sh.light = "STUDIO"
    sh.color_type = "MATERIAL"
    sh.show_shadows = True
    sh.show_cavity = True
    sh.cavity_type = "WORLD"
    if sc.world is None:
        sc.world = bpy.data.worlds.new("w")
    sc.world.color = (0.80, 0.84, 0.88)
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    cam.data.type = "ORTHO"
    sc.collection.objects.link(cam)
    sc.camera = cam
    txt = bpy.data.curves.new("label", "FONT")
    tob = bpy.data.objects.new("label", txt)
    sc.collection.objects.link(tob)
    tob.parent = cam
    tmat = bpy.data.materials.new("label_mat")
    tmat.diffuse_color = (0.05, 0.05, 0.08, 1)
    txt.materials.append(tmat)
    ground = bpy.data.meshes.new("ground")
    ground.from_pydata([(-20, -20, 0), (20, -20, 0), (20, 20, 0), (-20, 20, 0)], [], [(0, 1, 2, 3)])
    gmat = bpy.data.materials.new("ground_mat")
    gmat.diffuse_color = (0.62, 0.66, 0.55, 1)
    ground.materials.append(gmat)
    gob = bpy.data.objects.new("ground", ground)
    sc.collection.objects.link(gob)
    return cam, tob


def fit_camera(cam, tob, objs, label):
    pts = []
    dg = bpy.context.evaluated_depsgraph_get()
    for o in objs:
        if o.type != "MESH" or not o.visible_get():  # skip importer bone-shape helpers
            continue
        ev = o.evaluated_get(dg)
        me = ev.to_mesh()
        pts += [ev.matrix_world @ v.co for v in me.vertices]
        ev.to_mesh_clear()
    lo = Vector((min(p.x for p in pts), min(p.y for p in pts), min(p.z for p in pts)))
    hi = Vector((max(p.x for p in pts), max(p.y for p in pts), max(p.z for p in pts)))
    centre = (lo + hi) / 2
    cam.location = centre + VIEW_DIR * 30
    cam.rotation_euler = (-VIEW_DIR).to_track_quat("-Z", "Y").to_euler()
    bpy.context.view_layer.update()
    inv = cam.matrix_world.inverted()
    loc = [inv @ p for p in pts]
    ext = max(max(p.x for p in loc) - min(p.x for p in loc), max(p.y for p in loc) - min(p.y for p in loc))
    scale = max(ext * 1.3, 0.6)
    cam.data.ortho_scale = scale
    cx = (max(p.x for p in loc) + min(p.x for p in loc)) / 2
    cy = (max(p.y for p in loc) + min(p.y for p in loc)) / 2
    cam.data.shift_x = cx / scale
    cam.data.shift_y = cy / scale - 0.04
    cam.data.clip_end = 100
    tob.data.body = label
    tob.data.size = scale * 0.075
    # camera-local view spans [cx +- scale/2] x [cy - 0.04*scale +- scale/2]
    tob.location = (cx - scale * 0.47, cy - scale * 0.04 - scale * 0.46, -2)


def render_to_array(path):
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(path)
    arr = np.empty(img.size[0] * img.size[1] * 4, dtype=np.float32)
    img.pixels.foreach_get(arr)
    arr = arr.reshape(img.size[1], img.size[0], 4)
    bpy.data.images.remove(img)
    os.remove(path)
    return arr


def save_grid(cells, cols, path):
    rows = (len(cells) + cols - 1) // cols
    sheet = np.ones((rows * CELL, cols * CELL, 4), dtype=np.float32)
    for i, a in enumerate(cells):
        r = rows - 1 - i // cols  # Blender images are bottom-up
        c = i % cols
        sheet[r * CELL:(r + 1) * CELL, c * CELL:(c + 1) * CELL] = a
    img = bpy.data.images.new("sheet", cols * CELL, rows * CELL, alpha=True)
    img.pixels.foreach_set(sheet.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    print("WROTE", path)


def clear_imported(keep):
    for o in list(bpy.data.objects):
        if o.name not in keep:
            bpy.data.objects.remove(o)
    for coll in (bpy.data.meshes, bpy.data.armatures, bpy.data.actions):
        for item in list(coll):
            if item.users == 0:
                coll.remove(item)


def import_glb(path):
    before = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=path)
    objs = [o for o in bpy.data.objects if o not in before]
    for o in objs:  # show skinned models in their bind pose
        if o.type == "ARMATURE":
            o.data.pose_position = "REST"
            if o.animation_data:
                o.animation_data.action = None
    for o in objs:
        if o.type == "MESH":
            for slot in o.material_slots:
                m = slot.material
                if m and m.node_tree:
                    b = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
                    if b:
                        m.diffuse_color = b.inputs["Base Color"].default_value
    return objs


def main():
    args = C.script_args()
    out_dir = args[0] if args else "/Users/rexxa/work/3d-data/screens"
    filt = args[1:]
    os.makedirs(out_dir, exist_ok=True)
    stamp = datetime.datetime.now().strftime("%Y%m%d_%H%M")
    with open(C.MANIFEST) as f:
        manifest = json.load(f)
    cam, tob = setup_scene()
    keep = {"cam", "label", "ground"}
    tmp = os.path.join(out_dir, "_cell.png")

    cells = []
    for mid, e in manifest["models"].items():
        if filt and not any(s in mid for s in filt):
            continue
        objs = import_glb(os.path.join(C.REPO, "assets", "3d", e["file"]))
        fit_camera(cam, tob, objs, f"{mid} {e['triangles']}t")
        cells.append(render_to_array(tmp))
        clear_imported(keep)
    if cells:
        save_grid(cells, 8, os.path.join(out_dir, f"blender_models_{stamp}.png"))

    # pose sheet straight from VAT data
    bpy.context.scene.display.shading.color_type = "VERTEX"
    cells = []
    for cid in [k for k, e in manifest["models"].items() if e.get("family") == "character"]:
        if filt and not any(s in cid for s in filt):
            continue
        meta = json.load(open(os.path.join(C.VAT_DIR, f"{cid}_vat.json")))
        data = np.fromfile(os.path.join(C.VAT_DIR, f"{cid}_vat.bin"), dtype="<f2").astype(np.float32)
        data = data.reshape(-1, meta["width"], 4)
        objs = import_glb(os.path.join(C.VAT_DIR, f"{cid}_static.glb"))
        ob = next(o for o in objs if o.type == "MESH")
        me = ob.data
        # textured (set B): show the atlas through UV0 instead of the tint mask
        bpy.context.scene.display.shading.color_type = "TEXTURE" if meta.get("textured") else "VERTEX"
        if meta.get("textured"):
            tex = ob.active_material.node_tree.nodes.new("ShaderNodeTexImage")
            tex.image = bpy.data.images.load(os.path.join(C.VAT_DIR, meta["atlas"]))
            ob.active_material.node_tree.nodes.active = tex
            me.uv_layers[0].active = me.uv_layers[0].active_render = True
        uv2 = me.uv_layers[1].data
        idx = np.zeros(len(me.vertices), dtype=np.int64)
        for li, loop in enumerate(me.loops):
            idx[loop.vertex_index] = int(uv2[li].uv[0] * meta["width"])
        for clip, c in meta["clips"].items():
            row = data[c["start"] + c["count"] // 2]
            for i, v in enumerate(me.vertices):
                x, y, z, _ = row[idx[i]]
                v.co = (x, -z, y)
            me.update()
            fit_camera(cam, tob, [ob], f"{cid}:{clip}")
            cells.append(render_to_array(tmp))
        clear_imported(keep)
    if cells:
        save_grid(cells, 6, os.path.join(out_dir, f"blender_poses_{stamp}.png"))


main()
