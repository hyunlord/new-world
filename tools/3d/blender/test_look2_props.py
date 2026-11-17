"""Check exported L2 silhouettes, original IDs/states and open vessel geometry.

python3 tools/3d/blender/test_look2_props.py
blender -b --factory-startup -P tools/3d/blender/test_look2_props.py -- --material-cache
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import math
import struct
import sys
from pathlib import Path
from tempfile import TemporaryDirectory

sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_fence_directions import check as check_fences
from test_fence_directions import positions

ROOT = Path(__file__).resolve().parents[3]
MODELS = ROOT / "assets/3d/models"


def check_material_cache() -> None:
    """Export smoke then ordinary snow through the same cached palette material."""
    import common as C
    import stations_w3d as W

    C.reset_scene()
    with TemporaryDirectory(prefix="look2-smoke-") as folder:
        for identity, alpha in (("smoke_probe", .16), ("snow_probe", 1.0)):
            model = C.Model(identity)
            if alpha < 1:
                W.smoke(model, 0, 0, 0)
            else:
                model.cyl("snow", .1, .2)
            obj = model.build_object()
            material = obj.data.materials[0]
            assert abs(material.node_tree.nodes["Principled BSDF"].inputs["Alpha"].default_value - alpha) < .0001
            assert abs(material.diffuse_color[3] - alpha) < .0001
            path = Path(folder) / f"{identity}.glb"
            C.export_glb(str(path), [obj])
            raw = path.read_bytes()
            size = struct.unpack_from("<I", raw, 12)[0]
            saved = json.loads(raw[20:20 + size])["materials"][0]
            expected = "BLEND" if alpha < 1 else "OPAQUE"
            assert saved.get("alphaMode", "OPAQUE") == expected, (identity, saved)
    print("PASS: smoke alpha and viewport alpha; following ordinary snow exports opaque")


def check_smoke_alpha() -> None:
    """Only authored smoke surfaces blend; other surfaces remain fully opaque."""
    smoky = {"charcoal_clamp", "kiln", "smelter", "asset_smelter_lit",
             "asset_charcoal_clamp_burning", "asset_kiln_lit", "asset_bloomery_lit",
             "w3d_bloomery_working", "w3d_charcoal_clamp_working", "w3d_kiln_working",
             "w3d_smelter_working"}
    seen = set()
    for fragment in ("look2_props_manifest.json", "asset_stations_manifest.json", "w3d_stations_manifest.json"):
        for identity, row in json.loads((MODELS / fragment).read_text())["models"].items():
            raw = (ROOT / "assets/3d" / row["file"]).read_bytes()
            size = struct.unpack_from("<I", raw, 12)[0]
            doc = json.loads(raw[20:20 + size])
            for material in doc["materials"]:
                alpha = material.get("pbrMetallicRoughness", {}).get("baseColorFactor", [1, 1, 1, 1])[3]
                mode = material.get("alphaMode", "OPAQUE")
                smoke = identity in smoky and material["name"] == "snow"
                if smoke:
                    assert mode == "BLEND" and .1 <= alpha <= .2, (identity, mode, alpha)
                    seen.add(identity)
                else:
                    assert mode == "OPAQUE" and alpha == 1, (identity, material["name"], mode, alpha)
    assert seen == smoky, (seen, smoky)


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
    check_smoke_alpha()
    check_fences()
    print("PASS: 118 L2 models; five yard IDs; vessel opening; bench support; smoke alpha11; states; bounds; 32 fence masks")


if __name__ == "__main__":
    if "--material-cache" in sys.argv:
        check_material_cache()
    else:
        check()
