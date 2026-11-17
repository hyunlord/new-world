"""W3D metalworking/craft stations: recognisable replacements for the M3D-ASSETS primitives.

    blender -b --factory-startup -P tools/3d/blender/stations_w3d.py                 # build
    blender -b --factory-startup -P tools/3d/blender/stations_w3d.py -- contact [DIR]  # review sheet

Writes assets/3d/models/w3d_<station>_<state>.glb and the fragment w3d_stations_manifest.json
(never manifest.json). Same conventions as stations.py: metres, floor origin at the footprint
centre, FRONT = Blender -Y (Godot +Z), flat shading, palette material names only. Lit parts use
`glow` (props3d.gd swaps it on/off); idle states have no glow. Smoke puffs use `snow` with a grey
default colour because the material API has no smoke name.
"""
import json
import math
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import stations as S  # noqa: E402
import look2_props_details as D
import look2_props_shapes as H

TAU = 2 * math.pi
FRAGMENT = os.path.join(C.MODELS_DIR, "w3d_stations_manifest.json")
SMOKE = (0.70, 0.70, 0.72)
MAX_TRIS = 6500

# station -> footprint, states, light socket (Blender coords, working state only)
W3D = {
    "smelter": dict(fp=(2, 2), states=("idle", "working"), light=(0, -0.55, 0.3)),
    "bloomery": dict(fp=(2, 2), states=("idle", "working"), light=(0, -0.5, 0.35)),
    "forge": dict(fp=(2, 2), states=("idle", "working"), light=(-0.35, 0.4, 0.9)),
    "charcoal_clamp": dict(fp=(3, 3), states=("idle", "working", "done"), light=(0, 0, 1.0)),
    "casting_bench": dict(fp=(2, 1), states=("idle", "working"), light=(0.6, -0.05, 0.4)),
    "kiln": dict(fp=(2, 2), states=("idle", "working"), light=(0, -0.62, 0.3)),
}
# the M3D-ASSETS model each state replaces (contact sheet only)
REPLACES = {"smelter": ("asset_smelter_unlit", "asset_smelter_lit"),
            "bloomery": ("asset_bloomery_unlit", "asset_bloomery_lit"),
            "forge": ("asset_forge_idle", "asset_forge_working"),
            "charcoal_clamp": ("asset_charcoal_clamp_idle", "asset_charcoal_clamp_burning",
                               "asset_charcoal_clamp_spent"),
            "casting_bench": ("asset_casting_bench_empty", "asset_casting_bench_working"),
            "kiln": ("asset_kiln_unlit", "asset_kiln_lit")}


# --------------------------------------------------------------------------- shared bits

class placed:
    """Parts added inside the block are scaled by `k`, turned by `yaw` about Z and moved to (x, y, z)."""

    def __init__(self, m, x, y, yaw=0.0, z=0.0, k=1.0):
        self.m, self.off, self.yaw, self.k = m, (x, y, z), yaw, k

    # an int layer marks the old verts (a set of BMVert wrappers loses members as the bmesh grows)
    def __enter__(self):
        self.lay = self.m.bm.verts.layers.int.new("placed_old")
        for v in self.m.bm.verts:
            v[self.lay] = 1

    def __exit__(self, *exc):
        c, s = math.cos(self.yaw), math.sin(self.yaw)
        for v in self.m.bm.verts:
            if not v[self.lay]:
                x, y = v.co.x * self.k, v.co.y * self.k
                v.co.x, v.co.y = x * c - y * s + self.off[0], x * s + y * c + self.off[1]
                v.co.z = v.co.z * self.k + self.off[2]
        self.m.bm.verts.layers.int.remove(self.lay)


def hot(m, working, idle_mat="wood_dark"):
    """Material for embers/fire: glow while working, cold ash/charcoal otherwise."""
    return "glow" if working else idle_mat


def smoke(m, x, y, z, n=3, r=0.14):
    """Separate puffs drifting downwind (+X) and growing as they rise."""
    m.colors["snow"] = SMOKE
    zi = z
    for i in range(n):
        ri = r * (1 + 0.4 * i)
        zi += ri * 1.25
        m.ball("snow", ri, loc=(x + 0.1 * i * i, y + 0.04 * i, zi), scale=(1.15, 1.15, 0.8), subdiv=2,
               jitter=0.06)
        zi += ri * 0.6


