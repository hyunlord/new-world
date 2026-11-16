"""Check exported fence connection geometry in Godot world directions.

Run: python3 tools/3d/blender/test_fence_directions.py
Reads binary POSITION vertices independently of the generator's connection table.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import struct
from pathlib import Path

MODELS = Path(__file__).resolve().parents[3] / "assets/3d/models"


def positions(path):
    raw = path.read_bytes()
    json_size = struct.unpack_from("<I", raw, 12)[0]
    gltf = json.loads(raw[20:20 + json_size])
    binary_start = 20 + json_size + 8
    vertices = []
    for mesh in gltf["meshes"]:
        for primitive in mesh["primitives"]:
            accessor = gltf["accessors"][primitive["attributes"]["POSITION"]]
            assert accessor["type"] == "VEC3" and accessor["componentType"] == 5126
            view = gltf["bufferViews"][accessor["bufferView"]]
            start = binary_start + view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
            stride = view.get("byteStride", 12)
            vertices.extend(struct.unpack_from("<fff", raw, start + i * stride)
                            for i in range(accessor["count"]))
    assert vertices
    return vertices


def check():
    for family in ("pen_fence", "palisade"):
        for mask in range(16):
            vertices = positions(MODELS / f"{family}_{mask:02x}.glb")
            xs = [v[0] for v in vertices]
            zs = [v[2] for v in vertices]
            # Godot north is -Z; east +X; south +Z; west -X.
            reached = (min(zs) < -0.4, max(xs) > 0.4, max(zs) > 0.4, min(xs) < -0.4)
            expected = tuple(bool(mask & (1 << bit)) for bit in range(4))
            assert reached == expected, (family, mask, reached, expected)
    print("PASS: 32 exported fence masks including all corners/Ts match Godot NESW directions")


if __name__ == "__main__":
    check()
