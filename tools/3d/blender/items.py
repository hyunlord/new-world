"""Items lying on the ground (fit inside 1 tile). Run alone: blender -b --factory-startup -P items.py

Material notes: meat flesh uses `fruit` (red) and bone `horn`; there is no dedicated meat name.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402
import look2_props_shapes as H
from stations import basket, pot, log  # noqa: E402  (shared pot/basket shapes)

TAU = 2 * math.pi

ITEMS = {
    "log_pile_s": dict(rows=(2,), length=0.8, r=0.09),
    "log_pile_m": dict(rows=(3, 2), length=0.85, r=0.09),
    "log_pile_l": dict(rows=(3, 2, 1), length=0.9, r=0.1),
    "firewood": dict(n=7),
    "stone_pile": dict(n=6, mat="stone", colors={}),
    "flint_pile": dict(n=6, mat="vein", colors={"stone": (0.78, 0.75, 0.68), "vein": (0.12, 0.12, 0.14)}),
    "ore_pile": dict(n=6, mat="vein", colors={"stone": (0.38, 0.36, 0.36)}),
    "ingot_stack": dict(layers=3),
    "pot": dict(),
    "pot_stack": dict(),
    "hide": dict(),
    "sack": dict(),
    "tool": dict(),
    "meat": dict(colors={"fruit": (0.62, 0.16, 0.14)}),
    "cloth_bolt": dict(),
    "basket": dict(),
}


def log_pile(m, p):
    r, ln = p["r"], p["length"]
    for layer, count in enumerate(p["rows"]):
        for i in range(count):
            x0 = (i - (count - 1) / 2) * r * 2.05
            z = r + layer * r * 1.75
            jy = 0.03 * ((i + layer) % 2)
            log(m, (x0, -ln / 2 + jy, z), (x0, ln / 2 + jy, z), r, segs=6)


def firewood(m, p):
    for i in range(p["n"]):
        layer = 0 if i < 4 else 1
        k = i if layer == 0 else i - 4
        x = (k - (1.5 if layer == 0 else 1.0)) * 0.1
        z = 0.05 + layer * 0.085
        m.cyl("wood", 0.06, 0.5, loc=(x, -0.25 + 0.02 * (i % 2), z), rot=(-math.pi / 2, 0, 0),
              segs=3, cap_mat="trunk", r2=0.055)
    m.rod("leather", (-0.25, 0.02, 0.0), (0.25, 0.02, 0.0), 0.015, segs=4)


def pile(m, p):
    n = p["n"]
    for i in range(n):
        a = TAU * i / (n - 1)
        d = 0.0 if i == 0 else 0.2
        mat = p["mat"] if i % 2 == 0 else "stone"
        if p["mat"] == "stone":
            mat = "stone"
        s = 0.13 if i == 0 else 0.1
        z = 0.12 if i == 0 else 0.06
        m.ball(mat, s, loc=(math.cos(a) * d, math.sin(a) * d, z), scale=(1.2, 1, 0.8), subdiv=1,
               jitter=0.18 if p["mat"] == "vein" else 0.12, ground=True)


def ingot_stack(m, p):
    for layer in range(p["layers"]):
        n = 3 - (layer % 2)
        for i in range(n):
            off = (i - (n - 1) / 2) * 0.12
            loc = (off, 0, 0.025 + layer * 0.05) if layer % 2 == 0 else (0, off, 0.025 + layer * 0.05)
            size = (0.1, 0.3, 0.05) if layer % 2 == 0 else (0.3, 0.1, 0.05)
            m.box("metal", size, loc=loc, taper=0.8)


def pot_item(m, p):
    pot(m, (0, 0, 0), h=0.38, r=0.17, segs=9)


def pot_stack(m, p):
    pot(m, (-0.13, 0, 0), h=0.3, r=0.13, segs=6)
    pot(m, (0.13, 0.02, 0), h=0.28, r=0.12, segs=6)
    m.lathe("clay", [(0.07, 0), (0.12, 0.12), (0.08, 0.22)], loc=(0.0, 0.0, 0.27), segs=7,
            top_mat="earth")  # small jar on top


def hide(m, p):
    m.box("hide", (0.75, 0.5, 0.025), loc=(0, 0, 0.012), taper=0.85, jitter=0.01)
    for sx in (-1, 1):  # leg flaps
        for sy in (-1, 1):
            m.box("hide", (0.14, 0.18, 0.02), loc=(sx * 0.37, sy * 0.25, 0.01),
                  rot=(0, 0, sx * sy * 0.6), taper=0.6)
    m.box("hide", (0.1, 0.22, 0.02), loc=(0.42, 0, 0.012), taper=0.5)  # neck


def sack(m, p):
    m.ball("cloth", 0.2, loc=(0, 0, 0.17), scale=(1.0, 0.9, 0.95), subdiv=2, jitter=0.05,
           ground=True)
    m.cyl("cloth", 0.06, 0.12, loc=(0, 0, 0.33), segs=6, r2=0.09)
    m.cyl("leather", 0.065, 0.025, loc=(0, 0, 0.35), segs=6)


def tool(m, p):
    m.rod("wood", (-0.3, -0.05, 0.025), (0.3, 0.05, 0.025), 0.022, segs=6)
    m.box("stone", (0.09, 0.2, 0.05), loc=(0.27, 0.0, 0.03), rot=(0, 0, 0.16), taper=0.65)
    m.rod("leather", (0.26, -0.035, 0.025), (0.28, 0.06, 0.025), 0.03, segs=5)


def meat(m, p):
    m.ball("fruit", 0.13, loc=(0.05, 0, 0.09), scale=(1.4, 1.0, 0.75), subdiv=2, jitter=0.05,
           ground=True)
    m.rod("horn", (-0.12, 0.0, 0.08), (-0.3, 0.02, 0.06), 0.03, segs=5)
    m.ball("horn", 0.045, loc=(-0.3, 0.02, 0.06), subdiv=1)


def cloth_bolt(m, p):
    m.cyl("cloth", 0.1, 0.55, loc=(-0.275, 0, 0.1), rot=(0, math.pi / 2, 0), segs=8,
          cap_mat="wood")
    m.box("cloth", (0.3, 0.2, 0.012), loc=(0.0, -0.17, 0.006), taper=0.95)
    for x in (-0.15, 0.15):
        m.cyl("leather", 0.105, 0.03, loc=(x, 0, 0.1), rot=(0, math.pi / 2, 0), segs=8)


def basket_item(m, p):
    basket(m, (0, 0, 0), h=0.26, r=0.2, segs=9)
    m.rod("wood", (-0.19, 0, 0.26), (0, 0, 0.42), 0.015, segs=4)
    m.rod("wood", (0, 0, 0.42), (0.19, 0, 0.26), 0.015, segs=4)


FNS = {"firewood": firewood, "stone_pile": pile, "flint_pile": pile, "ore_pile": pile,
       "ingot_stack": ingot_stack, "pot": pot_item, "pot_stack": pot_stack, "hide": hide,
       "sack": sack, "tool": tool, "meat": meat, "cloth_bolt": cloth_bolt, "basket": basket_item}


def build(entries):
    for iid, p in ITEMS.items():
        m = C.Model(iid)
        m.colors.update(p.get("colors", {}))
        (log_pile if iid.startswith("log_pile") else FNS[iid])(m, p)
        H.finish(m)
        H.fit(m, (1, 1))
        C.save_static(m, (1, 1), entries, {"family": "item"})
    C.check_budget({k: e for k, e in entries.items() if e.get("family") == "item"}, 20, 150)
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}))
