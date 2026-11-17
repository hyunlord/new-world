"""Check exported L2 silhouettes, original IDs/states and open vessel geometry.

python3 tools/3d/blender/test_look2_props.py
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import math
import struct
from pathlib import Path

from test_fence_directions import check as check_fences
from test_fence_directions import positions

ROOT = Path(__file__).resolve().parents[3]
MODELS = ROOT / "assets/3d/models"


def check_bench_support() -> None:
    """Low benches must not inherit pegs floating at the tall wood bench height."""
    for identity, ceiling in (("pottery_bench", .43), ("casting_bench", .34)):
        raw = (MODELS / f"{identity}.glb").read_bytes()
        size = struct.unpack_from("<I", raw, 12)[0]
        doc = json.loads(raw[20:20 + size])
        heights = []
        for mesh in doc["meshes"]:
            for part in mesh["primitives"]:
                if doc["materials"][part["material"]]["name"] != "wood_dark":
                    continue
                accessor = doc["accessors"][part["attributes"]["POSITION"]]
                view = doc["bufferViews"][accessor["bufferView"]]
                start = 28 + size + view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
                stride = view.get("byteStride", 12)
                heights.extend(struct.unpack_from("<fff", raw, start + i * stride)[1]
                               for i in range(accessor["count"]))
        assert heights and max(heights) <= ceiling, (identity, max(heights), ceiling)


def check_open_pot() -> None:
    """The old flat cap sealed the mouth; no triangle may cover its top centre."""
    raw = (MODELS / "pot.glb").read_bytes()
    size = struct.unpack_from("<I", raw, 12)[0]
    doc = json.loads(raw[20:20 + size])
    binary = 28 + size

    def read(index: int) -> list[tuple]:
        accessor = doc["accessors"][index]
        view = doc["bufferViews"][accessor["bufferView"]]
        count = {"VEC3": 3, "SCALAR": 1}[accessor["type"]]
        scalar = {5126: "f", 5125: "I", 5123: "H", 5121: "B"}[accessor["componentType"]]
        fmt = "<" + scalar * count
        stride = view.get("byteStride", struct.calcsize(fmt))
        start = binary + view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        return [struct.unpack_from(fmt, raw, start + n * stride) for n in range(accessor["count"])]

    top_faces = 0
    for mesh in doc["meshes"]:
        for primitive in mesh["primitives"]:
            verts = read(primitive["attributes"]["POSITION"])
            indices = [v[0] for v in read(primitive["indices"])]
            for i in range(0, len(indices), 3):
                triangle = [verts[j] for j in indices[i:i + 3]]
                if min(p[1] for p in triangle) < .375:
                    continue
                top_faces += 1
                x = sum(p[0] for p in triangle) / 3
                z = sum(p[2] for p in triangle) / 3
                assert math.hypot(x, z) > .075, "pot mouth sealed by a cap"
    assert top_faces >= 12, "rolled rim missing"


def check() -> None:
    """All exported props fit their contract and working states alone emit light."""
    rows = {}
    for name, count in (("look2_props_manifest.json", 42), ("asset_stations_manifest.json", 63),
                        ("w3d_stations_manifest.json", 13)):
        family = json.loads((MODELS / name).read_text())["models"]
        assert len(family) == count, name
        assert not rows.keys() & family.keys(), name
        rows.update(family)
    canonical = json.loads((MODELS / "manifest.json").read_text())["models"]
    replacements = json.loads((MODELS / "look2_props_manifest.json").read_text())["models"]
    assert all(canonical[k] == v for k, v in replacements.items())
    for identity, row in rows.items():
        vertices = positions(ROOT / "assets/3d" / row["file"])
        assert all(math.isfinite(x) for v in vertices for x in v), identity
        assert min(v[1] for v in vertices) >= -.0001, identity
        assert 0 < row["triangles"] <= 6500, identity
        assert len(row["materials"]) <= 11, identity
        if identity != "door":
            w, h = row["footprint"]
            assert max(abs(v[0]) for v in vertices) <= w / 2 + .001, identity
            assert max(abs(v[2]) for v in vertices) <= h / 2 + .001, identity
        if identity.startswith("w3d_"):
            working = identity.endswith("_working")
            assert ("glow" in row["materials"]) == working, identity
            assert ("light" in row["sockets"]) == working, identity
        if identity.startswith(("asset_smelter_", "asset_bloomery_", "asset_kiln_", "asset_charcoal_clamp_")):
            active = row["state"] in ("lit", "burning", "working")
            assert ("glow" in row["materials"]) == active, identity
            assert ("light" in row["sockets"]) == active, identity
    assert set(k for k in rows if k.startswith("yard_")) == {
        "yard_cart", "yard_hay", "yard_woodpile", "yard_tool_rack", "yard_grain_baskets"}
    check_open_pot()
    check_bench_support()
    check_fences()
    print("PASS: 118 L2 models; five yard IDs; vessel opening; bench support; states; bounds; 32 fence masks")


if __name__ == "__main__":
    check()
