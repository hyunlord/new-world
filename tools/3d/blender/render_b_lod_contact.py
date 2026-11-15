"""Full vs LOD contact sheet for set B characters (review only, not part of the asset build).

    blender -b --factory-startup -P tools/3d/blender/render_b_lod_contact.py -- [out_dir]

For each of b_man/b_woman/b_child/b_goat/b_aurochs, re-poses the full and the `_lod` static mesh
from their VAT .bin at the middle of the `walk` clip, textured through the (shared) atlas, and
writes <out_dir>/blender_b_lod_<YYYYmmdd_HHMM>.png with one row per character: [full, lod].
"""
import datetime
import json
import os
import sys

import bpy
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import render_contact as RC  # noqa: E402

SUBJECTS = ["b_man", "b_woman", "b_child", "b_goat", "b_aurochs"]


def render_walk_mid(cid, cam, tob, tmp):
    meta = json.load(open(os.path.join(C.VAT_DIR, f"{cid}_vat.json")))
    data = np.fromfile(os.path.join(C.VAT_DIR, f"{cid}_vat.bin"), dtype="<f2").astype(np.float32)
    data = data.reshape(-1, meta["width"], 4)
    objs = RC.import_glb(os.path.join(C.VAT_DIR, f"{cid}_static.glb"))
    ob = next(o for o in objs if o.type == "MESH")
    me = ob.data
    tex = ob.active_material.node_tree.nodes.new("ShaderNodeTexImage")
    tex.image = bpy.data.images.load(os.path.join(C.VAT_DIR, meta["atlas"]))
    ob.active_material.node_tree.nodes.active = tex
    me.uv_layers[0].active = me.uv_layers[0].active_render = True
    uv2 = me.uv_layers[1].data
    idx = np.zeros(len(me.vertices), dtype=np.int64)
    for li, loop in enumerate(me.loops):
        idx[loop.vertex_index] = int(uv2[li].uv[0] * meta["width"])
    c = meta["clips"]["walk"]
    row = data[c["start"] + c["count"] // 2]
    for i, v in enumerate(me.vertices):
        x, y, z, _ = row[idx[i]]
        v.co = (x, -z, y)
    me.update()
    RC.fit_camera(cam, tob, [ob], f"{cid} {meta['split_vertex_count']}v/{meta['width']}col")
    cell = RC.render_to_array(tmp)
    RC.clear_imported({"cam", "label", "ground"})
    return cell


def main():
    args = C.script_args()
    out_dir = args[0] if args else "/Users/rexxa/work/3d-data/screens"
    os.makedirs(out_dir, exist_ok=True)
    stamp = datetime.datetime.now().strftime("%Y%m%d_%H%M")
    cam, tob = RC.setup_scene()
    bpy.context.scene.display.shading.color_type = "TEXTURE"
    tmp = os.path.join(out_dir, "_b_lod_cell.png")
    cells = []
    for cid in SUBJECTS:
        cells.append(render_walk_mid(cid, cam, tob, tmp))
        cells.append(render_walk_mid(cid + "_lod", cam, tob, tmp))
    RC.save_grid(cells, 2, os.path.join(out_dir, f"blender_b_lod_{stamp}.png"))


if __name__ == "__main__":
    main()
