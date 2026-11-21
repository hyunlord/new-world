"""Stored hide piles (INTEG1): irregular pelts with a fur side and a tan flesh side, in 1 / 3 / 6 pelt stacks.

The M3D-ASSETS hide items were stacked rectangles in one pale `hide` colour (glTF factor 0.62/0.47/0.32 is
linear, so they imported as near-white beige and graded to grey blobs). Each pelt here has a ragged outline with
leg, neck and tail tabs, a slight drape, a `fur` top (or `hide` for cured leather) and a `hide` underside/edge.
Writes assets/3d/models/w3d_pelt_<kind>_<s|m|l>.glb and the fragment w3d_pelts_manifest.json; bindings3d
`items.by_id` points the hide items at them. Colours are linear glTF factors (Godot shows their sRGB).

Usage: blender -b --factory-startup -P tools/3d/blender/s2_pelts.py
"""
import json
import math
import os
import random
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

FRAGMENT = os.path.join(C.MODELS_DIR, "w3d_pelts_manifest.json")
SEGMENTS = 44
RINGS = (0.35, 0.7, 1.0)
# kind: (top surface, top colour, flesh colour, size, tab strength, thickness)
KINDS = {
    "hide": ("fur", (0.52, 0.27, 0.10), (0.72, 0.52, 0.29), 1.0, 1.0, 0.016),
    "wolf_hide": ("fur", (0.36, 0.30, 0.22), (0.60, 0.38, 0.18), 1.04, 1.1, 0.018),
    "goat_hide": ("fur", (0.50, 0.36, 0.20), (0.62, 0.40, 0.19), 0.86, 0.9, 0.015),
    "cured_hide": ("hide", (0.48, 0.26, 0.11), (0.48, 0.26, 0.11), 0.98, 0.45, 0.009),
}
STACKS = {"s": 1, "m": 3, "l": 6}


def outline(rng: random.Random, size: float, tabs: float) -> list[float]:
    """Radius scale per segment: an oval body with four legs, a neck and a tail, and a ragged edge."""
    bumps = [(0.0, 0.16, 0.20), (math.pi, 0.11, 0.10)]  # neck, tail: (angle, height, width)
    for base in (0.62, math.pi - 0.62, math.pi + 0.62, -0.62):
        bumps.append((base + rng.uniform(-0.1, 0.1), rng.uniform(0.24, 0.31), rng.uniform(0.09, 0.13)))
    out = []
    for s in range(SEGMENTS):
        t = math.tau * s / SEGMENTS
        r = 1.0 + rng.uniform(-0.035, 0.035)
        for angle, height, width in bumps:
            d = math.atan2(math.sin(t - angle), math.cos(t - angle))
            r += tabs * height * math.exp(-(d / width) ** 2)
        out.append(r * size)
    return out


