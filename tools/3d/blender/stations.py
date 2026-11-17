"""Stations / buildings (blueprint style ids of the sim). Run alone:
blender -b --factory-startup -P stations.py

Footprint (w, h) is in tiles: w along X (Godot X), h along depth (Godot Z). FRONT faces Godot +Z
(Blender -Y). Sockets are given in Blender coords here and written to the manifest in Godot coords.
`work` defaults to just in front of the footprint: (0, -(h/2 + 0.35), 0).
Water surfaces use `metal` (blue-grey default) because the material API has no water name.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import look2_props_details as D
import look2_props_shapes as H

TAU = 2 * math.pi

# id -> footprint, sockets, default-colour overrides, builder params
STATIONS = {
    "hearth": dict(fp=(1, 1), light=(0, 0, 0.35), lit=True, colors={"earth": (0.24, 0.22, 0.21)}),
    "hearth_unlit": dict(fp=(1, 1), lit=False, colors={"earth": (0.30, 0.29, 0.28)}),
    "lean_to": dict(fp=(2, 2), ridge_h=1.7, work=(0, -0.2, 0)),
    "knapping_spot": dict(fp=(1, 1), colors={"vein": (0.12, 0.12, 0.14)}),
    "wood_bench": dict(fp=(2, 1), top_h=0.72),
    "drying_rack": dict(fp=(2, 1), bar_h=1.5),
    "grinding_stone": dict(fp=(1, 1), colors={"thatch": (0.86, 0.74, 0.42)}),
    "pottery_bench": dict(fp=(2, 1), colors={"earth": (0.15, 0.10, 0.07)}),
    "loom": dict(fp=(2, 1)),
    "retting_pit": dict(fp=(2, 2), colors={"metal": (0.30, 0.38, 0.36)}),
    "clay_oven": dict(fp=(1, 1), light=(0, -0.36, 0.22), colors={"wood_dark": (0.08, 0.06, 0.05)}),
    "charcoal_clamp": dict(fp=(3, 3), holes=6, light=(0, 0, 0.9)),
    "smelter": dict(fp=(2, 2), shaft_h=1.6, light=(0, -0.5, 0.2)),
    "casting_bench": dict(fp=(2, 1), light=(0.62, -0.05, 0.3)),
    "anvil": dict(fp=(1, 1)),
    "kiln": dict(fp=(2, 2), light=(0, -0.82, 0.22)),
    "storage_shelf": dict(fp=(2, 1), shelves=(0.3, 0.85, 1.4)),
    "bedroll": dict(fp=(2, 1)),
    "door": dict(fp=(1, 1), width=0.9, height=2.0, thick=0.08),
    "well_body": dict(fp=(3, 3), colors={"metal": (0.18, 0.26, 0.30)}),
    "bee_skep": dict(fp=(1, 1)),
}


# --------------------------------------------------------------------------- shared bits

# These signatures are shared by the building, station and item generators.
pot, basket, log = H.pot, H.basket, H.log


def stone_ring(m, n, rad, size, z=0.0, scale=(1.2, 1.0, 0.8)):
    for i in range(n):
        a = TAU * i / n + m.rng.uniform(-0.1, 0.1)
        m.ball("stone", size * m.rng.uniform(0.85, 1.1), loc=(math.cos(a) * rad, math.sin(a) * rad,
               z + size * 0.5), scale=scale, rot=(0, 0, a), subdiv=1, jitter=0.15, ground=True)


# --------------------------------------------------------------------------- builders

def hearth(m, p):
    stone_ring(m, 9, 0.36, 0.13)
    m.cyl("earth", 0.32, 0.03, segs=9)
    for i in range(4):
        a = TAU * i / 4 + 0.4
        m.rod("wood_dark", (math.cos(a) * 0.3, math.sin(a) * 0.3, 0.03), (0, 0, 0.32), 0.045, segs=5)
    if p["lit"]:
        m.ball("glow", 0.17, loc=(0, 0, 0.03), scale=(1, 1, 0.45), subdiv=1, jitter=0.1)
        for i, (x, y, h) in enumerate([(0, 0, 0.5), (0.08, 0.05, 0.36), (-0.07, -0.04, 0.32)]):
            m.cyl("glow", 0.1 if i == 0 else 0.07, h, loc=(x, y, 0.04), segs=5, r2=0.0,
                  rot=(0, 0, i))


def lean_to(m, p):
    rh = p["ridge_h"]
    yr, yb = -0.55, 0.65  # ridge line (front) and back ground line (steep: inside reads from the south)
    for x in (-0.85, 0.85):  # forked posts
        m.rod("wood", (x, yr, 0), (x, yr, rh + 0.05), 0.06, segs=6)
        m.rod("wood", (x, yr, rh - 0.1), (x + 0.12 * (1 if x > 0 else -1), yr - 0.05, rh + 0.25),
              0.035, segs=5)
    m.rod("wood", (-0.98, yr, rh), (0.98, yr, rh), 0.06, segs=6)
    for i in range(7):  # rafters leaning from the ridge to the ground
        x = -0.9 + 0.3 * i
        m.rod("wood_dark", (x, yr - 0.08, rh + 0.08), (x, yb, 0), 0.035, segs=5)
    ang = math.atan2(rh, yb - yr)
    ln = math.hypot(rh, yb - yr)
    for i in range(4):  # overlapping bark/thatch courses, lowest first (shingled)
        f = (i + 0.5) / 4
        y = yr + (yb - yr) * (1 - f)
        z = rh * f + 0.08
        m.box("thatch", (2.1 - 0.05 * i, ln / 4 + 0.22, 0.11), loc=(0, y, z), rot=(-ang, 0, 0),
              jitter=0.025)
    for i in range(8):  # leafy branches thrown on top
        f = 0.12 + 0.76 * ((i * 0.37) % 1.0)
        x = -0.85 + 0.243 * i
        m.ball("canopy", 0.24, loc=(x, yr + (yb - yr) * f, rh * (1 - f) + 0.16), scale=(1.4, 1, 0.5),
               subdiv=1, jitter=0.12)
    for x in (-1.0, 1.0):  # side walls of leaning branches
        for k in range(3):
            m.rod("wood_dark", (x, yr + 0.1 + 0.25 * k, rh * (1 - 0.18 * k)),
                  (x * 0.98, yr + 0.55 + 0.3 * k, 0), 0.03, segs=4)
    m.box("hide", (1.1, 0.8, 0.03), loc=(0.15, 0.3, 0.015), rot=(0, 0, 0.08), taper=0.9)


def knapping_spot(m, p):
    m.box("hide", (0.75, 0.6, 0.02), loc=(0, -0.08, 0.01), rot=(0, 0, 0.1), taper=0.85)
    log(m, (-0.3, 0.3, 0.13), (0.3, 0.3, 0.13), r=0.13, segs=7)
    m.ball("stone", 0.2, loc=(0, -0.1, 0.05), scale=(1.2, 1.0, 0.5), subdiv=1, jitter=0.1,
           ground=True)
    m.ball("stone", 0.07, loc=(0.18, -0.12, 0.13), scale=(1.2, 1, 0.9), subdiv=1, jitter=0.1)
    for i, (x, y) in enumerate([(-0.2, -0.25), (0.25, -0.3), (-0.05, -0.32)]):
        m.ball("vein", 0.07, loc=(x, y, 0.05), scale=(1.3, 1, 0.8), subdiv=1, jitter=0.15)
    for i in range(8):
        a = TAU * i / 8 + 0.3
        d = 0.18 + 0.12 * ((i * 0.618) % 1)
        m.ball("vein", 0.04, loc=(math.cos(a) * d, -0.1 + math.sin(a) * d, 0.025),
               scale=(1.4, 1.0, 0.25), rot=(0, 0, a), subdiv=0)


def wood_bench(m, p):
    th = p["top_h"]
    for y in (-0.2, 0, 0.2):
        m.box("wood", (1.7, 0.19, 0.1), loc=(0, y, th), jitter=0.012)
    for x in (-0.7, 0.7):
        for y in (-0.22, 0.22):
            m.rod("wood_dark", (x * 1.08, y * 1.25, 0), (x, y, th - 0.05), 0.045, segs=5)
    log(m, (-0.6, 0.05, th + 0.17), (0.25, 0.0, th + 0.17), r=0.12, segs=7)
    # stone axe resting on the bench
    m.rod("wood", (0.35, -0.15, th + 0.06), (0.75, 0.1, th + 0.06), 0.022, segs=5)
    m.box("stone", (0.08, 0.16, 0.05), loc=(0.73, 0.06, th + 0.07), rot=(0, 0, 0.55), taper=0.6)
    for i in range(6):  # chips
        a = TAU * i / 6
        m.box("wood", (0.08, 0.04, 0.015), loc=(0.2 + 0.4 * math.cos(a), -0.15 + 0.2 * math.sin(a),
              0.01), rot=(0, 0, a))
    m.cyl("trunk", 0.2, 0.42, loc=(-0.72, -0.55 + 0.12, 0), segs=7, cap_mat="wood")


def drying_rack(m, p):
    bh = p["bar_h"]
    for x in (-0.85, 0.85):
        m.rod("wood", (x, -0.38, 0), (x, 0.0, bh + 0.1), 0.045, segs=5)
        m.rod("wood", (x, 0.38, 0), (x, 0.0, bh + 0.1), 0.045, segs=5)
    m.rod("wood", (-0.95, 0, bh), (0.95, 0, bh), 0.04, segs=6)
    m.rod("wood", (-0.88, -0.15, bh * 0.55), (0.88, -0.15, bh * 0.55), 0.03, segs=5)
    for i, x in enumerate((-0.45, 0.25)):  # hides hanging over the bar
        m.box("hide", (0.55, 0.035, 0.75), loc=(x, -0.02, bh - 0.38), rot=(0.05, 0, 0.03 * i),
              taper=0.85, jitter=0.02)
        m.box("hide", (0.55, 0.035, 0.4), loc=(x, 0.04, bh - 0.2), rot=(-0.12, 0, 0), jitter=0.02)
    for i in range(4):  # meat / fish strips on the lower bar
        x = 0.4 + 0.12 * i
        m.box("leather", (0.05, 0.03, 0.32), loc=(x, -0.15, bh * 0.55 - 0.16), rot=(0, 0.05 * i, 0))
    m.rod("leather", (-0.65, -0.15, bh * 0.55), (-0.65, -0.15, bh * 0.55 - 0.3), 0.03, segs=4)


def grinding_stone(m, p):
    m.box("hide", (0.85, 0.7, 0.02), loc=(0, 0.02, 0.01), taper=0.9)
    m.ball("stone", 0.32, loc=(0, 0.0, 0.08), scale=(1.25, 0.75, 0.42), subdiv=2, jitter=0.06,
           ground=True)
    m.ball("stone", 0.13, loc=(0.05, -0.02, 0.22), scale=(1.3, 0.7, 0.55), subdiv=1, jitter=0.08)
    basket(m, (0.28, -0.3, 0.0), h=0.16, r=0.13)
    m.ball("thatch", 0.11, loc=(0.28, -0.3, 0.15), scale=(1, 1, 0.35), subdiv=1)
    m.ball("thatch", 0.09, loc=(-0.28, 0.0, 0.17), scale=(1.4, 1, 0.3), subdiv=1)  # flour heap
    m.ball("stone", 0.06, loc=(-0.3, -0.3, 0.03), scale=(1.3, 1, 0.8), subdiv=1, jitter=0.1)


def pottery_bench(m, p):
    m.box("wood", (1.0, 0.5, 0.07), loc=(-0.4, 0.05, 0.45))
    for x in (-0.82, 0.02):
        for y in (-0.15, 0.25):
            m.rod("wood_dark", (x, y, 0), (x, y, 0.42), 0.035, segs=5)
    for i, (x, h, r) in enumerate([(-0.7, 0.32, 0.13), (-0.38, 0.24, 0.11), (-0.12, 0.2, 0.09)]):
        pot(m, (x, 0.05, 0.485), h=h, r=r)
    # wheel: stone pivot, spindle, wooden turntable, wet clay lump
    m.ball("stone", 0.14, loc=(0.55, -0.05, 0.05), scale=(1, 1, 0.6), subdiv=1, ground=True)
    m.rod("wood_dark", (0.55, -0.05, 0.05), (0.55, -0.05, 0.36), 0.03, segs=5)
    m.cyl("wood", 0.3, 0.06, loc=(0.55, -0.05, 0.34), segs=12)
    m.lathe("clay", [(0.12, 0), (0.13, 0.08), (0.09, 0.16), (0.1, 0.19)], loc=(0.55, -0.05, 0.4),
            segs=8, top_mat="earth")
    m.box("stone", (0.25, 0.25, 0.25), loc=(0.85, 0.3, 0.12), rot=(0, 0, 0.3), jitter=0.03)
    m.ball("clay", 0.16, loc=(0.85, 0.3, 0.27), scale=(1, 1, 0.6), subdiv=1, jitter=0.1)


def loom(m, p):
    tilt = 0.22
    top = 1.9
    for x in (-0.8, 0.8):
        m.rod("wood", (x, -0.15, 0), (x, -0.15 + math.sin(tilt) * top, top), 0.05, segs=6)
        m.rod("wood_dark", (x, 0.45, 0), (x, -0.1 + math.sin(tilt) * top, top - 0.15), 0.035,
              segs=5)
    yt = -0.15 + math.sin(tilt) * (top - 0.08)
    m.rod("wood", (-0.92, yt, top - 0.08), (0.92, yt, top - 0.08), 0.05, segs=7)
    # woven cloth hanging from the cloth beam, warp below it
    ch = 0.75
    zc = top - 0.12 - ch / 2
    yc = -0.15 + math.sin(tilt) * zc - 0.02
    m.box("cloth", (1.4, 0.025, ch), loc=(0, yc, zc), rot=(-tilt, 0, 0))
    m.rod("wood_dark", (-0.85, yc - 0.12, zc - 0.45), (0.85, yc - 0.12, zc - 0.45), 0.025, segs=5)
    for i in range(7):  # warp bundles down to the weights
        x = -0.6 + 0.2 * i
        z0 = zc - ch / 2
        m.box("cloth", (0.05, 0.01, z0 - 0.35), loc=(x, -0.15 + math.sin(tilt) * (z0 / 2 + 0.17),
              (z0 + 0.35) / 2), rot=(-tilt, 0, 0))
        m.ball("clay", 0.065, loc=(x, -0.15 + math.sin(tilt) * 0.33, 0.3), scale=(1, 0.7, 1.1),
               subdiv=1)


def retting_pit(m, p):
    n = 10
    for i in range(n):
        a = TAU * i / n
        m.ball("earth", 0.32, loc=(math.cos(a) * 0.82, math.sin(a) * 0.72, 0.0),
               scale=(1.2, 0.9, 0.5), rot=(0, 0, a + math.pi / 2), subdiv=1, jitter=0.1,
               ground=True)
    m.cyl("metal", 0.8, 0.04, loc=(0, 0, 0.0), segs=12, scale=(1, 0.88, 1))
    for i in range(5):  # flax bundles soaking, tied in the middle
        y = -0.4 + 0.2 * i
        a = 0.1 * ((i * 0.7) % 1 - 0.5)
        x0, x1 = -0.5 + 0.06 * (i % 2), 0.45 + 0.06 * (i % 3)
        m.rod("thatch", (x0, y + a, 0.06), (x1, y - a, 0.06), 0.06, segs=6)
        m.rod("leather", ((x0 + x1) / 2 - 0.02, y, 0.06), ((x0 + x1) / 2 + 0.02, y, 0.06), 0.068,
              segs=6)
    for x, y in ((-0.2, -0.1), (0.25, 0.2)):
        m.ball("stone", 0.11, loc=(x, y, 0.13), scale=(1.2, 1, 0.7), subdiv=1, jitter=0.1)


def clay_oven(m, p):
    m.cyl("stone", 0.45, 0.12, segs=9, jitter=0.01)
    m.ball("clay", 0.4, loc=(0, 0.03, 0.12), scale=(1, 1.05, 0.95), subdiv=2, jitter=0.03,
           ground=True)
    m.box("wood_dark", (0.24, 0.12, 0.22), loc=(0, -0.36, 0.23), taper=0.7)  # dark mouth
    m.box("glow", (0.14, 0.04, 0.08), loc=(0, -0.39, 0.17), taper=0.8)  # embers inside
    m.box("clay", (0.34, 0.1, 0.06), loc=(0, -0.38, 0.36), taper=0.9)  # lintel
    m.cyl("clay", 0.065, 0.12, loc=(0, 0.1, 0.47), segs=6, r2=0.055, cap_mat="wood_dark")
    for i in range(3):
        log(m, (0.32, -0.25 + 0.1 * i, 0.04 + (0.07 if i == 1 else 0)),
            (0.45, 0.15 + 0.1 * i, 0.04 + (0.07 if i == 1 else 0)), r=0.04, segs=5)


def charcoal_clamp(m, p):
    """Share the authored furnace form while retaining the legacy ID and sockets."""
    import stations_w3d as W
    W.charcoal_clamp(m, "working")


def smelter(m, p):
    """Share the authored furnace form while retaining the legacy ID and sockets."""
    import stations_w3d as W
    W.smelter(m, True)


def casting_bench(m, p):
    for x in (-0.6, 0.15):
        m.box("stone", (0.3, 0.45, 0.4), loc=(x, 0.05, 0.2), jitter=0.02)
    m.box("wood", (1.15, 0.55, 0.08), loc=(-0.22, 0.05, 0.44))
    for i in range(3):  # stone moulds with cast bars in them
        x = -0.6 + 0.33 * i
        m.box("stone", (0.26, 0.16, 0.07), loc=(x, 0.02, 0.515))
        m.box("metal", (0.16, 0.06, 0.03), loc=(x, 0.02, 0.555))
    # crucible sitting in a little ember bed at the right
    m.cyl("stone", 0.22, 0.08, loc=(0.65, -0.05, 0), segs=8)
    m.ball("glow", 0.16, loc=(0.65, -0.05, 0.08), scale=(1, 1, 0.35), subdiv=1)
    m.lathe("clay", [(0.07, 0), (0.11, 0.1), (0.1, 0.18)], loc=(0.65, -0.05, 0.1), segs=8,
            top_mat="glow")
    m.rod("wood_dark", (0.4, -0.35, 0.02), (0.75, 0.05, 0.3), 0.02, segs=4)  # tongs
    m.rod("wood_dark", (0.45, -0.38, 0.02), (0.78, 0.02, 0.26), 0.02, segs=4)


def anvil(m, p):
    m.cyl("trunk", 0.24, 0.42, segs=8, r2=0.22, cap_mat="wood", jitter=0.008)
    for i in range(4):
        a = TAU * i / 4 + 0.6
        m.cyl("trunk", 0.1, 0.22, loc=(math.cos(a) * 0.17, math.sin(a) * 0.17, 0.06),
              rot=(1.9, 0, a + math.pi / 2), segs=5, r2=0.055)
    m.ball("stone", 0.2, loc=(0, 0, 0.5), scale=(1.25, 0.85, 0.55), subdiv=2, jitter=0.05)
    m.ball("stone", 0.06, loc=(0.18, -0.25, 0.04), scale=(1.3, 1, 0.9), subdiv=1, jitter=0.1)
    m.box("metal", (0.12, 0.05, 0.02), loc=(-0.04, -0.03, 0.61), rot=(0, 0, 0.3))


def kiln(m, p):
    """Share the authored furnace form while retaining the legacy ID and sockets."""
    import stations_w3d as W
    W.kiln(m, True)


def storage_shelf(m, p):
    for x in (-0.85, 0.85):
        for y in (-0.22, 0.22):
            m.rod("wood_dark", (x, y, 0), (x, y, 1.65), 0.04, segs=5)
    items = [[("pot", -0.55, 0.3), ("basket", 0.0, 0.2), ("pot", 0.5, 0.26)],
             [("basket", -0.5, 0.22), ("pot", 0.1, 0.3), ("pot", 0.5, 0.2)],
             [("pot", -0.45, 0.22), ("basket", 0.35, 0.18)]]
    for z, row in zip(p["shelves"], items):
        m.box("wood", (1.8, 0.52, 0.05), loc=(0, 0, z))
        for kind, x, h in row:
            if kind == "pot":
                pot(m, (x, 0.0, z + 0.025), h=h, r=h * 0.5, segs=7)
            else:
                basket(m, (x, 0.0, z + 0.025), h=h, r=h * 0.85, segs=7)


def bedroll(m, p):
    m.box("hide", (1.75, 0.8, 0.04), loc=(0.05, 0, 0.02), taper=0.92, jitter=0.015)
    m.ball("fur", 0.5, loc=(0.25, 0.0, 0.04), scale=(1.6, 0.78, 0.16), subdiv=2, jitter=0.05)
    m.cyl("hide", 0.13, 0.82, loc=(-0.72, 0.41, 0.13), rot=(math.pi / 2, 0, 0), segs=10,
          cap_mat="leather")
    for y in (-0.22, 0.22):
        m.cyl("leather", 0.14, 0.05, loc=(-0.72, y + 0.025, 0.13), rot=(math.pi / 2, 0, 0), segs=10)


def door(m, p):
    w, h, t = p["width"], p["height"], p["thick"]
    n = 4
    pw = w / n
    for i in range(n):  # planks; hinge (pivot) on the LEFT edge at x = 0
        top = h - 0.03 * ((i * 7) % 3)
        m.box("wood", (pw - 0.006, t * 0.75, top), loc=(pw * (i + 0.5), 0, top / 2))
    for z in (0.35, h - 0.35):  # ledges on the front face
        m.box("wood_dark", (w - 0.08, t * 0.4, 0.13), loc=(w / 2, -t * 0.55, z))
    d = math.hypot(w - 0.2, h - 0.7)
    m.box("wood_dark", (0.11, t * 0.4, d), loc=(w / 2, -t * 0.55, h / 2),
          rot=(0, math.atan2(w - 0.2, h - 0.7), 0))
    for z in (0.35, h - 0.35):  # leather hinge straps + pegs
        m.box("leather", (0.22, t * 0.3, 0.07), loc=(0.1, -t * 0.8, z))
        for x in (0.25, 0.45, 0.65, 0.8):
            m.box("wood_dark", (0.03, 0.03, 0.03), loc=(x, -t * 0.8, z))
    m.rod("leather", (w - 0.12, -t * 0.6, 1.05), (w - 0.12, -t * 0.6 - 0.06, 0.9), 0.015, segs=4)


def well_body(m, p):
    for course, (z, off) in enumerate(((0.16, 0.0), (0.46, 0.5))):
        n = 11
        for i in range(n):
            a = TAU * (i + off) / n
            m.box("stone", (0.5, 0.3, 0.3), loc=(math.cos(a) * 0.9, math.sin(a) * 0.9, z),
                  rot=(0, 0, a + math.pi / 2), jitter=0.025)
    m.cyl("metal", 0.75, 0.02, loc=(0, 0, 0.25), segs=11)
    for x in (-1.1, 1.1):
        m.rod("wood", (x, 0, 0), (x, 0, 1.95), 0.07, segs=6)
    m.rod("wood_dark", (-1.2, 0, 1.6), (1.2, 0, 1.6), 0.07, segs=7)  # windlass
    m.rod("wood_dark", (1.2, 0, 1.6), (1.2, -0.3, 1.6), 0.03, segs=4)  # crank
    m.rod("leather", (0, 0, 1.55), (0, 0, 0.95), 0.012, segs=4)  # rope
    m.lathe("wood", [(0.12, 0), (0.15, 0.22)], loc=(0, 0, 0.75), segs=8, top_mat="metal")
    H.thatch(m, (2.7, 1.55, .5), 1.91)


def bee_skep(m, p):
    m.cyl("stone", 0.32, 0.28, segs=8, r2=0.28, jitter=0.02)
    m.lathe("thatch", [(0.27, 0.0), (0.28, 0.12), (0.26, 0.28), (0.2, 0.44), (0.1, 0.55),
                       (0.0, 0.58)], loc=(0, 0, 0.28), segs=12)
    for i, (r, z) in enumerate([(0.29, 0.06), (0.285, 0.2), (0.245, 0.34), (0.175, 0.47)]):
        m.cyl("thatch", r, 0.035, loc=(0, 0, 0.28 + z), segs=12)
    m.box("wood_dark", (0.1, 0.06, 0.05), loc=(0, -0.27, 0.305))
    m.cyl("thatch", 0.14, 0.18, loc=(0, 0, 0.8), segs=8, r2=0.0)  # straw hackle


def build(entries):
    for sid, p in STATIONS.items():
        m = C.Model(sid)
        m.colors.update(p.get("colors", {}))
        fn = globals()["hearth" if sid.startswith("hearth") else sid]
        fn(m, p)
        D.detail(m, "hearth" if sid == "hearth_unlit" else sid)
        H.finish(m)
        w, h = p["fp"]
        if sid != "door":
            H.fit(m, (w, h))
        if "light" in p:
            m.sockets["light"] = p["light"]
        if sid != "door":
            m.sockets["work"] = p.get("work", (0, -(h / 2 + 0.35), 0))
        extra = {"family": "station"}
        if sid == "door":
            extra["pivot"] = "left edge (x=0); leaf spans +X to width, front faces +Z"
        C.save_static(m, (w, h), entries, extra)
    C.check_budget({k: e for k, e in entries.items() if e.get("family") == "station"}, 200, 1200)
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}))
