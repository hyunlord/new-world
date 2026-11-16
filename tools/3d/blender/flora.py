"""Flora: trees, bushes, stump, sapling. Run alone: blender -b --factory-startup -P flora.py"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

# Species proportions (metres). Ranges are (min, max) sampled per variant with its own seed.
SPECIES = {
    "oak": dict(kind="broadleaf", trunk_h=(2.2, 2.7), trunk_r=0.24, canopy_z=(3.7, 4.1),
                clusters=(4, 6), cluster_r=(0.8, 1.1), spread=1.15, main_r=(1.45, 1.65),
                squash=0.85, branches=2, colors={}),
    "birch": dict(kind="broadleaf", trunk_h=(4.4, 5.0), trunk_r=0.15, canopy_z=(3.9, 4.2),
                  clusters=(5, 6), cluster_r=(0.6, 0.8), spread=0.6, main_r=(1.05, 1.2),
                  squash=1.9, branches=2,
                  colors={"trunk": (0.80, 0.78, 0.72), "canopy": (0.45, 0.62, 0.22)}),
    "pine": dict(kind="conifer", trunk_h=(1.2, 1.5), trunk_r=0.18, tiers=(4, 5),
                 base_r=(1.35, 1.6), top=(5.6, 6.6), colors={"canopy": (0.10, 0.30, 0.16)}),
}

TREES = {  # id -> (species, seed, per-variant overrides of the species table)
    "tree_oak_a": ("oak", 11, {}), "tree_oak_b": ("oak", 12, {"clusters": (5, 6)}),
    "tree_oak_c": ("oak", 13, {"trunk_h": (2.6, 2.8), "branches": 3}),
    "tree_birch_a": ("birch", 21, {}), "tree_birch_b": ("birch", 22, {"squash": 1.5}),
    "tree_pine_a": ("pine", 31, {"tiers": (4, 4)}),
    "tree_pine_b": ("pine", 32, {"tiers": (5, 5), "base_r": (1.15, 1.2), "top": (6.8, 7.0)}),
    "tree_pine_c": ("pine", 33, {"tiers": (4, 4), "base_r": (1.75, 1.85), "top": (4.8, 5.0),
                                 "trunk_h": (0.9, 1.0)}),
}

SMALL = {  # id -> params
    "bush": dict(height=0.75, radius=0.5, lumps=4, berries=0),
    "berry_bush": dict(height=0.8, radius=0.5, lumps=4, berries=9),
    "sapling": dict(height=1.4),
    "stump": dict(radius=0.26, height=0.38),
}


def u(r, a):
    return r.uniform(*a) if isinstance(a, tuple) else a


def broadleaf(m, sp):
    r = m.rng
    th = u(r, sp["trunk_h"])
    cz = u(r, sp["canopy_z"])
    m.cyl("trunk", sp["trunk_r"], cz - 0.3, segs=6, r2=sp["trunk_r"] * 0.55, jitter=0.01)
    # root flare
    m.cyl("trunk", sp["trunk_r"] * 1.6, 0.25, segs=6, r2=sp["trunk_r"], jitter=0.01)
    for i in range(sp["branches"]):
        a = r.uniform(0, 2 * math.pi)
        m.cyl("trunk", sp["trunk_r"] * 0.45, 1.3, loc=(0, 0, th * 0.75), segs=5, r2=0.04,
              rot=(0.75, 0, a))
    sq = sp["squash"]
    mr = u(r, sp["main_r"])
    m.ball("canopy", mr, loc=(0, 0, cz), scale=(1, 1, sq), subdiv=2, jitter=0.07)
    n = r.randint(*sp["clusters"])
    for i in range(n):
        a = 2 * math.pi * (i + r.uniform(-0.2, 0.2)) / n
        cr = u(r, sp["cluster_r"])
        d = sp["spread"] * r.uniform(0.75, 1.05)
        z = cz + r.uniform(-0.45, 0.55) * mr * sq
        m.ball("canopy", cr, loc=(d * math.cos(a), d * math.sin(a), z), scale=(1, 1, sq),
               subdiv=1, jitter=0.12)


def conifer(m, sp):
    r = m.rng
    th = u(r, sp["trunk_h"])
    top = u(r, sp["top"])
    m.cyl("trunk", sp["trunk_r"], th + 0.6, segs=6, r2=sp["trunk_r"] * 0.7, jitter=0.01)
    tiers = r.randint(*sp["tiers"])
    br = u(r, sp["base_r"])
    span = top - th
    for i in range(tiers):
        f = i / tiers
        z0 = th + span * f * 0.82
        rad = br * (1 - 0.68 * f)
        h = span * (1 - f * 0.82) * (0.62 if i < tiers - 1 else 1.0)
        z0 = min(z0, top - h)
        # skirt tier: thick drooping rim then a cone to the apex
        m.tube("canopy", [(0, 0, z0), (0, 0, z0 + 0.22), (0, 0, z0 + h)], [rad, rad * 0.82, 0.0],
               segs=10, jitter=0.05)
    return top


def stump(m, p):
    rad, h = p["radius"], p["height"]
    m.cyl("trunk", rad, h, segs=8, r2=rad * 0.92, cap_mat="wood", jitter=0.008)
    for i in range(4):
        a = i * math.pi / 2 + 0.4
        m.cyl("trunk", 0.11, 0.26, loc=(math.cos(a) * rad * 0.7, math.sin(a) * rad * 0.7, 0.07),
              rot=(1.9, 0, a + math.pi / 2), segs=5, r2=0.06)


def bush(m, p):
    r = m.rng
    rad, h = p["radius"], p["height"]
    m.ball("canopy", rad, loc=(0, 0, h * 0.5), scale=(1.15, 1.0, 0.85), subdiv=2, jitter=0.08,
           ground=True)
    for i in range(p["lumps"]):
        a = 2 * math.pi * i / p["lumps"] + 0.5
        m.ball("canopy", rad * 0.6, loc=(math.cos(a) * rad * 0.65, math.sin(a) * rad * 0.65,
               h * 0.38), subdiv=1, jitter=0.12, ground=True)
    for i in range(p["berries"]):
        a = 2.39996 * i
        z = h * (0.35 + 0.5 * ((i * 0.618) % 1.0))
        rr = rad * 1.02 * math.sqrt(max(0.0, 1 - ((z - h * 0.5) / (h * 0.55)) ** 2)) + 0.05
        m.ball("fruit", 0.065, loc=(math.cos(a) * rr, math.sin(a) * rr, z), subdiv=1)


def sapling(m, p):
    h = p["height"]
    m.cyl("trunk", 0.04, h, segs=5, r2=0.02)
    m.cyl("earth", 0.22, 0.05, segs=7, r2=0.16)
    m.ball("canopy", 0.32, loc=(0, 0, h), scale=(1, 1, 1.2), subdiv=1, jitter=0.1)
    m.ball("canopy", 0.22, loc=(0.18, 0.05, h - 0.3), subdiv=1, jitter=0.1)
    m.ball("canopy", 0.2, loc=(-0.15, -0.08, h - 0.42), subdiv=1, jitter=0.1)
    m.ball("canopy", 0.2, loc=(0.0, 0.18, h - 0.2), subdiv=1, jitter=0.1)


def build(entries):
    for tid, (species, seed, over) in TREES.items():
        sp = dict(SPECIES[species], **over)
        m = C.Model(tid, seed=seed)
        m.colors.update(sp["colors"])
        (broadleaf if sp["kind"] == "broadleaf" else conifer)(m, sp)
        C.save_static(m, (1, 1), entries, {"family": "flora", "species": species})
    for sid, p in SMALL.items():
        m = C.Model(sid)
        {"bush": bush, "berry_bush": bush, "sapling": sapling, "stump": stump}[sid](m, p)
        C.save_static(m, (1, 1), entries, {"family": "flora"})
    C.check_budget({k: v for k, v in entries.items() if k.startswith("tree_")}, 150, 400)
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}))
