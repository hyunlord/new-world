"""M3D-ASSETS 'living' skinned subjects (assets/3d/living/) -> vertex-animation textures, baked the
same way as chars_b.py (and sharing its helpers) for 7 animals and 18 human wardrobe variants.

    blender -b --factory-startup -P tools/3d/blender/chars_c.py [-- c_wolf p_adult_F_bronze ...]
    blender -b --factory-startup -P tools/3d/blender/chars_c.py -- contact [out_dir]

Differences from chars_b.py's set B source:
  * animals (`c_<species>`, assets/3d/living/<dir>/<dir>.glb) have a single skinned body mesh and no
    extras (no held tool/carry bundle): same game clip aliases as b_goat (run->walk, rest->lie_down,
    attack->idle). The body mesh's Blender object is named after the source directory, which is the
    species id except wild_goat (source directory `goat`).
  * people (`p_<stem>`, assets/3d/living/people/person_<stem>.glb) reuse the shared man/woman/child
    body (by age/sex) but each age/era variant adds its own wardrobe meshes: separate skinned mesh
    objects (same armature) with SOLID-colour materials that have no texture of their own, unlike the
    body and the atlas-textured CarryLoad bundle. Since the baked VAT keeps one shared material/atlas,
    a wardrobe mesh's UV0 is overwritten with one texel sampled from the body's own atlas.png, picked
    automatically: the atlas pixel closest to the wardrobe material's base colour, inside a locally
    uniform patch so a single point sample never straddles a UV-island seam (`_solid_texel`). A
    HandTool (chars_b._hand_tool) is added exactly as for b_man/b_woman/b_child, with its own
    wood/head texels found the same way (nearest-to-brown for the haft, least-saturated patch for the
    head) instead of chars_b.TOOL_UV (hand-picked from a different, unrelated atlas image).
  * manifest rows never touch assets/3d/models/manifest.json (owned by other model data this pass):
    they are written to the sibling fragment `vat_c_manifest.json`, same pattern as stations_w3d.py.
"""
import json
import os
import shutil
import sys

import bpy
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import chars_b as B  # noqa: E402

SRC_DIR = os.path.join(C.REPO, "assets", "3d", "living")
PEOPLE_DIR = os.path.join(SRC_DIR, "people")
FRAGMENT = os.path.join(C.MODELS_DIR, "vat_c_manifest.json")

# cid -> source directory/file stem (== the species id except wild_goat, whose source kept its old
# `goat` name from before the wild_* rename).
ANIMALS = {
    "c_aurochs": "aurochs", "c_deer": "deer", "c_wild_goat": "goat", "c_reindeer": "reindeer",
    "c_wild_boar": "wild_boar", "c_wild_sheep": "wild_sheep", "c_wolf": "wolf",
}
PEOPLE_STEMS = sorted(f[len("person_"):-len(".glb")] for f in os.listdir(PEOPLE_DIR)
                      if f.startswith("person_") and f.endswith(".glb"))

TARGET_WOOD = (0.55, 0.35, 0.15)  # brown, same intent as chars_b's hand-picked Set B wood texels


def _subject_of(stem):
    """person_<stem>.glb's body subject (man/woman/child), by its age_sex_era stem."""
    age = stem.split("_", 1)[0]
    return "child" if age == "child" else ("man" if "_M_" in stem else "woman")


def _atlas_pixels(path):
    img = bpy.data.images.load(path)
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return px.reshape(h, w, 4).astype(np.float64)


def _solid_texel(px, target=None, patch=3):
    """UV of a pixel in atlas `px` (h, w, 4) close to `target` colour, or (if None) the least
    saturated pixel in the whole atlas. Matching is done on a `patch` x `patch` box-blur of the atlas
    (not the raw pixels), so a single noisy/AA pixel can't win the nearest-colour search by chance and
    the picked UV sits in a patch that is actually close to the target, not just one lucky texel."""
    h, w, _ = px.shape
    rgb = px[:, :, :3]
    pad = patch // 2
    padded = np.pad(rgb, ((pad, pad), (pad, pad), (0, 0)), mode="edge")
    smooth = sum(padded[dy:dy + h, dx:dx + w] for dy in range(patch) for dx in range(patch)) / (patch * patch)
    score = (smooth.max(-1) - smooth.min(-1)) if target is None else ((smooth - np.array(target)) ** 2).sum(-1)
    j, i = np.unravel_index(int(np.argmin(score)), score.shape)
    return (i + 0.5) / w, (j + 0.5) / h


