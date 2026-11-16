"""Shared helpers for the WorldSim procedural model pipeline (Blender 5.x, headless).

Conventions (see README.md):
  * 1 Blender unit = 1 m = 1 tile. Build in Blender Z-up with the model FRONT facing -Y;
    the glTF exporter converts (x, y, z) -> Godot (x, z, -y) so the front faces Godot +Z.
  * Origin = centre of the footprint on the ground (z = 0).
  * Material names are an API (Godot recolours by name); only names in PALETTE are allowed.
"""
import json
import math
import os
import random

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
MODELS_DIR = os.path.join(REPO, "assets", "3d", "models")
VAT_DIR = os.path.join(REPO, "assets", "3d", "vat")
MANIFEST = os.path.join(MODELS_DIR, "manifest.json")

# name -> (linear-ish base colour RGB, emission strength). Godot overrides these by name.
PALETTE = {
    "trunk": ((0.30, 0.18, 0.09), 0.0),
    "canopy": ((0.20, 0.45, 0.14), 0.0),
    "fruit": ((0.70, 0.08, 0.12), 0.0),
    "stone": ((0.52, 0.50, 0.47), 0.0),
    "vein": ((0.80, 0.42, 0.18), 0.0),
    "earth": ((0.36, 0.25, 0.15), 0.0),
    "wood": ((0.58, 0.40, 0.22), 0.0),
    "wood_dark": ((0.30, 0.19, 0.10), 0.0),
    "clay": ((0.68, 0.38, 0.22), 0.0),
    "metal": ((0.62, 0.60, 0.56), 0.0),
    "glow": ((1.00, 0.45, 0.08), 6.0),
    "thatch": ((0.78, 0.64, 0.33), 0.0),
    "hide": ((0.62, 0.47, 0.32), 0.0),
    "cloth": ((0.72, 0.66, 0.52), 0.0),
    "skin": ((0.80, 0.58, 0.44), 0.0),
    "hair": ((0.22, 0.13, 0.07), 0.0),
    "leather": ((0.36, 0.22, 0.12), 0.0),
    "horn": ((0.86, 0.80, 0.66), 0.0),
    "fur": ((0.45, 0.33, 0.22), 0.0),
    "fur_light": ((0.85, 0.80, 0.70), 0.0),
    "snow": ((0.94, 0.96, 1.00), 0.0),
}


def rng(seed):
    """Deterministic RNG per model (seed may be a string)."""
    if isinstance(seed, str):
        seed = sum((i + 1) * ord(c) for i, c in enumerate(seed))
    return random.Random(seed)


def to_godot(v):
    """Blender Z-up (x, y, z) -> Godot Y-up (x, z, -y)."""
    return [round(v[0], 4), round(v[2], 4), round(-v[1], 4)]


# --------------------------------------------------------------------------- scene

def reset_scene():
    for coll in (bpy.data.objects, bpy.data.meshes, bpy.data.materials, bpy.data.armatures,
                 bpy.data.actions, bpy.data.curves, bpy.data.cameras, bpy.data.lights,
                 bpy.data.images, bpy.data.texts):
        for item in list(coll):
            coll.remove(item)
    scene = bpy.context.scene
    scene.render.fps = 30
    scene.frame_start = 0
    scene.frame_current = 0


def material(name):
    if name not in PALETTE:
        raise ValueError(f"material '{name}' is not in the material API: {sorted(PALETTE)}")
    mat = bpy.data.materials.get(name)
    if mat:
        return mat
    col, emit = PALETTE[name]
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*col, 1.0)
    bsdf.inputs["Roughness"].default_value = 0.85
    if emit > 0:
        bsdf.inputs["Emission Color"].default_value = (*col, 1.0)
        bsdf.inputs["Emission Strength"].default_value = emit
    mat.diffuse_color = (*col, 1.0)  # viewport / workbench colour
    return mat


def plain_material(name, col=(0.8, 0.8, 0.8)):
    """Material outside the API (used only for the VAT static mesh, which Godot replaces)."""
    mat = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    mat.use_nodes = True
    mat.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = (*col, 1.0)
    mat.diffuse_color = (*col, 1.0)
    return mat


# --------------------------------------------------------------------------- mesh builder

def _matrix(loc=(0, 0, 0), rot=(0, 0, 0), scale=(1, 1, 1)):
    s = Matrix.Diagonal((*scale, 1.0))
    return Matrix.Translation(Vector(loc)) @ Euler(rot, "XYZ").to_matrix().to_4x4() @ s