def flames(m, x, y, z, s=1.0):
    for i, (dx, dy, h) in enumerate([(0, 0, 0.42), (0.08, 0.05, 0.3), (-0.07, -0.04, 0.26)]):
        m.cyl("glow", (0.09 if i == 0 else 0.06) * s, h * s, loc=(x + dx * s, y + dy * s, z), segs=5,
              r2=0.0, rot=(0, 0, i))


def bellows(m, x, y, yaw, nozzle=0.45, lift=0.0, k=1.0):
    """Wedge (accordion) bellows: boards + leather body + handle; nozzle points along local +X.
    `lift` raises the whole thing (onto a stand), `k` scales it."""
    with placed(m, x, y, yaw, lift, k):
        m.box("wood_dark", (0.62, 0.28, 0.04), loc=(0, 0, 0.14))
        m.ball("hide", 0.27, loc=(-0.06, 0, 0.25), scale=(1.1, 0.95, 0.45), rot=(0, 0.12, 0), subdiv=1)
        m.box("wood_dark", (0.56, 0.22, 0.04), loc=(0, 0, 0.37), rot=(0, 0.24, 0), taper=0.8)
        m.rod("wood_dark", (-0.28, 0, 0.42), (-0.48, 0, 0.62), 0.025, segs=4)  # handle
        m.rod("wood_dark", (0.28, 0, 0.18), (0.28 + nozzle, 0, 0.18), 0.035, segs=5)  # nozzle
        for dx in (-0.22, 0.22):  # stone props under the bottom board
            m.box("stone", (0.12, 0.3, 0.12), loc=(dx, 0, 0.06), jitter=0.01)


def charcoal_pile(m, x, y, r=0.25, n=7):
    for i in range(n):
        a = TAU * i / n
        d = r * 0.55 * (i % 2)
        m.ball("wood_dark", r * 0.45, loc=(x + math.cos(a) * d, y + math.sin(a) * d, r * 0.12 * (2 - i % 2)),
               scale=(1.2, 1, 0.8), rot=(0, 0, a), subdiv=0, jitter=0.2)


def tongs(m, p_bottom, p_top, w=0.05):
    """Long smithing tongs leaning: two crossed iron rods with a hinge near the jaws."""
    x0, y0, z0 = p_bottom
    x1, y1, z1 = p_top
    m.rod("metal", (x0 - w, y0, z0), (x1 + w, y1, z1), 0.016, segs=4)
    m.rod("metal", (x0 + w, y0, z0), (x1 - w, y1, z1), 0.016, segs=4)
    t = 0.18
    m.ball("metal", 0.035, loc=(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, z0 + (z1 - z0) * t), subdiv=0)


def shaft_furnace(m, h, r0, r1, working, front_y, colors=None, lip=True):
    """Clay shaft on a stone footing, tapping arch at the front (-Y), throat at the top."""
    S.stone_ring(m, 9, r0 + 0.06, 0.15, scale=(1.3, 1.0, 0.75))
    m.lathe("clay", [(r0, 0.0), (r0 * 0.98, 0.35), (r0 * 0.78, h * 0.6), (r1, h - 0.08), (r1 * 1.08, h)],
            segs=10, top_mat=hot(m, working))
    if lip:
        m.cyl("clay", r1 * 1.15, 0.08, loc=(0, 0, h - 0.06), segs=10, r2=r1 * 1.1, caps=False)
    m.box(hot(m, working), (0.28, 0.12, 0.28), loc=(0, front_y + 0.02, 0.16), taper=0.7)  # tapping arch
    m.box("clay", (0.46, 0.14, 0.08), loc=(0, front_y, 0.34))
    for x in (-0.19, 0.19):
        m.box("clay", (0.09, 0.14, 0.34), loc=(x, front_y, 0.17))
    D.furnace_courses(m, (h, r0, r1))


# --------------------------------------------------------------------------- builders