def pelt(rng: random.Random, kind: str, z0: float, yaw: float, shift: tuple[float, float],
         points: list, faces: list, mats: list) -> None:
    top_mat, _, _, size, tabs, thick = KINDS[kind]
    radii = outline(rng, size, tabs)
    curl = [rng.uniform(0.0, 0.018) for _ in range(SEGMENTS)]
    a, b = 0.33, 0.235  # body half-length / half-width before tabs (m)

    def place(x: float, y: float, z: float) -> tuple[float, float, float]:
        c, s = math.cos(yaw), math.sin(yaw)
        return (shift[0] + c * x - s * y, shift[1] + s * x + c * y, z)

    for layer, dz in ((0, thick), (1, 0.0)):  # top (fur), bottom (flesh)
        first = len(points)
        points.append(place(0.0, 0.0, z0 + dz + 0.012))
        for f in RINGS:
            for s in range(SEGMENTS):
                t = math.tau * s / SEGMENTS
                # A low drape: the middle rides higher, the tabs curl a little at the very edge.
                z = z0 + dz + 0.012 * (1.0 - f * f) + (curl[s] if f == 1.0 else 0.0)
                points.append(place(a * radii[s] * f * math.cos(t), b * radii[s] * f * math.sin(t), z))
        for s in range(SEGMENTS):
            n = (s + 1) % SEGMENTS
            tri = (first, first + 1 + s, first + 1 + n)
            faces.append(tri if layer == 0 else tuple(reversed(tri)))
            mats.append(top_mat if layer == 0 else "hide")
        for ring in range(len(RINGS) - 1):
            r0, r1 = first + 1 + ring * SEGMENTS, first + 1 + (ring + 1) * SEGMENTS
            for s in range(SEGMENTS):
                n = (s + 1) % SEGMENTS
                quad = (r0 + s, r1 + s, r1 + n, r0 + n)
                faces.append(quad if layer == 0 else tuple(reversed(quad)))
                mats.append(top_mat if layer == 0 else "hide")
    # The cut edge between the two outer rings is flesh-coloured.
    top_edge = len(points) - 2 * (1 + len(RINGS) * SEGMENTS) + 1 + (len(RINGS) - 1) * SEGMENTS
    bottom_edge = top_edge + 1 + len(RINGS) * SEGMENTS
    for s in range(SEGMENTS):
        n = (s + 1) % SEGMENTS
        faces.append((top_edge + s, bottom_edge + s, bottom_edge + n, top_edge + n))
        mats.append("hide")


def build(kind: str, bucket: str) -> bpy.types.Object:
    rng = random.Random(f"{kind}/{bucket}")
    points: list = []
    faces: list = []
    mats: list = []
    for k in range(STACKS[bucket]):
        yaw = rng.uniform(-0.25, 0.25) + (math.pi if k % 2 else 0.0)
        shift = (rng.uniform(-0.05, 0.05), rng.uniform(-0.04, 0.04))
        pelt(rng, kind, k * (KINDS[kind][5] + 0.022), yaw, shift, points, faces, mats)
    mesh = bpy.data.meshes.new(f"w3d_pelt_{kind}_{bucket}")
    mesh.from_pydata(points, [], faces)
    _, top_colour, flesh_colour, *_ = KINDS[kind]
    names = sorted(set(mats))
    for name in names:
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        colour = flesh_colour if name == "hide" else top_colour
        bsdf.inputs["Base Color"].default_value = (*colour, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.9
        mesh.materials.append(mat)
    mesh.polygons.foreach_set("material_index", [names.index(m) for m in mats])
    mesh.polygons.foreach_set("use_smooth", [True] * len(faces))
    mesh.validate()
    mesh.update()
    obj = bpy.data.objects.new(mesh.name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def main() -> None:
    entries = {}
    for kind in KINDS:
        for bucket, count in STACKS.items():
            C.reset_scene()
            obj = build(kind, bucket)
            model_id = obj.name
            C.export_glb(os.path.join(C.MODELS_DIR, f"{model_id}.glb"), [obj])
            x0, x1, y0, y1 = C.bbox_xy(obj)
            entries[model_id] = {
                "file": f"models/{model_id}.glb", "footprint": [1, 1], "height": round(C.height_of(obj), 3),
                "triangles": C.tri_count(obj), "materials": sorted({m.name for m in obj.data.materials}),
                "sockets": {}, "family": "item", "content_id": kind, "stack_display_count": count,
            }
            if max(abs(x0), abs(x1), abs(y0), abs(y1)) > 0.5:
                entries[model_id]["overhang"] = True  # pelt tabs may reach a little past the tile, as the old piles did
            print("PELT %s tris=%d height=%.3f bbox=%.3f..%.3f, %.3f..%.3f" % (
                model_id, entries[model_id]["triangles"], entries[model_id]["height"], x0, x1, y0, y1))
    with open(FRAGMENT, "w") as f:
        json.dump({"models": entries}, f, indent=1)
        f.write("\n")


main()
