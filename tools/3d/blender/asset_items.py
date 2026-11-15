"""M3D-ASSETS per-code ground items, data-driven small/medium/large stacks.

Run: blender -b --factory-startup -P tools/3d/blender/asset_items.py
The pinned table records the provisional local C3 source without changing sim data.
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
import items as I
import stations as S

COLORS = {"copper": (0.73, 0.32, 0.13), "bronze": (0.63, 0.47, 0.18),
          "tin": (0.70, 0.73, 0.75), "iron": (0.25, 0.29, 0.32),
          "mud_brick": (0.61, 0.42, 0.27), "limestone": (0.77, 0.75, 0.62),
          "granite": (0.43, 0.46, 0.48), "clay": (0.68, 0.38, 0.22),
          "flint": (0.13, 0.15, 0.16), "charcoal": (0.055, 0.06, 0.065),
          "copper_ore": (0.18, 0.48, 0.32), "tin_ore": (0.54, 0.52, 0.42),
          "iron_ore": (0.52, 0.22, 0.12), "bog_iron": (0.30, 0.20, 0.10)}


def item(m, row):
    name = row["id"]
    tint = row["tint"]
    for metal in ("copper", "bronze", "iron", "tin"):
        if metal in name:
            tint = metal
    tint = {"clay": "clay", "raw_pot": "clay", "flint": "flint", "flint_blade": "flint",
            "charcoal": "charcoal", "copper_ore": "copper_ore", "tin_ore": "tin_ore",
            "iron_ore": "iron_ore", "bog_iron": "bog_iron"}.get(name, tint)
    blade = "metal" if any(x in name for x in ("copper", "bronze", "iron", "steel")) else "stone"
    if tint in COLORS:
        m.colors.update({k: COLORS[tint] for k in ("metal", "stone", "clay", "vein")})
    if name == "log":
        I.log_pile(m, {"rows": (1,), "length": 0.7, "r": 0.09})
    elif name == "firewood":
        I.firewood(m, {"n": 3})
    elif "hide" in name or "cloak" in name:
        I.hide(m, {})
    elif "cloth" in name or "tunic" in name or "wrap" in name or name in ("poultice", "leather_armor"):
        I.cloth_bolt(m, {})
    elif name == "basket":
        I.basket_item(m, {})
    elif "pot" in name or name in ("stew", "porridge", "fruit_porridge", "beer", "milk", "honey"):
        S.pot(m, (0, 0, 0), h=0.3, r=0.16)
        m.colors["earth"] = (0.7, 0.55, 0.24) if name not in ("raw_pot", "clay_pot") else (0.18, 0.12, 0.08)
    elif "meat" in name:
        I.meat(m, {})
    elif name in ("bow", "composite_bow"):
        m.tube("wood", [(-0.34, 0, 0.04), (-0.2, 0.12, 0.04), (0, 0.18, 0.04), (0.2, 0.12, 0.04), (0.34, 0, 0.04)], [0.02] * 5)
        m.rod("leather", (-0.34, 0, 0.04), (0.34, 0, 0.04), 0.006, segs=4)
    elif "sickle" in name:
        m.rod("wood", (-0.3, 0, 0.03), (0, 0, 0.03), 0.025)
        m.tube(blade, [(0, 0, 0.03), (0.2, 0, 0.03), (0.25, 0.18, 0.03), (0.1, 0.27, 0.03)], [0.04, 0.04, 0.03, 0])
    elif "hoe" in name or "adze" in name:
        m.rod("wood", (-0.32, 0, 0.04), (0.28, 0, 0.04), 0.025)
        m.box(blade, (0.10, 0.26, 0.045), loc=(0.25, -0.08, 0.04), taper=0.65)
    elif "needle" in name or "awl" in name:
        mat = "horn" if name == "bone_needle" else "metal"
        m.rod(mat, (-0.18, 0, 0.02), (0.22, 0, 0.02), 0.016, segs=5, r2=0)
        if "awl" in name:
            m.box("wood", (0.13, 0.045, 0.045), loc=(-0.15, 0, 0.025))
    elif name == "hand_spindle":
        m.rod("wood", (-0.25, 0, 0.09), (0.25, 0, 0.09), 0.015)
        m.cyl("stone", 0.09, 0.025, loc=(-0.1, 0, 0.09), rot=(0, 1.57079632679, 0))
    elif "spear" in name or name == "arrow":
        length, radius = (0.25, 0.009) if name == "arrow" else (0.36, 0.018)
        m.rod("wood", (-length, 0, 0.03), (length * 0.65, 0, 0.03), radius)
        m.rod(blade, (length * 0.65, 0, 0.03), (length * 1.12, 0, 0.03), radius * 2.5, r2=0)
        if name == "arrow":
            m.box("cloth", (0.08, 0.07, 0.012), loc=(-0.2, 0, 0.03), taper=0.3)
    elif "axe" in name:
        I.tool(m, {})
    elif any(x in name for x in ("blade", "knife", "sword", "saw", "edge")):
        m.box("wood", (0.18, 0.05, 0.04), loc=(-0.25, 0, 0.02))
        m.box(blade, (0.46, 0.10, 0.025), loc=(0.05, 0, 0.02), taper=0.7)
    elif "ingot" in name or name in ("wrought_iron", "cut_stone", "mud_brick", "tablet", "clay_mould"):
        m.box("metal" if "ingot" in name or name == "wrought_iron" else "stone", (0.35, 0.19, 0.10), loc=(0, 0, 0.05), taper=0.85)
    elif name in ("hay", "flax_stalks", "flax_fibre", "herbs"):
        for k in range(7):
            m.rod("thatch", (-0.3, -0.10 + 0.03 * k, 0.04), (0.3, -0.10 + 0.03 * k, 0.04), 0.012, segs=4)
        m.box("leather", (0.03, 0.25, 0.06), loc=(0, 0, 0.04))
    elif name in ("wool_yarn", "linen_thread", "cord", "sling"):
        m.cyl("cloth", 0.12, 0.12, segs=10, r2=0.11)
        m.rod("cloth", (0, 0, 0.02), (0.3, 0.1, 0.02), 0.012, segs=4)
    elif name == "fish":
        m.ball("metal", 0.13, loc=(0, 0, 0.06), scale=(1.9, 0.6, 0.5), ground=True)
        m.box("metal", (0.12, 0.18, 0.02), loc=(-0.27, 0, 0.04), taper=0.2)
        m.ball("earth", 0.015, loc=(0.17, -0.07, 0.07))
    elif name == "coin":
        m.cyl("metal", 0.12, 0.025, segs=12)
        m.cyl("metal", 0.085, 0.01, loc=(0, 0, 0.025), segs=8)
    elif name == "nails":
        for y in (-0.08, 0, 0.08):
            m.rod("metal", (-0.16, y, 0.03), (0.16, y, 0.03), 0.012, segs=4, r2=0)
            m.box("metal", (0.025, 0.045, 0.04), loc=(-0.16, y, 0.03))
    elif name == "book":
        m.box("leather", (0.3, 0.22, 0.07), loc=(0, 0, 0.035))
        m.box("cloth", (0.27, 0.21, 0.045), loc=(0.01, -0.01, 0.035))
    elif name == "wheel":
        m.cyl("wood", 0.24, 0.04, segs=10)
        m.cyl("wood_dark", 0.045, 0.08, segs=6)
    elif "helmet" in name or "boss" in name:
        m.ball("metal", 0.19, loc=(0, 0, 0), scale=(1, 1, 0.8), subdiv=2, ground=True)
    else:
        mat = {"clay": "clay", "wool": "cloth", "roots": "earth", "roasted_roots": "earth",
               "hazelnuts": "wood", "lentil": "thatch", "pea": "canopy"}.get(name, "stone")
        if name in ("berries", "apple", "dried_berries"):
            mat = "fruit"
        if row["shape"] == "Ore":
            mat = "vein"
        if name.endswith("seed") or name in ("wild_grain", "barley", "rye", "millet", "flour", "bread", "cheese", "beeswax"):
            mat = "thatch"
        m.ball(mat, 0.15, loc=(0, 0, 0.09), scale=(1.2, 0.8, 0.7), subdiv=1, ground=True, jitter=0.12)


def build():
    with open(os.path.join(C.HERE, "asset_items.json")) as f:
        data = json.load(f)
    rows = data["items"]
    assert len(rows) == 119 and len({r["code"] for r in rows}) == 119
    entries = {}
    for row in rows:
        for suffix, count in (("s", 1), ("m", 3), ("l", 6)):
            m = C.Model(f"item_{row['code']:03d}_{row['id']}_{suffix}")
            item(m, row)
            if count > 1:
                import bmesh
                original = list(m.bm.verts) + list(m.bm.edges) + list(m.bm.faces)
                for k in range(1, count):
                    result = bmesh.ops.duplicate(m.bm, geom=original)
                    verts = [v for v in result["geom"] if isinstance(v, bmesh.types.BMVert)]
                    bmesh.ops.translate(m.bm, verts=verts, vec=((k % 2) * 0.07, (k % 3) * 0.06, k * 0.05))
            C.save_static(m, (1, 1), entries, {"family": "item", "item_code": row["code"],
                          "content_id": row["id"], "stack_display_count": count,
                          "source_sha": data["source_sha"]})
    assert len(entries) == 357
    with open(os.path.join(C.MODELS_DIR, "asset_items_manifest.json"), "w") as f:
        json.dump({"models": entries}, f, indent=1)


if __name__ == "__main__":
    C.reset_scene()
    build()