def smelter(m, working):
    m.colors["clay"] = (0.74, 0.46, 0.28)
    shaft_furnace(m, 1.3, 0.5, 0.3, working, front_y=-0.49)
    # tuyere into the right side, double bellows behind it pointing at the furnace (-X)
    m.rod("clay", (0.66, 0.0, 0.26), (0.4, 0.0, 0.3), 0.06, segs=6)
    for y in (-0.22, 0.22):
        bellows(m, 0.66, y, math.pi + (0.3 if y > 0 else -0.3), nozzle=0.08, k=0.7)
    # slag pit in front of the arch: dark rim, run of slag (glowing while tapping)
    for i in range(7):
        a = TAU * i / 7
        m.ball("earth", 0.1, loc=(0.0 + math.cos(a) * 0.24, -0.74 + math.sin(a) * 0.12, 0.0),
               scale=(1.4, 1, 0.5), rot=(0, 0, a), subdiv=0, ground=True)
    m.cyl(hot(m, working, "stone"), 0.22, 0.03, loc=(0, -0.74, 0), segs=8, scale=(1, 0.55, 1))
    m.rod(hot(m, working, "stone"), (0, -0.56, 0.06), (0, -0.7, 0.03), 0.045, segs=5)
    m.colors["stone"] = (0.40, 0.38, 0.37)
    for i in range(4):  # spent slag cakes
        m.ball("stone", 0.09, loc=(-0.72 + 0.1 * (i % 2), -0.62 + 0.15 * i, 0.04), scale=(1.3, 1, 0.6),
               subdiv=0, jitter=0.2)
    S.basket(m, (-0.62, 0.55, 0.0), h=0.24, r=0.17)  # ore
    m.ball("vein", 0.13, loc=(-0.62, 0.55, 0.22), scale=(1, 1, 0.45), subdiv=1, jitter=0.2)
    charcoal_pile(m, -0.3, 0.75, r=0.22)
    if working:
        flames(m, 0, 0, 1.3, s=0.8)
        smoke(m, 0.04, 0.03, 1.65)


def bloomery(m, working):
    m.colors["clay"] = (0.50, 0.30, 0.20)
    shaft_furnace(m, 2.05, 0.46, 0.24, working, front_y=-0.45)
    # tuyere on the left, double bellows on a low stand pointing +X
    m.rod("clay", (-0.66, 0.08, 0.28), (-0.38, 0.05, 0.3), 0.065, segs=6)
    for y in (-0.15, 0.32):
        bellows(m, -0.6, y, -0.3 if y > 0 else 0.25, nozzle=0.1, k=0.75)
    # bloom tongs leaning on the shaft, front right
    tongs(m, (0.62, -0.52, 0.0), (0.3, -0.26, 1.35))
    # stone anvil block with the bloom on it
    m.box("stone", (0.36, 0.3, 0.26), loc=(0.6, -0.68, 0.13), jitter=0.02, taper=0.85)
    m.ball(hot(m, working, "earth"), 0.13, loc=(0.6, -0.68, 0.32), scale=(1.2, 1, 0.75), subdiv=1, jitter=0.15)
    m.rod("wood", (0.82, -0.52, 0.02), (0.9, -0.2, 0.42), 0.022, segs=4)  # hammer handle
    m.box("stone", (0.14, 0.09, 0.09), loc=(0.9, -0.2, 0.44), rot=(0, 0, 0.2))
    m.ball("stone", 0.09, loc=(-0.25, -0.72, 0.03), scale=(1.4, 1, 0.5), subdiv=0, jitter=0.2)  # slag
    m.ball("stone", 0.07, loc=(-0.05, -0.8, 0.02), scale=(1.4, 1, 0.5), subdiv=0, jitter=0.2)
    charcoal_pile(m, 0.55, 0.6, r=0.3, n=8)
    S.basket(m, (-0.55, 0.72, 0.0), h=0.22, r=0.16)
    m.ball("vein", 0.12, loc=(-0.55, 0.72, 0.2), scale=(1, 1, 0.45), subdiv=1, jitter=0.2)
    if working:
        flames(m, 0, 0, 2.05, s=0.75)
        smoke(m, 0.04, 0.02, 2.45)


