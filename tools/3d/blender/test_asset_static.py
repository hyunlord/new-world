"""Check M3D-ASSETS GLB coverage, geometry and palette without Blender.

Run: python3 tools/3d/blender/test_asset_static.py
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import struct
from pathlib import Path

from test_fence_directions import positions

ROOT = Path(__file__).resolve().parents[3]
MODELS = ROOT / "assets/3d/models"
PALETTE = {"trunk", "canopy", "fruit", "stone", "vein", "earth", "wood", "wood_dark", "clay", "metal", "glow", "thatch", "hide", "cloth", "skin", "hair", "leather", "horn", "fur", "fur_light", "snow"}


def check():
    fragments = [json.loads((MODELS / f"asset_{family}_manifest.json").read_text())["models"]
                 for family in ("stations", "items")]
    assert [len(f) for f in fragments] == [63, 357]
    codes = {}
    for entries in fragments:
        for name, entry in entries.items():
            raw = (ROOT / "assets/3d" / entry["file"]).read_bytes()
            magic, version, length = struct.unpack_from("<4sII", raw)
            assert (magic, version, length) == (b"glTF", 2, len(raw)), name
            size, chunk = struct.unpack_from("<I4s", raw, 12)
            assert chunk == b"JSON", name
            gltf = json.loads(raw[20:20 + size])
            assert gltf["meshes"] and entry["triangles"] > 0, name
            assert set(entry["materials"]) <= PALETTE, name
            assert entry["height"] > 0, name
            if "item_code" in entry:
                codes.setdefault(entry["item_code"], set()).add(entry["stack_display_count"])
    for suffix in ("s", "m", "l"):
        vertices = positions(MODELS / f"item_035_hand_spindle_{suffix}.glb")
        assert min(v[1] for v in vertices) >= -1e-6, suffix
    item_entries = fragments[1]
    for code, content, material in ((7, "clay", "clay"), (41, "bone_needle", "horn"),
                                    (3, "flint", "stone"), (50, "hoe", "stone"), (105, "iron_hoe", "metal")):
        assert material in item_entries[f"item_{code:03d}_{content}_s"]["materials"]
    ore_colors = []
    for code, content in ((73, "copper_ore"), (74, "tin_ore"), (75, "iron_ore"), (101, "bog_iron")):
        raw = (MODELS / f"item_{code:03d}_{content}_s.glb").read_bytes()
        size = struct.unpack_from("<I", raw, 12)[0]
        gltf = json.loads(raw[20:20 + size])
        ore_colors.append(tuple(gltf["materials"][0]["pbrMetallicRoughness"]["baseColorFactor"]))
    assert len(set(ore_colors)) == 4
    assert item_entries["item_054_arrow_s"]["triangles"] != item_entries["item_015_spear_s"]["triangles"]
    assert set(codes) == set(range(119))
    assert all(counts == {1, 3, 6} for counts in codes.values())
    print("PASS: 63 station/construction pieces; 119 item codes × 3 stack counts; 420 valid GLBs")


if __name__ == "__main__":
    check()
