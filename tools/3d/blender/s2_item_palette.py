"""Style 2 palette copies of the pale/pink M3D-ASSETS item piles and stations (INTEG1). Plain Python.

The M3D-ASSETS item glbs store their PALETTE colours as glTF baseColorFactor, which glTF defines as linear, so
Godot shows them one sRGB step lighter: clay reads pink, grain and stone near-white, berries bubblegum pink.
This copies each listed item's s/m/l models to models/w3d_item_<id>_<bucket>.glb with the listed surfaces set to
an S2 palette colour (given in sRGB here, written as linear factors), copies their manifest rows into the fragment
w3d_items_manifest.json; bindings3d `items.by_id` / `stations.by_style` point at them (a source's textures3d
`model_surfaces` override is repeated by hand for its copy, e.g. w3d_item_clay_*).
Geometry and every other surface are unchanged.

Usage: python3 tools/3d/blender/s2_item_palette.py
"""
import json
import os
import struct

REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
MODELS = os.path.join(REPO, "assets", "3d", "models")
FRAGMENT = os.path.join(MODELS, "w3d_items_manifest.json")
# item id -> (source model prefix, {surface: sRGB colour}).
PALETTE = {
    "clay": ("item_007_clay", {"clay": (0.56, 0.41, 0.30)}),            # wet raw clay, grey-brown
    "raw_pot": ("item_008_raw_pot", {"clay": (0.63, 0.49, 0.36)}),      # unfired greenware, buff
    "porridge": ("item_023_porridge", {"clay": (0.60, 0.35, 0.21), "earth": (0.74, 0.63, 0.43)}),
    "berries": ("item_017_berries", {"fruit": (0.50, 0.09, 0.15)}),     # deep red
    "dried_berries": ("item_026_dried_berries", {"fruit": (0.34, 0.11, 0.12)}),  # dark maroon
    "emmer_seed": ("item_031_emmer_seed", {"thatch": (0.69, 0.57, 0.33)}),       # golden grain
    "wild_grain": ("item_019_wild_grain", {"thatch": (0.69, 0.57, 0.33)}),
    "stone": ("item_002_stone", {"stone": (0.55, 0.54, 0.50)}),
    "hazelnuts": ("item_018_hazelnuts", {"wood": (0.55, 0.38, 0.22)}),  # hazel shell brown
    "clay_pot": ("item_009_clay_pot", {"clay": (0.60, 0.35, 0.21)}),    # fired pot, same family as raw_pot
}
# Whole models (stations) -> {surface: sRGB}; written as w3d_station_<id>.glb (bindings3d stations.by_style).
STATIONS = {
    "pottery_bench": {"clay": (0.63, 0.49, 0.36)},  # its greenware pots read white in S2
}
BUCKETS = ("s", "m", "l")


def linear(c: float) -> float:
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def recolour(src: str, dst: str, colours: dict) -> list:
    data = open(src, "rb").read()
    json_len = struct.unpack_from("<I", data, 12)[0]
    doc = json.loads(data[20:20 + json_len])
    rest = data[20 + json_len:]
    hit = []
    for mat in doc.get("materials", []):
        if mat.get("name") in colours:
            pbr = mat.setdefault("pbrMetallicRoughness", {})
            pbr["baseColorFactor"] = [round(linear(c), 5) for c in colours[mat["name"]]] + [1.0]
            hit.append(mat["name"])
    if sorted(hit) != sorted(colours):
        raise ValueError(f"{src}: surfaces {sorted(colours)} expected, found {hit}")
    chunk = json.dumps(doc, separators=(",", ":")).encode()
    chunk += b" " * (-len(chunk) % 4)
    body = struct.pack("<I4s", len(chunk), b"JSON") + chunk + rest
    with open(dst, "wb") as f:
        f.write(struct.pack("<4sII", b"glTF", 2, 12 + len(body)) + body)
    return hit


def main() -> None:
    manifest = json.load(open(os.path.join(REPO, "assets", "3d", "assets_manifest.json")))["models"]
    # The explicit models/manifest.json row wins a duplicate id (data3d merge order).
    manifest.update(json.load(open(os.path.join(MODELS, "manifest.json")))["models"])
    entries = {}
    for item, (prefix, colours) in PALETTE.items():
        for bucket in BUCKETS:
            src_id, dst_id = f"{prefix}_{bucket}", f"w3d_item_{item}_{bucket}"
            recolour(os.path.join(MODELS, src_id + ".glb"), os.path.join(MODELS, dst_id + ".glb"), colours)
            row = dict(manifest[src_id])
            row["file"] = f"models/{dst_id}.glb"
            row.pop("source_sha", None)
            entries[dst_id] = row
            print("ITEM_PALETTE", dst_id, colours)
    for src_id, colours in STATIONS.items():
        dst_id = f"w3d_station_{src_id}"
        recolour(os.path.join(MODELS, src_id + ".glb"), os.path.join(MODELS, dst_id + ".glb"), colours)
        entries[dst_id] = dict(manifest[src_id], file=f"models/{dst_id}.glb")
        print("ITEM_PALETTE", dst_id, colours)
    with open(FRAGMENT, "w") as f:
        json.dump({"models": entries}, f, indent=1)
        f.write("\n")


main()