class Model:
    """Accumulates primitives into one bmesh; each primitive has one material and an
    optional bone (for rigid skinning of characters)."""

    def __init__(self, model_id, seed=None):
        self.id = model_id
        self.bm = bmesh.new()
        self.mats = []
        self.bone_layer = self.bm.verts.layers.deform.verify()
        self.bones = []  # bone names in vertex-group order
        self.rng = rng(model_id if seed is None else seed)
        self.sockets = {}
        self.colors = {}  # material name -> default RGB override baked into this glb only

    # ---- internals
    def _finish_part(self, verts, mat, bone, jitter=0.0):
        if mat not in self.mats:
            self.mats.append(mat)
        mi = self.mats.index(mat)
        faces = {f for v in verts for f in v.link_faces}
        for f in faces:
            f.material_index = mi
            f.smooth = False
        if jitter:
            for v in verts:
                v.co += Vector(tuple(self.rng.uniform(-jitter, jitter) for _ in range(3)))
        if bone is not None:
            if bone not in self.bones:
                self.bones.append(bone)
            gi = self.bones.index(bone)
            for v in verts:
                v[self.bone_layer][gi] = 1.0
        return verts

    # ---- primitives (all take a material name first)
    def box(self, mat, size, loc=(0, 0, 0), rot=(0, 0, 0), bone=None, jitter=0.0, taper=1.0):
        """Box of `size` centred at loc. taper < 1 shrinks the top face (+Z) in X/Y."""
        r = bmesh.ops.create_cube(self.bm, size=1.0, matrix=Matrix.Identity(4))
        verts = r["verts"]
        for v in verts:
            if taper != 1.0 and v.co.z > 0:
                v.co.x *= taper
                v.co.y *= taper
            v.co = _matrix(loc, rot, size) @ v.co
        return self._finish_part(verts, mat, bone, jitter)

    def cyl(self, mat, r, h, loc=(0, 0, 0), rot=(0, 0, 0), segs=8, r2=None, bone=None,
            jitter=0.0, scale=(1, 1, 1), caps=True, cap_mat=None):
        """Cylinder/cone with its BASE at loc, extending along local +Z by h.
        cap_mat gives the end caps their own material (e.g. cut log ends)."""
        r2 = r if r2 is None else r2
        res = bmesh.ops.create_cone(self.bm, cap_ends=caps, cap_tris=False, segments=segs,
                                    radius1=r, radius2=max(r2, 0.0), depth=h,
                                    matrix=Matrix.Translation((0, 0, h / 2)))
        verts = res["verts"]
        if r2 <= 0.0:  # merge the degenerate tip ring into one apex vertex
            tip = [v for v in verts if v.co.z > h * 0.99]
            bmesh.ops.pointmerge(self.bm, verts=tip, merge_co=Vector((0, 0, h)))
            verts = [v for v in verts if v.is_valid]
        caps_faces = [f for f in {f for v in verts for f in v.link_faces}
                      if max(v.co.z for v in f.verts) - min(v.co.z for v in f.verts) < 1e-6]
        m = _matrix(loc, rot, scale)
        for v in verts:
            v.co = m @ v.co
        self._finish_part(verts, mat, bone, jitter)
        if cap_mat:
            if cap_mat not in self.mats:
                self.mats.append(cap_mat)
            for f in caps_faces:
                f.material_index = self.mats.index(cap_mat)
        return verts

    def ball(self, mat, r, loc=(0, 0, 0), scale=(1, 1, 1), rot=(0, 0, 0), subdiv=1, bone=None,
             jitter=0.0, flat_bottom=None, ground=False):
        """Icosphere (subdiv 1 = 20 tris, 2 = 80 tris). flat_bottom clamps local z >= value."""
        res = bmesh.ops.create_icosphere(self.bm, subdivisions=subdiv, radius=r,
                                         matrix=Matrix.Identity(4))
        verts = res["verts"]
        if jitter:
            for v in verts:
                v.co *= 1.0 + self.rng.uniform(-jitter, jitter)
        if flat_bottom is not None:
            for v in verts:
                v.co.z = max(v.co.z, flat_bottom)
        m = _matrix(loc, rot, scale)
        for v in verts:
            v.co = m @ v.co
            if ground:
                v.co.z = max(v.co.z, 0.0)
        return self._finish_part(verts, mat, bone)

    def rod(self, mat, p0, p1, r, segs=6, r2=None, cap_mat=None, bone=None, jitter=0.0):
        """Cylinder (or cone if r2 == 0) from point p0 to point p1."""
        d = Vector(p1) - Vector(p0)
        rot = Vector((0, 0, 1)).rotation_difference(d.normalized()).to_euler("XYZ")
        return self.cyl(mat, r, d.length, loc=p0, rot=tuple(rot), segs=segs, r2=r2,
                        cap_mat=cap_mat, bone=bone, jitter=jitter)

    def lathe(self, mat, profile, loc=(0, 0, 0), segs=8, top_mat=None, bone=None, scale=(1, 1, 1)):
        """Surface of revolution around local Z: profile = [(radius, z), ...] bottom to top.
        top_mat colours the top cap (e.g. dark pot opening, glowing furnace throat)."""
        pts = [(loc[0], loc[1], loc[2] + z) for _, z in profile]
        verts = self.tube(mat, pts, [r for r, _ in profile], segs=segs, bone=bone)
        if scale != (1, 1, 1):
            for v in verts:
                v.co = Vector(loc) + Vector(((v.co.x - loc[0]) * scale[0], (v.co.y - loc[1]) * scale[1],
                                             (v.co.z - loc[2]) * scale[2]))
        if top_mat and profile[-1][0] > 0:
            if top_mat not in self.mats:
                self.mats.append(top_mat)
            ztop = max(v.co.z for v in verts)
            for f in {f for v in verts for f in v.link_faces}:
                if len(f.verts) == segs and all(abs(v.co.z - ztop) < 1e-5 for v in f.verts):
                    f.material_index = self.mats.index(top_mat)
        return verts

    def tube(self, mat, points, radii, segs=6, bone=None, jitter=0.0):
        """Bent tube through `points` (list of xyz) with per-point radii; closed tip if radius 0."""
        rings = []
        for i, p in enumerate(points):
            p = Vector(p)
            if radii[i] <= 0.0:  # pointed tip: a single apex vertex
                rings.append([self.bm.verts.new(p)] * segs)
                continue
            d = (Vector(points[min(i + 1, len(points) - 1)]) - Vector(points[max(i - 1, 0)])).normalized()
            q = Vector((0, 0, 1)).rotation_difference(d)
            ring = []
            for k in range(segs):
                a = 2 * math.pi * k / segs
                off = q @ Vector((math.cos(a), math.sin(a), 0)) * radii[i]
                ring.append(self.bm.verts.new(p + off))
            rings.append(ring)
        verts = list(dict.fromkeys(v for ring in rings for v in ring))
        for a, b in zip(rings, rings[1:]):
            for k in range(segs):
                quad = list(dict.fromkeys((a[k], a[(k + 1) % segs], b[(k + 1) % segs], b[k])))
                self.bm.faces.new(quad)
        if radii[0] > 0:
            self.bm.faces.new(list(reversed(rings[0])))
        if radii[-1] > 0:
            self.bm.faces.new(rings[-1])
        return self._finish_part(verts, mat, bone, jitter)

    # ---- output
    def build_object(self, name=None):
        bm = self.bm
        bm.normal_update()
        me = bpy.data.meshes.new(name or self.id)
        bm.to_mesh(me)
        bm.free()
        for m in self.mats:
            mat = material(m)
            col = self.colors.get(m, PALETTE[m][0])
            mat.node_tree.nodes.get("Principled BSDF").inputs["Base Color"].default_value = (*col, 1.0)
            mat.diffuse_color = (*col, 1.0)
            me.materials.append(mat)
        obj = bpy.data.objects.new(name or self.id, me)
        bpy.context.scene.collection.objects.link(obj)
        for b in self.bones:
            obj.vertex_groups.new(name=b)
        return obj