def _force_uv(me, uv):
    me.uv_layers[0].data.foreach_set("uv", list(uv) * len(me.loops))


def _material_rgb(obj):
    bsdf = obj.data.materials[0].node_tree.nodes.get("Principled BSDF")
    return tuple(bsdf.inputs["Base Color"].default_value)[:3]


def build_one(cid, glb, subject, atlas_src, game_clips, source, entries, lod=False):
    out_id = f"{cid}_lod" if lod else cid
    C.reset_scene()
    bpy.ops.import_scene.gltf(filepath=glb)
    arm = next(o for o in bpy.data.objects if o.type == "ARMATURE")
    skinned = sorted((o for o in bpy.data.objects if o.type == "MESH"
                      and any(m.type == "ARMATURE" for m in o.modifiers)), key=lambda o: o.name)
    body = next(o for o in skinned if o.name == subject)
    attachments = [o for o in skinned if o is not body]  # era wardrobe: full res, never decimated
    extras = sorted((o for o in bpy.data.objects if o.type == "MESH" and o not in skinned
                     and o.animation_data), key=lambda o: o.name)  # e.g. CarryLoad
    is_human = game_clips is B.HUMAN

    px = _atlas_pixels(atlas_src) if (attachments or is_human) else None
    for att in attachments:
        _force_uv(att.data, _solid_texel(px, _material_rgb(att)))
    if is_human:
        wood, head = _solid_texel(px, TARGET_WOOD), _solid_texel(px, None)
        extras.append(B._hand_tool(out_id, subject, arm, wood, head))

    if lod:
        extras_verts = sum(len(o.data.vertices) for o in extras + attachments)
        B._decimate_body(body, B.LOD_WIDTH_BUDGET - extras_verts)
    meshes = [body] + attachments + extras
    clips = B._clip_bindings([arm] + extras)
    width = sum(len(o.data.vertices) for o in meshes)

    rows, meta_clips, ranges = [], {}, {}
    for name in sorted(clips, key=lambda c: (c not in game_clips, game_clips.index(c) if c in game_clips else 0, c)):
        binding = clips[name]
        B._play(binding)
        f0, f1 = binding[0][1].frame_range
        dur = (f1 - f0) / C.FPS
        loop = bool(np.abs(B._positions(meshes, f0) - B._positions(meshes, f1)).max() < B.LOOP_TOL)
        count = round(dur * C.VAT_FPS) + (0 if loop else 1)
        meta_clips[name] = {"start": len(rows), "count": count, "loop": loop, "duration": round(dur, 4)}
        ranges[name] = f0
        for k in range(count):
            p = B._positions(meshes, f0 + k * C.FPS / C.VAT_FPS)
            rows.append(np.stack([p[:, 0], p[:, 2], -p[:, 1], np.ones(width)], 1))
    for name in game_clips:
        if name not in meta_clips:
            meta_clips[name] = dict(meta_clips[B.ALIASES[name]], alias_of=B.ALIASES[name])

    # static mesh = idle frame 0 (frame 0 of the VAT's idle row), so the rest mesh and the bake agree
    B._play(clips["idle"])
    split, n_split = B._static_mesh(out_id, meshes, B._positions(meshes, ranges["idle"]))
    C.export_glb(os.path.join(C.VAT_DIR, f"{out_id}_static.glb"), [split], vertex_colors=True)

    pos = np.stack(rows)
    assert np.isfinite(pos).all(), f"{out_id}: non-finite VAT positions"
    pos.astype("<f2").tofile(os.path.join(C.VAT_DIR, f"{out_id}_vat.bin"))
    atlas = f"{cid}_atlas.png"  # shared with the full bake; the LOD json points at it, never its own copy
    if not lod:
        shutil.copyfile(atlas_src, os.path.join(C.VAT_DIR, atlas))
    height = C.height_of(body)
    if lod:
        assert width <= B.LOD_WIDTH_BUDGET, f"{out_id}: width {width} > {B.LOD_WIDTH_BUDGET}"
        assert n_split <= B.LOD_SPLIT_BUDGET, f"{out_id}: split {n_split} > {B.LOD_SPLIT_BUDGET}"
    meta = {
        "width": width, "vertex_count": width, "split_vertex_count": n_split, "frames": len(rows),
        "fps": C.VAT_FPS, "normals_offset": -1, "clips": meta_clips,
        "bounds_min": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).min(0)],
        "bounds_max": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).max(0)],
        "height": round(height, 4),
        "format": "float16 LE RGBA, row-major; one row of positions (Godot xyz, w=1) per frame; texel x = UV2.x * "
                  "width (the unsplit vertex); no normal rows: the shader derives flat face normals; UV0 = atlas UVs",
        "textured": True, "atlas": atlas, "source_commit": B.SOURCE_COMMIT, "source": source,
    }
    with open(os.path.join(C.VAT_DIR, f"{out_id}_vat.json"), "w") as f:
        json.dump(meta, f, indent=1)
        f.write("\n")
    entries[out_id] = {
        "file": source, "footprint": [1, 1], "height": round(height, 3),
        "triangles": sum(C.tri_count(o) for o in meshes), "materials": [], "sockets": {},
        "family": "character", "animations": list(meta_clips), "bones": [b.name for b in arm.data.bones],
        "vat": {"static": f"vat/{out_id}_static.glb", "bin": f"vat/{out_id}_vat.bin",
                "json": f"vat/{out_id}_vat.json", "atlas": f"vat/{atlas}", "vertex_count": n_split},
        "textured": True, "source_commit": B.SOURCE_COMMIT, "subject": subject,
    }
    print(f"[chars_c] {out_id}: width={width} split={n_split} frames={len(rows)} "
          f"clips={ {k: (v['count'], v['loop']) for k, v in meta_clips.items()} } height={height:.3f}")


