"""Handmade vessel, timber and thatch geometry shared by L2 station builders.

Loaded by Blender: blender -b --factory-startup -P tools/3d/blender/look2_props.py
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import math

import bmesh
from mathutils import Vector

import common as C


def rim(m: C.Model, loc: tuple[float, float, float], size: tuple[str, float, float]) -> None:
    """An open rolled rim; the aperture never receives a cap."""
    material, radius, thickness = size
    x, y, z = loc
    points = [(x + math.cos(i * math.tau / 12) * radius,
               y + math.sin(i * math.tau / 12) * radius, z) for i in range(13)]
    m.tube(material, points, [thickness] * 13, segs=4)


def pot(m: C.Model, loc: tuple[float, float, float], h: float = 0.32,
        r: float = 0.15, mat: str = "clay", segs: int = 8,
        mouth: str = "earth") -> None:
    """Keep the legacy builder API; a rolled lip surrounds a recessed inner bowl."""
    n = max(10, segs)
    profile = [(r * .55, 0), (r * .94, h * .24), (r, h * .52),
               (r * .84, h * .76), (r * .65, h * .88), (r * .73, h * .94),
               (r * .72, h)]
    verts = m.lathe(mat, profile, loc=loc, segs=n, top_mat=mouth)
    faces = {f for v in verts for f in v.link_faces}
    for face in faces:
        face.smooth = len(face.verts) == 4
    x, y, z = loc
    caps = [f for f in faces if all(abs(v.co.z - z - h) < .0001 for v in f.verts)]
    bmesh.ops.delete(m.bm, geom=caps, context="FACES_ONLY")
    m.cyl(mat, r * .53, h * .3, loc=(x, y, z + h * .7), r2=r * .6, segs=n,
          caps=False)
    m.cyl(mouth, r * .53, .004, loc=(x, y, z + h * .7), segs=n)
    rim(m, (x, y, z + h), (mat, r * .65, r * .085))
    # Broad clay coils remain visible at game scale without another material surface.
    for radius, level in ((r * .96, .28), (r * .87, .74)):
        m.cyl(mat, radius * 1.02, h * .04, loc=(x, y, z + h * level),
              segs=n, caps=False)


def basket(m: C.Model, loc: tuple[float, float, float], h: float = .22,
           r: float = .17, segs: int = 8) -> None:
    """Woven ribs and a thick open rim, preserving the old basket signature."""
    x, y, z = loc
    n = max(8, segs)
    verts = m.lathe("thatch", [(r * .73, 0), (r, h * .85), (r * 1.04, h)],
                    loc=loc, segs=n)
    caps = [f for f in {f for v in verts for f in v.link_faces}
            if all(abs(v.co.z - z - h) < .0001 for v in f.verts)]
    bmesh.ops.delete(m.bm, geom=caps, context="FACES_ONLY")
    m.cyl("earth", r * .78, .006, loc=(x, y, z + h * .2), segs=n)
    rim(m, (x, y, z + h), ("thatch", r * 1.02, r * .06))
    for level in range(4):
        rz = h * (.14 + level * .23)
        radius = r * (.77 + level * .082)
        m.cyl("thatch", radius, h * .055, loc=(x, y, z + rz), segs=n, caps=False)
    for i in range(n):
        a = math.tau * i / n
        m.rod("thatch", (x + math.cos(a) * r * .76, y + math.sin(a) * r * .76, z + .01),
              (x + math.cos(a) * r * 1.02, y + math.sin(a) * r * 1.02, z + h),
              r * .035, segs=4)


def log(m: C.Model, p0: tuple[float, float, float], p1: tuple[float, float, float],
        r: float = .06, segs: int = 6) -> None:
    """A bowed, tapered log with visible endgrain; arguments match existing callers."""
    a, b = Vector(p0), Vector(p1)
    middle = a.lerp(b, .48)
    middle.x += r * .12
    middle.y -= r * .09
    m.tube("trunk", [tuple(a), tuple(middle), tuple(b)], [r, r * 1.06, r * .91],
           segs=max(6, segs))
    axis = (b - a).normalized()
    for point, direction, radius in ((a, -axis, r), (b, axis, r * .91)):
        start = point + direction * .001
        end = point + direction * .006
        m.rod("wood", tuple(start), tuple(end), radius * .92, segs=max(6, segs))
        m.rod("trunk", tuple(end), tuple(end + direction * .001), radius * .23, segs=6)


def lash(m: C.Model, center: tuple[float, float, float], radius: float) -> None:
    """Three chunky cord wraps around a vertical joint."""
    x, y, z = center
    for k in range(3):
        m.cyl("thatch", radius, .025, loc=(x, y, z + (k - 1) * .032), segs=8, caps=False)


def thatch(m: C.Model, size: tuple[float, float, float], base: float) -> None:
    """Overlapping gable bundles, irregular eaves and a raised tied ridge."""
    width, depth, rise = size
    count = max(5, round(width / .22))
    angle = math.atan2(rise, depth / 2)
    run = math.hypot(depth / 2, rise)
    for side in (-1, 1):
        for course in range(3):
            t = (course + .5) / 3
            for strip in range(count):
                x = -width / 2 + (strip + .5) * width / count
                rough = m.rng.uniform(-.025, .025)
                m.box("thatch", (width / count * 1.08, run / 3 + .12, .105),
                      loc=(x, side * depth / 2 * (1 - t), base + rise * t + rough),
                      rot=(-side * angle, 0, rough), taper=.9, jitter=.012)
    log(m, (-width / 2, 0, base + rise + .09), (width / 2, 0, base + rise + .09), r=.055)
    for x in (-width * .32, width * .32):
        m.rod("wood", (x, -depth * .35, base + rise * .35 + .11),
              (x, depth * .35, base + rise * .35 + .11), .018, segs=4)


def fit(m: C.Model, footprint: tuple[float, float]) -> None:
    """Keep decorative timber and stone inside the established centred footprint."""
    for axis, extent in enumerate(footprint):
        maximum = max(abs(v.co[axis]) for v in m.bm.verts)
        factor = min(1, extent / 2 / maximum)
        for vertex in m.bm.verts:
            vertex.co[axis] *= factor


def finish(m: C.Model) -> None:
    """Small chamfers soften large block edges while preserving all material IDs."""
    m.bm.normal_update()
    candidates = []
    for edge in m.bm.edges:
        if len(edge.link_faces) != 2 or edge.calc_length() < .16:
            continue
        material = m.mats[edge.link_faces[0].material_index]
        if material not in ("wood", "wood_dark", "stone", "clay"):
            continue
        if edge.calc_face_angle() > .75:
            candidates.append(edge)
    # Bevel only broad structural edges; reeds, rope and tiny tools keep their low cost.
    if candidates:
        bmesh.ops.bevel(m.bm, geom=candidates, offset=.012, segments=1, affect="EDGES",
                        clamp_overlap=True)
    for vertex in m.bm.verts:
        vertex.co.z = max(0, vertex.co.z)
    m.bm.normal_update()