def tri_count(obj):
    return sum(len(p.vertices) - 2 for p in obj.data.polygons)


def height_of(obj):
    return max((obj.matrix_world @ v.co).z for v in obj.data.vertices)


def bbox_xy(obj):
    xs = [(obj.matrix_world @ v.co).x for v in obj.data.vertices]
    ys = [(obj.matrix_world @ v.co).y for v in obj.data.vertices]
    return min(xs), max(xs), min(ys), max(ys)


# --------------------------------------------------------------------------- export

GLTF_COMMON = dict(
    export_format="GLB",
    use_selection=True,
    export_yup=True,
    export_apply=True,
    export_extras=False,
    export_cameras=False,
    export_lights=False,
    export_texcoords=True,
    export_normals=True,
    export_tangents=False,
    export_materials="EXPORT",
    export_image_format="NONE",
    check_existing=False,
)


def export_glb(path, objects, animations=False, vertex_colors=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.object.select_all(action="DESELECT")
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    opts = dict(GLTF_COMMON)
    opts.update(
        export_animations=animations,
        export_skins=animations,
        export_morph=False,
        export_vertex_color="ACTIVE" if vertex_colors else "NONE",
    )
    if animations:
        opts.update(export_animation_mode="ACTIONS", export_force_sampling=True,
                    export_frame_step=1, export_reset_pose_bones=True,
                    export_optimize_animation_size=True, export_anim_single_armature=True)
    bpy.ops.export_scene.gltf(filepath=path, **opts)


def save_static(model, footprint, out_list, extra=None):
    """Build, export and register a static (non-skinned) model."""
    obj = model.build_object()
    tris = tri_count(obj)
    path = os.path.join(MODELS_DIR, f"{model.id}.glb")
    export_glb(path, [obj])
    entry = {
        "file": f"models/{model.id}.glb",
        "footprint": list(footprint),
        "height": round(height_of(obj), 3),
        "triangles": tris,
        "materials": list(model.mats),
        "sockets": {k: to_godot(v) for k, v in sorted(model.sockets.items())},
    }
    if extra:
        entry.update(extra)
    x0, x1, y0, y1 = bbox_xy(obj)
    w, h = footprint
    if x0 < -w / 2 - 1e-3 or x1 > w / 2 + 1e-3 or y0 < -h / 2 - 1e-3 or y1 > h / 2 + 1e-3:
        entry["overhang"] = True  # trees may overhang their tile; stations must not
    out_list[model.id] = entry
    bpy.data.objects.remove(obj)
    return entry


def update_manifest(entries, replace=False):
    """Merge entries into manifest.json (family scripts run alone) or rewrite it (build_all)."""
    data = {}
    if not replace and os.path.exists(MANIFEST):
        with open(MANIFEST) as f:
            data = json.load(f)
    data.setdefault("models", {}).update(entries)
    data["units"] = "1 unit = 1 m = 1 tile; Godot coords (Y up, front +Z); origin = footprint centre on ground"
    data["models"] = dict(sorted(data["models"].items()))
    os.makedirs(MODELS_DIR, exist_ok=True)
    with open(MANIFEST, "w") as f:
        json.dump(data, f, indent=1, sort_keys=False)
        f.write("\n")


def check_budget(entries, lo, hi):
    bad = {k: e["triangles"] for k, e in entries.items() if not lo <= e["triangles"] <= hi}
    if bad:
        print(f"WARNING triangle budget {lo}-{hi} violated: {bad}")
    return bad


def script_args():
    import sys
    return sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []


# --------------------------------------------------------------------------- characters
# Rigid-skinned low-poly characters: every primitive is weighted 100% to one bone (Model(bone=)).
# Poses are written in WORLD axes and converted to each bone's local rest frame, so bone roll
# never matters. Convention: rotation about +X ("pitch") turns forward(-Y) towards down(-Z):
# positive pitch = lean/nod forward for upward/forward bones, swing BACK for downward limbs.

FPS = 30          # scene/key rate (glTF animation time = frame / 30)
VAT_FPS = 15      # VAT sampling rate
X = Vector((1, 0, 0))
Y = Vector((0, 1, 0))
Z = Vector((0, 0, 1))

VAT_TINT = {  # COLOR_0 tint regions for the VAT static mesh
    "skin": (1, 0, 0), "hair": (0, 1, 0), "cloth": (0, 0, 1), "leather": (0, 0, 0),
    "fur": (1, 0, 0), "fur_light": (0, 1, 0), "horn": (0, 0, 1),
}


def make_armature(name, bones):
    """bones: list of (name, head, tail, parent_name_or_None) in Blender coords."""
    arm = bpy.data.armatures.new(name + "_rig")
    obj = bpy.data.objects.new(name + "_rig", arm)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode="EDIT")
    for bname, head, tail, parent in bones:
        eb = arm.edit_bones.new(bname)
        eb.head, eb.tail, eb.roll = head, tail, 0.0
        if parent:
            eb.parent = arm.edit_bones[parent]
    bpy.ops.object.mode_set(mode="OBJECT")
    for pb in obj.pose.bones:
        pb.rotation_mode = "QUATERNION"
    return obj


