"""Set B characters (accepted art, textured) -> vertex-animation textures.

    blender -b --factory-startup -P tools/3d/blender/chars_b.py [-- b_man b_goat ...]

Unlike people.py/animals.py nothing is modelled here: the skinned, atlas-textured glbs accepted on
v8/art-3d-chars (commit SOURCE_COMMIT) are copied once into assets/3d/source/chars/B/<subject>/ (see SOURCE.md)
and this script bakes them like common.export_character, with three differences:
  * the static mesh keeps the source UV0 (atlas UVs, copied per loop by the split) and is posed at idle
    frame 0 (the bind pose is an A-pose; the VAT shader replaces positions anyway);
  * extra meshes animated as objects (the humans' CarryLoad, scaled to ~0 outside `carry`) become extra
    VAT columns after the body's;
  * the json adds textured/atlas/source_commit, and clip names the game asks for that the source lacks
    are aliases (same rows, "alias_of"). The atlas is byte-copied to vat/<id>_atlas.png.
The source scale is kept (man 1.7 m, woman 1.7 m, child 1.2 m as in the receipts).
"""
import json
import os
import shutil
import sys

import bpy
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

SOURCE_COMMIT = "f0b91f999"
SRC_DIR = os.path.join(C.REPO, "assets", "3d", "source", "chars", "B")
HUMAN = ["idle", "walk", "work", "carry", "sleep"]
ANIMAL = ["idle", "walk", "run", "graze", "rest", "attack"]
ALIASES = {"run": "walk", "rest": "lie_down", "attack": "idle"}  # game name -> source clip, used if missing
SUBJECTS = {"b_man": ("man", HUMAN), "b_woman": ("woman", HUMAN), "b_child": ("child", HUMAN),
            "b_goat": ("goat", ANIMAL), "b_aurochs": ("aurochs", ANIMAL)}
LOOP_TOL = 0.01  # m: a clip loops when its last pose matches its first


def _clip_bindings(objs):
    """{clip: [(obj, action, slot)]} from the NLA strips the glTF importer creates (one slot per object)."""
    out = {}
    for o in objs:
        for t in o.animation_data.nla_tracks:
            for s in t.strips:
                out.setdefault(s.name, []).append((o, s.action, s.action_slot))
    return out


def _play(binding):
    for o, act, slot in binding:
        ad = o.animation_data
        for t in ad.nla_tracks:
            t.mute = True
        ad.action = act
        ad.action_slot = slot


def _positions(meshes, frame):
    """World positions (Blender coords) of every vertex of `meshes`, concatenated in order."""
    scene = bpy.context.scene
    scene.frame_set(int(frame), subframe=frame - int(frame))
    dg = bpy.context.evaluated_depsgraph_get()
    parts = []
    for o in meshes:
        ev = o.evaluated_get(dg)
        me = ev.to_mesh()
        co = np.empty(len(me.vertices) * 3, dtype=np.float64)
        me.vertices.foreach_get("co", co)
        ev.to_mesh_clear()
        m = np.array(ev.matrix_world, dtype=np.float64)
        co = co.reshape(-1, 3)
        parts.append(co @ m[:3, :3].T + m[:3, 3])
    return np.concatenate(parts)


def _static_mesh(cid, meshes, pos):
    """Split copy of `meshes` posed at `pos`: one vertex per loop (flat), UV0 = source UV per loop,
    UV1 = ((unsplit index + 0.5) / width, 0.5), black COLOR_0 (no tint region), one material."""
    width = len(pos)
    verts, faces, uv0, orig = [], [], [], []
    base = 0
    for o in meshes:
        me = o.data
        lv = np.empty(len(me.loops), dtype=np.int64)
        me.loops.foreach_get("vertex_index", lv)
        uv = np.empty(len(me.loops) * 2, dtype=np.float64)
        me.uv_layers[0].data.foreach_get("uv", uv)
        for poly in me.polygons:
            ls = range(poly.loop_start, poly.loop_start + poly.loop_total)
            faces.append(list(range(len(verts), len(verts) + poly.loop_total)))
            for li in ls:
                orig.append(base + lv[li])
                verts.append(tuple(pos[base + lv[li]]))
                uv0 += [uv[2 * li], uv[2 * li + 1]]
        base += len(me.vertices)
    me = bpy.data.meshes.new(cid + "_vat")
    me.from_pydata(verts, [], faces)
    n = len(me.vertices)
    assert len(me.loops) == n, "split mesh must have one vertex per loop"
    lv = np.empty(n, dtype=np.int64)
    me.loops.foreach_get("vertex_index", lv)
    orig = np.array(orig)[lv]  # loop -> its (split) vertex -> unsplit column
    me.uv_layers.new(name="UVMap").data.foreach_set("uv", np.array(uv0))
    me.uv_layers.new(name="UV2").data.foreach_set("uv", np.stack([(orig + 0.5) / width, np.full(n, 0.5)], 1).ravel())
    attr = me.color_attributes.new("Color", "FLOAT_COLOR", "POINT")
    attr.data.foreach_set("color", np.tile([0.0, 0.0, 0.0, 1.0], n))
    me.color_attributes.active_color = attr
    me.color_attributes.render_color_index = me.color_attributes.find("Color")
    me.materials.append(C.plain_material("vat_" + cid))
    obj = bpy.data.objects.new(cid + "_vat", me)
    bpy.context.scene.collection.objects.link(obj)
    return obj, n


