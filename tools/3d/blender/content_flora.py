"""C3 plants, crops and material-specific outcrops; Blender headless entry point.

blender -b --factory-startup -P tools/3d/blender/content_flora.py
Writes a separate manifest fragment; the asset pack build merges it with M3D-1.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import math
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common as C
import rocks

SOURCE_SHA = "cfeb57c9e1906eedd4e5d14447ba4faca20c4047"
# kind, height, seed/flower colour; visual authoring only, never simulation content.
SPECIES = {
    "berry_bush": ("shrub", 0.75, (0.65, 0.12, 0.18)),
    "hazel": ("shrub", 1.35, (0.46, 0.25, 0.10)),
    "wild_flax": ("flower", 0.70, (0.35, 0.48, 0.80)),
    "wild_grain": ("grain", 0.82, (0.72, 0.55, 0.20)),
    "wild_root": ("root", 0.30, (0.73, 0.48, 0.28)),
    "wild_barley": ("barley", 0.72, (0.76, 0.60, 0.24)),
    "wild_rye": ("grain", 1.08, (0.62, 0.52, 0.28)),
    "wild_lentil": ("legume", 0.34, (0.48, 0.40, 0.20)),
    "wild_pea": ("legume", 0.58, (0.38, 0.56, 0.22)),
    "wild_millet": ("millet", 0.76, (0.66, 0.45, 0.16)),
    "wild_apple": ("shrub", 1.50, (0.72, 0.22, 0.12)),
}
CROPS = {
    "emmer": "wild_grain", "flax": "wild_flax", "barley": "wild_barley",
    "rye": "wild_rye", "lentil": "wild_lentil", "pea": "wild_pea", "millet": "wild_millet",
}
PHASES = ("growing", "ripe", "spent", "dormant")
OUTCROPS = {
    "copper_outcrop": (0.76, 0.37, 0.13),
    "tin_outcrop": (0.52, 0.55, 0.59),
    "iron_outcrop": (0.46, 0.20, 0.11),
    "bog_iron_outcrop": (0.29, 0.18, 0.10),
}


def plant(model, spec, phase):
    """Author readable plant silhouettes with visible seasonal fruit and stubble."""
    kind, height, fruit = spec
    green = phase in ("growing", "ripe")
    model.colors.update(canopy=(0.25, 0.43, 0.14) if green else (0.48, 0.37, 0.19), fruit=fruit)
    scale = 0.58 if phase == "growing" else 1.0
    height *= scale
    if kind == "shrub":
        model.cyl("trunk", 0.045, height * 0.67, segs=5)
        for i in range(3):
            angle = i * math.tau / 3
            x, y = math.cos(angle) * 0.20, math.sin(angle) * 0.20
            model.rod("trunk", (0, 0, height * 0.2), (x, y, height * 0.75), 0.025, segs=5)
            if phase != "dormant":
                model.ball("canopy", height * 0.22, loc=(x, y, height * 0.7), scale=(1, 1, 0.85))
            if phase == "ripe":
                model.ball("fruit", 0.055, loc=(x, y - height * 0.2, height * 0.72))
        return
    count = 7 if kind in ("grain", "barley", "millet") else 5
    for i in range(count):
        angle = i * 2.399963
        x, y = math.cos(angle) * (0.08 + i * 0.025), math.sin(angle) * (0.08 + i * 0.025)
        h = height * (0.8 + (i % 3) * 0.1)
        if phase in ("spent", "dormant"):
            h *= 0.20 if phase == "spent" else 0.09
        model.rod("canopy", (x, y, 0), (x + 0.025, y, h), 0.012, segs=4)
        if phase in ("spent", "dormant"):
            continue
        model.ball("canopy", 0.065, loc=(x + 0.035, y, h * 0.48), scale=(1.7, 0.4, 0.5))
        match kind:
            case "grain" | "barley" | "millet":
                if phase == "ripe":
                    model.ball("fruit", 0.035, loc=(x + 0.025, y, h), scale=(1, 1, 3.3 if kind != "millet" else 4.5))
                    if kind == "barley":
                        for side in (-1, 1):
                            model.rod("fruit", (x + 0.025, y, h), (x + side * 0.075, y, h + 0.15), 0.004, segs=3)
            case "flower":
                if phase == "ripe":
                    model.ball("fruit", 0.045, loc=(x + 0.025, y, h), scale=(1, 1, 0.3))
            case "legume":
                model.ball("canopy", 0.075, loc=(x, y, h * 0.72), scale=(1.5, 0.5, 0.6))
                if phase == "ripe":
                    model.ball("fruit", 0.033, loc=(x + 0.05, y, h * 0.7), scale=(0.65, 0.65, 2))
            case "root":
                model.ball("canopy", 0.10, loc=(x, y, h * 0.5), scale=(0.7, 0.4, 1.5))
                if phase == "ripe":
                    model.ball("fruit", 0.055, loc=(x, y, 0.04), scale=(1, 1, 0.7))
            case _:
                raise AssertionError(f"Unknown authored silhouette: {kind}")


def build(entries):
    """Export all explicit registry identities and phases without editing the base manifest."""
    groups = (("plant", SPECIES), ("crop", {k: SPECIES[v] for k, v in CROPS.items()}))
    for family, rows in groups:
        for identity, spec in rows.items():
            for phase in PHASES:
                mid = f"{family}_{identity}_{phase}"
                model = C.Model(mid, seed=0)
                plant(model, spec, phase)
                C.save_static(model, (1, 1), entries, {"family": "flora", "content_family": family,
                              "content_id": identity, "phase": phase, "source_sha": SOURCE_SHA})
    for identity, color in OUTCROPS.items():
        model = C.Model(identity, seed=0)
        model.colors.update(vein=color)
        rocks.ore(model, {"size": 0.8, "crystals": 4})
        C.save_static(model, (1, 1), entries, {"family": "rocks", "content_family": "outcrop",
                      "content_id": identity, "source_sha": SOURCE_SHA})
    expected = {f"{family}_{identity}_{phase}" for family, rows in groups
                for identity in rows for phase in PHASES} | set(OUTCROPS)
    assert expected and expected <= entries.keys()
    assert all(row["height"] > 0 and row["triangles"] > 0 for row in entries.values())
    return entries


if __name__ == "__main__":
    C.reset_scene()
    result = build({})
    path = Path(C.MODELS_DIR).parent / "flora_manifest.json"
    path.write_text(json.dumps({"models": result}, indent=2) + "\n")
    print(f"CONTENT_FLORA_DONE {len(result)} models: {path}")