def skin_to(mesh_obj, arm_obj):
    mesh_obj.parent = arm_obj
    mod = mesh_obj.modifiers.new("Armature", "ARMATURE")
    mod.object = arm_obj


def _apply_pose(arm_obj, pose, prev):
    """pose: {bone: [(axis_world, angle), ...] or {'rot': [...], 'loc': world_vec}}"""
    for pb in arm_obj.pose.bones:
        spec = pose.get(pb.name, [])
        rots, loc = (spec.get("rot", []), spec.get("loc")) if isinstance(spec, dict) else (spec, None)
        rinv = pb.bone.matrix_local.to_3x3().inverted()
        q = Quaternion()
        for axis, ang in rots:
            q = Quaternion(rinv @ Vector(axis), ang) @ q
        if prev.get(pb.name) is not None and q.dot(prev[pb.name]) < 0:
            q.negate()  # keep quaternion keys sign-continuous
        prev[pb.name] = q.copy()
        pb.rotation_quaternion = q
        pb.location = (rinv @ Vector(loc)) if loc is not None else Vector()


def add_action(arm_obj, name, dur, pose_fn):
    """Key pose_fn(s) (s = normalised time 0..1) every frame at 30 fps and park it on the NLA."""
    ad = arm_obj.animation_data or arm_obj.animation_data_create()
    act = bpy.data.actions.new(name)
    act.use_fake_user = True
    ad.action = act
    n = round(dur * FPS)
    prev = {}
    for f in range(n + 1):
        _apply_pose(arm_obj, pose_fn(f / n), prev)
        for pb in arm_obj.pose.bones:
            pb.keyframe_insert("rotation_quaternion", frame=f, group=pb.name)
            pb.keyframe_insert("location", frame=f, group=pb.name)
    track = ad.nla_tracks.new()
    track.name = name
    strip = track.strips.new(name, 0, act)
    if hasattr(strip, "action_slot") and act.slots:
        strip.action_slot = act.slots[0]
    ad.action = None
    return act