def build_one(cid, entries):
    subject, game_clips = SUBJECTS[cid]
    src = os.path.join(SRC_DIR, subject)
    glb = os.path.join(src, f"{subject}.glb")
    C.reset_scene()  # 30 fps: glTF time t lands on frame 30 t
    bpy.ops.import_scene.gltf(filepath=glb)
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    body = next(o for o in bpy.data.objects if o.type == "MESH" and any(m.type == "ARMATURE" for m in o.modifiers))
    extras = sorted((o for o in bpy.data.objects if o.type == "MESH" and o is not body and o.animation_data),
                    key=lambda o: o.name)
    meshes = [body] + extras
    clips = _clip_bindings([arm] + extras)
    width = sum(len(o.data.vertices) for o in meshes)

    rows, meta_clips, ranges = [], {}, {}
    for name in sorted(clips, key=lambda c: (c not in game_clips, game_clips.index(c) if c in game_clips else 0, c)):
        binding = clips[name]
        _play(binding)
        f0, f1 = binding[0][1].frame_range
        dur = (f1 - f0) / C.FPS
        loop = bool(np.abs(_positions(meshes, f0) - _positions(meshes, f1)).max() < LOOP_TOL)
        count = round(dur * C.VAT_FPS) + (0 if loop else 1)
        meta_clips[name] = {"start": len(rows), "count": count, "loop": loop, "duration": round(dur, 4)}
        ranges[name] = f0
        for k in range(count):
            p = _positions(meshes, f0 + k * C.FPS / C.VAT_FPS)
            rows.append(np.stack([p[:, 0], p[:, 2], -p[:, 1], np.ones(width)], 1))
    for name in game_clips:
        if name not in meta_clips:
            meta_clips[name] = dict(meta_clips[ALIASES[name]], alias_of=ALIASES[name])

    # static mesh = idle frame 0 (frame 0 of the VAT's idle row), so the rest mesh and the bake agree
    _play(clips["idle"])
    split, n_split = _static_mesh(cid, meshes, _positions(meshes, ranges["idle"]))
    C.export_glb(os.path.join(C.VAT_DIR, f"{cid}_static.glb"), [split], vertex_colors=True)

    pos = np.stack(rows)
    assert np.isfinite(pos).all(), f"{cid}: non-finite VAT positions"
    pos.astype("<f2").tofile(os.path.join(C.VAT_DIR, f"{cid}_vat.bin"))
    atlas = f"{cid}_atlas.png"
    shutil.copyfile(os.path.join(src, "atlas.png"), os.path.join(C.VAT_DIR, atlas))
    height = C.height_of(body)  # bind pose, source scale
    if subject == "man":
        assert abs(height - 1.7) < 0.01, f"b_man must be 1.7 m tall, is {height:.3f}"
    meta = {
        "width": width, "vertex_count": width, "split_vertex_count": n_split, "frames": len(rows),
        "fps": C.VAT_FPS, "normals_offset": -1, "clips": meta_clips,
        "bounds_min": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).min(0)],
        "bounds_max": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).max(0)],
        "height": round(height, 4),
        "format": "float16 LE RGBA, row-major; one row of positions (Godot xyz, w=1) per frame; texel x = UV2.x * "
                  "width (the unsplit vertex); no normal rows: the shader derives flat face normals; UV0 = atlas UVs",
        "textured": True, "atlas": atlas, "source_commit": SOURCE_COMMIT,
        "source": f"source/chars/B/{subject}/{subject}.glb",
    }
    with open(os.path.join(C.VAT_DIR, f"{cid}_vat.json"), "w") as f:
        json.dump(meta, f, indent=1)
        f.write("\n")
    entries[cid] = {
        "file": meta["source"], "footprint": [1, 1], "height": round(height, 3),
        "triangles": sum(C.tri_count(o) for o in meshes), "materials": [], "sockets": {},
        "family": "character", "animations": list(meta_clips), "bones": [b.name for b in arm.data.bones],
        "vat": {"static": f"vat/{cid}_static.glb", "bin": f"vat/{cid}_vat.bin", "json": f"vat/{cid}_vat.json",
                "atlas": f"vat/{atlas}", "vertex_count": n_split},
        "textured": True, "source_commit": SOURCE_COMMIT, "subject": subject,
    }
    print(f"[chars_b] {cid}: width={width} split={n_split} frames={len(rows)} "
          f"clips={ {k: (v['count'], v['loop']) for k, v in meta_clips.items()} } height={height:.3f}")


def build(entries, only=None):
    for cid in SUBJECTS:
        if not only or cid in only:
            build_one(cid, entries)


if __name__ == "__main__":
    try:
        out = {}
        build(out, C.script_args())
        C.update_manifest(out)
    except Exception:
        import traceback
        traceback.print_exc()
        sys.exit(1)
