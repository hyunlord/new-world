"""Blueprint-code models assembled from the shared station and wall builders.

Run: blender -b --factory-startup -P tools/3d/blender/asset_blueprints.py
IDs reuse the existing M3D-1 style models wherever the blueprint shares that style.
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

BLUEPRINTS = (
    ("hut", "lean_to"), ("knapping_spot", "knapping_spot"), ("wood_bench", "wood_bench"),
    ("drying_rack", "drying_rack"), ("log_wall", "wall_log_complete"), ("stone_wall", "wall_stone_complete"),
    ("door", "door"), ("packed_floor", "floor_packed"), ("bedroll", "bedroll"),
    ("grinding_stone", "grinding_stone"), ("pottery_bench", "pottery_bench"), ("hearth_fire", "asset_hearth_lit"),
    ("food_drying_rack", "drying_rack"), ("retting_pit", "retting_pit"), ("loom", "loom"),
    ("fence", "pen_fence_0a"), ("clay_oven", "clay_oven"), ("smoking_rack", "drying_rack"),
    ("charcoal_clamp", "asset_charcoal_clamp_idle"), ("smelter", "asset_smelter_unlit"),
    ("casting_bench", "asset_casting_bench_empty"), ("mud_brick_wall", "wall_mud_brick_complete"),
    ("palisade", "palisade_0a"), ("kiln", "asset_kiln_unlit"), ("stone_masonry_wall", "wall_stone_complete"),
    ("bloomery", "asset_bloomery_unlit"), ("forge", "asset_forge_idle"), ("anvil", "asset_anvil_idle"),
    ("mason_bench", "mason_bench_idle"), ("cut_stone_wall", "wall_stone_complete"),
    ("town_wall", "wall_stone_complete"), ("town_gate", "door"), ("paved_road", "floor_paved_road"),
)


def build():
    m = C.Model("floor_packed")
    m.box("earth", (1, 1, 0.03), loc=(0, 0, 0.015))
    entries = {}
    C.save_static(m, (1, 1), entries, {"family": "floor"})
    for code, (name, model) in enumerate(BLUEPRINTS):
        if model not in entries:
            path = os.path.join(C.MODELS_DIR, f"{model}.glb")
            assert os.path.isfile(path), path
    result = {"source_sha": "cfeb57c9e1906eedd4e5d14447ba4faca20c4047", "models": entries,
              "blueprints": {str(code): {"content_id": name, "model": model}
                             for code, (name, model) in enumerate(BLUEPRINTS)}}
    assert len(result["blueprints"]) == 33
    with open(os.path.join(C.MODELS_DIR, "asset_blueprints_manifest.json"), "w") as f:
        json.dump(result, f, indent=1)


if __name__ == "__main__":
    C.reset_scene()
    build()