def _use_action(arm_obj, act):
    ad = arm_obj.animation_data
    for t in ad.nla_tracks:
        t.mute = True
    ad.action = act
    if act is not None and hasattr(ad, "action_slot") and act.slots:
        ad.action_slot = act.slots[0]


def _split_copy(obj, cid):
    """Copy of obj with every face's vertices split (flat normals, one vertex per loop),
    COLOR_0 tint mask, UV0 (zeros) + UV1 (vertex index lookup) and a single material."""
    me = obj.data.copy()
    me.name = cid + "_vat"
    n_orig = len(me.vertices)
    bm = bmesh.new()
    bm.from_mesh(me)
    tag = bm.verts.layers.int.new("orig")
    for v in bm.verts:
        v[tag] = v.index
    bmesh.ops.split_edges(bm, edges=bm.edges[:])
    bm.verts.ensure_lookup_table()
    orig = [v[tag] for v in bm.verts]
    bm.to_mesh(me)
    bm.free()
    n = len(me.vertices)
    names = [m.name for m in me.materials]
    col = [0.0] * (n * 4)
    for poly in me.polygons:
        tint = VAT_TINT.get(names[poly.material_index], (0, 0, 0))
        for vi in poly.vertices:
            col[vi * 4:vi * 4 + 4] = (*tint, 1.0)
    attr = me.color_attributes.new("Color", "FLOAT_COLOR", "POINT")
    attr.data.foreach_set("color", col)
    me.color_attributes.active_color = attr
    me.color_attributes.render_color_index = me.color_attributes.find("Color")
    while me.uv_layers:
        me.uv_layers.remove(me.uv_layers[0])
    uv0 = me.uv_layers.new(name="UVMap")
    uv1 = me.uv_layers.new(name="UV2")
    vidx = [0] * len(me.loops)
    me.loops.foreach_get("vertex_index", vidx)
    uv0.data.foreach_set("uv", [0.0] * (2 * len(me.loops)))
    # The VAT column is the ORIGINAL (unsplit) vertex: split copies share it (flat normals come from the shader).
    uv1.data.foreach_set("uv", [c for vi in vidx for c in ((orig[vi] + 0.5) / n_orig, 0.5)])
    me.materials.clear()
    me.materials.append(plain_material("vat_" + cid))
    for poly in me.polygons:
        poly.material_index = 0
    split = bpy.data.objects.new(cid + "_vat", me)
    bpy.context.scene.collection.objects.link(split)
    for g in obj.vertex_groups:
        split.vertex_groups.new(name=g.name)
    return split


