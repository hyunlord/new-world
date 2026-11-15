"""Set B characters (accepted art, textured) -> vertex-animation textures.

    blender -b --factory-startup -P tools/3d/blender/chars_b.py [-- b_man b_goat ...]

Unlike people.py/animals.py nothing is modelled here: the skinned, atlas-textured glbs accepted on
v8/art-3d-chars (commit SOURCE_COMMIT) are copied once into assets/3d/source/chars/B/<subject>/ (see SOURCE.md)
and this script bakes them like common.export_character, with three differences:
  * the static mesh keeps the source UV0 (atlas UVs, copied per loop by the split) and is posed at idle
    frame 0 (the bind pose is an A-pose; the VAT shader replaces positions anyway);
  * extra meshes animated as objects (the humans' CarryLoad, scaled to ~0 outside `carry`; and a
    procedurally-built HandTool, scaled to ~0 outside `work`) become extra VAT columns after the body's;
  * the json adds textured/atlas/source_commit, and clip names the game asks for that the source lacks
    are aliases (same rows, "alias_of"). The atlas is byte-copied to vat/<id>_atlas.png.
The source scale is kept (man 1.7 m, woman 1.7 m, child 1.2 m as in the receipts).

For every subject a second, low-detail `<id>_lod` bake is also produced (same clips/fps/conventions,
same atlas file) for crowd rendering: the body mesh is Decimate-collapsed, inserted BEFORE the Armature
modifier so the fixed decimated topology is what gets skinned and posed per frame (skin weights carry
over: Decimate remaps the mesh's vertex-group layer when it merges vertices). See `_decimate_body`.
"""
import json
import os
import shutil
import sys

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

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
LOD_WIDTH_BUDGET = 400   # VAT columns (unsplit vertices) ceiling for the whole character
LOD_SPLIT_BUDGET = 900   # static mesh (one vertex per loop) ceiling for the whole character

TOOL_LEN = 0.55          # haft length, m
TOOL_HAFT_R = 0.016      # haft radius, m
TOOL_HAND_BONE = "hand_R"
# (u, v) texels hand-picked (by scanning the atlas for locally-uniform patches, see SOURCE.md
# commit) from each subject's own source atlas.png: a neutral wood-brown patch for the haft and
# the atlas's least-saturated patch for the head. man and child have no true grey pixel anywhere in
# their atlas (every uniform patch is a skin/hair/cloth brown); their "head" texel is simply the
# darkest, most desaturated patch found there, not literal grey -- see README's Set B section.
TOOL_UV = {
    "man":   {"wood": (0.7588, 0.6279), "head": (0.0400, 0.6553)},
    "woman": {"wood": (0.8389, 0.2510), "head": (0.6279, 0.9365)},
    "child": {"wood": (0.0596, 0.0146), "head": (0.1670, 0.3135)},
}


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


def _decimate_body(body, width_budget):
    """Collapse-decimate body's rest mesh (BEFORE the Armature modifier, so posing happens on the
    fixed decimated topology) to at most `width_budget` referenced vertices. The body is built from
    many disconnected low-poly islands (fingers, straps, ...): Decimate can fully collapse a small
    island's faces away while leaving its vertices in the mesh, so after applying the modifier any
    vertex no longer used by a face is deleted. Deterministic: no RNG, same input -> same output."""
    arm_mod = next(m for m in body.modifiers if m.type == "ARMATURE")
    idx = list(body.modifiers).index(arm_mod)
    bpy.context.view_layer.objects.active = body
    dec = body.modifiers.new("lod_decimate", "DECIMATE")
    dec.decimate_type = "COLLAPSE"
    bpy.ops.object.modifier_move_to_index(modifier=dec.name, index=idx)  # before the armature modifier

    def referenced_at(ratio):
        dec.ratio = ratio
        dg = bpy.context.evaluated_depsgraph_get()
        ev = body.evaluated_get(dg)
        me = ev.to_mesh()
        ref = {vi for p in me.polygons for vi in p.vertices}
        ev.to_mesh_clear()
        return len(ref)

    lo, hi = 1e-4, 1.0
    for _ in range(20):
        mid = (lo + hi) / 2
        if referenced_at(mid) <= width_budget:
            lo = mid
        else:
            hi = mid
    dec.ratio = lo
    bpy.ops.object.modifier_apply(modifier=dec.name)
    bm = bmesh.new()
    bm.from_mesh(body.data)
    bmesh.ops.delete(bm, geom=[v for v in bm.verts if not v.link_faces], context="VERTS")
    bm.to_mesh(body.data)
    bm.free()
    body.data.update()