def forge(m, working):
    # raised stone hearth against the back, fire bed on top
    hx, hy = -0.35, 0.42
    m.box("stone", (1.05, 0.78, 0.62), loc=(hx, hy, 0.31), jitter=0.015)
    m.box("stone", (1.1, 0.83, 0.08), loc=(hx, hy, 0.66))
    m.box("stone", (1.05, 0.16, 0.5), loc=(hx, hy + 0.36, 0.95))  # fire-back wall
    m.ball(hot(m, working), 0.24, loc=(hx, hy - 0.02, 0.69), scale=(1.4, 1, 0.35), subdiv=1, jitter=0.1)
    if working:
        flames(m, hx, hy, 0.72, s=0.7)
        m.box("glow", (0.34, 0.05, 0.04), loc=(0.42, -0.36, 0.69), rot=(0, 0, 0.25))  # bar on the anvil
    # hood (pyramid) and chimney over the fire, on two corner posts
    m.box("clay", (1.0, 0.78, 0.5), loc=(hx, hy + 0.04, 1.45), taper=0.38)
    m.box("clay", (0.34, 0.34, 0.95), loc=(hx, hy + 0.08, 2.1), jitter=0.01)
    m.box("stone", (0.42, 0.42, 0.08), loc=(hx, hy + 0.08, 2.58))
    for x in (hx - 0.46, hx + 0.46):
        m.rod("wood_dark", (x, hy - 0.32, 0.7), (x, hy - 0.3, 1.22), 0.04, segs=5)
    # bellows on the right, nozzle into the hearth side
    bellows(m, 0.6, 0.55, math.pi, nozzle=0.25, lift=0.38, k=0.75)
    m.box("wood_dark", (0.42, 0.26, 0.38), loc=(0.6, 0.55, 0.19))  # stand
    # anvil on its stump, front right (M3D-1 anvil reused)
    with placed(m, 0.42, -0.36, 0.2):
        S.anvil(m, {})
    m.box("metal", (0.34, 0.14, 0.12), loc=(0.42, -0.36, 0.58), rot=(0, 0, 0.2))  # iron anvil face
    m.cyl("metal", 0.07, 0.16, loc=(0.6, -0.33, 0.58), rot=(0, math.pi / 2, 0.2), segs=6, r2=0.0)  # horn
    m.rod("wood", (0.15, -0.6, 0.02), (0.25, -0.62, 0.55), 0.02, segs=4)  # hammer leaning on stump
    m.box("metal", (0.06, 0.06, 0.14), loc=(0.25, -0.62, 0.58), rot=(0, 1.4, 0))
    tongs(m, (0.85, -0.05, 0.0), (0.73, -0.25, 0.62))
    # quench trough, front left
    m.box("wood", (0.78, 0.36, 0.3), loc=(-0.52, -0.62, 0.15))
    m.box("metal", (0.66, 0.24, 0.02), loc=(-0.52, -0.62, 0.3))
    m.colors["metal"] = (0.42, 0.44, 0.46)


