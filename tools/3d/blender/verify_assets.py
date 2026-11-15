"""Check delivered model fragments: python3 tools/3d/blender/verify_assets.py <manifest...>.

Uses the shipped GLB reader; no Blender or additional dependencies required.
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

from verify_vat import accessor, read_glb

ROOT = Path(__file__).resolve().parents[3]
ASSETS = ROOT / "assets/3d"


def check_manifest(path: Path) -> int:
    """Require real, finite triangle geometry for every registered model."""
    with path.open() as stream:
        models = json.load(stream)["models"]
    assert models, f"empty manifest: {path}"
    files: set[str] = set()
    for identity, entry in models.items():
        relative = Path(entry["file"])
        assert not relative.is_absolute() and ".." not in relative.parts, identity
        assert relative.as_posix() not in files, f"duplicate file: {identity}"
        files.add(relative.as_posix())
        model = ASSETS / relative
        raw = model.read_bytes()
        magic, version, length = struct.unpack_from("<III", raw)
        assert (magic, version, length) == (0x46546C67, 2, len(raw)), identity
        document, binary = read_glb(model)
        triangles = 0
        vertices = 0
        for mesh in document["meshes"]:
            for primitive in mesh["primitives"]:
                assert primitive.get("mode", 4) == 4, identity
                positions = accessor(document, binary, primitive["attributes"]["POSITION"])
                assert positions and all(math.isfinite(v) for p in positions for v in p), identity
                vertices += len(positions)
                indices = accessor(document, binary, primitive["indices"])
                assert len(indices) % 3 == 0, identity
                assert all(0 <= i[0] < len(positions) for i in indices), identity
                triangles += len(indices) // 3
        assert vertices > 0 and triangles == entry["triangles"] > 0, identity
        assert math.isfinite(entry["height"]) and entry["height"] > 0, identity
        assert len(entry["footprint"]) == 2 and all(v > 0 for v in entry["footprint"]), identity
        if entry.get("animations"):
            names = {clip["name"] for clip in document.get("animations", [])}
            assert set(entry["animations"]) <= names and document.get("skins"), identity
    print(f"PASS {path.relative_to(ROOT)}: {len(models)} models")
    return len(models)


def main() -> None:
    """Validate explicitly selected fragments; absent inputs fail closed."""
    assert len(sys.argv) > 1, "pass one or more manifest paths"
    total = sum(check_manifest(Path(path).resolve()) for path in sys.argv[1:])
    print(f"PASS total={total}")


if __name__ == "__main__":
    main()
