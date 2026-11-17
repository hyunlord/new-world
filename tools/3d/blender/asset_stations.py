"""M3D-ASSETS station states and modular construction pieces.

Run: blender -b --factory-startup -P tools/3d/blender/asset_stations.py
Uses the M3D-1 palette, feet origin, metre units and +Z Godot forward.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C
import stations as S
import look2_props_shapes as H
import stations_w3d as W


def masonry(m, style, height):
    mat = {"mud_brick": "clay", "stone": "stone", "log": "trunk"}[style]
    if style == "log":
        for z in range(round(height / 0.2)):
            drift = 0.012 * ((z % 3) - 1)
            m.rod(mat, (-0.49, drift, z * 0.2 + 0.1), (0.49, -drift, z * 0.2 + 0.1),
                  0.1, r2=0.092, cap_mat="wood", jitter=0.003)
            for x in (-.39, .39):
                m.box("wood_dark", (.025, .12, .025), loc=(x, 0, z * .2 + .185))
    else:
        for z in range(round(height / 0.2)):
            for x in range(4):
                m.box(mat, (0.239, 0.24, 0.19), loc=(-0.375 + x * 0.25, 0, 0.1 + z * 0.2),
                      jitter=.004, taper=.96)


def bench(m, kind, working):
    for x in (-0.6, 0.6):
        for y in (-0.25, 0.25):
            m.box("wood_dark", (0.08, 0.08, 0.7), loc=(x, y, 0.35))
    m.box("wood", (1.4, 0.65, 0.1), loc=(0, 0, 0.75))
    if kind == "mason_bench":
        m.box("stone", (0.45, 0.35, 0.25), loc=(-0.2, 0, 0.925), jitter=0.015)
        m.rod("wood", (0.3, -0.2, 0.82), (0.55, 0.1, 0.82), 0.025)
        m.box("stone", (0.14, 0.08, 0.08), loc=(0.55, 0.1, 0.84))
    else:
        m.cyl("wood_dark", 0.25, 0.03, loc=(0, 0, 0.82), segs=12)
        S.pot(m, (0, 0, 0.85), h=0.28, r=0.15)
    if working:
        m.ball("clay", 0.08, loc=(0.4, 0.1, 0.85), ground=True)


def fence(m, mask, palisade):
    height = 1.8 if palisade else 0.9
    m.cyl("trunk", 0.07, height, segs=6, r2=0.02 if palisade else 0.07)
    H.lash(m, (0, 0, .71), .081)
    for bit, (x, y) in enumerate(((0, 0.5), (0.5, 0), (0, -0.5), (-0.5, 0))):
        if mask & (1 << bit):
            for z in (0.3, 0.7):
                m.rod("wood", (0, 0, z), (x, y, z), 0.035, segs=5)
            if not palisade:
                for k in range(5):
                    z = .23 + k * .105
                    sway = .035 if k % 2 else -.035
                    m.tube("wood", [(0, 0, z), (x * .5 - y * sway, y * .5 + x * sway, z + .012),
                                    (x, y, z)], [.018] * 3, segs=4)
                m.rod("trunk", (x * .5, y * .5, .1), (x * .5, y * .5, .8), .026, segs=5)
            if palisade:
                for k in range(1, 5):
                    m.cyl("trunk", 0.065, height, loc=(x * k / 4, y * k / 4, 0), segs=5, r2=0.02)


def build():
    entries = {}
    for source, states in {"smelter": ("lit", "unlit"), "charcoal_clamp": ("idle", "burning", "spent"),
                           "casting_bench": ("empty", "working"), "kiln": ("lit", "unlit"),
                           "anvil": ("idle", "working"), "hearth": ("lit", "unlit"),
                           "bloomery": ("lit", "unlit"), "forge": ("idle", "working")}.items():
        base = {"bloomery": "smelter", "forge": "hearth"}.get(source, source)
        for state in states:
            m = C.Model(f"asset_{source}_{state}")
            p = dict(S.STATIONS[base])
            active = state in ("lit", "burning", "working")
            match source:  # noqa: MATCH_OK — authored registry above is the complete input domain.
                case "charcoal_clamp":
                    W.charcoal_clamp(m, {"idle": "idle", "burning": "working", "spent": "done"}[state])
                case "smelter" | "bloomery" | "kiln":
                    getattr(W, source)(m, active)
                case "casting_bench" | "anvil" | "hearth" | "forge":
                    getattr(S, base)(m, p)
                case _:
                    raise AssertionError("Unregistered state builder: " + source)
            if not active:
                for face in m.bm.faces:
                    if m.mats[face.material_index] == "glow":
                        if "wood_dark" not in m.mats:
                            m.mats.append("wood_dark")
                        face.material_index = m.mats.index("wood_dark")
            if source == "casting_bench" and state == "empty":
                import bmesh
                faces = [f for f in m.bm.faces if m.mats[f.material_index] == "metal"]
                bmesh.ops.delete(m.bm, geom=faces, context="FACES")
            if source == "forge":
                S.anvil(m, {})
                m.ball("leather", 0.2, loc=(0.3, 0.15, 0.22), scale=(0.7, 1, 0.5))
                m.rod("clay", (0.3, 0.15, 0.2), (0, 0, 0.2), 0.04)
            if source == "anvil" and active:
                m.box("glow", (0.23, 0.07, 0.04), loc=(0, 0, 0.65))
                m.rod("wood", (-0.2, -0.15, 0.1), (0.2, -0.15, 0.5), 0.025)
            if active:
                m.sockets["light"] = p.get("light", (0, 0, 0.5))
            m.sockets["work"] = (0, -(p["fp"][1] / 2 + 0.35), 0)
            H.finish(m)
            H.fit(m, p["fp"])
            C.save_static(m, p["fp"], entries, {"family": "station", "state": state})
    for kind in ("mason_bench", "pottery_wheel"):
        for working in (False, True):
            m = C.Model(f"{kind}_{'working' if working else 'idle'}")
            bench(m, kind, working)
            H.finish(m)
            C.save_static(m, (2, 1), entries, {"family": "station"})
    for style in ("mud_brick", "stone", "log"):
        for stage, height in (("foundation", 0.2), ("half_built", 1.0), ("complete", 2.0)):
            m = C.Model(f"wall_{style}_{stage}")
            masonry(m, style, height)
            H.finish(m)
            C.save_static(m, (1, 1), entries, {"family": "wall", "stage": stage})
    for palisade in (False, True):
        for mask in range(16):
            m = C.Model(f"{'palisade' if palisade else 'pen_fence'}_{mask:02x}")
            fence(m, mask, palisade)
            C.save_static(m, (1.2, 1.2), entries, {"family": "fence", "connections": mask,
                          "connection_bits": ["north", "east", "south", "west"]})
    m = C.Model("floor_paved_road")
    for y in range(4):
        for x in range(4):
            m.box("stone", (0.24, 0.24, 0.04), loc=(-0.375 + x * 0.25, -0.375 + y * 0.25, 0.02))
    C.save_static(m, (1, 1), entries, {"family": "floor"})
    assert len(entries) == 63, len(entries)
    with open(os.path.join(C.MODELS_DIR, "asset_stations_manifest.json"), "w") as f:
        json.dump({"models": entries}, f, indent=1)
    return entries


if __name__ == "__main__":
    C.reset_scene()
    build()