def charcoal_clamp(m, state):
    R = 1.25
    if state == "idle":  # stacked wood cone around a centre pole, turf waiting beside it
        m.lathe("trunk", [(1.1, 0.0), (1.0, 0.45), (0.72, 0.85), (0.32, 1.05), (0.1, 1.1)], segs=12)
        m.rod("wood", (0, 0, 0.9), (0, 0, 1.45), 0.05, segs=5)  # centre (chimney) pole
        tiers = [(26, 1.2, 0.0, 1.02, 0.6), (18, 0.95, 0.45, 0.7, 0.95), (10, 0.62, 0.85, 0.32, 1.13)]
        for t, (n, r0, z0, r1, z1) in enumerate(tiers):
            for i in range(n):
                a = TAU * (i + 0.5 * t) / n
                S.log(m, (math.cos(a) * r0, math.sin(a) * r0, z0),
                      (math.cos(a) * r1, math.sin(a) * r1, z1 + 0.04 * (i % 2)), r=0.075, segs=5)
        for i in range(5):  # turf sods ready to cover the stack
            m.box("canopy", (0.3, 0.22, 0.08), loc=(1.25, -1.15 + 0.12 * i, 0.04 + 0.08 * (i % 3)),
                  rot=(0, 0, 0.3 * i), jitter=0.02)
        m.ball("earth", 0.3, loc=(-1.15, -1.1, 0), scale=(1, 1, 0.5), subdiv=1, jitter=0.1, ground=True)
        m.rod("wood", (-1.3, -0.9, 0.0), (-1.0, -1.3, 0.9), 0.025, segs=4)  # shovel
        m.box("wood_dark", (0.16, 0.04, 0.2), loc=(-1.3, -0.88, 0.08), rot=(0.3, 0, 0.9))
    elif state == "working":  # turf-covered smoking mound
        m.lathe("earth", [(R, 0.0), (R * 0.97, 0.25), (R * 0.82, 0.6), (R * 0.5, 0.92), (0.15, 1.05)],
                segs=14, top_mat="glow")
        for i in range(10):  # turf patches
            a = TAU * i / 10 + 0.2
            d = 0.55 + 0.35 * (i % 2)
            z = 1.05 * (1 - (d / R) ** 2) ** 0.5 - 0.06
            m.box("canopy", (0.32, 0.24, 0.07), loc=(math.cos(a) * d, math.sin(a) * d, z),
                  rot=(0.35 * math.sin(a), -0.35 * math.cos(a), a), jitter=0.02)
        for i in range(5):  # vent holes, glowing, with their own smoke
            a = TAU * i / 5 + 0.8
            d = 0.95
            x, y = math.cos(a) * d, math.sin(a) * d
            z = 0.42
            m.ball("glow", 0.09, loc=(x * 1.02, y * 1.02, z), scale=(1, 1, 0.7), subdiv=0)
        smoke(m, 0.0, 0.0, 1.0, n=4, r=0.16)
        for i in range(5):  # spare logs leaning on the mound
            a = TAU * i / 5 + 0.5
            S.log(m, (math.cos(a) * 1.36, math.sin(a) * 1.36, 0.0),
                  (math.cos(a + 0.1) * 1.2, math.sin(a + 0.1) * 1.2, 0.4), r=0.05, segs=5)
        m.rod("wood", (-1.08, -1.2, 0.02), (-0.5, -0.95, 0.9), 0.025, segs=4)  # rake
        m.box("wood_dark", (0.4, 0.06, 0.05), loc=(-1.1, -1.22, 0.04), rot=(0, 0, -0.35))
    else:  # opened: black charcoal heap, burnt crust raked to a ring, charcoal in baskets
        m.colors["earth"] = (0.24, 0.18, 0.13)
        m.colors["wood_dark"] = (0.06, 0.055, 0.05)
        m.lathe("wood_dark", [(R * 0.9, 0.0), (R * 0.75, 0.15), (R * 0.4, 0.32), (0.1, 0.38)], segs=12)
        for i in range(26):
            a = TAU * i * 0.381966
            d = 0.15 + 0.85 * ((i * 0.618) % 1)
            z = 0.38 * (1 - (d / (R * 0.95)) ** 2) + 0.02
            m.box("wood_dark", (0.2, 0.08, 0.08), loc=(math.cos(a) * d, math.sin(a) * d, max(z, 0.04)),
                  rot=(0.3 * (i % 3), 0.2, a), jitter=0.015)
        for i in range(12):  # raked crust
            a = TAU * i / 12
            m.ball("earth", 0.22, loc=(math.cos(a) * 1.24, math.sin(a) * 1.24, 0.0), scale=(1.4, 0.8, 0.5),
                   rot=(0, 0, a + math.pi / 2), subdiv=0, jitter=0.15, ground=True)
        for x, y in ((0.92, -1.18), (1.22, -0.92)):
            S.basket(m, (x, y, 0.0), h=0.26, r=0.17)
            m.ball("wood_dark", 0.15, loc=(x, y, 0.25), scale=(1, 1, 0.45), subdiv=1, jitter=0.2)
        m.rod("wood", (-1.08, -1.2, 0.02), (-0.4, -0.85, 0.85), 0.025, segs=4)  # rake
        m.box("wood_dark", (0.4, 0.06, 0.05), loc=(-1.1, -1.22, 0.04), rot=(0, 0, -0.4))