def build(entries=None, only=None):
    """Build every living animal and person variant and write the fragment manifest
    (assets/3d/models/manifest.json is never touched)."""
    entries = {} if entries is None else entries
    out = {}
    for cid, stem in ANIMALS.items():
        if only and cid not in only:
            continue
        glb = os.path.join(SRC_DIR, stem, f"{stem}.glb")
        atlas = os.path.join(SRC_DIR, stem, "atlas.png")
        source = f"living/{stem}/{stem}.glb"
        build_one(cid, glb, stem, atlas, B.ANIMAL, source, out)
        build_one(cid, glb, stem, atlas, B.ANIMAL, source, out, lod=True)
    for stem in PEOPLE_STEMS:
        cid = f"p_{stem}"
        if only and cid not in only:
            continue
        glb = os.path.join(PEOPLE_DIR, f"person_{stem}.glb")
        subject = _subject_of(stem)
        atlas = os.path.join(SRC_DIR, subject, "atlas.png")
        source = f"living/people/person_{stem}.glb"
        build_one(cid, glb, subject, atlas, B.HUMAN, source, out)
        build_one(cid, glb, subject, atlas, B.HUMAN, source, out, lod=True)
    with open(FRAGMENT, "w") as f:
        json.dump({"models": dict(sorted(out.items()))}, f, indent=1)
        f.write("\n")
    entries.update(out)
    print(f"[chars_c] {len(out)} models -> {FRAGMENT}")
    return out


# --------------------------------------------------------------------------- review sheet

def contact(out_dir):
    """Full-vs-LOD contact sheet (review only), same mechanics as render_b_lod_contact.py (reused
    here, not duplicated: its render_walk_mid is already generic by id)."""
    import datetime
    import render_b_lod_contact as RBL  # noqa: E402
    import render_contact as RC  # noqa: E402

    os.makedirs(out_dir, exist_ok=True)
    stamp = datetime.datetime.now().strftime("%Y%m%d_%H%M")
    cam, tob = RC.setup_scene()
    bpy.context.scene.display.shading.color_type = "TEXTURE"
    tmp = os.path.join(out_dir, "_c_vat_cell.png")
    ids = list(ANIMALS) + [f"p_{s}" for s in PEOPLE_STEMS]
    cells = []
    for cid in ids:
        cells.append(RBL.render_walk_mid(cid, cam, tob, tmp))
        cells.append(RBL.render_walk_mid(cid + "_lod", cam, tob, tmp))
    RC.save_grid(cells, 2, os.path.join(out_dir, f"blender_c_vat_{stamp}.png"))


if __name__ == "__main__":
    args = C.script_args()
    try:
        if args[:1] == ["contact"]:
            contact(args[1] if len(args) > 1 else "/Users/rexxa/work/3d-data/screens")
        else:
            build(only=args or None)
    except Exception:
        import traceback
        traceback.print_exc()
        sys.exit(1)
