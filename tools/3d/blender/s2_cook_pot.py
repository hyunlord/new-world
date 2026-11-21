"""Style 2 cook fixture: a smooth round-bottomed clay pot with a sooted rim, belly and inside (INTEG1).

Replaces the faceted 24-sided "Hearth vessel" that s2clips_capture.fixture("hearth") baked into
assets/3d/vat/fixture_cook.glb. The pot keeps that vessel's footprint (centre, rim height 0.44, base 0.153,
outer radius 0.225) so mocap3d.fixture_offset (the merged AABB centre) and the ladle contact do not move.
Surfaces: `clay` (fired clay body, atlas tile via textures3d `surfaces`) and `earth` (soot; textures3d
`model_surfaces.fixture_cook` maps it to the charcoal tile).

Usage: blender -b --factory-startup -P tools/3d/blender/s2_cook_pot.py [-- <out.glb>]
"""
import math
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

CENTRE = (-0.13, -0.44)  # Blender x, y (glTF x, -z) of the old vessel.
SEGMENTS = 48
SOOT = (0.085, 0.066, 0.052)
CLAY = (0.30, 0.135, 0.062)  # linear; sRGB ~ (0.58, 0.40, 0.27) fired clay brown


def profile() -> list[tuple[float, float, bool]]:
    """(radius, height, sooted) rings from the outer bottom pole over the lip to the inner bottom."""
    rings: list[tuple[float, float, bool]] = []
    # Round belly: a squat ellipse from the bottom pole to just above its widest point.
    for i in range(1, 15):
        a = -math.pi / 2 + (math.pi / 2 + 0.42) * i / 14
        z = 0.296 + 0.143 * math.sin(a)
        rings.append((0.225 * math.cos(a), z, z < 0.215))
    # Shoulder into the neck, then a rolled, flared rim.
    for r, z in ((0.200, 0.372), (0.182, 0.392), (0.170, 0.405), (0.172, 0.418)):
        rings.append((r, z, False))
    for r, z in ((0.184, 0.428), (0.194, 0.436), (0.192, 0.441), (0.182, 0.442), (0.171, 0.437)):
        rings.append((r, z, True))
    # Inside: down the neck, then the belly ellipse 0.016 thinner, to just above the inner pole; all sooted.
    rings += [(0.160, 0.420, True), (0.156, 0.400, True)]
    for i in range(12):
        a = 0.42 - (math.pi / 2 + 0.42 - 0.2) * i / 11
        rings.append((0.209 * math.cos(a), 0.296 + 0.127 * math.sin(a), True))
    return rings


def build() -> bpy.types.Object:
    rings = profile()
    points = [(0.0, 0.0, 0.153)]
    for r, z, _ in rings:
        for s in range(SEGMENTS):
            t = math.tau * s / SEGMENTS
            # A slight hand-thrown wobble keeps the outline from reading as lathe-perfect.
            w = r * (1.0 + 0.006 * math.sin(3 * t + 0.7) + 0.004 * math.sin(5 * t))
            points.append((w * math.cos(t), w * math.sin(t), z))
    inner_pole = len(points)
    points.append((0.0, 0.0, 0.296 - 0.127))
    faces: list[tuple[int, ...]] = []
    soot: list[bool] = []
    for s in range(SEGMENTS):
        faces.append((0, 1 + (s + 1) % SEGMENTS, 1 + s))
        soot.append(True)
    for row in range(len(rings) - 1):
        a, b = 1 + row * SEGMENTS, 1 + (row + 1) * SEGMENTS
        for s in range(SEGMENTS):
            n = (s + 1) % SEGMENTS
            faces.append((a + s, a + n, b + n, b + s))
            soot.append(rings[row][2] and rings[row + 1][2])
    last = 1 + (len(rings) - 1) * SEGMENTS
    for s in range(SEGMENTS):
        faces.append((last + s, last + (s + 1) % SEGMENTS, inner_pole))
        soot.append(True)
    mesh = bpy.data.meshes.new("Hearth vessel")
    mesh.from_pydata(points, [], faces)
    mesh.validate()
    for name, colour in (("clay", CLAY), ("earth", SOOT)):
        mat = bpy.data.materials.new(name)
        mat.use_nodes = True
        bsdf = mat.node_tree.nodes.get("Principled BSDF")
        bsdf.inputs["Base Color"].default_value = (*colour, 1.0)
        bsdf.inputs["Roughness"].default_value = 0.82 if name == "clay" else 0.95
        mesh.materials.append(mat)
    mesh.polygons.foreach_set("material_index", [1 if s else 0 for s in soot])
    mesh.polygons.foreach_set("use_smooth", [True] * len(faces))
    mesh.update()
    pot = bpy.data.objects.new("Hearth vessel", mesh)
    pot.location = (*CENTRE, 0.0)
    bpy.context.scene.collection.objects.link(pot)
    return pot


def main() -> None:
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    out = argv[0] if argv else os.path.join(C.VAT_DIR, "fixture_cook.glb")
    C.reset_scene()
    pot = build()
    bpy.context.view_layer.update()
    xs = [(pot.matrix_world @ v.co) for v in pot.data.vertices]
    print("COOK_POT verts=%d faces=%d min=%s max=%s" % (len(xs), len(pot.data.polygons),
          [round(min(v[i] for v in xs), 4) for i in range(3)], [round(max(v[i] for v in xs), 4) for i in range(3)]))
    C.export_glb(out, [pot])


main()