def casting_bench(m, working):
    # bench with three sand moulds (wooden frames, sand, cast-shape cavities)
    bx = -0.32
    m.box("wood", (1.25, 0.6, 0.08), loc=(bx, 0.05, 0.62), jitter=0.008)
    for x in (bx - 0.52, bx + 0.52):
        for y in (-0.18, 0.28):
            m.rod("wood_dark", (x, y, 0), (x, y, 0.58), 0.04, segs=5)
    m.rod("wood_dark", (bx - 0.52, 0.05, 0.18), (bx + 0.52, 0.05, 0.18), 0.03, segs=4)
    m.colors["thatch"] = (0.76, 0.66, 0.46)  # moulding sand
    for i in range(3):
        x = bx - 0.4 + 0.4 * i
        m.box("wood_dark", (0.34, 0.4, 0.1), loc=(x, 0.03, 0.71))
        m.box("thatch", (0.28, 0.34, 0.02), loc=(x, 0.03, 0.765))
        cav = "glow" if working and i == 2 else ("metal" if i == 0 else "wood_dark")
        m.box(cav, (0.08, 0.2, 0.012), loc=(x, 0.03, 0.776), taper=1.0)  # axe-head cavity
        m.box(cav, (0.15, 0.06, 0.012), loc=(x, 0.15, 0.776))
    # ladle resting across the bench end
    m.rod("wood", (bx - 0.6, -0.2, 0.69), (bx + 0.05, -0.22, 0.69), 0.02, segs=4)
    m.lathe("metal", [(0.03, 0), (0.08, 0.04), (0.09, 0.07)], loc=(bx + 0.12, -0.22, 0.66), segs=8,
            top_mat=hot(m, working, "metal"))
    # crucible standing in a small charcoal hearth at the right end
    cx, cy = 0.6, -0.05
    for i in range(7):
        a = TAU * i / 7
        m.ball("stone", 0.09, loc=(cx + math.cos(a) * 0.22, cy + math.sin(a) * 0.22, 0.06),
               scale=(1.2, 1, 0.9), rot=(0, 0, a), subdiv=0, jitter=0.12, ground=True)
    m.ball(hot(m, working), 0.18, loc=(cx, cy, 0.04), scale=(1, 1, 0.4), subdiv=1, jitter=0.1)
    m.lathe("clay", [(0.09, 0), (0.14, 0.12), (0.15, 0.24), (0.12, 0.32)], loc=(cx, cy, 0.08), segs=8,
            top_mat=hot(m, working, "earth"))
    tongs(m, (0.25, -0.4, 0.02), (0.9, -0.3, 0.02))


def kiln(m, working):
    # updraft kiln: round clay chamber on a fire box, stoke hole at the front, pots stacked in the top
    m.cyl("earth", 0.86, 0.08, segs=12, r2=0.8)
    m.lathe("clay", [(0.72, 0.0), (0.7, 0.4), (0.64, 0.9), (0.56, 1.05), (0.58, 1.12)],
            loc=(0, 0.06, 0.06), segs=12, top_mat=hot(m, working, "earth"))
    for z in (0.35, 0.75):  # clay coil seams
        m.cyl("clay", 0.71 - 0.07 * (z > 0.5), 0.05, loc=(0, 0.06, z), segs=12, caps=False)
    m.colors["clay"] = (0.62, 0.36, 0.22)
    # pots stacked in the open top: a lower ring and one on top
    for i in range(5):
        a = TAU * i / 5 + 0.3
        S.pot(m, (math.cos(a) * 0.32, 0.06 + math.sin(a) * 0.32, 1.08), h=0.28, r=0.13, mat="horn",
              segs=7)
    S.pot(m, (0, 0.06, 1.08), h=0.42, r=0.18, mat="horn", segs=8)
    m.colors["horn"] = (0.80, 0.56, 0.38)  # unfired/biscuit pot colour, distinct from the kiln wall
    # stoke hole arch with fire bed (or ash) and fuel
    m.box(hot(m, working), (0.32, 0.14, 0.26), loc=(0, -0.66, 0.2), taper=0.75)
    m.box("clay", (0.56, 0.2, 0.1), loc=(0, -0.68, 0.39))
    for x in (-0.24, 0.24):
        m.box("clay", (0.12, 0.2, 0.36), loc=(x, -0.68, 0.18))
    if working:
        flames(m, 0, -0.74, 0.08, s=0.55)
        smoke(m, 0.05, 0.08, 1.55, n=3, r=0.15)
    for i in range(4):  # stoke wood
        S.log(m, (-0.85 + 0.07 * i, -0.62, 0.05 + 0.08 * (i % 2)),
              (-0.5 + 0.07 * i, -0.92, 0.05 + 0.08 * (i % 2)), r=0.045, segs=5)
    # finished pots set out to cool, front right
    S.pot(m, (0.66, -0.68, 0.0), h=0.32, r=0.14, segs=7)
    S.pot(m, (0.86, -0.42, 0.0), h=0.24, r=0.11, segs=7)


