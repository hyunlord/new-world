"""Merge explicit asset fragments without changing M3D-1's manifest.

python3 tools/3d/blender/merge_assets.py <fragment...>
Writes assets/3d/assets_manifest.json; rerun after generating or adding a family.
"""
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ASSETS = ROOT / "assets/3d"


def main() -> None:
    """Reject model collisions and dangling blueprint mappings before writing."""
    assert len(sys.argv) > 1, "pass all delivered manifest fragments"
    baseline = ASSETS / "models/manifest.json"
    with baseline.open() as stream:
        merged = json.load(stream)
    merged["fragments"] = []
    merged["blueprints"] = {}
    for argument in sys.argv[1:]:
        path = Path(argument).resolve()
        with path.open() as stream:
            fragment = json.load(stream)
        models = fragment["models"]
        assert models, f"empty fragment: {path}"
        collisions = merged["models"].keys() & models.keys()
        assert not collisions, f"duplicate model IDs: {sorted(collisions)}"
        merged["models"].update(models)
        merged["fragments"].append(path.relative_to(ROOT).as_posix())
        blueprints = fragment.get("blueprints", {})
        assert not merged["blueprints"].keys() & blueprints.keys(), "duplicate blueprint codes"
        merged["blueprints"].update(blueprints)
    for identity, entry in merged["models"].items():
        assert (ASSETS / entry["file"]).is_file(), identity
    for code, entry in merged["blueprints"].items():
        assert entry["model"] in merged["models"], code
    merged["models"] = dict(sorted(merged["models"].items()))
    output = ASSETS / "assets_manifest.json"
    output.write_text(json.dumps(merged, indent=1) + "\n")
    print(f"ASSETS_MERGED models={len(merged['models'])} blueprints={len(merged['blueprints'])}")


if __name__ == "__main__":
    main()