def _hand_tool(cid, subject, arm):
    """Low-poly hand tool (hex haft + wedge head, 20 verts) for set B humans: built directly in the
    armature's rest space, grip centred on TOOL_HAND_BONE with the haft along that bone's rest
    head->tail axis, then rigidly skinned 100% to it (the standard rigid-part technique: see
    common.Model), so normal skin deformation carries it through every pose unchanged -- no manual
    per-frame tracking needed. UV0 is two solid texels from TOOL_UV (wood haft, grey-ish head).
    Scale is keyframed per clip (1 in `work`, 0 elsewhere), same mechanism as CarryLoad's NLA tracks,
    so it is picked up by `_clip_bindings` like any other extra and appears only during `work`."""
    bone = arm.data.bones[TOOL_HAND_BONE]
    head_pt, tail_pt = Vector(bone.head_local), Vector(bone.tail_local)
    axis = (tail_pt - head_pt).normalized()
    grip = head_pt.lerp(tail_pt, 0.5)
    rot = Vector((0, 0, 1)).rotation_difference(axis).to_matrix().to_4x4()
    base = Matrix.Translation(grip - axis * 0.15) @ rot  # local z=0.15 lands on the grip

    bm = bmesh.new()
    haft = bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=6,
                                 radius1=TOOL_HAFT_R, radius2=TOOL_HAFT_R, depth=TOOL_LEN,
                                 matrix=Matrix.Translation((0, 0, TOOL_LEN / 2)))["verts"]
    n_haft = len(haft)
    bmesh.ops.create_cube(bm, size=1.0, matrix=Matrix.Translation((0, 0, TOOL_LEN)) @
                          Matrix.Diagonal((0.05, 0.035, 0.09, 1.0)))
    bm.verts.ensure_lookup_table()
    for v in bm.verts:
        v.co = base @ v.co
    me = bpy.data.meshes.new(cid + "_HandTool")
    bm.to_mesh(me)
    bm.free()
    uv = me.uv_layers.new(name="UVMap")
    lv = np.empty(len(me.loops), dtype=np.int64)
    me.loops.foreach_get("vertex_index", lv)
    wood, head = TOOL_UV[subject]["wood"], TOOL_UV[subject]["head"]
    uv.data.foreach_set("uv", np.array([c for vi in lv for c in (wood if vi < n_haft else head)]))
    obj = bpy.data.objects.new(cid + "_HandTool", me)
    bpy.context.scene.collection.objects.link(obj)
    obj.vertex_groups.new(name=TOOL_HAND_BONE).add(range(len(me.vertices)), 1.0, "REPLACE")
    C.skin_to(obj, arm)

    ad = obj.animation_data_create()
    for t in arm.animation_data.nla_tracks:
        name = t.strips[0].name
        f0, f1 = t.strips[0].action.frame_range
        n = round(f1 - f0)
        act = bpy.data.actions.new(f"{cid}_tool_{name}")
        act.use_fake_user = True
        ad.action = act
        s = 1.0 if name == "work" else 0.0
        obj.scale = (s, s, s)
        obj.keyframe_insert("scale", frame=0)
        obj.keyframe_insert("scale", frame=n)
        track = ad.nla_tracks.new()
        track.name = name
        strip = track.strips.new(name, 0, act)
        if hasattr(strip, "action_slot") and act.slots:
            strip.action_slot = act.slots[0]
    ad.action = None
    return obj