# --------------------------------------------------------------------------- build

def build(entries=None):
    """Build every w3d station state and write the fragment manifest (manifest.json untouched)."""
    entries = {} if entries is None else entries
    out = {}
    for name, p in W3D.items():
        for state in p["states"]:
            m = C.Model(f"w3d_{name}_{state}")
            fn = globals()[name]
            fn(m, state) if name == "charcoal_clamp" else fn(m, state == "working")
            H.finish(m)
            w, h = p["fp"]
            if state == "working":
                m.sockets["light"] = p["light"]
            m.sockets["work"] = (0, -(h / 2 + 0.35), 0)
            for v in m.bm.verts:  # nothing below the floor (piles of jittered lumps sink a little)
                v.co.z = max(v.co.z, 0.0)
            lo = [min(v.co[i] for v in m.bm.verts) for i in range(3)]
            hi = [max(v.co[i] for v in m.bm.verts) for i in range(3)]
            assert lo[0] >= -w / 2 and hi[0] <= w / 2 and lo[1] >= -h / 2 and hi[1] <= h / 2 and lo[2] >= -1e-4, \
                f"{m.id} leaves its {w}x{h} footprint: {lo} {hi}"
            e = C.save_static(m, (w, h), out, {"family": "station", "state": state})
            assert e["triangles"] <= MAX_TRIS, f"{m.id}: {e['triangles']} > {MAX_TRIS} triangles"
            assert ("glow" in e["materials"]) == (state == "working"), f"{m.id}: glow only while working"
    with open(FRAGMENT, "w") as f:
        json.dump({"models": dict(sorted(out.items()))}, f, indent=1)
        f.write("\n")
    entries.update(out)
    print(f"[stations_w3d] {len(out)} models -> {FRAGMENT}")
    return out


# --------------------------------------------------------------------------- review sheet