def export_character(cid, mesh_obj, arm_obj, clips, entries, extra=None):
    """clips: {name: (duration_s, loop, action)}. Writes models/<cid>.glb (skinned + actions) and
    vat/<cid>_static.glb, <cid>_vat.bin, <cid>_vat.json."""
    import numpy as np
    scene = bpy.context.scene
    skin_to(mesh_obj, arm_obj)
    tris = tri_count(mesh_obj)
    height = height_of(mesh_obj)
    for t in arm_obj.animation_data.nla_tracks:
        t.mute = False
    export_glb(os.path.join(MODELS_DIR, f"{cid}.glb"), [arm_obj, mesh_obj], animations=True)

    # ---- VAT
    split = _split_copy(mesh_obj, cid)
    skin_to(split, arm_obj)
    n = len(split.data.vertices)
    arm_obj.data.pose_position = "REST"
    _use_action(arm_obj, None)
    scene.frame_set(0)
    export_glb(os.path.join(VAT_DIR, f"{cid}_static.glb"), [split], vertex_colors=True)
    arm_obj.data.pose_position = "POSE"

    # Positions of the ORIGINAL skinned mesh (one column per unsplit vertex), float16: about a sixth of the
    # split float32 position+normal layout, so every .bin stays well under the repository's 5 MB file limit.
    n_orig = len(mesh_obj.data.vertices)
    pos_rows, meta_clips = [], {}
    co = np.empty(n_orig * 3, dtype=np.float32)
    for name, (dur, loop, act) in clips.items():
        _use_action(arm_obj, act)
        count = round(dur * VAT_FPS) + (0 if loop else 1)
        meta_clips[name] = {"start": len(pos_rows), "count": count, "loop": loop,
                            "duration": dur}
        for k in range(count):
            scene.frame_set(round(k * FPS / VAT_FPS))
            dg = bpy.context.evaluated_depsgraph_get()
            ev = mesh_obj.evaluated_get(dg)
            me = ev.to_mesh()
            assert len(me.vertices) == n_orig, "armature evaluation changed the vertex count"
            me.vertices.foreach_get("co", co)
            ev.to_mesh_clear()
            b = co.reshape(n_orig, 3)
            row = np.empty((n_orig, 4), dtype=np.float32)
            row[:, 0], row[:, 1], row[:, 2], row[:, 3] = b[:, 0], b[:, 2], -b[:, 1], 1.0
            pos_rows.append(row)
    _use_action(arm_obj, None)
    pos = np.stack(pos_rows)
    data = pos.astype("<f2")
    os.makedirs(VAT_DIR, exist_ok=True)
    data.tofile(os.path.join(VAT_DIR, f"{cid}_vat.bin"))
    meta = {
        "width": n_orig, "vertex_count": n_orig, "split_vertex_count": n, "frames": len(pos_rows), "fps": VAT_FPS,
        "normals_offset": -1, "clips": meta_clips,
        "bounds_min": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).min(0)],
        "bounds_max": [round(float(v), 4) for v in pos[:, :, :3].reshape(-1, 3).max(0)],
        "height": round(height, 4),
        "format": "float16 LE RGBA, row-major; one row of positions (Godot xyz, w=1) per frame; texel x = UV2.x * "
                  "width (the unsplit vertex); no normal rows: the shader derives flat face normals",
    }
    with open(os.path.join(VAT_DIR, f"{cid}_vat.json"), "w") as f:
        json.dump(meta, f, indent=1)
        f.write("\n")
    entry = {
        "file": f"models/{cid}.glb", "footprint": [1, 1], "height": round(height, 3),
        "triangles": tris, "materials": [m.name for m in mesh_obj.data.materials],
        "sockets": {}, "family": "character", "animations": list(clips),
        "bones": [b.name for b in arm_obj.data.bones],
        "vat": {"static": f"vat/{cid}_static.glb", "bin": f"vat/{cid}_vat.bin",
                "json": f"vat/{cid}_vat.json", "vertex_count": n},
    }
    if extra:
        entry.update(extra)
    entries[cid] = entry
    return entry


def wave(s, phase=0.0, k=1):
    """sin over normalised loop time s (0..1); k = cycles per loop."""
    return math.sin(2 * math.pi * (k * s + phase))