def build_one(cid, entries, lod=False):
    subject, game_clips = SUBJECTS[cid]
    out_id = f"{cid}_lod" if lod else cid
    src = os.path.join(SRC_DIR, subject)
    glb = os.path.join(src, f"{subject}.glb")
    C.reset_scene()  # 30 fps: glTF time t lands on frame 30 t
    bpy.ops.import_scene.gltf(filepath=glb)
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    body = next(o for o in bpy.data.objects if o.type == "MESH" and any(m.type == "ARMATURE" for m in o.modifiers))
    extras = sorted((o for o in bpy.data.objects if o.type == "MESH" and o is not body and o.animation_data),
                    key=lambda o: o.name)
    if game_clips is HUMAN:
        extras.append(_hand_tool(out_id, subject, arm))  # appended after existing extras (e.g. CarryLoad)
    if lod:
        # extras (e.g. the humans' CarryLoad) are kept at full resolution: they're already tiny (24
        # verts) and decimating them further risks breaking their silhouette for no real VAT savings.
        extras_verts = sum(len(o.data.vertices) for o in extras)
        _decimate_body(body, LOD_WIDTH_BUDGET - extras_verts)
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
    split, n_split = _static_mesh(out_id, meshes, _positions(meshes, ranges["idle"]))
    C.export_glb(os.path.join(C.VAT_DIR, f"{out_id}_static.glb"), [split], vertex_colors=True)

    pos = np.stack(rows)
    assert np.isfinite(pos).all(), f"{out_id}: non-finite VAT positions"
    pos.astype("<f2").tofile(os.path.join(C.VAT_DIR, f"{out_id}_vat.bin"))
    atlas = f"{cid}_atlas.png"  # shared with the full bake; the LOD json points at it, never its own copy
    if not lod:
        shutil.copyfile(os.path.join(src, "atlas.png"), os.path.join(C.VAT_DIR, atlas))
    height = C.height_of(body)  # bind pose, source scale
    if subject == "man" and not lod:
        assert abs(height - 1.7) < 0.01, f"b_man must be 1.7 m tall, is {height:.3f}"
    if lod:
        assert width <= LOD_WIDTH_BUDGET, f"{out_id}: width {width} > {LOD_WIDTH_BUDGET}"
        assert n_split <= LOD_SPLIT_BUDGET, f"{out_id}: split {n_split} > {LOD_SPLIT_BUDGET}"
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
    with open(os.path.join(C.VAT_DIR, f"{out_id}_vat.json"), "w") as f:
        json.dump(meta, f, indent=1)
        f.write("\n")
    entries[out_id] = {
        "file": meta["source"], "footprint": [1, 1], "height": round(height, 3),
        "triangles": sum(C.tri_count(o) for o in meshes), "materials": [], "sockets": {},
        "family": "character", "animations": list(meta_clips), "bones": [b.name for b in arm.data.bones],
        "vat": {"static": f"vat/{out_id}_static.glb", "bin": f"vat/{out_id}_vat.bin",
                "json": f"vat/{out_id}_vat.json", "atlas": f"vat/{atlas}", "vertex_count": n_split},
        "textured": True, "source_commit": SOURCE_COMMIT, "subject": subject,
    }
    print(f"[chars_b] {out_id}: width={width} split={n_split} frames={len(rows)} "
          f"clips={ {k: (v['count'], v['loop']) for k, v in meta_clips.items()} } height={height:.3f}")


def build(entries, only=None):
    for cid in SUBJECTS:
        if not only or cid in only:
            build_one(cid, entries)
            build_one(cid, entries, lod=True)


if __name__ == "__main__":
    try:
        out = {}
        build(out, C.script_args())
        C.update_manifest(out)
    except Exception:
        import traceback
        traceback.print_exc()
        sys.exit(1)
