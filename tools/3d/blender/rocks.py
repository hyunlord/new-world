"""Rocks and ground features. Run alone: blender -b --factory-startup -P rocks.py

Material notes: flint nodules use `vein` (default near-black) so Godot can tint them like ore veins;
the ore outcrop's crystals are `vein` (Godot tints copper vs tin).
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C  # noqa: E402

ROCKS = {
    "boulder_large": dict(size=1.2, lumps=1, colors={}),
    "boulder_small": dict(size=0.55, lumps=1, colors={}),
    "flint_outcrop": dict(size=0.9, nodules=5,
                          colors={"stone": (0.78, 0.75, 0.68), "vein": (0.10, 0.10, 0.12)}),
    "ore_outcrop": dict(size=1.0, crystals=6,
                        colors={"stone": (0.38, 0.36, 0.36), "vein": (0.82, 0.45, 0.18)}),
    "clay_bank": dict(size=1.6, patches=2, colors={}),
}


def boulder(m, p):
    s = p["size"]
    big = s > 1.0
    m.ball("stone", s * 0.5, loc=(0, 0, s * 0.3), scale=(1.1, 0.95, 0.8), subdiv=2 if big else 1,
           jitter=0.12, ground=True)
    m.ball("stone", s * 0.28, loc=(s * 0.38, -s * 0.18, s * 0.1), scale=(1.0, 1.0, 0.8),
           subdiv=1, jitter=0.15, ground=True)


def flint(m, p):
    s = p["size"]
    m.ball("stone", s * 0.5, loc=(0, 0, s * 0.1), scale=(1.15, 1.0, 0.5), subdiv=1, jitter=0.12,
           ground=True)
    m.ball("stone", s * 0.3, loc=(s * 0.35, s * 0.2, s * 0.05), scale=(1.0, 1.0, 0.6), subdiv=1,
           jitter=0.12, ground=True)
    r = m.rng
    for i in range(p["nodules"]):  # dark nodules poking out of the pale cortex
        a = 2 * math.pi * i / p["nodules"] + r.uniform(-0.3, 0.3)
        d = s * (0.12 if i == 0 else r.uniform(0.3, 0.45))
        m.ball("vein", s * r.uniform(0.11, 0.14), loc=(math.cos(a) * d, math.sin(a) * d,
               s * 0.32 - d * 0.45), scale=(1.3, 1, 0.8), subdiv=1, jitter=0.15)


def ore(m, p):
    s = p["size"]
    r = m.rng
    # angular, tall-ish base: low-subdivision rocks with heavy jitter read as broken rock
    m.ball("stone", s * 0.48, loc=(0, 0, s * 0.25), scale=(1.1, 0.9, 0.9), subdiv=1, jitter=0.25,
           ground=True)
    m.ball("stone", s * 0.3, loc=(s * 0.38, s * 0.15, s * 0.12), subdiv=1, jitter=0.25, ground=True)
    # jagged crystal spikes radiating from the top (separate `vein` surface)
    for i in range(p["crystals"]):
        a = 2 * math.pi * i / p["crystals"] + r.uniform(-0.25, 0.25)
        tilt = r.uniform(0.25, 0.75)
        ln = s * r.uniform(0.45, 0.75) * (1.25 if i == 0 else 1.0)
        base = (math.cos(a) * s * 0.15, math.sin(a) * s * 0.15, s * 0.35)
        d = (math.cos(a) * math.sin(tilt), math.sin(a) * math.sin(tilt), math.cos(tilt))
        pts = [base, tuple(b + dd * ln * 0.7 for b, dd in zip(base, d)),
               tuple(b + dd * ln for b, dd in zip(base, d))]
        m.tube("vein", pts, [s * 0.09, s * 0.09, 0.0], segs=5)


def clay_bank(m, p):
    s = p["size"]
    m.ball("earth", s * 0.5, loc=(0, 0, 0.0), scale=(1.0, 0.8, 0.42), subdiv=2, jitter=0.08,
           ground=True)
    for i in range(p["patches"]):
        x = (-0.18 + 0.36 * i) * s
        m.ball("clay", s * 0.26, loc=(x, -s * 0.12, s * 0.05), scale=(1.1, 0.9, 0.55), subdiv=1,
               jitter=0.1, ground=True)


def build(entries):
    fns = {"boulder_large": boulder, "boulder_small": boulder, "flint_outcrop": flint,
           "ore_outcrop": ore, "clay_bank": clay_bank}
    for rid, p in ROCKS.items():
        m = C.Model(rid)
        m.colors.update(p["colors"])
        fns[rid](m, p)
        fp = (2, 2) if p["size"] > 1.3 else (1, 1)
        C.save_static(m, fp, entries, {"family": "rocks"})
    C.check_budget({k: e for k, e in entries.items() if e.get("family") == "rocks"}, 40, 150)
    return entries


if __name__ == "__main__":
    C.reset_scene()
    C.update_manifest(build({}))