def contact(out_dir):
    """Rows: M3D-ASSETS primitive at game zoom | new model at game zoom | new model closer (2x).
    Game camera: 30 deg FOV, 50 deg pitch, 64 px per ground metre at 1x."""
    import bpy
    import numpy as np
    from bpy_extras.object_utils import world_to_camera_view
    from mathutils import Vector
    cell = 512
    with open(FRAGMENT) as f:
        rows = sorted(json.load(f)["models"].items(), key=lambda kv: list(W3D).index(kv[0][4:].rsplit("_", 1)[0]))
    C.reset_scene()
    sc = bpy.context.scene
    sc.render.engine = "BLENDER_WORKBENCH"
    sc.render.resolution_x = sc.render.resolution_y = cell
    sc.display.shading.color_type = "MATERIAL"
    sc.display.shading.light = "STUDIO"
    sc.display.shading.show_shadows = True
    sc.world = bpy.data.worlds.new("w")
    sc.world.color = (0.80, 0.84, 0.88)
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    sc.collection.objects.link(cam)
    cam.data.angle = math.radians(30)
    cam.data.clip_end = 500
    sc.camera = cam
    ground = bpy.data.meshes.new("ground")
    ground.from_pydata([(-40, -40, 0), (40, -40, 0), (40, 40, 0), (-40, 40, 0)], [], [(0, 1, 2, 3)])
    gmat = bpy.data.materials.new("ground")
    gmat.diffuse_color = (0.47, 0.52, 0.36, 1)
    ground.materials.append(gmat)
    sc.collection.objects.link(bpy.data.objects.new("ground", ground))
    text = bpy.data.curves.new("label", "FONT")
    label = bpy.data.objects.new("label", text)
    sc.collection.objects.link(label)
    label.parent = cam
    tmat = bpy.data.materials.new("label")
    tmat.diffuse_color = (0.02, 0.02, 0.02, 1)
    text.materials.append(tmat)
    tmp = os.path.join(out_dir, "_w3d_cell.png")

    def render(model_id, caption, zoom):
        before = set(bpy.data.objects)
        bpy.ops.import_scene.gltf(filepath=os.path.join(C.MODELS_DIR, f"{model_id}.glb"))
        imported = set(bpy.data.objects) - before
        for o in imported:
            for slot in o.material_slots:
                bsdf = slot.material and slot.material.node_tree and slot.material.node_tree.nodes.get(
                    "Principled BSDF")
                if bsdf:
                    slot.material.diffuse_color = bsdf.inputs["Base Color"].default_value
        pitch = math.radians(50)
        d = Vector((math.cos(pitch) / math.sqrt(2), -math.cos(pitch) / math.sqrt(2), math.sin(pitch)))
        cam.location = d * (cell / (2 * math.tan(math.radians(15)) * 64 * zoom))
        cam.data.shift_y = 0.12
        cam.rotation_euler = (-d).to_track_quat("-Z", "Y").to_euler()
        bpy.context.view_layer.update()
        o = world_to_camera_view(sc, cam, Vector((0, 0, 0)))
        one = world_to_camera_view(sc, cam, Vector((1 / math.sqrt(2), 1 / math.sqrt(2), 0)))
        assert abs((one.x - o.x) * cell - 64 * zoom) < 0.5, "camera scale drift"
        w = 4 * math.tan(math.radians(15))
        label.location = (-w * 0.48, w * (-0.46 + cam.data.shift_y), -2)
        text.size = w * 0.026
        text.body = f"{caption}\n{zoom}x  {64 * zoom}px/m  FOV30 pitch50"
        sc.render.filepath = tmp
        bpy.ops.render.render(write_still=True)
        img = bpy.data.images.load(tmp)
        px = np.empty(cell * cell * 4, dtype=np.float32)
        img.pixels.foreach_get(px)
        bpy.data.images.remove(img)
        for ob in imported:
            bpy.data.objects.remove(ob, do_unlink=True)
        return px.reshape(cell, cell, 4)

    cells = []
    for mid, e in rows:
        name, state = mid[4:].rsplit("_", 1)
        old = REPLACES[name][W3D[name]["states"].index(state)]
        cells.append([render(old, f"OLD {old}", 1), render(mid, f"NEW {mid}  {e['triangles']} tris", 1),
                      render(mid, f"NEW {mid} (closer)", 2)])
    os.remove(tmp)
    n = len(cells)
    page = np.ones((n * cell, 3 * cell, 4), dtype=np.float32)
    for r, row in enumerate(cells):
        for c, px in enumerate(row):
            page[(n - 1 - r) * cell:(n - r) * cell, c * cell:(c + 1) * cell] = px
    path = os.path.join(out_dir, f"blender_w3d_stations_{time.strftime('%Y%m%d_%H%M')}.png")
    img = bpy.data.images.new("page", 3 * cell, n * cell, alpha=True)
    img.pixels.foreach_set(page.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    print(f"[stations_w3d] contact sheet -> {path}")


if __name__ == "__main__":
    args = C.script_args()
    try:
        if args[:1] == ["contact"]:
            contact(args[1] if len(args) > 1 else "/Users/rexxa/work/3d-data/screens")
        else:
            C.reset_scene()
            build()
    except Exception:
        import traceback
        traceback.print_exc()
        sys.exit(1)
